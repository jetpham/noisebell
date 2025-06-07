mod discord;
mod gpio;

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

struct ShutdownGuard {
    discord_client: Arc<discord::DiscordClient>,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        info!("Shutdown guard triggered");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            if let Err(e) = self.discord_client.send_shutdown_message().await {
                error!("Failed to send shutdown message: {}", e);
            }
        });
    }
}

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

    // Create shutdown guard that will send message on any exit
    let _guard = ShutdownGuard {
        discord_client: Arc::clone(&discord_client),
    };

    discord_client.send_startup_message().await?;

    const DEFAULT_GPIO_PIN: u8 = 17;

    info!("initializing gpio monitor");
    let mut gpio_monitor = gpio::GpioMonitor::new(DEFAULT_GPIO_PIN, Duration::from_millis(100))?;

    // Send initial state
    discord_client
        .clone()
        .send_circuit_event(&gpio_monitor.get_current_state())
        .await?;

    let debounce_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(Mutex::new(None));
    let debounce_duration = Duration::from_secs(5);

    let callback = move |event: gpio::CircuitEvent| {
        info!("Circuit state changed to: {:?}", event);
        let discord_client = discord_client.clone();
        let debounce_handle = debounce_handle.clone();

        tokio::spawn(async move {
            // Cancel any existing debounce timer
            {
                let mut handle_guard = debounce_handle.lock().await;
                if let Some(handle) = handle_guard.take() {
                    handle.abort();
                }
            }

            // Start new debounce timer
            let new_handle = tokio::spawn({
                let discord_client = discord_client.clone();
                async move {
                    tokio::time::sleep(debounce_duration).await;
                    info!(
                        "Debounce period elapsed, sending Discord message for: {:?}",
                        event
                    );
                    if let Err(e) = discord_client.send_circuit_event(&event).await {
                        error!("Failed to send Discord message: {}", e);
                    }
                }
            });

            // Store the new handle
            {
                let mut handle_guard = debounce_handle.lock().await;
                *handle_guard = Some(new_handle);
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
