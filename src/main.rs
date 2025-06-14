mod logging;
mod api;
mod storage;
mod monitor;
mod gpio_monitor;
mod web_monitor;
mod webhook_sender;

use std::{fmt, time::Duration, sync::Arc, env};
use tokio::sync::RwLock;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

const DEFAULT_GPIO_PIN: u8 = 17;
const DEFAULT_DEBOUNCE_DELAY_SECS: u64 = 5;
const DEFAULT_API_PORT: u16 = 3000;
const DEFAULT_WEB_MONITOR_PORT: u16 = 8080;

// Shared state types
pub type SharedMonitor = Arc<RwLock<Box<dyn monitor::Monitor>>>;
pub type SharedStorage = Arc<storage::Storage>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEvent {
    Open,
    Closed,
}

impl fmt::Display for StatusEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusEvent::Open => write!(f, "open"),
            StatusEvent::Closed => write!(f, "closed"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init()?;

    info!("initializing storage");
    let storage = storage::Storage::new();
    let shared_storage: SharedStorage = Arc::new(storage);

    // Check environment variable for monitor type
    let monitor_type = env::var("MONITOR_TYPE").unwrap_or_else(|_| "gpio".to_string());
    let web_monitor_port = env::var("WEB_MONITOR_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WEB_MONITOR_PORT);

    info!("initializing {} monitor", monitor_type);
    let monitor = monitor::create_monitor(
        &monitor_type,
        DEFAULT_GPIO_PIN,
        Duration::from_secs(DEFAULT_DEBOUNCE_DELAY_SECS),
        Some(web_monitor_port),
    )?;

    let shared_monitor: SharedMonitor = Arc::new(RwLock::new(monitor));

    let monitor_for_task = shared_monitor.clone();
    
    let callback = {
        let storage = shared_storage.clone();
        Box::new(move |event: StatusEvent| {
            info!("Circuit state changed to: {:?}", event);
            
            // Send webhooks asynchronously
            let storage = storage.clone();
            tokio::spawn(async move {
                if let Err(e) = webhook_sender::send_webhooks(&storage, event).await {
                    error!("Failed to send webhooks: {}", e);
                }
            });
        })
    };

    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = monitor_for_task.write().await.monitor(callback) {
            error!("Monitor error: {}", e);
        }
    });

    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_server(DEFAULT_API_PORT, shared_monitor, shared_storage).await {
            error!("API server error: {}", e);
        }
    });

    if monitor_type == "web" {
        info!("Web monitor UI available at: http://localhost:{}", web_monitor_port);
    }
    info!("Monitor and API server started.");

    let _ = tokio::join!(monitor_handle, api_handle);
    
    Ok(())
}
