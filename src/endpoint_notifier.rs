use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, error, warn};
use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::StatusEvent;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub retry_attempts: u32,
}

pub struct EndpointNotifier {
    config: EndpointConfig,
    client: Client,
}

impl EndpointNotifier {
    pub fn new(config: EndpointConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }

    pub async fn notify_endpoints(&self, event: StatusEvent) -> Result<()> {
        // Convert StatusEvent to lowercase string for API compatibility
        let status = match event {
            StatusEvent::Open => "open",
            StatusEvent::Closed => "closed",
        };
        
        let payload = json!({
            "status": status,
        });

        info!("Notifying endpoint at {} about status: {}", self.config.url, status);

        let mut success = false;
        let mut last_error = None;
        
        for attempt in 1..=self.config.retry_attempts {
            match self.send_request(&payload).await {
                Ok(_) => {
                    info!("Successfully notified endpoint");
                    success = true;
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.retry_attempts {
                        warn!("Attempt {} failed: {}. Retrying...", attempt, last_error.as_ref().unwrap());
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }

        if !success {
            let error_msg = last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error"));
            error!("Failed to notify endpoint after {} attempts: {}", self.config.retry_attempts, error_msg);
            return Err(error_msg);
        }

        info!("Endpoint notification completed successfully");
        Ok(())
    }

    async fn send_request(&self, payload: &serde_json::Value) -> Result<()> {
        let mut request = self.client
            .post(&self.config.url)
            .json(payload);

        // Add API key to Authorization header if provided
        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP request failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        Ok(())
    }
} 