use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use socketcan::{CanFdFrame, EmbeddedFrame, Id, StandardId};
use tokio::sync::watch::{Receiver, Sender};
use tracing::error;

use crate::error::Error;
use crate::traits::{CanBuilderTrait, MessageTrait, SignalTrait, SocketUtilsTrait};

#[derive(Clone)]
pub struct CanActor<S: SignalTrait> {
    pub tx: Sender<HashMap<S, f64>>,
}

impl<S: SignalTrait> CanActor<S> {
    pub fn send(&self, signal_label: S, value: f64) {
        self.tx.send_if_modified(|signal_map| {
            let mut changed = false;
            let signal_value = signal_map.get_mut(&signal_label);
            match signal_value {
                Some(v) => {
                    if *v != value {
                        *v = value;
                        changed = true;
                    }
                }
                None => {
                    signal_map.insert(signal_label, value);
                    changed = true;
                }
            }
            changed
        });
    }
}

pub struct SenderSession<S: SignalTrait> {
    pub socket: Box<dyn SocketUtilsTrait<S>>,
    pub cycle_time: Duration,
    pub rx: Receiver<HashMap<S, f64>>,
}

impl<S: SignalTrait> SenderSession<S> {
    pub fn start_task(mut self) {
        let mut interval = tokio::time::interval(self.cycle_time);
        let signal_map = self.rx.borrow().clone();
        let mut can_frame = self.socket.create_frame(signal_map);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = self.rx.changed() => can_frame = self.socket.create_frame(self.rx.borrow().clone()),
                    _ = interval.tick() => self.socket.send_can(can_frame).await,
                }
            }
        });
    }
}

// --- Mock Implementations for Standalone Simulator ---

pub struct MockSocket {
    pub id: u32,
}

#[async_trait]
impl<S: SignalTrait> SocketUtilsTrait<S> for MockSocket {
    fn create_frame(&self, signal_map: HashMap<S, f64>) -> CanFdFrame {
        let id = match StandardId::new(self.id as u16) {
            Some(id) => Id::Standard(id),
            None => {
                error!("Failed to create CAN ID");
                return CanFdFrame::default();
            }
        };
        let mut signal_vector = signal_map.iter().collect::<Vec<_>>();
        signal_vector.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));
        let mut data = [0u8; 8];
        for (i, (_signal, value)) in signal_vector.iter().enumerate() {
            if i >= 8 { break; }
            data[i] = **value as u8;
        }

        match CanFdFrame::new(id, &data) {
            Some(frame) => frame,
            None => {
                error!("Failed to create CAN frame");
                CanFdFrame::default()
            }
        }
    }
    async fn send_can(&self, frame: CanFdFrame) {
        println!("{:?}", frame);
    }
}

pub struct MockBuilder<S: SignalTrait, M: MessageTrait> {
    pub interface: String,
    pub message_label: Option<M>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: SignalTrait, M: MessageTrait> Default for MockBuilder<S, M> {
    fn default() -> Self {
        Self {
            interface: String::new(),
            message_label: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: SignalTrait, M: MessageTrait> MockBuilder<S, M> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S: SignalTrait, M: MessageTrait> CanBuilderTrait<S, M> for MockBuilder<S, M> {
    fn set_interface(mut self, interface: &str) -> Self {
        self.interface = interface.to_string();
        self
    }

    fn set_message_label(mut self, message_label: M) -> Self {
        self.message_label = Some(message_label);
        self
    }

    fn build(self) -> Result<CanActor<S>, Error> {
        let (tx, rx) = tokio::sync::watch::channel(HashMap::new());
        let Some(message_label) = self.message_label else {
            return Err(Error::MessageLabelNone);
        };
        let socket = MockSocket { id: message_label.raw_id() };
        let can_sender = SenderSession {
            socket: Box::new(socket),
            cycle_time: message_label.cycle_time(),
            rx,
        };
        can_sender.start_task();
        Ok(CanActor { tx })
    }
}
