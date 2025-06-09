mod webhooks;
mod status;
mod health;

use axum::{
    routing::{get},
    Router,
};
use std::net::SocketAddr;
use tracing::info;
use crate::{SharedMonitor, SharedStorage};

#[derive(Clone)]
pub struct AppState {
    pub monitor: SharedMonitor,
    pub storage: SharedStorage,
}

pub fn create_router(shared_monitor: SharedMonitor, shared_storage: SharedStorage) -> Router {
    let state = AppState {
        monitor: shared_monitor,
        storage: shared_storage,
    };

    Router::new()
        .route("/webhooks", get(webhooks::get_webhook)
            .post(webhooks::post_webhook))
        .route("/status", get(status::get_status))
        .route("/health", get(health::get_health))
        .with_state(state)
}

pub async fn start_server(
    port: u16, 
    shared_monitor: SharedMonitor, 
    shared_storage: SharedStorage,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(shared_monitor, shared_storage);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    info!("Starting API server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service()
    ).await?;
    
    Ok(())
} 