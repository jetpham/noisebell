use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, error, warn};
use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::StatusEvent;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub url: String,
    pub name: Option<String>,
    pub timeout_secs: Option<u64>,
    pub retry_attempts: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub endpoints: Vec<Endpoint>,
}

impl EndpointConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read endpoint config file: {}", e))?;
        
        let config: EndpointConfig = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse endpoint config JSON: {}", e))?;
        
        Ok(config)
    }
}

pub struct EndpointNotifier {
    config: EndpointConfig,
    client: Client,
}

impl EndpointNotifier {
    pub fn new(config: EndpointConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }

    pub async fn notify_endpoints(&self, event: StatusEvent) -> Result<()> {
        let payload = json!({
            "event": event.to_string(),
        });

        info!("Notifying {} endpoints about event: {:?}", self.config.endpoints.len(), event);

        let mut tasks = Vec::new();
        
        for endpoint in &self.config.endpoints {
            let task = self.notify_single_endpoint(endpoint, payload.clone());
            tasks.push(task);
        }

        // Wait for all notifications to complete
        let results = futures::future::join_all(tasks).await;
        
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!("Endpoint notification failed: {}", e);
                    failure_count += 1;
                }
            }
        }

        info!("Endpoint notifications completed: {} success, {} failures", success_count, failure_count);
        
        Ok(())
    }

    async fn notify_single_endpoint(&self, endpoint: &Endpoint, payload: serde_json::Value) -> Result<()> {
        let timeout = Duration::from_secs(endpoint.timeout_secs.unwrap_or(30));
        let retry_attempts = endpoint.retry_attempts.unwrap_or(3);
        
        let endpoint_name = endpoint.name.as_deref().unwrap_or(&endpoint.url);
        
        for attempt in 1..=retry_attempts {
            match self.send_request(endpoint, &payload, timeout).await {
                Ok(_) => {
                    info!("Successfully notified endpoint '{}'", endpoint_name);
                    return Ok(());
                }
                Err(e) => {
                    if attempt == retry_attempts {
                        error!("Failed to notify endpoint '{}' after {} attempts: {}", endpoint_name, retry_attempts, e);
                        return Err(e);
                    } else {
                        warn!("Attempt {} failed for endpoint '{}': {}. Retrying...", attempt, endpoint_name, e);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
        
        unreachable!()
    }

    async fn send_request(&self, endpoint: &Endpoint, payload: &serde_json::Value, timeout: Duration) -> Result<()> {
        let response = self.client
            .post(&endpoint.url)
            .json(payload)
            .timeout(timeout)
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