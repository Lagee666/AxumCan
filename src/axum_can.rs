use std::{collections::HashMap, sync::Arc};

use crate::registry::{Registry, WsMessage};
use axum::{
    Router,
    extract::{
        State,
        ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
struct CanSignal {
    vcan1: HashMap<String, HashMap<String, u64>>,
    vcan2: HashMap<String, HashMap<String, u64>>,
    vcan3: HashMap<String, HashMap<String, u64>>,
    vcan4: HashMap<String, HashMap<String, u64>>,
    vcan5: HashMap<String, HashMap<String, u64>>,
    vcan6: HashMap<String, HashMap<String, u64>>,
}

use tower_http::services::ServeDir;

pub struct AppState {
    pub registry: Arc<Registry>,
}

pub async fn serve(listener: TcpListener, state: Arc<AppState>) {
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("dashboard/dist"))
        .with_state(state);

    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.registry.broadcast_tx.subscribe();

    // Send initial state
    let init_msg = WsMessage::Init {
        signals: Box::new(state.registry.initial_signals.clone()),
    };
    if let Ok(json) = serde_json::to_string(&init_msg) {
        if let Err(e) = sender.send(AxumMessage::Text(json.into())).await {
            error!("Failed to send init message: {}", e);
            return;
        }
    }

    let registry = state.registry.clone();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(AxumMessage::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(AxumMessage::Text(text))) = receiver.next().await {
            match serde_json::from_str::<WsMessage>(&text) {
                Ok(msg) => match msg {
                    WsMessage::ClientUpdate { signal, value } => {
                        info!("Received update from client: {} = {}", signal, value);
                        registry.update(signal, value, false);
                    }
                    WsMessage::SetArbitration {
                        signal,
                        allow_backend,
                    } => {
                        info!(
                            "Set arbitration for {}: allow_backend = {}",
                            signal, allow_backend
                        );
                        registry.set_arbitration(signal, allow_backend);
                    }
                    _ => {}
                },
                Err(e) => {
                    error!("Failed to parse WebSocket message: {}. Text: {}", e, text);
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
