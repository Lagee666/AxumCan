use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{
    can_sender::CanActor,
    signals::Signals,
};

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
    pub broadcast_tx: broadcast::Sender<WsMessage>,
    pub initial_signals: Signals,
}

impl Default for Registry {
    fn default() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            actors: HashMap::new(),
            arbitration: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            initial_signals: Signals::default(),
        }
    }
}

impl Registry {
    pub fn add_actor(&mut self, signal_label: String, actor: CanActor) {
        self.actors.entry(signal_label).or_default().push(actor);
    }

    pub fn set_initial_signals(&mut self, signals: Signals) {
        self.initial_signals = signals;
    }

    pub fn update(&self, signal_label: String, value: f64, is_backend: bool) {
        if is_backend {
            let arbitration = self.arbitration.lock().unwrap();
            if let Some(&allow_backend) = arbitration.get(&signal_label) {
                if !allow_backend {
                    return;
                }
            }
        }

        if let Some(actors) = self.actors.get(&signal_label) {
            for actor in actors {
                actor.send(signal_label.clone(), value);
            }
        }
    }

    pub fn set_arbitration(&self, signal_label: String, allow_backend: bool) {
        let mut arbitration = self.arbitration.lock().unwrap();
        arbitration.insert(signal_label, allow_backend);
    }

    pub fn update_dashboard(&self, signal_label: String, value: f64) {
        // Respect arbitration
        let arbitration = self.arbitration.lock().unwrap();
        if let Some(&allow_backend) = arbitration.get(&signal_label) {
            if !allow_backend {
                return;
            }
        }
        drop(arbitration);

        // Update CAN actors
        if let Some(actors) = self.actors.get(&signal_label) {
            for actor in actors {
                actor.send(signal_label.clone(), value);
            }
        }

        // Send to frontend
        let _ = self.broadcast_tx.send(WsMessage::StateChanged {
            signal: signal_label,
            value,
        });
    }

    pub fn send_to_monitor(&self, signal_name: String, value: f64) {
        let _ = self.broadcast_tx.send(WsMessage::StateChanged {
            signal: signal_name,
            value,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::SetArbitration {
            signal: "VehSpeed".to_string(),
            allow_backend: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        // Check that variant is camelCase (setArbitration) and field is renamed (allowBackend)
        assert!(json.contains("\"type\":\"setArbitration\""));
        assert!(json.contains("\"allowBackend\":false"));

        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        if let WsMessage::SetArbitration {
            signal,
            allow_backend,
        } = deserialized
        {
            assert_eq!(signal, "VehSpeed");
            assert!(!allow_backend);
        } else {
            panic!("Deserialized to wrong variant");
        }
    }

    #[tokio::test]
    async fn test_arbitration_logic() {
        let registry = Registry::default();
        let label = "VehSpeed".to_string();
        let mut rx = registry.broadcast_tx.subscribe();

        // 1. Enable arbitration (default is allow)
        registry.set_arbitration(label.clone(), true);
        registry.update_dashboard(label.clone(), 10.0);

        // Should receive message
        let msg = rx.recv().await.unwrap();
        if let WsMessage::StateChanged { value, .. } = msg {
            assert_eq!(value, 10.0);
        }

        // 2. Disable arbitration
        registry.set_arbitration(label.clone(), false);
        registry.update_dashboard(label.clone(), 20.0);

        // Should NOT receive a new message
        let result = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not have received message when arbitration is disabled"
        );
    }

    #[tokio::test]
    async fn test_send_to_monitor() {
        let registry = Registry::default();
        let mut rx = registry.broadcast_tx.subscribe();

        registry.send_to_monitor("UnknownSignal".to_string(), 99.0);

        let msg = rx.recv().await.unwrap();
        if let WsMessage::StateChanged { signal, value } = msg {
            assert_eq!(signal, "UnknownSignal");
            assert_eq!(value, 99.0);
        }
    }
}
