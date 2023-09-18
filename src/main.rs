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

    main::init_logging();

    let db = main::connect_database().await?;

    // Initialize application state
    let state = repository::ArticlesRepository::new(db);

    // Set routes
    let app: axum::Router = axum::Router::new()

        // entry points
        .route("/", routing::get(
            || async { Redirect::permanent("/blog") })
        )
        .route("/blog", routing::get(handlers::get_articles))

        // navigation
        .route("/blog/next/:id", routing::get(handlers::next_article))
        .route("/blog/next/:id/", routing::get(
            |Path(id): Path<i64>| async move { Redirect::permanent(&format!("/blog/next/{id}")) })
        )
        .route("/blog/prev/:id", routing::get(handlers::prev_article))
        .route("/blog/prev/:id/", routing::get(
            |Path(id): Path<i64>| async move { Redirect::permanent(&format!("/blog/prev/{id}")) })
        )
        .route("/blog/:title", routing::get(handlers::get_article))
        .route("/blog/:title/", routing::get(
            |Path(title): Path<String>| async move { Redirect::permanent(&format!("/blog/{title}")) })
        )

        // manipulation
        .route("/blog/:title", routing::post(handlers::create_article))
        .route("/blog/:title/", routing::post(
            |Path(title): Path<String>| async move { Redirect::permanent(&format!("/blog/{title}")) })
        )
        .route("/blog/:title", routing::delete(handlers::delete_article))
        .route("/blog/:title/", routing::delete(
            |Path(title): Path<String>| async move { Redirect::permanent(&format!("/blog/{title}")) })
        )

        // host files or images
        .nest_service("/content", {
            let dir = services::ServeDir::new("content");
            routing::get_service(dir)
                .handle_error(|e| async move { eprintln!("{e:?}") })
        })

        // set application state
        .with_state(state);

    // Start hosting
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

mod main {
    // Don't uses logging if `RUST_LOG` is unset
    pub fn init_logging() {
        let rust_log = std::env::var("RUST_LOG");
        if matches!(rust_log.as_deref(), Ok("trace")) {
            tracing_subscriber::fmt::init();
        }
    }

    // Connects to a database using `DATABASE_URL` from .env
    pub async fn connect_database() -> Result<
        super::database::Database,
        Box<dyn std::error::Error>,
    > {
        let database_url = &std::env::var("DATABASE_URL")
            .map_err(|_| "No DATABASE_URL set, check README.md")?;

        let mysql = sqlx::mysql::MySqlPool::connect(database_url).await?;

        let db = super::database::Database::new_migrate(mysql).await;

        Ok(db)
    }
}
