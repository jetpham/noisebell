mod gpio;
mod webhook;

use std::time::Duration;

use anyhow::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    info!("Starting noisebell...");

    tracing_subscriber::fmt::init();

    const DEFAULT_GPIO_PIN: u8 = 17;
    let gpio_pin = std::env::var("GPIO_PIN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_GPIO_PIN);

    let webhook_notifier = webhook::WebhookNotifier::new()?;
    let mut gpio_monitor = gpio::GpioMonitor::new(gpio_pin, Duration::from_millis(100))?;

    let callback = move |event: gpio::CircuitEvent| {
        info!("Circuit state changed: {:?}", event);

        let notifier = webhook_notifier.clone();

        tokio::spawn(async move {
            notifier.notify_all("circuit_state_change", event).await;
        });
    };

    info!("starting gpio_monitor");

    if let Err(e) = gpio_monitor.monitor(callback).await {
        error!("GPIO monitoring error: {}", e);
    }

    Ok(())
}
