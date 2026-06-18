use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use socketcan::{CanFdFrame, EmbeddedFrame, Id, StandardId};
use tokio::sync::watch::{Receiver, Sender};
use tracing::error;

use crate::{error::Error, message_utils::MessageProvider};

#[derive(Default)]
pub struct Builder {
    socket: Option<Box<dyn SocketUtils>>,
    interface: String,
    message_id: Option<Box<dyn MessageProvider>>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> Result<CanActor, Error> {
        let (tx, rx) = tokio::sync::watch::channel(HashMap::new());
        if self.socket.is_none() {
            self.socket = Some(Box::new(MockSocket))
        }
        if self.message_id.is_none() {
            return Err(Error::MessageLabelNone);
        }
        let cycle_time = self.message_id.as_ref().unwrap().get_cycle_time();
        let message_id = self.message_id.take().unwrap().to_id();
        let can_sender = SenderSession {
            builder: self,
            cycle_time,
            message_id,
            rx,
        };
        can_sender.start_task();
        Ok(CanActor { tx })
    }

    pub fn set_interface(mut self, interface: &str) -> Self {
        self.interface = interface.to_string();
        self
    }

    pub fn set_id(mut self, message_id: Box<dyn MessageProvider>) -> Self {
        self.message_id = Some(message_id);
        self
    }
}

#[async_trait]
pub trait SocketUtils: Send + Sync + 'static {
    fn create_frame(&self, message_id: u16, signal_map: HashMap<String, f64>) -> CanFdFrame;
    async fn send_can(&self, frame: CanFdFrame);
}

#[derive(Default)]
pub struct MockSocket;

#[async_trait]
impl SocketUtils for MockSocket {
    fn create_frame(&self, message_id: u16, signal_map: HashMap<String, f64>) -> CanFdFrame {
        let id = match StandardId::new(message_id) {
            Some(id) => Id::Standard(id),
            None => {
                error!("Failed to create CAN ID");
                return CanFdFrame::default();
            }
        };
        let mut signal_vector = signal_map.iter().collect::<Vec<_>>();
        signal_vector.sort_by(|a, b| a.0.cmp(b.0));
        let mut data = [0u8; 8];
        for (i, (_signal, value)) in signal_vector.iter().enumerate() {
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

#[derive(Clone)]
pub struct CanActor {
    tx: Sender<HashMap<String, f64>>,
}

impl CanActor {
    pub fn send(&self, signal_label: String, value: f64) {
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

pub struct SenderSession {
    builder: Builder,
    message_id: u16,
    cycle_time: Duration,
    rx: Receiver<HashMap<String, f64>>,
}

impl SenderSession {
    fn start_task(mut self) {
        let mut interval = tokio::time::interval(self.cycle_time);
        let signal_map = self.rx.borrow().clone();
        let socket = self.builder.socket.unwrap();
        let mut can_frame = socket.create_frame(self.message_id, signal_map);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = self.rx.changed() => can_frame = socket.create_frame(self.message_id,self.rx.borrow().clone()),
                    _ = interval.tick() => socket.send_can(can_frame).await,
                }
            }
        });
    }
}
