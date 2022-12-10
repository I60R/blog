

use axum::{routing, extract};
use std::net::SocketAddr;


#[tokio::main]
async fn main() {

    let app: axum::Router<(), axum::body::Body> = axum::Router::new()
        .route("/", routing::get(get_about))
        .route("/blog", routing::get(get_articles))
        .route("/blog/:title", routing::post(create_article))
        .route("/blog/:title", routing::get(get_article))
        .route("/image/:title", routing::get(get_article));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server Error");
}



async fn get_articles() -> &'static str {
    "Hello world"
}

async fn get_about() -> &'static str {
    "Hello world: about"
}

async fn get_article(
    extract::Path(title): extract::Path<String>,
) -> String {
    format!("Hello world: {title}")
}

async fn create_article(
    extract::Path(title): extract::Path<String>,
    payload: String,
) -> String {
    format!("title: {title}, payload: {payload}")
}
