use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub gpio: GpioConfig,
    pub web_monitor: WebMonitorConfig,
    pub logging: LoggingConfig,
    pub monitor: MonitorConfig,
    pub endpoint: EndpointConfig,
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
    pub fn from_file_and_env() -> Result<Self> {
        // Load TOML configuration from file
        let config_content = fs::read_to_string("config.toml")
            .map_err(|e| anyhow::anyhow!("Failed to read config.toml: {}", e))?;
        
        let mut config: Config = toml::from_str(&config_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config.toml: {}", e))?;
        
        // Load API key from environment variable (for security)
        config.endpoint.api_key = std::env::var("ENDPOINT_API_KEY").ok();
        
        Ok(config)
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