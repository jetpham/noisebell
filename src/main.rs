mod gpio;
mod webhook;

use std::time::Duration;
use std::fs;

use anyhow::Result;
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    fs::create_dir_all("logs")?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("noisebell")
        .filename_suffix("log")
        .max_log_files(7)
        .build("logs")?;

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // Only show our logs and hide hyper logs
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("noisebell", LevelFilter::INFO)
        .with_target("hyper", LevelFilter::WARN)
        .with_target("hyper_util", LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default().with_writer(std::io::stdout))
        .with(fmt::Layer::default().with_writer(non_blocking))
        .init();

    info!("Starting noisebell...");

    const DEFAULT_GPIO_PIN: u8 = 17;
    const DEFAULT_WEBHOOK_RETRIES: u32 = 3;

    let gpio_pin = std::env::var("GPIO_PIN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_GPIO_PIN);

    let webhook_retries = std::env::var("WEBHOOK_RETRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_WEBHOOK_RETRIES);

    let webhook_notifier = webhook::WebhookNotifier::new(webhook_retries)?;
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
