mod gpio;
mod webhook;

use std::time::Duration;
use std::fs;
use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
};
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::filter::LevelFilter;

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

    const DEFAULT_GPIO_PIN: u8 = 17;
    const DEFAULT_WEBHOOK_RETRIES: u32 = 3;
    const DEFAULT_SERVER_PORT: u16 = 8080;

    info!("initializing webhook notifier");
    let webhook_notifier = Arc::new(webhook::WebhookNotifier::new(DEFAULT_WEBHOOK_RETRIES)?);

    info!("initializing gpio monitor");
    let mut gpio_monitor = gpio::GpioMonitor::new(DEFAULT_GPIO_PIN, Duration::from_millis(100))?;

    let app = Router::new()
        .route("/endpoints", post(add_endpoint))
        .with_state(webhook_notifier.clone());

    let server_addr = format!("127.0.0.1:{}", DEFAULT_SERVER_PORT);
    info!("Starting API server on http://{}", server_addr);
    
    let server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&server_addr).await?;
        axum::serve(listener, app.into_make_service())
            .await?;
        Ok::<_, anyhow::Error>(())
    });

    let callback = move |event: gpio::CircuitEvent| {
        info!("Circuit state changed: {:?}", event);

        let notifier = webhook_notifier.clone();

        tokio::spawn(async move {
            notifier.notify_all("circuit_state_change", event).await;
        });
    };

    info!("starting GPIO monitor");

    if let Err(e) = gpio_monitor.monitor(callback).await {
        error!("GPIO monitoring error: {}", e);
    }

    // Wait for the server to complete (it shouldn't unless there's an error)
    if let Err(e) = server.await? {
        error!("Server error: {}", e);
    }

    Ok(())
}

async fn add_endpoint(
    State(notifier): State<Arc<webhook::WebhookNotifier>>,
    Json(endpoint): Json<webhook::Endpoint>,
) -> Result<(), axum::http::StatusCode> {
    notifier.add_endpoint(endpoint)
        .await
        .map_err(|e| {
            error!("Failed to add endpoint: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
}
