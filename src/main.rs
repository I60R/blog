mod handlers;
mod database;

use axum::{
    extract::Path,
    routing,
    response::Redirect,
};
use std::net::SocketAddr;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let connection = sqlx::sqlite::SqlitePool::connect(
        &std::env::var("DATABASE_URL")?
    ).await?;
    let db = database::Database::new(connection);
    db.init().await;

    let app: axum::Router<_, axum::body::Body> = axum::Router::new()
        .route("/", routing::get(
            || async { Redirect::permanent("/blog") })
        )
        .route("/blog", routing::get(handlers::get_articles))
        .route("/blog/", routing::get(
            || async { Redirect::permanent("/blog") })
        )
        .route("/blog/next/after_id=:id", routing::get(handlers::next_article))
        .route("/blog/prev/after_id=:id", routing::get(handlers::prev_article))
        .route("/blog/:title", routing::post(handlers::create_article))
        .route("/blog/:title", routing::delete(handlers::delete_article))
        .route("/blog/:title", routing::get(handlers::get_article))
        .route("/blog/:title/", routing::get(
            |Path(title): Path<String>| async move { Redirect::permanent(&format!("/blog/{title}")) })
        )
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
