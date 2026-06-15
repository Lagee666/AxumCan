use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use socketcan::{CanFdFrame, EmbeddedFrame, Id, StandardId};
use tokio::sync::watch::{Receiver, Sender};
use tracing::error;

use crate::error::Error;
use crate::traits::{CanBuilderTrait, MessageTrait, SignalTrait, SocketUtilsTrait};

#[derive(Clone)]
pub struct CanActor<Signal: SignalTrait> {
    pub tx: Sender<HashMap<Signal, f64>>,
}

impl<Signal: SignalTrait> CanActor<Signal> {
    pub fn send(&self, signal_label: Signal, value: f64) {
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

pub struct SenderSession<Signal: SignalTrait> {
    pub socket: Box<dyn SocketUtilsTrait<Signal>>,
    pub cycle_time: Duration,
    pub rx: Receiver<HashMap<Signal, f64>>,
}

impl<Signal: SignalTrait> SenderSession<Signal> {
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

// --- Zero-Config Mock Implementations ---

pub struct MockSocket {
    pub id: u32,
}

#[async_trait]
impl<Signal: SignalTrait> SocketUtilsTrait<Signal> for MockSocket {
    fn create_frame(&self, signal_map: HashMap<Signal, f64>) -> CanFdFrame {
        let id = match StandardId::new((self.id & 0x7FF) as u16) {
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

/// The default builder used when the user wants to mock CAN data.
pub struct MockBuilder<Signal: SignalTrait, Message: MessageTrait> {
    pub interface: String,
    pub message_label: Option<Message>,
    _phantom: std::marker::PhantomData<Signal>,
}

impl<Signal: SignalTrait, Message: MessageTrait> Default for MockBuilder<Signal, Message> {
    fn default() -> Self {
        Self {
            interface: String::new(),
            message_label: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Signal: SignalTrait, Message: MessageTrait> CanBuilderTrait<Signal, Message> for MockBuilder<Signal, Message> {
    fn set_interface(mut self, interface: &str) -> Self {
        self.interface = interface.to_string();
        self
    }

    fn set_message_label(mut self, message_label: Message) -> Self {
        self.message_label = Some(message_label);
        self
    }

    fn build(self) -> Result<CanActor<Signal>, Error> {
        let (tx, rx) = tokio::sync::watch::channel(HashMap::new());
        let Some(message_label) = self.message_label else {
            return Err(Error::MessageLabelNone);
        };
        
        // Mock logic: generate stable fake metadata from the message label string
        let label_str = message_label.to_string();
        let fake_id = match label_str.as_str() {
            "test1" => 0x100,
            "test2" => 0x200,
            "test3" => 0x300,
            _ => label_str.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32)),
        };
        let fake_cycle = match label_str.as_str() {
            "test1" => Duration::from_millis(100),
            _ => Duration::from_millis(500),
        };

        let socket = MockSocket { id: fake_id };
        let can_sender = SenderSession {
            socket: Box::new(socket),
            cycle_time: fake_cycle,
            rx,
        };
        can_sender.start_task();
        Ok(CanActor { tx })
    }
}
