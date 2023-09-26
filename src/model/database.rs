use super::article;


#[derive(Clone)]
pub struct Database {
    db: sqlx::mysql::MySqlPool
}

impl Database {
    pub async fn new_migrate(connection: sqlx::mysql::MySqlPool) -> Database {
        sqlx::migrate!("./migrations")
            .run(&connection).await
            .expect("Cannot migrate database");
        Database { db: connection }
    }


    #[tracing::instrument]
    pub async fn fetch_articles(&self) -> Vec<article::ListItem> {
        let q = sqlx::query_as!(
            article::ListItem,
            "SELECT id, added, title FROM blogs
        ");

        q.fetch_all(&self.db).await
            .expect("Query failed")
    }

    #[tracing::instrument]
    pub async fn fetch_article(&self, title: &str) -> Option<article::Item> {
        let q = sqlx::query_as!(
            article::Item,
            "SELECT id, title, body,
                (id <=> (SELECT MAX(id) FROM blogs)) AS 'is_last: bool',
                (id <=> (SELECT MIN(id) FROM blogs)) AS 'is_first: bool'
                FROM blogs
                WHERE title = ?
        ", title);

        q.fetch_one(&self.db).await.ok()
    }


    #[tracing::instrument]
    pub async fn fetch_next_article_title_after_id(&self, id: u32) -> String {
        let id = id.saturating_add(1);
        let q = sqlx::query!("
            SELECT COALESCE(
                (SELECT title FROM blogs WHERE id = ?),
                (SELECT title FROM blogs WHERE id > ? ORDER BY id LIMIT 1)
            ) AS title;
        ", id, id);

        q.fetch_one(&self.db).await
            .expect("Query failed")
            .title
            .unwrap_or_else(|| String::from("Deleted"))
    }

    #[tracing::instrument]
    pub async fn fetch_prev_article_title_before_id(&self, id: u32) -> String {
        let id = id.saturating_sub(1);
        let q = sqlx::query!("
            SELECT COALESCE(
                (SELECT title FROM blogs WHERE id = ?),
                (SELECT title FROM blogs WHERE id < ? ORDER BY id DESC LIMIT 1)
            ) AS title;
        ", id, id);

        q.fetch_one(&self.db).await
            .expect("Query failed")
            .title
            .unwrap_or_else(|| String::from("Deleted"))
    }

    #[tracing::instrument]
    pub async fn create_article(&self, title: &str, body: &str) -> bool {
        let q = sqlx::query!("
             INSERT IGNORE INTO blogs (title, body, added)
                VALUES (?, ?, NOW());
        ", title, body);

        q.execute(&self.db).await
            .expect("Query failed")
            .rows_affected() != 0
    }

    #[tracing::instrument]
    pub async fn delete_article(&self, title: &str) -> bool {
        let q = sqlx::query!("
            DELETE FROM blogs WHERE title = ?
        ", title);

        q.execute(&self.db).await
            .expect("Query failed")
            .rows_affected() != 0
    }
}


impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("🖴 ")
    }
}