use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, error};

use crate::{
    can_sender::CanActor,
    error::Error,
    signals::Signals,
    traits::{CanBuilderTrait, MessageTrait, SignalTrait},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
#[serde(bound(serialize = "S: Serialize", deserialize = "S: serde::de::DeserializeOwned"))]
pub enum WsMessage<S: SignalTrait> {
    Init {
        signals: Signals,
    },
    ClientUpdate {
        signal: S,
        value: f64,
    },
    SetArbitration {
        signal: S,
        #[serde(rename = "allowBackend")]
        allow_backend: bool,
    },
    StateChanged {
        signal: String, // Keep as String for broadcast if needed, or make it S
        value: f64,
    },
}

pub struct Registry<S, M, B>
where
    S: SignalTrait,
    M: MessageTrait,
    B: CanBuilderTrait<S, M>,
{
    actors: HashMap<S, Vec<CanActor<S>>>,
    arbitration: Arc<Mutex<HashMap<S, bool>>>,
    pub broadcast_tx: broadcast::Sender<WsMessage<S>>,
    pub initial_signals: Signals,
    _phantom: std::marker::PhantomData<(M, B)>,
}

impl<S, M, B> Default for Registry<S, M, B>
where
    S: SignalTrait,
    M: MessageTrait,
    B: CanBuilderTrait<S, M>,
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

impl<S, M, B> Registry<S, M, B>
where
    S: SignalTrait,
    M: MessageTrait,
    B: CanBuilderTrait<S, M>,
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
            let message_label = match M::from_str(&message_str) {
                Ok(label) => label,
                Err(_) => {
                    error!("Parse message {} failed", message_str);
                    continue;
                }
            };

            let actor = match B::default()
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
                let signal_label = match S::from_str(&signal_str) {
                    Ok(label) => label,
                    Err(_) => {
                        error!("Parse signal {} failed", signal_str);
                        continue;
                    }
                };
                debug!(
                    "Register Channel: {:?}, message: {:?}, id: {:X?}, signal: {:?}",
                    channel, message_label, message_label.raw_id(), signal_label
                );
                actor.send(signal_label.clone(), value as f64);
                self.add(signal_label, actor.clone());
            }
        }
    }

    fn add(&mut self, signal_label: S, actor: CanActor<S>) {
        self.actors.entry(signal_label).or_default().push(actor);
    }

    pub fn update(&self, signal_label: S, value: f64, is_backend: bool) {
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

    pub fn set_arbitration(&self, signal_label: S, allow_backend: bool) {
        let mut arbitration = self.arbitration.lock().unwrap();
        arbitration.insert(signal_label, allow_backend);
    }

    /// FEATURE 1: Update a signal that exists in the dashboard and sync its UI value.
    /// This also respects arbitration and updates the underlying CAN actors.
    pub fn update_dashboard(&self, signal_label: S, value: f64) {
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

    /// FEATURE 2: Send an arbitrary signal to the frontend Monitor only.
    /// This supports any signal name (String), even if it doesn't exist in the signal map.
    pub fn send_to_monitor(&self, signal_name: String, value: f64) {
        let _ = self.broadcast_tx.send(WsMessage::StateChanged {
            signal: signal_name,
            value,
        });
    }
}
