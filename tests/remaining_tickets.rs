use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use axum_can::{
    error::Error,
    model::{ByteOrder, CanChannel, CanFrame, CanMessage, CanModel, SignalSpec},
    registry::Registry,
    source::CanModelSource,
    transport::{CanTransport, TransportFactory, TransportMode},
};

fn model_with_channels(names: &[&str]) -> CanModel {
    CanModel {
        channels: names
            .iter()
            .map(|name| CanChannel {
                name: (*name).into(),
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
                        initial_value: 1.0,
                    }],
                }],
            })
            .collect(),
    }
}

struct StaticSource(CanModel);

#[async_trait]
impl CanModelSource for StaticSource {
    async fn load(&self) -> Result<CanModel, Error> {
        Ok(self.0.clone())
    }
}

#[derive(Default)]
struct TransportState {
    created_channels: Vec<String>,
    starts: usize,
    sends: usize,
    stops: usize,
    fail_send: bool,
}

struct RecordingTransport {
    state: Arc<Mutex<TransportState>>,
}

#[async_trait]
impl CanTransport for RecordingTransport {
    async fn start(&self) -> Result<(), Error> {
        self.state.lock().unwrap().starts += 1;
        Ok(())
    }

    async fn send(&self, _frame: &CanFrame) -> Result<(), Error> {
        let mut state = self.state.lock().unwrap();
        state.sends += 1;
        if state.fail_send {
            Err(Error::Transport("test send failure".into()))
        } else {
            Ok(())
        }
    }

    async fn stop(&self) -> Result<(), Error> {
        self.state.lock().unwrap().stops += 1;
        Ok(())
    }
}

struct RecordingFactory {
    state: Arc<Mutex<TransportState>>,
}

impl TransportFactory for RecordingFactory {
    fn create(&self, channel: &str) -> Result<Arc<dyn CanTransport>, Error> {
        self.state
            .lock()
            .unwrap()
            .created_channels
            .push(channel.into());
        Ok(Arc::new(RecordingTransport {
            state: self.state.clone(),
        }))
    }
}

#[tokio::test(start_paused = true)]
async fn custom_factory_receives_each_channel_and_lifecycle_runs() {
    let state = Arc::new(Mutex::new(TransportState::default()));
    let mut registry = Registry::default();
    registry.set_model_source(Box::new(StaticSource(model_with_channels(&[
        "vcan0", "vcan1",
    ]))));
    registry.set_transport_mode(TransportMode::Custom(Arc::new(RecordingFactory {
        state: state.clone(),
    })));

    registry.init().await.unwrap();
    {
        let state = state.lock().unwrap();
        assert_eq!(state.created_channels, ["vcan0", "vcan1"]);
        assert_eq!(state.starts, 2);
    }
    registry.shutdown().await;
    assert_eq!(state.lock().unwrap().stops, 2);
}

#[tokio::test(start_paused = true)]
async fn send_failures_do_not_stop_periodic_sending() {
    let state = Arc::new(Mutex::new(TransportState {
        fail_send: true,
        ..Default::default()
    }));
    let mut registry = Registry::default();
    registry.set_model_source(Box::new(StaticSource(model_with_channels(&["vcan0"]))));
    registry.set_transport_mode(TransportMode::Custom(Arc::new(RecordingFactory {
        state: state.clone(),
    })));

    registry.init().await.unwrap();
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;
    assert!(state.lock().unwrap().sends >= 2);
    registry.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn repeated_initialization_does_not_duplicate_senders() {
    let state = Arc::new(Mutex::new(TransportState::default()));
    let mut registry = Registry::default();
    registry.set_model_source(Box::new(StaticSource(model_with_channels(&["vcan0"]))));
    registry.set_transport_mode(TransportMode::Custom(Arc::new(RecordingFactory {
        state: state.clone(),
    })));

    registry.init().await.unwrap();
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;
    registry.init().await.unwrap();
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    assert_eq!(state.lock().unwrap().sends, 2);
    registry.shutdown().await;
}

#[tokio::test]
async fn missing_socketcan_interface_fails_initialization() {
    let mut registry = Registry::default();
    registry.set_model_source(Box::new(StaticSource(model_with_channels(&[
        "interface-that-does-not-exist",
    ]))));
    registry.set_transport_mode(TransportMode::SocketCan);

    assert!(registry.init().await.is_err());
}

#[tokio::test]
async fn logical_signal_updates_are_broadcast_to_multiple_clients() {
    let state = Arc::new(Mutex::new(TransportState::default()));
    let mut registry = Registry::default();
    registry.set_model_source(Box::new(StaticSource(model_with_channels(&[
        "vcan0", "vcan1",
    ]))));
    registry.set_transport_mode(TransportMode::Custom(Arc::new(RecordingFactory { state })));
    registry.init().await.unwrap();

    let mut client_a = registry.broadcast_tx.subscribe();
    let mut client_b = registry.broadcast_tx.subscribe();
    assert!(registry.update("Speed".into(), 42.0, false));

    for client in [&mut client_a, &mut client_b] {
        let message = client.recv().await.unwrap();
        assert!(matches!(
            message,
            axum_can::registry::WsMessage::StateChanged { value: 42.0, .. }
        ));
    }
    registry.shutdown().await;
}
