use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use socketcan::{CanFdFrame, EmbeddedFrame, Id, StandardId};
use tokio::sync::watch::{Receiver, Sender};
use tracing::error;

use crate::{
    error::Error,
    traits::CanHardware,
};

#[derive(Clone)]
pub struct CanActor {
    pub tx: Sender<HashMap<String, f64>>,
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
    pub socket: Box<dyn SocketUtils>,
    pub cycle_time: Duration,
    pub rx: Receiver<HashMap<String, f64>>,
}

impl SenderSession {
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

#[async_trait]
pub trait SocketUtils: Send + Sync + 'static {
    fn create_frame(&self, signal_map: HashMap<String, f64>) -> CanFdFrame;
    async fn send_can(&self, frame: CanFdFrame);
}

// --- Mock Hardware Implementation ---

pub struct MockSocket {
    pub id: u32,
}

#[async_trait]
impl SocketUtils for MockSocket {
    fn create_frame(&self, signal_map: HashMap<String, f64>) -> CanFdFrame {
        let id = match StandardId::new((self.id & 0x7FF) as u16) {
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

pub struct MockHardware;

impl CanHardware for MockHardware {
    fn create_actor(
        &self,
        _channel: &str,
        message_name: &str,
        _signals: &HashMap<String, u64>,
    ) -> Result<CanActor, Error> {
        let (tx, rx) = tokio::sync::watch::channel(HashMap::new());
        
        // Mock logic: generate stable fake metadata from the message name string
        let fake_id = match message_name {
            "test1" => 0x100,
            "test2" => 0x200,
            "test3" => 0x300,
            _ => message_name.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32)),
        };
        let fake_cycle = match message_name {
            "test1" => Duration::from_millis(100),
            _ => Duration::from_millis(500),
        };

        let socket = MockSocket { id: fake_id };
        let session = SenderSession {
            socket: Box::new(socket),
            cycle_time: fake_cycle,
            rx,
        };
        session.start_task();
        Ok(CanActor { tx })
    }
}
