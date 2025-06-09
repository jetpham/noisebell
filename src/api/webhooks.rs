use axum::{
    response::IntoResponse,
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, instrument};
use crate::api::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookRequest {
    url: String,
}

#[instrument(skip(state))]
pub async fn get_webhook(
    State(state): State<AppState>
) -> impl IntoResponse {
    info!("Received webhook list request");
    let webhooks = state.storage.get_webhooks().await;
    info!("Found {} webhooks", webhooks.len());
    Json(json!({
        "status": "success",
        "data": {
            "webhooks": webhooks
        }
    })).into_response()
}

#[instrument(skip(state), fields(url = %payload.url))]
pub async fn post_webhook(
    State(state): State<AppState>,
    Json(payload): Json<WebhookRequest>,
) -> impl IntoResponse {
    info!("Received webhook registration request");
    let webhook = state.storage.add_webhook(&payload.url).await;
    info!("Successfully registered webhook");
    Json(json!({
        "status": "success",
        "message": "Webhook added successfully",
        "data": webhook
    })).into_response()
} 