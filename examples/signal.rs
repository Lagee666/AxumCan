use std::{collections::HashMap, path::Path};

use axum_can::{
    error::Error,
    signals::{ChannelInfo, MessageInfo},
};
use serde::{Deserialize, Serialize};

fn main() {}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Signals {
    pub vcan1: HashMap<String, HashMap<String, u64>>,
    pub vcan2: HashMap<String, HashMap<String, u64>>,
    pub vcan3: HashMap<String, HashMap<String, u64>>,
    pub vcan4: HashMap<String, HashMap<String, u64>>,
    pub vcan5: HashMap<String, HashMap<String, u64>>,
    pub vcan6: HashMap<String, HashMap<String, u64>>,
}

impl Signals {
    pub async fn init(path: &Path) -> Result<Self, Error> {
        let content = tokio::fs::read_to_string(path).await?;
        serde_json::from_str(&content).map_err(|err| err.into())
    }

    pub fn to_channel_info(&self) -> Vec<ChannelInfo> {
        let mut channels = Vec::new();

        for (channel_name, messages) in [
            ("vcan1", &self.vcan1),
            ("vcan2", &self.vcan2),
            ("vcan3", &self.vcan3),
            ("vcan4", &self.vcan4),
            ("vcan5", &self.vcan5),
            ("vcan6", &self.vcan6),
        ] {
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
                channel: channel_name.to_string(),
                messages: message_infos,
            });
        }

        channels
    }
}
