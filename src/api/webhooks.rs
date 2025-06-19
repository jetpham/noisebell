use axum::{
    response::IntoResponse,
    extract::{State, Request},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, instrument};
use crate::api::AppState;
use crate::storage::WebhookError;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookRequest {
    url: String,
}

#[instrument(skip(state))]
pub async fn get_webhook(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    let uri = request.uri().clone();
    info!("Received webhook list request at {}", uri);
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
    match state.storage.add_webhook(&payload.url).await {
        Ok(webhook) => {
            info!("Successfully registered webhook: {}", payload.url);
            Json(json!({
                "status": "success",
                "message": "Webhook added successfully",
                "data": webhook
            })).into_response()
        },
        Err(WebhookError::DuplicateUrl) => {
            info!("Failed to register webhook: duplicate URL - {}", payload.url);
            (StatusCode::CONFLICT, Json(json!({
                "status": "error",
                "message": format!("Webhook endpoint already exists: {}", payload.url)
            }))).into_response()
        },
        Err(WebhookError::InvalidUrl) => {
            info!("Failed to register webhook: invalid URL - {}", payload.url);
            (StatusCode::BAD_REQUEST, Json(json!({
                "status": "error",
                "message": format!("Invalid URL format: {}", payload.url)
            }))).into_response()
        }
    }
} 