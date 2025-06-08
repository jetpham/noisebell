use std::fs;
use anyhow::Result;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

const LOG_DIR: &str = "logs";
const LOG_PREFIX: &str = "noisebell";
const LOG_SUFFIX: &str = "log";
const MAX_LOG_FILES: usize = 7;

pub fn init() -> Result<()> {
    tracing::info!("creating logs directory");
    fs::create_dir_all(LOG_DIR)?;

    tracing::info!("initializing logging");
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

    Ok(())
} 