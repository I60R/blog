#[derive(Clone)]
pub struct Database {
    db: sqlx::sqlite::SqlitePool
}

impl Database {
    pub fn new(connection: sqlx::sqlite::SqlitePool) -> Database {
        Database { db: connection }
    }


    pub async fn init(&self) {
        let q = sqlx::query("
            CREATE TABLE blogs (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                added TEXT NOT NULL
            );
        ");

        if let Err(e) = q.execute(&self.db).await {
            if !e.to_string().contains("table blogs already exists") {
                panic!("{e}")
            }
        }
    }

    pub async fn fetch_articles(&self) -> Vec<ArticleListItem> {
        let q = sqlx::query_as!(
            ArticleListItem,
            "SELECT added, title FROM blogs
        ");

        q.fetch_all(&self.db).await
            .expect("Query failed")
    }


    pub async fn fetch_article(&self, title: &str) -> ArticleItem {
        let q = sqlx::query_as!(
            ArticleItem,
            "SELECT id, title, body,
                (id = (SELECT MAX(id) FROM blogs)) AS 'is_last: bool',
                (id = (SELECT MIN(id) FROM blogs)) AS 'is_first: bool'
                FROM blogs
                WHERE title = ?
        ", title);

        q.fetch_one(&self.db).await
            .expect("Query failed")
    }


    pub async fn fetch_article_title_by_id(&self, id: i64) -> String {
        let q = sqlx::query!("
            SELECT title FROM blogs WHERE id = ?;
        ", id);

        q.fetch_one(&self.db).await
            .expect("Query failed")
            .title
    }


    pub async fn create_article(&self, title: &str, body: &str) -> bool {
        let q = sqlx::query!("
             INSERT OR IGNORE INTO blogs (title, body, added)
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
    pub added: String,
    pub title: String
}

#[derive(Debug)]
pub struct ArticleItem {
    pub body: String,
    pub title: String,
    pub is_last: bool,
    pub is_first: bool,
    pub id: i64,

}