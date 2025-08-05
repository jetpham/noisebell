use std::sync::Arc;
use anyhow::Result;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{Html, IntoResponse},
    routing::{get},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Mutex};
use tracing::{info, error};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tower_http::services::ServeDir;

use crate::{StatusEvent, monitor::Monitor};

#[derive(Clone)]
pub struct WebMonitor {
    port: u16,
    current_state: Arc<RwLock<StatusEvent>>,
    callback: Arc<Mutex<Option<Box<dyn FnMut(StatusEvent) + Send + 'static>>>>,
}

#[derive(Clone)]
struct AppState {
    current_state: Arc<RwLock<StatusEvent>>,
    callback: Arc<Mutex<Option<Box<dyn FnMut(StatusEvent) + Send + 'static>>>>,
}

#[derive(Serialize, Deserialize)]
struct StateChangeMessage {
    event: String,
    state: String,
}

impl WebMonitor {
    pub fn new(port: u16) -> Result<Self> {
        Ok(Self {
            port,
            current_state: Arc::new(RwLock::new(StatusEvent::Closed)), // Default to closed
            callback: Arc::new(Mutex::new(None)),
        })
    }

    async fn serve_html() -> impl IntoResponse {
        Html(include_str!("../static/monitor.html"))
    }

    async fn websocket_handler(
        ws: WebSocketUpgrade,
        State(state): State<AppState>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Self::handle_websocket(socket, state))
    }

    async fn handle_websocket(socket: WebSocket, state: AppState) {
        let (mut sender, mut receiver) = socket.split();

        // Send current state immediately
        let current_state = *state.current_state.read().await;
        let initial_message = StateChangeMessage {
            event: "state_update".to_string(),
            state: current_state.to_string(),
        };
        
        if let Ok(msg) = serde_json::to_string(&initial_message) {
            if let Err(e) = sender.send(Message::Text(msg.into())).await {
                error!("Failed to send initial state: {}", e);
                return;
            }
        }

        // Handle incoming messages from client
        let state_for_receiver = state.clone();
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let text_str = text.to_string();
                    if let Ok(state_msg) = serde_json::from_str::<StateChangeMessage>(&text_str) {
                        if state_msg.event == "state_change" {
                            let new_state = match state_msg.state.as_str() {
                                "open" => StatusEvent::Open,
                                "closed" => StatusEvent::Closed,
                                _ => continue,
                            };

                            // Update current state
                            {
                                let mut current = state_for_receiver.current_state.write().await;
                                *current = new_state;
                            }

                            // Trigger callback
                            {
                                let mut callback_guard = state_for_receiver.callback.lock().await;
                                if let Some(ref mut callback) = callback_guard.as_mut() {
                                    callback(new_state);
                                }
                            }

                            info!("Web monitor state changed to: {:?}", new_state);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    async fn start_server(&self) -> Result<()> {
        let app_state = AppState {
            current_state: self.current_state.clone(),
            callback: self.callback.clone(),
        };

        let app = Router::new()
            .route("/", get(Self::serve_html))
            .route("/ws", get(Self::websocket_handler))
            .nest_service("/media", ServeDir::new("media"))
            .with_state(app_state);

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting web monitor server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

impl Monitor for WebMonitor {
    fn monitor(&mut self, callback: Box<dyn FnMut(StatusEvent) + Send>) -> Result<()> {
        // Store the callback synchronously to ensure it's available immediately
        let callback_arc = self.callback.clone();
        let rt = tokio::runtime::Handle::current();
        tokio::task::block_in_place(|| {
            rt.block_on(async {
                let mut guard = callback_arc.lock().await;
                *guard = Some(callback);
            });
        });

        // Run the web server in a blocking task to avoid runtime conflicts
        let server = self.clone();
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(server.start_server()) {
                error!("Web monitor server error: {}", e);
            }
        });

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    fn get_current_state(&self) -> StatusEvent {
        // This is a synchronous function, but we need to read async state
        // We'll use a blocking operation here similar to how GPIO reads work
        let rt = tokio::runtime::Handle::current();
        tokio::task::block_in_place(|| {
            rt.block_on(async {
                *self.current_state.read().await
            })
        })
    }
}