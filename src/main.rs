

use axum::{routing, extract};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};


#[derive(Clone)]
struct State {
    db: Arc<Mutex<sqlite::Connection>>
}


#[tokio::main]
async fn main() {
    let db = sqlite::open("blogs.sqlite")
        .expect("Cannot open DB");

    let q = "
        CREATE TABLE blogs (
            title TEXT NOT NULL PRIMARY_KEY,
            body TEXT NOT NULL,
            added TEXT NOT NULL,
        );";
    if let Err(sqlite::Error { message: Some(message), .. }) = db.execute(q) {
        if message != "table blogs already exists" {
            panic!("{message}")
        }
    }
    let db = Arc::new(Mutex::new(db));


    let state = State {
        db,
    };

    let app: axum::Router<_, axum::body::Body> = axum::Router::new()
        .route("/", routing::get(get_about))
        .route("/blog", routing::get(get_articles))
        .route("/blog/:title", routing::post(create_article))
        .route("/blog/:title", routing::get(get_article))
        .route("/image/:title", routing::get(get_article))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server Error");
}



async fn get_articles(
    extract::State(st): extract::State<State>,
) -> String {
    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();

    let q = "
        SELECT added, title FROM blogs
    ";

    let mut resp = String::new();

    db.iterate(q, |pairs| {
        for &(column, value) in pairs {
            match (column, value) {
                ("added", Some(date)) => {
                    resp.push('\n');
                    resp.push_str(date);
                    resp.push_str(" - ");
                },
                ("title", Some(title)) => {
                    resp.push_str(title)
                },
                _ => {
                    resp = String::from("invalid format");
                }
            }
        }
        true
    }).unwrap();

    resp
}

async fn get_about() -> &'static str {
    "Hello world: about"
}

async fn get_article(
    extract::State(st): extract::State<State>,
    extract::Path(title): extract::Path<String>,
) -> String {
    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();
    let q = format!("
        SELECT title, body FROM blogs WHERE title = '{title}'
    ");

    let mut resp = String::new();
    db.iterate(q, |pairs| {
        for &(column, value) in pairs {
            match (column, value) {
                ("title", Some(title)) => {
                    resp.push_str(title);
                    resp.push_str("\n");
                },
                ("body", Some(body)) => {
                    resp.push_str(body);
                    return true;
                },
                _ => {
                    resp = String::from("invalid format");
                }
            }
        }
        true
    }).unwrap();

    resp
}

async fn create_article(
    extract::State(st): extract::State<State>,
    extract::Path(title): extract::Path<String>,
    payload: String,
) -> String {
    let db = Arc::clone(&st.db);
    let db = db.lock().unwrap();
    let q = format!("
         INSERT INTO blogs (title, body, added)
            VALUES ('{title}', '{payload}', DATE('now'));
    ");
    db.execute(q).unwrap();

    format!("title: {title}, payload: {payload}")
}
