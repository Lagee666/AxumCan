//! Inspect the channels and signals loaded from `can_signal.json`.
//!
//! Channel names are read from the configuration; this example does not
//! assume a fixed set of interfaces.

use axum_can::source::{CanModelSource, JsonModelSource};

#[tokio::main]
async fn main() -> Result<(), axum_can::error::Error> {
    let model = JsonModelSource::new("can_signal.json").load().await?;

    for channel in model.channels {
        println!("channel: {}", channel.name);
        for message in channel.messages {
            println!(
                "  message: {} (id=0x{:x}, cycle={}ms)",
                message.name,
                message.can_id,
                message.cycle_time.as_millis()
            );
            for signal in message.signals {
                println!(
                    "    signal: {} (start_bit={}, bit_length={}, initial={})",
                    signal.name, signal.start_bit, signal.bit_length, signal.initial_value
                );
            }
        }
    }

    Ok(())
}
