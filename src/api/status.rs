use axum::{
    response::IntoResponse,
    extract::State,
    Json,
};
use serde_json::json;
use tracing::{info, instrument};
use crate::api::AppState;

#[instrument(skip(state))]
pub async fn get_status(
    State(state): State<AppState>
) -> impl IntoResponse {
    info!("Received status request");
    let status = state.monitor.read().await.get_current_state();
    info!("Current state: {}", status);
    
    Json(json!({
        "status": "success",
        "data": {
            "state": status.to_string()
        }
    })).into_response()
} 