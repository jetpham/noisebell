use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use std::time::Duration;
use futures::future::join_all;

use crate::gpio::CircuitEvent;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Endpoint {
    url: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct EndpointsConfig {
    endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize)]
struct WebhookPayload {
    event_type: String,
    timestamp: String,
    new_state: String,
}

#[derive(Clone)]
pub struct WebhookNotifier {
    client: Client,
    endpoints: Arc<RwLock<Vec<Endpoint>>>,
    max_retries: u32,
}

impl WebhookNotifier {
    pub fn new(max_retries: u32) -> Result<Self> {
        let endpoints = if let Ok(config) = fs::read_to_string("endpoints.json") {
            let endpoints_config: EndpointsConfig = serde_json::from_str(&config)?;
            endpoints_config.endpoints
        } else {
            Vec::new()
        };
        
        Ok(Self {
            client: Client::new(),
            endpoints: Arc::new(RwLock::new(endpoints)),
            max_retries,
        })
    }

    pub async fn add_endpoint(&self, endpoint: Endpoint) -> Result<()> {
        let mut endpoints = self.endpoints.write().await;
        
        endpoints.retain(|e| e.description != endpoint.description);
        
        endpoints.push(endpoint);
        
        self.save_endpoints().await?;
        
        Ok(())
    }

    async fn save_endpoints(&self) -> Result<()> {
        let endpoints = self.endpoints.read().await;
        let config = EndpointsConfig {
            endpoints: endpoints.clone(),
        };
        fs::write("endpoints.json", serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    async fn send_webhook(&self, endpoint: &Endpoint, payload: &WebhookPayload) -> Result<()> {
        match self.client
            .post(&endpoint.url)
            .json(payload)
            .send()
            .await 
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!(
                        "Successfully sent webhook to {}: {}",
                        endpoint.description,
                        response.status()
                    );
                    Ok(())
                } else {
                    error!(
                        "Webhook request to {} failed with status: {}",
                        endpoint.description,
                        response.status()
                    );
                    Err(anyhow::anyhow!("Failed with status: {}", response.status()))
                }
            }
            Err(e) => {
                error!(
                    "Failed to send webhook to {}: {}",
                    endpoint.description,
                    e
                );
                Err(anyhow::anyhow!("Request failed: {}", e))
            }
        }
    }

    async fn send_webhook_with_retries(&self, endpoint: &Endpoint, payload: &WebhookPayload) {
        let mut attempts = 0;
        
        while attempts < self.max_retries {
            match self.send_webhook(endpoint, payload).await {
                Ok(_) => break,
                Err(e) => {
                    attempts += 1;
                    if attempts < self.max_retries {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    } else {
                        error!("Failed to send webhook to {} after {} attempts: {}", 
                            endpoint.description, self.max_retries, e);
                    }
                }
            }
        }
    }

    pub async fn notify_all(&self, event_type: &str, state: CircuitEvent) {
        let payload = WebhookPayload {
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            new_state: state.to_string(),
        };

        let endpoints = self.endpoints.read().await;
        let webhook_futures: Vec<_> = endpoints.iter()
            .map(|endpoint| {
                info!("Sending webhook to {}: {}", endpoint.description, serde_json::to_string(&payload).unwrap());
                self.send_webhook_with_retries(endpoint, &payload)
            })
            .collect();

        join_all(webhook_futures).await;
    }
} 