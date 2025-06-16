mod api;
mod error;
mod models;
mod server;

use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use eyre::Result;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_url = format!("{}:{}", host, port);

    // Initialize the server state
    let server_state = server::ServerState::new().await?;
    
    tracing::info!("Starting Amazon Q Developer API server at http://{}", server_url);

    // Start the HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(actix_web::web::Data::new(server_state.clone()))
            .configure(api::configure_routes)
    })
    .bind(server_url)?
    .run()
    .await?;

    Ok(())
}
