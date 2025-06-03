mod gpio;

use std::time::Duration;

use anyhow::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize GPIO monitor
    const DEFAULT_GPIO_PIN: u8 = 17;
    let gpio_pin = std::env::var("GPIO_PIN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_GPIO_PIN);

    let mut gpio_monitor = gpio::GpioMonitor::new(gpio_pin, Duration::from_millis(100))?;

    // Simple callback function that just logs the event
    let callback = |event: gpio::CircuitEvent| {
        info!("Circuit state changed: {:?}", event);
    };

    // Start GPIO monitoring
    if let Err(e) = gpio_monitor.monitor(callback).await {
        error!("GPIO monitoring error: {}", e);
    }

    Ok(())
}
