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

struct StaticSource(CanModel);

#[async_trait]
impl CanModelSource for StaticSource {
    async fn load(&self) -> Result<CanModel, Error> {
        Ok(self.0.clone())
    }
}

#[derive(Clone)]
struct CaptureFactory {
    frames: Arc<Mutex<Vec<CanFrame>>>,
}

struct CaptureTransport {
    frames: Arc<Mutex<Vec<CanFrame>>>,
}

#[async_trait]
impl CanTransport for CaptureTransport {
    async fn send(&self, frame: &CanFrame) -> Result<(), Error> {
        self.frames.lock().unwrap().push(frame.clone());
        Ok(())
    }
}

impl TransportFactory for CaptureFactory {
    fn create(&self, _channel: &str) -> Result<Arc<dyn CanTransport>, Error> {
        Ok(Arc::new(CaptureTransport {
            frames: self.frames.clone(),
        }))
    }
}

#[tokio::test(start_paused = true)]
async fn registry_periodically_sends_a_canonical_message() {
    let frames = Arc::new(Mutex::new(Vec::new()));
    let model = CanModel {
        channels: vec![CanChannel {
            name: "test-bus".into(),
            messages: vec![CanMessage {
                name: "VehicleStatus".into(),
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
                    initial_value: 42.0,
                }],
            }],
        }],
    };

    let mut registry = Registry::default();
    registry.set_model_source(Box::new(StaticSource(model)));
    registry.set_transport_mode(TransportMode::Custom(Arc::new(CaptureFactory {
        frames: frames.clone(),
    })));
    registry.init().await.unwrap();

    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    let frames = frames.lock().unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].id, 0x100);
    assert!(!frames[0].is_extended);
    assert_eq!(frames[0].data, vec![42, 0, 0, 0, 0, 0, 0, 0]);
}
