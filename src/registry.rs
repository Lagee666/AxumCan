use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{debug, error};

use crate::{
    can_sender::{CanActor, spawn_sender},
    error::Error,
    signals::Signals,
    source::{CanModelSource, JsonModelSource, legacy_signals},
    transport::{CanTransport, TransportMode},
};

const JSON_FILE_PATH: &str = "can_signal.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsMessage {
    Init {
        signals: Box<Signals>,
    },
    ClientUpdate {
        signal: String,
        value: f64,
    },
    SetArbitration {
        signal: String,
        #[serde(rename = "allowBackend")]
        allow_backend: bool,
    },
    StateChanged {
        signal: String,
        value: f64,
    },
}

pub struct Registry {
    actors: HashMap<String, Vec<CanActor>>,
    arbitration: Arc<Mutex<HashMap<String, bool>>>,
    source: Box<dyn CanModelSource>,
    mode: TransportMode,
    tasks: Vec<JoinHandle<()>>,
    transports: Vec<Arc<dyn CanTransport>>,
    pub broadcast_tx: broadcast::Sender<WsMessage>,
    pub initial_signals: Signals,
}

impl Default for Registry {
    fn default() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            actors: HashMap::new(),
            arbitration: Arc::new(Mutex::new(HashMap::new())),
            source: Box::new(JsonModelSource::new(JSON_FILE_PATH)),
            mode: TransportMode::default(),
            tasks: Vec::new(),
            transports: Vec::new(),
            broadcast_tx,
            initial_signals: Signals::default(),
        }
    }
}

impl Registry {
    pub fn set_signal_path(&mut self, path: PathBuf) {
        self.source = Box::new(JsonModelSource::new(path));
    }
    pub fn set_transport_mode(&mut self, mode: TransportMode) {
        self.mode = mode;
    }
    pub fn set_model_source(&mut self, source: Box<dyn CanModelSource>) {
        self.source = source;
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        self.shutdown().await;
        let model = self.source.load().await?;
        model.validate()?;
        let factory = self.mode.factory();
        let mut transports: HashMap<String, Arc<dyn CanTransport>> = HashMap::new();
        for channel in &model.channels {
            if transports.contains_key(&channel.name) {
                return Err(Error::InvalidModel(format!(
                    "duplicate CAN channel {}",
                    channel.name
                )));
            }
            let transport = match factory.create(&channel.name) {
                Ok(transport) => transport,
                Err(error) => {
                    for previous in transports.values() {
                        let _ = previous.stop().await;
                    }
                    return Err(error);
                }
            };
            if let Err(error) = transport.start().await {
                let _ = transport.stop().await;
                for previous in transports.values() {
                    let _ = previous.stop().await;
                }
                return Err(error);
            }
            transports.insert(channel.name.clone(), transport);
        }
        self.initial_signals = legacy_signals(&model);
        self.actors.clear();
        self.transports = transports.values().cloned().collect();
        for channel in model.channels {
            let transport = transports
                .get(&channel.name)
                .expect("transport created above")
                .clone();
            for message in channel.messages {
                let sender = spawn_sender(channel.name.clone(), message.clone(), transport.clone());
                for signal in &message.signals {
                    debug!(channel = %channel.name, message = %message.name, signal = %signal.name, "registered Logical Signal");
                    self.actors
                        .entry(signal.name.clone())
                        .or_default()
                        .push(sender.actor.clone());
                }
                self.tasks.push(sender.task);
            }
        }
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
        for transport in self.transports.drain(..) {
            if let Err(error) = transport.stop().await {
                error!(%error, "failed to stop CAN transport");
            }
        }
        self.actors.clear();
    }

    pub fn update(&self, signal_label: String, value: f64, is_backend: bool) -> bool {
        if is_backend && !self.backend_allowed(&signal_label) {
            return false;
        }
        let Some(actors) = self.actors.get(&signal_label) else {
            return false;
        };
        for actor in actors {
            actor.send(signal_label.clone(), value);
        }
        let _ = self.broadcast_tx.send(WsMessage::StateChanged {
            signal: signal_label,
            value,
        });
        true
    }

    pub fn set_arbitration(&self, signal_label: String, allow_backend: bool) {
        self.arbitration
            .lock()
            .unwrap()
            .insert(signal_label, allow_backend);
    }
    pub fn update_dashboard(&self, signal_label: String, value: f64) -> bool {
        self.update(signal_label, value, true)
    }
    pub fn send_to_monitor(&self, signal_name: String, value: f64) {
        let _ = self.broadcast_tx.send(WsMessage::StateChanged {
            signal: signal_name,
            value,
        });
    }
    fn backend_allowed(&self, signal: &str) -> bool {
        self.arbitration
            .lock()
            .unwrap()
            .get(signal)
            .copied()
            .unwrap_or(true)
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::SetArbitration {
            signal: "VehSpeed".into(),
            allow_backend: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"setArbitration\""));
        assert!(json.contains("\"allowBackend\":false"));
        assert!(matches!(
            serde_json::from_str::<WsMessage>(&json).unwrap(),
            WsMessage::SetArbitration {
                allow_backend: false,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn unknown_client_update_is_not_broadcast() {
        let registry = Registry::default();
        let mut rx = registry.broadcast_tx.subscribe();
        assert!(!registry.update("Speed".into(), 10.0, false));
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), rx.recv())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn backend_arbitration_blocks_backend_updates() {
        let registry = Registry::default();
        registry.set_arbitration("Speed".into(), false);
        assert!(!registry.update_dashboard("Speed".into(), 10.0));
    }
}
