use std::{collections::HashMap, sync::Arc};

use tokio::{sync::watch, task::JoinHandle};
use tracing::{error, info};

use crate::{encoder::encode_message, model::CanMessage, transport::CanTransport};

#[derive(Clone)]
pub struct CanActor {
    tx: watch::Sender<HashMap<String, f64>>,
}

impl CanActor {
    pub fn send(&self, signal_label: String, value: f64) {
        self.tx
            .send_if_modified(|values| match values.get_mut(&signal_label) {
                Some(current) if *current == value => false,
                Some(current) => {
                    *current = value;
                    true
                }
                None => {
                    values.insert(signal_label, value);
                    true
                }
            });
    }
}

pub struct SenderTask {
    pub actor: CanActor,
    pub task: JoinHandle<()>,
}

pub fn spawn_sender(
    channel: String,
    message: CanMessage,
    transport: Arc<dyn CanTransport>,
) -> SenderTask {
    let initial_values = message
        .signals
        .iter()
        .map(|signal| (signal.name.clone(), signal.initial_value))
        .collect::<HashMap<_, _>>();
    let (tx, mut rx) = watch::channel(initial_values);
    let actor = CanActor { tx };
    let task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(message.cycle_time);
        let mut frame = match encode_message(&message, &rx.borrow()) {
            Ok(frame) => frame,
            Err(error) => {
                error!(%channel, message = %message.name, %error, "failed to encode initial CAN frame");
                return;
            }
        };
        loop {
            tokio::select! {
                changed = rx.changed() => {
                    if changed.is_err() { info!(%channel, message = %message.name, "CAN sender state closed"); break; }
                    match encode_message(&message, &rx.borrow()) {
                        Ok(next) => frame = next,
                        Err(error) => error!(%channel, message = %message.name, %error, "failed to encode CAN frame"),
                    }
                }
                _ = interval.tick() => {
                    if let Err(error) = transport.send(&frame).await {
                        error!(%channel, message = %message.name, can_id = format_args!("0x{:x}", frame.id), %error, "failed to send CAN frame");
                    }
                }
            }
        }
    });
    SenderTask { actor, task }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::Error,
        model::{ByteOrder, CanFrame, SignalSpec},
        transport::CanTransport,
    };
    use async_trait::async_trait;
    use std::{sync::Mutex, time::Duration};

    struct CaptureTransport(Mutex<Vec<CanFrame>>);
    #[async_trait]
    impl CanTransport for CaptureTransport {
        async fn send(&self, frame: &CanFrame) -> Result<(), Error> {
            self.0.lock().unwrap().push(frame.clone());
            Ok(())
        }
    }

    #[tokio::test(start_paused = true)]
    async fn sender_uses_latest_signal_value() {
        let transport = Arc::new(CaptureTransport(Mutex::new(Vec::new())));
        let message = CanMessage {
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
                minimum: None,
                maximum: None,
                initial_value: 0.0,
            }],
        };
        let sender = spawn_sender("vcan0".into(), message, transport.clone());
        sender.actor.send("Speed".into(), 42.0);
        tokio::time::advance(Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
        assert_eq!(transport.0.lock().unwrap()[0].data[0], 42);
        sender.task.abort();
    }
}
