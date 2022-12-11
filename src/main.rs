mod handlers;

use axum::routing;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};


#[derive(Clone)]
pub struct Database {
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
        );
    ";
    if let Err(sqlite::Error { message: Some(message), .. }) = db
        .execute(q)
    {
        if message != "table blogs already exists" {
            panic!("{message}")
        }
    }

    let db = Arc::new(Mutex::new(db));
    let state = Database {
        db,
    };

    let app: axum::Router<_, axum::body::Body> = axum::Router::new()
        .route("/", routing::get(handlers::get_about))
        .route("/blog", routing::get(handlers::get_articles))
        .route("/blog/:title", routing::get(handlers::get_article))
        .route("/blog/:title", routing::post(handlers::create_article))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server Error");
}
