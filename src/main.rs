mod model;
mod view;
mod controller;
mod logging;

use std::net::SocketAddr;

pub const ADDR: &str = "http://127.0.0.1:3000";


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
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}