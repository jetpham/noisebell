use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::Result;
use dotenvy::dotenv;
use tracing::info;

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

impl GpioConfig {
    pub fn from_env() -> Result<Self> {
        let pin = std::env::var("NOISEBELL_GPIO_PIN")
            .unwrap_or_else(|_| "17".to_string())
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("Invalid GPIO pin number"))?;
            
        let debounce_delay_secs = std::env::var("NOISEBELL_GPIO_DEBOUNCE_DELAY_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid debounce delay"))?;
            
        Ok(Self {
            pin,
            debounce_delay_secs,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebMonitorConfig {
    pub port: u16,
    pub enabled: bool,
}

impl WebMonitorConfig {
    pub fn from_env() -> Result<Self> {
        let port = std::env::var("NOISEBELL_WEB_MONITOR_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| anyhow::anyhow!("Invalid web monitor port"))?;
            
        let enabled = std::env::var("NOISEBELL_WEB_MONITOR_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .map_err(|_| anyhow::anyhow!("Invalid web monitor enabled flag"))?;
            
        Ok(Self {
            port,
            enabled,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_buffered_lines: usize,
}

impl LoggingConfig {
    pub fn from_env() -> Result<Self> {
        let level = std::env::var("NOISEBELL_LOGGING_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
            
        let file_path = std::env::var("NOISEBELL_LOGGING_FILE_PATH")
            .unwrap_or_else(|_| "logs/noisebell.log".to_string());
            
        let max_buffered_lines = std::env::var("NOISEBELL_LOGGING_MAX_BUFFERED_LINES")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("Invalid max buffered lines"))?;
            
        Ok(Self {
            level,
            file_path,
            max_buffered_lines,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub monitor_type: String,
}

impl MonitorConfig {
    pub fn from_env() -> Result<Self> {
        let monitor_type = std::env::var("NOISEBELL_MONITOR_TYPE")
            .unwrap_or_else(|_| "web".to_string());
            
        Ok(Self {
            monitor_type,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub retry_attempts: u32,
}

impl EndpointConfig {
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("NOISEBELL_ENDPOINT_URL")
            .unwrap_or_else(|_| "https://noisebell.jetpham.com/api/status".to_string());
            
        let api_key = std::env::var("ENDPOINT_API_KEY").ok();
        
        let timeout_secs = std::env::var("NOISEBELL_ENDPOINT_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid endpoint timeout"))?;
            
        let retry_attempts = std::env::var("NOISEBELL_ENDPOINT_RETRY_ATTEMPTS")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid retry attempts"))?;
            
        Ok(Self {
            url,
            api_key,
            timeout_secs,
            retry_attempts,
        })
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Self::load_env()?;
        
        let config = Config {
            gpio: GpioConfig::from_env()?,
            web_monitor: WebMonitorConfig::from_env()?,
            logging: LoggingConfig::from_env()?,
            monitor: MonitorConfig::from_env()?,
            endpoint: EndpointConfig::from_env()?,
        };
        
        Ok(config)
    }

    pub fn load_env() -> Result<()> {
        // Try to load from .env file, but don't fail if it doesn't exist
        match dotenv() {
            Ok(_) => {
                info!("Successfully loaded environment variables from .env file");
                Ok(())
            }
            Err(dotenvy::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
                info!("No .env file found, using system environment variables");
                Ok(())
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to load .env file: {}", e))
            }
        }
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