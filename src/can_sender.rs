use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use socketcan::{CanFdFrame, EmbeddedFrame, Id, StandardId};
use tokio::sync::watch::{Receiver, Sender};
use tracing::error;

use crate::error::Error;

pub trait CanBuilder {
    fn build(self) -> Result<CanActor, Error>;
    fn cycle_time(&self) -> tokio::time::Duration;
}

#[derive(Default)]
pub struct MockBuilder {
    interface: String,
    message_id: Option<String>,
    cycle_time: Duration,
}

impl MockBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_interface(mut self, interface: &str) -> Self {
        self.interface = interface.to_string();
        self
    }

    pub fn set_id(mut self, message_id: String) -> Self {
        self.message_id = Some(message_id);
        self
    }

    fn to_id(&self) -> u16 {
        let value = self.as_str();
        match value {
            "test1" => 0x100,
            "test2" => 0x200,
            "test3" => 0x300,
            "test4" => 0x400,
            "test5" => 0x500,
            _ => 0,
        }
    }

    fn get_cycle_time(&self) -> Duration {
        let value = self.as_str();
        match value {
            "test1" => Duration::from_millis(100),
            "test2" => Duration::from_millis(200),
            "test3" => Duration::from_millis(300),
            "test4" => Duration::from_millis(400),
            "test5" => Duration::from_millis(500),
            _ => Duration::from_secs(10000),
        }
    }
}

impl CanBuilder for MockBuilder {
    fn build(self) -> Result<CanActor, Error> {
        let (tx, rx) = tokio::sync::watch::channel(HashMap::new());
        let mut socket = MockSocket::default();
        let Some(id) = self.message_id else {
            return Err(Error::MessageLabelNone);
        };
        socket.id = id;
        let can_sender = SenderSession {
            socket: Box::new(socket),
            builder: Box::new(self),
            rx,
        };
        can_sender.start_task();
        Ok(CanActor { tx })
    }

    fn cycle_time(&self) -> Duration {
        self.cycle_time
    }
}

#[async_trait]
pub trait SocketUtils {
    fn create_frame(&self, signal_map: HashMap<String, f64>) -> CanFdFrame;
    async fn send_can(&self, frame: CanFdFrame);
}

#[derive(Default)]
pub struct MockSocket {
    pub id: u16,
}

#[async_trait]
impl SocketUtils for MockSocket {
    fn create_frame(&self, signal_map: HashMap<String, f64>) -> CanFdFrame {
        let id = match StandardId::new(self.id) {
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
    socket: Box<dyn SocketUtils + Send>,
    builder: Box<dyn CanBuilder>,
    rx: Receiver<HashMap<String, f64>>,
}

impl SenderSession {
    fn start_task(mut self) {
        let cycle_time = self.builder.cycle_time();
        let mut interval = tokio::time::interval(cycle_time);
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
