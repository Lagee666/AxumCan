use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::Error;

pub type Channel = String;
pub type Message = String;
pub type Signal = String;
pub type SignalValue = u64;

pub struct ChannelInfo {
    pub channel: Channel,
    pub messages: HashMap<Message, MessageInfo>,
}

pub struct MessageInfo {
    pub message: Message,
    pub cycle_time: u64,
    pub signals: HashMap<Signal, SignalValue>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Signals {
    #[serde(flatten)]
    pub channels: HashMap<Channel, HashMap<Message, HashMap<Signal, SignalValue>>>,
}

impl Signals {
    pub async fn init(path: &Path) -> Result<Self, Error> {
        let content = tokio::fs::read_to_string(path).await?;
        serde_json::from_str(&content).map_err(|err| err.into())
    }

    pub fn to_channel_info(&self) -> Vec<ChannelInfo> {
        let mut channels = Vec::new();

        for (channel_name, messages) in &self.channels {
            let mut message_infos = HashMap::new();
            for (message_name, signals) in messages {
                let mut signal_infos = HashMap::new();
                for (signal_name, value) in signals {
                    signal_infos.insert(signal_name.clone(), *value);
                }
                message_infos.insert(
                    message_name.clone(),
                    MessageInfo {
                        message: message_name.clone(),
                        cycle_time: 100, // Default cycle time, can be adjusted as needed
                        signals: signal_infos,
                    },
                );
            }
            channels.push(ChannelInfo {
                channel: channel_name.clone(),
                messages: message_infos,
            });
        }

        channels
    }
}
