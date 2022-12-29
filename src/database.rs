#[derive(Clone)]
pub struct Database {
    db: sqlx::mysql::MySqlPool
}

impl Database {
    pub fn new(connection: sqlx::mysql::MySqlPool) -> Database {
        Database { db: connection }
    }

    pub async fn migrate(&self) {
        sqlx::migrate!("./migrations")
            .run(&self.db).await
            .expect("Failed to migrate database")
    }

    pub async fn fetch_articles(&self) -> Vec<ArticleListItem> {
        let q = sqlx::query_as!(
            ArticleListItem,
            "SELECT added, title FROM blogs ORDER BY id DESC
        ");

        q.fetch_all(&self.db).await
            .expect("Query failed")
    }


    pub async fn fetch_article(&self, title: &str) -> Option<ArticleItem> {
        let q = sqlx::query_as!(
            ArticleItem,
            "SELECT id, title, body,
                (id <=> (SELECT MAX(id) FROM blogs)) AS 'is_last: bool',
                (id <=> (SELECT MIN(id) FROM blogs)) AS 'is_first: bool'
                FROM blogs
                WHERE title = ?
        ", title);

        q.fetch_one(&self.db).await.ok()
    }


    pub async fn fetch_next_article_title_after_id(&self, id: i64) -> String {
        let id = id + 1;
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


    pub async fn fetch_prev_article_title_before_id(&self, id: i64) -> String {
        let id = id - 1;
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


    pub async fn create_article(&self, title: &str, body: &str) -> bool {
        let q = sqlx::query!("
             INSERT IGNORE INTO blogs (title, body, added)
                VALUES (?, ?, DATE('now'));
        ", title, body);

        q.execute(&self.db).await
            .expect("Query failed")
            .rows_affected() != 0
    }

    pub async fn delete_article(&self, title: &str) -> bool {
        let q = sqlx::query!("
            DELETE FROM blogs WHERE title = ?
        ", title);

        q.execute(&self.db).await
            .expect("Query failed")
            .rows_affected() != 0
    }

}


#[derive(Debug)]
pub struct ArticleListItem {
    pub added: sqlx::types::time::Date,
    pub title: String
}


#[derive(Debug)]
pub struct ArticleItem {
    pub body: String,
    pub title: String,
    pub id: u32,
    pub is_last: bool,
    pub is_first: bool,
}
