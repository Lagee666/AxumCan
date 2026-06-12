use std::collections::HashMap;

use socketcan::CanFdFrame;
use tokio::sync::watch::{Receiver, Sender};

use crate::error::Error;

#[derive(Default)]
pub struct Builder {
    interface: String,
    message_lable: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_interface(mut self, interface: &str) -> Self {
        self.interface = interface.to_string();
        self
    }

    pub fn set_message_label(mut self, message_label: String) -> Self {
        self.message_lable = Some(message_label);
        self
    }

    pub fn build(self) -> Result<CanActor, Error> {
        let (tx, rx) = tokio::sync::watch::channel(HashMap::new());
        let socket = Socket::bind(&self.interface, true).unwrap();
        let channel = Channel::from_str(&self.interface).unwrap();
        let Some(message_label) = self.message_lable else {
            return Err(Error::MessageLabelNone);
        };
        let can_sender = SenderSession {
            socket,
            channel,
            message_label,
            rx,
        };
        can_sender.start_task();
        Ok(CanActor { tx })
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
    socket: Socket,
    channel: Channel,
    message_label: String,
    rx: Receiver<HashMap<String, f64>>,
}

impl SenderSession {
    fn start_task(mut self) {
        let cycle_time = self.message_label.cycle_time();
        let mut interval = tokio::time::interval(cycle_time);
        let signal_map = self.rx.borrow().clone();
        let mut can_frame = self.create_frame(signal_map);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = self.rx.changed() => can_frame = self.create_frame(self.rx.borrow().clone()),
                    _ = interval.tick() => self.send_can(can_frame).await,
                }
            }
        });
    }

    fn create_frame(&self, signal_map: HashMap<String, f64>) -> CanFdFrame {
        construct_frame(signal_map)
    }

    async fn send_can(&self, can_frame: CanFdFrame) {
        self.socket.send(&can_frame).await.unwrap();
    }
}
