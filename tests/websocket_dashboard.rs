use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use axum_can::{
    axum_can::{AppState, router},
    error::Error,
    model::{ByteOrder, CanChannel, CanMessage, CanModel, SignalSpec},
    registry::{Registry, WsMessage},
    source::CanModelSource,
    transport::TransportMode,
};
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

struct StaticSource(CanModel);

#[async_trait]
impl CanModelSource for StaticSource {
    async fn load(&self) -> Result<CanModel, Error> {
        Ok(self.0.clone())
    }
}

fn model() -> CanModel {
    CanModel {
        channels: vec![CanChannel {
            name: "vcan1".into(),
            messages: vec![CanMessage {
                name: "Status".into(),
                can_id: 0x100,
                is_extended: false,
                cycle_time: Duration::from_millis(100),
                signals: vec![SignalSpec {
                    name: "Speed".into(),
                    start_bit: 0,
                    bit_length: 8,
                    byte_order: ByteOrder::LittleEndian,
                    is_signed: false,
                    factor: 1.0,
                    offset: 0.0,
                    minimum: Some(0.0),
                    maximum: Some(255.0),
                    initial_value: 0.0,
                }],
            }],
        }],
    }
}

async fn next_ws_message(
    stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> WsMessage {
    let message = stream.next().await.unwrap().unwrap();
    let Message::Text(text) = message else {
        panic!("expected a text WebSocket message");
    };
    serde_json::from_str(&text).unwrap()
}

#[tokio::test]
async fn clients_receive_initial_state_and_broadcast_updates() {
    let mut registry = Registry::default();
    registry.set_model_source(Box::new(StaticSource(model())));
    registry.set_transport_mode(TransportMode::Print);
    registry.init().await.unwrap();
    let state = Arc::new(AppState {
        registry: Arc::new(registry),
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, router(state)).await;
    });
    let url = format!("ws://{address}/ws");
    let (mut client_a, _) = connect_async(&url).await.unwrap();
    let (mut client_b, _) = connect_async(&url).await.unwrap();

    assert!(matches!(
        next_ws_message(&mut client_a).await,
        WsMessage::Init { .. }
    ));
    assert!(matches!(
        next_ws_message(&mut client_b).await,
        WsMessage::Init { .. }
    ));

    client_a
        .send(Message::Text(
            serde_json::to_string(&WsMessage::ClientUpdate {
                signal: "Speed".into(),
                value: 42.0,
            })
            .unwrap()
            .into(),
        ))
        .await
        .unwrap();

    for client in [&mut client_a, &mut client_b] {
        let message = tokio::time::timeout(Duration::from_secs(1), next_ws_message(client))
            .await
            .unwrap();
        assert!(matches!(
            message,
            WsMessage::StateChanged { value: 42.0, .. }
        ));
    }

    client_a.close(None).await.unwrap();
    client_b.close(None).await.unwrap();
    server.abort();
}
