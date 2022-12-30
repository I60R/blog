mod handlers;
mod database;
mod article;
mod repository;
mod view;

use axum::{
    extract::Path,
    routing,
    response::Redirect,
};
use tower_http::services;
use std::net::SocketAddr;

pub const ADDR: &str = "http://127.0.0.1:3000";


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = &std::env::var("DATABASE_URL")
        .map_err(|_| "No DATABASE_URL set, check README.md")?;
    let mysql = sqlx::mysql::MySqlPool::connect(database_url).await?;
    let db = database::Database::new_migrate(mysql).await;

    let state = repository::ArticlesRepository::new(db);

    let app: axum::Router = axum::Router::new()
        .route("/", routing::get(
            || async { Redirect::permanent("/blog") })
        )
        .route("/blog", routing::get(handlers::get_articles))
        .route("/blog/next/:id", routing::get(handlers::next_article))
        .route("/blog/next/:id/", routing::get(
            |Path(id): Path<i64>| async move { Redirect::permanent(&format!("/blog/next/{id}")) })
        )
        .route("/blog/prev/:id", routing::get(handlers::prev_article))
        .route("/blog/prev/:id/", routing::get(
            |Path(id): Path<i64>| async move { Redirect::permanent(&format!("/blog/prev/{id}")) })
        )
        .route("/blog/:title", routing::post(handlers::create_article))
        .route("/blog/:title/", routing::post(
            |Path(title): Path<String>| async move { Redirect::permanent(&format!("/blog/{title}")) })
        )
        .route("/blog/:title", routing::delete(handlers::delete_article))
        .route("/blog/:title/", routing::delete(
            |Path(title): Path<String>| async move { Redirect::permanent(&format!("/blog/{title}")) })
        )
        .route("/blog/:title", routing::get(handlers::get_article))
        .route("/blog/:title/", routing::get(
            |Path(title): Path<String>| async move { Redirect::permanent(&format!("/blog/{title}")) })
        )
        .nest_service("/content", {
            let dir = services::ServeDir::new("content");
            routing::get_service(dir)
                .handle_error(|e| async move { eprintln!("{e:?}") })
        })
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
