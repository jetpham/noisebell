mod logging;
mod api;
mod monitor;
mod gpio_monitor;
mod web_monitor;
mod endpoint_notifier;
mod config;

use std::{fmt, sync::Arc};
use tokio::sync::RwLock;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

// Shared state types
pub type SharedMonitor = Arc<RwLock<Box<dyn monitor::Monitor>>>;

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
    // Load and validate configuration
    let config = config::Config::from_env()?;
    config.validate()?;
    
    info!("Configuration loaded successfully");
    info!("Monitor type: {}", config.monitor.monitor_type);
    info!("API server: {}:{}", config.api.host, config.api.port);
    if config.web_monitor.enabled {
        info!("Web monitor: port {}", config.web_monitor.port);
    }

    // Initialize logging with config
    logging::init(&config.logging)?;

    // Load endpoint configuration
    info!("Loading endpoint configuration from: {}", config.endpoints.config_file);
    let endpoint_config = endpoint_notifier::EndpointConfig::from_file(&config.endpoints.config_file)?;
    let notifier = Arc::new(endpoint_notifier::EndpointNotifier::new(endpoint_config));

    info!("initializing {} monitor", config.monitor.monitor_type);
    let monitor = monitor::create_monitor(
        &config.monitor.monitor_type,
        config.gpio.pin,
        config.get_debounce_delay(),
        if config.web_monitor.enabled { Some(config.web_monitor.port) } else { None },
    )?;

    let shared_monitor: SharedMonitor = Arc::new(RwLock::new(monitor));

    let monitor_for_task = shared_monitor.clone();
    
    let callback = {
        let notifier = notifier.clone();
        Box::new(move |event: StatusEvent| {
            info!("Circuit state changed to: {:?}", event);
            
            // Notify all configured endpoints
            let notifier = notifier.clone();
            tokio::spawn(async move {
                if let Err(e) = notifier.notify_endpoints(event).await {
                    error!("Failed to notify endpoints: {}", e);
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
        if let Err(e) = api::start_server(&config.api, shared_monitor).await {
            error!("API server error: {}", e);
        }
    });

    info!("Monitor and API server started.");

    let _ = tokio::join!(monitor_handle, api_handle);
    
    Ok(())
}
