use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub gpio: GpioConfig,
    pub api: ApiConfig,
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
pub struct ApiConfig {
    pub port: u16,
    pub host: String,
    pub max_connections: usize,
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
    pub max_files: usize,
    pub max_file_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub monitor_type: String,
    pub health_check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub config_file: String,
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

        let api = ApiConfig {
            port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid API_PORT"))?,
            host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            max_connections: std::env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid MAX_CONNECTIONS"))?,
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
            max_files: std::env::var("LOG_MAX_FILES")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid LOG_MAX_FILES"))?,
            max_file_size_mb: std::env::var("LOG_MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid LOG_MAX_FILE_SIZE_MB"))?,
        };

        let monitor = MonitorConfig {
            monitor_type: std::env::var("MONITOR_TYPE").unwrap_or_else(|_| "gpio".to_string()),
            health_check_interval_secs: std::env::var("HEALTH_CHECK_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid HEALTH_CHECK_INTERVAL_SECS"))?,
        };

        let endpoints = EndpointConfig {
            config_file: std::env::var("ENDPOINT_CONFIG_FILE").unwrap_or_else(|_| "endpoints.json".to_string()),
        };

        Ok(Config {
            gpio,
            api,
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

        if self.gpio.debounce_delay_secs == 0 {
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

    #[allow(dead_code)]
    pub fn get_health_check_interval(&self) -> Duration {
        Duration::from_secs(self.monitor.health_check_interval_secs)
    }
} 