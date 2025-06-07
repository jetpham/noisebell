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

const DEFAULT_GPIO_PIN: u8 = 17;
const DEFAULT_POLL_INTERVAL_MS: u64 = 100;
const DEFAULT_DEBOUNCE_DELAY_SECS: u64 = 5;
const LOG_DIR: &str = "logs";
const LOG_PREFIX: &str = "noisebell";
const LOG_SUFFIX: &str = "log";
const MAX_LOG_FILES: usize = 7;

#[tokio::main]
async fn main() -> Result<()> {
    info!("creating logs directory");
    fs::create_dir_all(LOG_DIR)?;

    info!("initializing logging");
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_PREFIX)
        .filename_suffix(LOG_SUFFIX)
        .max_log_files(MAX_LOG_FILES)
        .build(LOG_DIR)?;

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

    info!("initializing gpio monitor");
    let mut gpio_monitor = gpio::GpioMonitor::new(
        DEFAULT_GPIO_PIN,
        Duration::from_millis(DEFAULT_POLL_INTERVAL_MS),
        Duration::from_secs(DEFAULT_DEBOUNCE_DELAY_SECS)
    )?;

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
