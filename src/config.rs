use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub gpio: GpioConfig,
    pub web_monitor: WebMonitorConfig,
    pub logging: LoggingConfig,
    pub monitor: MonitorConfig,
    pub endpoints: EndpointConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioConfig {
    pub pin: u8,
    pub debounce_delay_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebMonitorConfig {
    pub port: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_buffered_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub monitor_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub retry_attempts: u32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let gpio = GpioConfig {
            pin: std::env::var("GPIO_PIN")
                .unwrap_or_else(|_| "17".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid GPIO_PIN"))?,
            debounce_delay_secs: std::env::var("DEBOUNCE_DELAY_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid DEBOUNCE_DELAY_SECS"))?,
        };

        let web_monitor = WebMonitorConfig {
            port: std::env::var("WEB_MONITOR_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid WEB_MONITOR_PORT"))?,
            enabled: std::env::var("WEB_MONITOR_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid WEB_MONITOR_ENABLED"))?,
        };

        let logging = LoggingConfig {
            level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            file_path: std::env::var("LOG_FILE_PATH").unwrap_or_else(|_| "logs/noisebell.log".to_string()),
            max_buffered_lines: std::env::var("LOG_MAX_BUFFERED_LINES")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid LOG_MAX_BUFFERED_LINES"))?,
        };

        let monitor = MonitorConfig {
            monitor_type: std::env::var("MONITOR_TYPE").unwrap_or_else(|_| "gpio".to_string()),
        };

        let endpoints = EndpointConfig {
            url: std::env::var("ENDPOINT_URL").unwrap_or_else(|_| "http://localhost:8080/api/status".to_string()),
            api_key: std::env::var("ENDPOINT_API_KEY").ok(),
            timeout_secs: std::env::var("ENDPOINT_TIMEOUT_SECS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid ENDPOINT_TIMEOUT_SECS"))?,
            retry_attempts: std::env::var("ENDPOINT_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid ENDPOINT_RETRY_ATTEMPTS"))?,
        };

        Ok(Config {
            gpio,
            web_monitor,
            logging,
            monitor,
            endpoints,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.gpio.pin > 40 {
            return Err(anyhow::anyhow!("GPIO pin must be between 1-40"));
        }

        if self.gpio.debounce_delay_secs <= 0 {
            return Err(anyhow::anyhow!("Debounce delay must be greater than 0"));
        }

        if !["gpio", "web"].contains(&self.monitor.monitor_type.as_str()) {
            return Err(anyhow::anyhow!("Unknown monitor type: {}", self.monitor.monitor_type));
        }

        Ok(())
    }

    pub fn get_debounce_delay(&self) -> Duration {
        Duration::from_secs(self.gpio.debounce_delay_secs)
    }
} 