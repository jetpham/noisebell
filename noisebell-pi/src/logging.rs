use std::fs;
use anyhow::Result;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use crate::config::LoggingConfig;

pub fn init(config: &LoggingConfig) -> Result<()> {
    tracing::info!("creating logs directory");
    let log_dir = std::path::Path::new(&config.file_path).parent().unwrap_or_else(|| std::path::Path::new("logs"));
    fs::create_dir_all(log_dir)?;

    tracing::info!("initializing logging");
    let file_appender = RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::NEVER)
        .filename_prefix("noisebell")
        .filename_suffix("log")
        .build(log_dir)?;

    let (non_blocking, _guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .buffered_lines_limit(config.max_buffered_lines)
        .finish(file_appender);

    // Parse log level from config
    let level_filter = match config.level.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    // Only show our logs and hide hyper logs
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("noisebell", level_filter)
        .with_target("hyper", LevelFilter::WARN)
        .with_target("hyper_util", LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default().with_writer(std::io::stdout))
        .with(fmt::Layer::default().with_writer(non_blocking))
        .init();

    Ok(())
} 