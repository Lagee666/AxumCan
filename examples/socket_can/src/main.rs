//! Minimal direct `socketcan` 3.5.0 example.
//!
//! The interface, CAN ID, payload, and send interval are read from the TOML
//! file next to this crate. No interface name is compiled into the program.

use std::{error::Error, path::PathBuf, time::Duration};

use serde::Deserialize;
use socketcan::{tokio::CanSocket, CanFrame, EmbeddedFrame, ExtendedId, Id, StandardId};

#[derive(Debug, Deserialize)]
struct Config {
    can: CanConfig,
}

#[derive(Debug, Deserialize)]
struct CanConfig {
    interface: String,
    id: u32,
    #[serde(default)]
    extended: bool,
    data: Vec<u8>,
    #[serde(default = "default_interval_ms")]
    interval_ms: u64,
}

fn default_interval_ms() -> u64 {
    1000
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("socket_can.toml");
    let config: Config = toml::from_str(&tokio::fs::read_to_string(config_path).await?)?;

    if config.can.data.len() > 8 {
        return Err("classic CAN payload cannot exceed 8 bytes".into());
    }

    let socket = CanSocket::open(&config.can.interface)?;
    let id = if config.can.extended {
        Id::Extended(ExtendedId::new(config.can.id).ok_or("invalid extended CAN ID")?)
    } else {
        Id::Standard(StandardId::new(config.can.id as u16).ok_or("invalid standard CAN ID")?)
    };

    let interval = Duration::from_millis(config.can.interval_ms);
    println!(
        "sending id=0x{:x} on {} every {:?}; press Ctrl-C to stop",
        config.can.id, config.can.interface, interval
    );

    let mut ticker = tokio::time::interval(interval);
    loop {
        ticker.tick().await;
        let frame = CanFrame::new(id, &config.can.data).ok_or("invalid CAN frame")?;
        socket.write_frame(frame).await?;
    }
}
