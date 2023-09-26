mod model;
mod view;
mod controller;
mod logging;

use std::net::SocketAddr;

pub const ADDR: &str = once_cell::Lazy::new(|| {
    std::env::var("BLOG_ADDR")
        .or(std::env::args().last())
        .expect("No BLOG_ADDR or <ADDR> arguments provided")
});


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    logging::init_logging();

    let db = model::connect_database().await?;

    // Initialize application state
    let state = model::repository::ArticlesRepository::new(db);

    // Set routes
    let app: axum::Router = controller::routes::create_routes()
        // set application state
        .with_state(state);

    // Start hosting
    let addr = SocketAddr::from(*ADDR);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}