mod gpio;
mod discord;

use std::fs;
use std::time::Duration;
use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    info!("creating logs directory");
    fs::create_dir_all("logs")?;

    info!("initializing logging");
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

    info!("initializing Discord client");
    let discord_client = discord::DiscordClient::new().await?;
    let discord_client = Arc::new(discord_client);

    discord_client.send_startup_message().await?;

    const DEFAULT_GPIO_PIN: u8 = 17;

    info!("initializing gpio monitor");
    let mut gpio_monitor = gpio::GpioMonitor::new(DEFAULT_GPIO_PIN, Duration::from_millis(100))?;

    // Send initial state
    discord_client.clone().send_circuit_event(&gpio_monitor.get_current_state()).await?;

    // Set up the callback for state changes
    let callback = move |event: gpio::CircuitEvent| {
        info!("Circuit state changed to: {:?}", event);
        let discord_client = discord_client.clone();
        tokio::spawn(async move {
            if let Err(e) = discord_client.send_circuit_event(&event).await {
                error!("Failed to send Discord message: {}", e);
            }
        });
    };

    // Start monitoring - this will block until an error occurs
    if let Err(e) = gpio_monitor.monitor(callback).await {
        error!("GPIO monitoring error: {}", e);
        return Err(anyhow::anyhow!("GPIO monitoring failed"));
    }

    Ok(())
}
