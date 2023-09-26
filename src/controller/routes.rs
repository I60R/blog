use axum::{
    extract::Path,
    routing,
    response::Redirect, body::Body,
};
use tower_http::services;

use crate::{
    controller::handlers,
    model::repository::ArticlesRepository
};


pub fn create_routes() -> axum::Router<ArticlesRepository, Body> {
    axum::Router::new()
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

        // admin panel
        .route("/login", routing::get(handlers::admin_login))
        .route("/login/", routing::get(
            || async move { Redirect::permanent("/login")}
        ))
        .route("/admin", routing::get(handlers::admin_panel))
        .route("/admin/", routing::get(
            || async move { Redirect::permanent("/admin")}
        ))

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
}