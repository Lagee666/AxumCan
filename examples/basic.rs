//! A small, self-contained AxumCan application.
//!
//! Run it with:
//!
//! ```text
//! cargo run --example basic
//! ```
//!
//! The example uses `TransportMode::Print`, so it works without a CAN
//! interface. Select `TransportMode::SocketCan` below when running on Linux
//! with an interface such as `vcan0`.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use axum_can::{
    axum_can::{AppState, serve},
    error::Error,
    model::{ByteOrder, CanChannel, CanMessage, CanModel, SignalSpec},
    registry::Registry,
    source::CanModelSource,
    transport::TransportMode,
};

struct ExampleModelSource;

#[async_trait]
impl CanModelSource for ExampleModelSource {
    async fn load(&self) -> Result<CanModel, Error> {
        Ok(CanModel {
            channels: vec![CanChannel {
                // This becomes the SocketCAN interface name when using
                // TransportMode::SocketCan.
                name: "vcan0".into(),
                messages: vec![CanMessage {
                    name: "VehicleStatus".into(),
                    can_id: 0x123,
                    is_extended: false,
                    cycle_time: Duration::from_millis(100),
                    signals: vec![
                        SignalSpec {
                            name: "Speed".into(),
                            start_bit: 0,
                            bit_length: 16,
                            byte_order: ByteOrder::LittleEndian,
                            is_signed: false,
                            factor: 0.1,
                            offset: 0.0,
                            minimum: Some(0.0),
                            maximum: Some(250.0),
                            initial_value: 0.0,
                        },
                        SignalSpec {
                            name: "Gear".into(),
                            start_bit: 16,
                            bit_length: 8,
                            byte_order: ByteOrder::LittleEndian,
                            is_signed: false,
                            factor: 1.0,
                            offset: 0.0,
                            minimum: Some(0.0),
                            maximum: Some(6.0),
                            initial_value: 1.0,
                        },
                    ],
                }],
            }],
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let mut registry = Registry::default();
    registry.set_model_source(Box::new(ExampleModelSource));
    registry.set_transport_mode(TransportMode::Print);
    // For a real Linux CAN/vCAN interface, use this instead:
    // registry.set_transport_mode(TransportMode::SocketCan);
    registry.init().await?;

    let registry = Arc::new(registry);
    let updater = Arc::clone(&registry);
    tokio::spawn(async move {
        let mut speed = 0.0;
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            speed = (speed + 10.0) % 251.0;
            updater.update("Speed".into(), speed, false);
        }
    });

    let state = Arc::new(AppState { registry });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8028").await?;
    println!("dashboard: http://127.0.0.1:8028");
    serve(listener, state).await;
    Ok(())
}
