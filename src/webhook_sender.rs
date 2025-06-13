use std::time::Duration;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use tracing::{info, warn, error, instrument};

use crate::{StatusEvent, storage::Storage};

const WEBHOOK_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

pub struct WebhookSender {
    client: Client,
}

impl WebhookSender {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(WEBHOOK_TIMEOUT_SECS))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    #[instrument(skip(self), fields(url = %url))]
    pub async fn send_webhook(&self, url: &str, event: StatusEvent) -> Result<()> {
        let payload = json!({
            "event": event.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "noisebell"
        });

        info!("Sending webhook notification");

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Noisebell/1.0")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Webhook sent successfully, status: {}", response.status());
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unable to read response body".to_string());
            error!("Webhook failed with status {}: {}", status, body);
            Err(anyhow::anyhow!("Webhook failed with status {}: {}", status, body))
        }
    }

    #[instrument(skip(self), fields(url = %url))]
    pub async fn send_webhook_with_retry(&self, url: &str, event: StatusEvent) -> Result<()> {
        for attempt in 0..=MAX_RETRIES {
            match self.send_webhook(url, event).await {
                Ok(_) => {
                    if attempt > 0 {
                        info!("Webhook succeeded on attempt {}", attempt + 1);
                    }
                    return Ok(());
                }
                Err(e) if attempt == MAX_RETRIES => {
                    error!("Webhook failed after {} attempts: {}", MAX_RETRIES + 1, e);
                    return Err(e);
                }
                Err(e) => {
                    warn!("Webhook attempt {} failed: {}, retrying in {} seconds", 
                          attempt + 1, e, 2_u64.pow(attempt));
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                }
            }
        }
        unreachable!()
    }
}

#[instrument(skip(storage))]
pub async fn send_webhooks(storage: &Storage, event: StatusEvent) -> Result<()> {
    let webhooks = storage.get_webhooks().await;
    
    if webhooks.is_empty() {
        info!("No webhooks configured, skipping notification");
        return Ok(());
    }

    info!("Sending notifications to {} webhook(s)", webhooks.len());

    let sender = WebhookSender::new();
    let futures: Vec<_> = webhooks
        .iter()
        .map(|webhook| sender.send_webhook_with_retry(&webhook.url, event))
        .collect();

    let results = futures::future::join_all(futures).await;
    
    let mut success_count = 0;
    let mut failed_webhooks = Vec::new();
    
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                let webhook_url = &webhooks[i].url;
                failed_webhooks.push(webhook_url.clone());
                error!("Failed to send webhook to {}: {}", webhook_url, e);
            }
        }
    }

    info!("Webhook sending completed: {} successful, {} failed", success_count, failed_webhooks.len());

    if !failed_webhooks.is_empty() {
        Err(anyhow::anyhow!("{} webhook(s) failed to send: {}", failed_webhooks.len(), failed_webhooks.join(", ")))
    } else {
        Ok(())
    }
} 