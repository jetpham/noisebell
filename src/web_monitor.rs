use std::sync::Arc;
use anyhow::Result;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{Html, IntoResponse},
    routing::{get},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast, Mutex};
use tracing::{info, error, warn};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tower_http::services::ServeDir;

use crate::{StatusEvent, monitor::Monitor};

#[derive(Clone)]
pub struct WebMonitor {
    port: u16,
    current_state: Arc<RwLock<StatusEvent>>,
    state_sender: broadcast::Sender<StatusEvent>,
    callback: Arc<Mutex<Option<Box<dyn FnMut(StatusEvent) + Send + 'static>>>>,
}

#[derive(Clone)]
struct AppState {
    current_state: Arc<RwLock<StatusEvent>>,
    state_sender: broadcast::Sender<StatusEvent>,
    callback: Arc<Mutex<Option<Box<dyn FnMut(StatusEvent) + Send + 'static>>>>,
}

#[derive(Serialize, Deserialize)]
struct StateChangeMessage {
    event: String,
    state: String,
}

impl WebMonitor {
    pub fn new(port: u16) -> Result<Self> {
        let (state_sender, _) = broadcast::channel(100);
        
        Ok(Self {
            port,
            current_state: Arc::new(RwLock::new(StatusEvent::Closed)), // Default to closed
            state_sender,
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
        let mut state_receiver = state.state_sender.subscribe();

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
        let receiver_task = tokio::spawn(async move {
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

                                // Broadcast to other clients
                                if let Err(e) = state_for_receiver.state_sender.send(new_state) {
                                    warn!("Failed to broadcast state change: {}", e);
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
        });

        // Handle outgoing state updates to client
        let sender_task = tokio::spawn(async move {
            while let Ok(new_state) = state_receiver.recv().await {
                let message = StateChangeMessage {
                    event: "state_update".to_string(),
                    state: new_state.to_string(),
                };
                
                if let Ok(msg) = serde_json::to_string(&message) {
                    if let Err(e) = sender.send(Message::Text(msg.into())).await {
                        error!("Failed to send state update: {}", e);
                        break;
                    }
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = receiver_task => {},
            _ = sender_task => {},
        }
    }

    async fn start_server(&self) -> Result<()> {
        let app_state = AppState {
            current_state: self.current_state.clone(),
            state_sender: self.state_sender.clone(),
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
        // Store the callback
        let callback_arc = self.callback.clone();
        let rt = tokio::runtime::Handle::current();
        rt.spawn(async move {
            let mut guard = callback_arc.lock().await;
            *guard = Some(callback);
        });

        // Start the web server
        let server = self.clone();
        rt.spawn(async move {
            if let Err(e) = server.start_server().await {
                error!("Web monitor server error: {}", e);
            }
        });

        Ok(())
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