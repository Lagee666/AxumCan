use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, error};

use crate::{
    can_sender::{CanActor, MockBuilder},
    error::Error,
    signals::Signals,
    traits::{CanBuilderTrait, MessageTrait, SignalTrait},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
#[serde(bound(serialize = "Signal: Serialize", deserialize = "Signal: serde::de::DeserializeOwned"))]
pub enum WsMessage<Signal: SignalTrait> {
    Init {
        signals: Signals,
    },
    ClientUpdate {
        signal: Signal,
        value: f64,
    },
    SetArbitration {
        signal: Signal,
        #[serde(rename = "allowBackend")]
        allow_backend: bool,
    },
    StateChanged {
        signal: String, 
        value: f64,
    },
}

/// The central coordinator for the CAN simulation.
/// Defaults to using Strings as labels and a MockBuilder for zero-config simulation.
pub struct Registry<Signal = String, Message = String, Builder = MockBuilder<Signal, Message>>
where
    Signal: SignalTrait,
    Message: MessageTrait,
    Builder: CanBuilderTrait<Signal, Message>,
{
    actors: HashMap<Signal, Vec<CanActor<Signal>>>,
    arbitration: Arc<Mutex<HashMap<Signal, bool>>>,
    pub broadcast_tx: broadcast::Sender<WsMessage<Signal>>,
    pub initial_signals: Signals,
    _phantom: std::marker::PhantomData<(Message, Builder)>,
}

impl<Signal, Message, Builder> Default for Registry<Signal, Message, Builder>
where
    Signal: SignalTrait,
    Message: MessageTrait,
    Builder: CanBuilderTrait<Signal, Message>,
{
    fn default() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            actors: HashMap::new(),
            arbitration: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            initial_signals: Signals::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Signal, Message, Builder> Registry<Signal, Message, Builder>
where
    Signal: SignalTrait,
    Message: MessageTrait,
    Builder: CanBuilderTrait<Signal, Message>,
{
    pub async fn init(&mut self) -> Result<(), Error> {
        self.actors.clear();
        let signals = Signals::init().await?;
        self.initial_signals = signals.clone();

        self.register_actors("vcan1", signals.vcan1);
        self.register_actors("vcan2", signals.vcan2);
        self.register_actors("vcan3", signals.vcan3);
        self.register_actors("vcan4", signals.vcan4);
        self.register_actors("vcan5", signals.vcan5);
        self.register_actors("vcan6", signals.vcan6);

        Ok(())
    }

    fn register_actors(&mut self, channel: &str, signals: HashMap<String, HashMap<String, u64>>) {
        for (message_str, signal_map) in signals {
            let message_label = match Message::from_str(&message_str) {
                Ok(label) => label,
                Err(_) => {
                    error!("Parse message {} failed", message_str);
                    continue;
                }
            };

            let actor = match Builder::default()
                .set_interface(channel)
                .set_message_label(message_label.clone())
                .build()
            {
                Ok(actor) => actor,
                Err(e) => {
                    error!("Build CAN actor failed: {}", e);
                    continue;
                }
            };

            for (signal_str, value) in signal_map {
                let signal_label = match Signal::from_str(&signal_str) {
                    Ok(label) => label,
                    Err(_) => {
                        error!("Parse signal {} failed", signal_str);
                        continue;
                    }
                };
                debug!(
                    "Register Channel: {:?}, message: {:?}, signal: {:?}",
                    channel, message_label, signal_label
                );
                actor.send(signal_label.clone(), value as f64);
                self.add(signal_label, actor.clone());
            }
        }
    }

    fn add(&mut self, signal_label: Signal, actor: CanActor<Signal>) {
        self.actors.entry(signal_label).or_default().push(actor);
    }

    pub fn update(&self, signal_label: Signal, value: f64, is_backend: bool) {
        if is_backend {
            let arbitration = self.arbitration.lock().unwrap();
            if let Some(&allow_backend) = arbitration.get(&signal_label) {
                if !allow_backend {
                    return;
                }
            }
        }

        let actors = self.actors.get(&signal_label);
        if let Some(actors) = actors {
            for actor in actors {
                actor.send(signal_label.clone(), value);
            }
        }
    }

    pub fn set_arbitration(&self, signal_label: Signal, allow_backend: bool) {
        let mut arbitration = self.arbitration.lock().unwrap();
        arbitration.insert(signal_label, allow_backend);
    }

    /// Update a signal that exists in the dashboard and sync its UI value.
    pub fn update_dashboard(&self, signal_label: Signal, value: f64) {
        // Respect arbitration
        let arbitration = self.arbitration.lock().unwrap();
        if let Some(&allow_backend) = arbitration.get(&signal_label) {
            if !allow_backend {
                return;
            }
        }
        drop(arbitration);

        // Update CAN actors
        let actors = self.actors.get(&signal_label);
        if let Some(actors) = actors {
            for actor in actors {
                actor.send(signal_label.clone(), value);
            }
        }

        // Send to frontend to update the control UI and monitor
        let _ = self.broadcast_tx.send(WsMessage::StateChanged {
            signal: signal_label.to_string(),
            value,
        });
    }

    /// Send an arbitrary signal to the frontend Monitor only.
    pub fn send_to_monitor(&self, signal_name: String, value: f64) {
        let _ = self.broadcast_tx.send(WsMessage::StateChanged {
            signal: signal_name,
            value,
        });
    }
}
