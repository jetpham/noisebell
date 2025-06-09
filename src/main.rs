mod logging;
mod api;
mod storage;
mod monitor;

use std::{fmt, time::Duration, sync::Arc};
use tokio::sync::RwLock;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

const DEFAULT_GPIO_PIN: u8 = 17;
const DEFAULT_DEBOUNCE_DELAY_SECS: u64 = 5;
const DEFAULT_API_PORT: u16 = 3000;

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

    info!("initializing monitor");
    let monitor = monitor::create_monitor(
        DEFAULT_GPIO_PIN,
        Duration::from_secs(DEFAULT_DEBOUNCE_DELAY_SECS),
    )?;

    let shared_monitor: SharedMonitor = Arc::new(RwLock::new(monitor));

    let monitor_for_task = shared_monitor.clone();
    
    let callback = Box::new(move |event: StatusEvent| {
        info!("Circuit state changed to: {:?}", event);
        
    });

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

    info!("Monitor and API server started.");

    let _ = tokio::join!(monitor_handle, api_handle);
    
    Ok(())
}
