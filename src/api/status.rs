use axum::{
    response::IntoResponse,
    extract::{State, Request},
    Json,
};
use serde_json::json;
use tracing::{info, instrument};
use crate::api::AppState;

#[instrument(skip(state))]
pub async fn get_status(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    let uri = request.uri().clone();
    let status = state.monitor.read().await.get_current_state();
    info!("Received status request at {} - Current state: {}", uri, status);
    
    Json(json!({
        "status": "success",
        "data": {
            "state": status.to_string()
        }
    })).into_response()
} 