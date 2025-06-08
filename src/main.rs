mod gpio;
mod discord;
mod logging;

use std::time::Duration;
use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info};

const DEFAULT_GPIO_PIN: u8 = 17;
const DEFAULT_POLL_INTERVAL_MS: u64 = 100;
const DEFAULT_DEBOUNCE_DELAY_SECS: u64 = 5;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init()?;

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
