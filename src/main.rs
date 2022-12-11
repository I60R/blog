mod handlers;
mod database;

use axum::routing;
use std::net::SocketAddr;




#[tokio::main]
async fn main() {
    let connection = sqlite::open("blogs.sqlite")
        .expect("Cannot open DB");
    let db = database::Database::new(connection);
    db.init();

    let app: axum::Router<_, axum::body::Body> = axum::Router::new()
        .route("/", routing::get(handlers::get_about))
        .route("/blog", routing::get(handlers::get_articles))
        .route("/blog/:title", routing::get(handlers::get_article))
        .route("/blog/:title", routing::post(handlers::create_article))
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server Error");
}
