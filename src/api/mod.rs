mod status;
mod health;

use axum::{
    routing::{get},
    Router,
};
use std::net::SocketAddr;
use tracing::info;
use crate::{SharedMonitor, config::ApiConfig};

#[derive(Clone)]
pub struct AppState {
    pub monitor: SharedMonitor,
}

pub fn create_router(
    shared_monitor: SharedMonitor, 
) -> Router {
    let state = AppState {
        monitor: shared_monitor,
    };

    Router::new()
        .route("/status", get(status::get_status))
        .route("/health", get(health::get_health))
        .with_state(state)
}

pub async fn start_server(
    config: &ApiConfig,
    shared_monitor: SharedMonitor, 
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(shared_monitor);
    let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;
    
    info!("Starting API server on {} (max connections: {})", addr, config.max_connections);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service()
    ).await?;
    
    Ok(())
} 