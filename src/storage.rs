use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub url: String,
    pub created_at: DateTime<Utc>,
}

pub struct Storage {
    webhooks: Arc<RwLock<Vec<WebhookEndpoint>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_webhook(&self, url: &str) -> WebhookEndpoint {
        let mut webhooks = self.webhooks.write().await;
        
        let webhook = WebhookEndpoint {
            url: url.to_string(),
            created_at: Utc::now(),
        };
        
        webhooks.push(webhook.clone());
        webhook
    }

    pub async fn get_webhooks(&self) -> Vec<WebhookEndpoint> {
        let webhooks = self.webhooks.read().await;
        webhooks.clone()
    }

    pub async fn delete_webhook(&self, url: &str) {
        let mut webhooks = self.webhooks.write().await;
        webhooks.retain(|w| w.url != url);
    }
} 