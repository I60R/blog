use std::sync::{Arc, Mutex};


#[derive(Clone)]
pub struct Database {
    db: Arc<Mutex<sqlite::Connection>>
}

impl Database {
    pub fn new(connection: sqlite::Connection) -> Database {
        let db = Arc::new(Mutex::new(connection));
        Database {
            db
        }
    }


    pub fn init(&self) {
        let db = Arc::clone(&self.db);
        let db = db
            .lock()
            .unwrap();

        let q = "
            CREATE TABLE blogs (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                added TEXT NOT NULL
            );
        ";

        if let Err(sqlite::Error { message: Some(message), .. }) = db
            .execute(q)
        {
            if message != "table blogs already exists" {
                panic!("{message}")
            }
        }
    }


    pub fn fetch_articles(&self) -> Vec<(String, String)> {
        let db = Arc::clone(&self.db);
        let db = db
            .lock()
            .unwrap();

        let q = "
            SELECT added, title FROM blogs
        ";

        let mut resp = vec![];

        db.iterate(q, |columns| {
            let mut added = None;

            for &column in columns {
                match column {
                    ("added", date_added) => {
                        added = date_added;
                    },
                    ("title", title) => {
                        resp.push((
                            added.unwrap().to_string(),
                            title.unwrap().to_string()
                        ));
                    },
                    _ => { }
                }
            }

            true
        }).unwrap();

        resp
    }


    pub fn fetch_article(&self, title: &str) -> (String, String, String, bool) {
        let db = Arc::clone(&self.db);
        let db = db
            .lock()
            .unwrap();

        let q = format!("
            SELECT
                (SELECT MAX(id) FROM blogs) AS max_id, id, title, body
                FROM blogs
                WHERE title = '{title}'
        ");

        let mut article_max_id = String::new();
        let mut article_id = String::new();
        let mut article_title = String::new();
        let mut article_body = String::new();

        db.iterate(q, |columns| {
            for &column in columns {
                match column {
                    ("max_id", Some(max_id)) => {
                        article_max_id = String::from(max_id);
                    },
                    ("id", Some(id)) => {
                        article_id = String::from(id);
                    },
                    ("title", Some(title)) => {
                        article_title = String::from(title);
                    },
                    ("body", Some(body)) => {
                        article_body = String::from(body);

                        return true;
                    },
                    _ => {
                        article_title = String::from("invalid format");
                    }
                }
            }
            true
        }).unwrap();

        let is_last = article_id == article_max_id;

        (article_id, article_title, article_body, is_last)
    }


    pub(crate) fn fetch_article_title_by_id(&self, id: &str) -> String {
        let db = Arc::clone(&self.db);
        let db = db
            .lock()
            .unwrap();

        let q = format!("
            SELECT title FROM blogs WHERE id = {id};
        ");

        let mut article_title = String::new();

        db.iterate(q, |columns| {
            for &column in columns {
                match column {
                    ("title", Some(title)) => {
                        article_title = String::from(title);
                        return true;
                    },
                    _ => {
                        article_title = String::from("invalid format");
                    }
                }
            }
            true
        }).unwrap();


        article_title
    }


    pub fn create_article(&self, title: &str, body: &str) -> bool {

        let db = Arc::clone(&self.db);
        let db = db
            .lock()
            .unwrap();

        let q = format!("
             INSERT OR IGNORE INTO blogs (title, body, added)
                VALUES ('{title}', '{body}', DATE('now'))
                RETURNING 0;
        ");

        let mut created = false;

        db.iterate(q, |_pairs| {
            created = true;
            true
        }).unwrap();

        created
    }

    pub fn delete_article(&self, title: &str) -> bool {

        let db = Arc::clone(&self.db);
        let db = db
            .lock()
            .unwrap();

        let q = format!("
            DELETE FROM blogs WHERE title = '{title}'
            RETURNING 0;
        ");

        let mut deleted = false;

        db.iterate(q, |_pairs| {
            deleted = true;
            true
        }).unwrap();

        deleted
    }

}


