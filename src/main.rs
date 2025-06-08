mod gpio;
mod discord;
mod logging;

use std::time::Duration;
use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info};

const DEFAULT_GPIO_PIN: u8 = 17;
const DEFAULT_DEBOUNCE_DELAY_SECS: u64 = 5;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init()?;

    info!("initializing Discord client");
    let discord_client = discord::DiscordClient::new().await?;
    let discord_client = Arc::new(discord_client);

    discord_client.handle_event(discord::SpaceEvent::Initializing).await?;

    info!("initializing gpio monitor");
    let mut gpio_monitor = gpio::GpioMonitor::new(
        DEFAULT_GPIO_PIN,
        Duration::from_secs(DEFAULT_DEBOUNCE_DELAY_SECS)
    )?;

    // Get a handle to the current runtime
    let runtime = tokio::runtime::Handle::current();

    // Set up the callback for state changes
    let callback = move |event: gpio::CircuitEvent| {
        info!("Circuit state changed to: {:?}", event);
        let discord_client = discord_client.clone();
        runtime.spawn(async move {
            let space_event = match event {
                gpio::CircuitEvent::Open => discord::SpaceEvent::Open,
                gpio::CircuitEvent::Closed => discord::SpaceEvent::Closed,
            };
            if let Err(e) = discord_client.handle_event(space_event).await {
                error!("Failed to send Discord message: {}", e);
            }
        });
    };

    if let Err(e) = gpio_monitor.monitor(callback) {
        error!("GPIO monitoring error: {}", e);
        return Err(anyhow::anyhow!("GPIO monitoring failed"));
    }

    info!("GPIO monitoring started. Press Ctrl+C to exit.");
    
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
