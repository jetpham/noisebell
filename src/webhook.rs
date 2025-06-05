use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{error, info};

use crate::gpio::CircuitEvent;

#[derive(Debug, Deserialize, Clone)]
struct Endpoint {
    url: String,
    description: String,
}

#[derive(Debug, Deserialize)]
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
    endpoints: Vec<Endpoint>,
}

impl WebhookNotifier {
    pub fn new() -> Result<Self> {
        let config = fs::read_to_string("endpoints.json")?;
        let endpoints_config: EndpointsConfig = serde_json::from_str(&config)?;
        
        Ok(Self {
            client: Client::new(),
            endpoints: endpoints_config.endpoints,
        })
    }

    pub async fn notify_all(&self, event_type: &str, state: CircuitEvent) {
        let payload = WebhookPayload {
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            new_state: state.to_string(),
        };

        for endpoint in &self.endpoints {
            info!("Sending webhook to {}: {}", endpoint.description, serde_json::to_string(&payload).unwrap());
            match self.client
                .post(&endpoint.url)
                .json(&payload)
                .send()
                .await 
            {
                Ok(response) => {
                    info!(
                        "Successfully sent webhook to {}: {}",
                        endpoint.description,
                        response.status()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to send webhook to {}: {}",
                        endpoint.description,
                        e
                    );
                }
            }
        }
    }
} 