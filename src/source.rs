use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    error::Error,
    message_utils::MessageProvider,
    model::{ByteOrder, CanChannel, CanMessage, CanModel, SignalSpec},
    signals::Signals,
};

#[async_trait]
pub trait CanModelSource: Send + Sync {
    async fn load(&self) -> Result<CanModel, Error>;
}

pub struct JsonModelSource {
    path: PathBuf,
}

impl JsonModelSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Debug, Deserialize)]
struct ExplicitConfig {
    channels: HashMap<String, ExplicitChannel>,
}

#[derive(Debug, Deserialize)]
struct ExplicitChannel {
    messages: HashMap<String, ExplicitMessage>,
}

#[derive(Debug, Deserialize)]
struct ExplicitMessage {
    can_id: u32,
    #[serde(default)]
    is_extended: bool,
    cycle_time_ms: u64,
    signals: HashMap<String, ExplicitSignal>,
}

#[derive(Debug, Deserialize)]
struct ExplicitSignal {
    start_bit: u8,
    bit_length: u8,
    #[serde(default = "default_byte_order")]
    byte_order: String,
    #[serde(default)]
    is_signed: bool,
    #[serde(default = "default_factor")]
    factor: f64,
    #[serde(default)]
    offset: f64,
    minimum: Option<f64>,
    maximum: Option<f64>,
    initial_value: f64,
}

fn default_byte_order() -> String {
    "little_endian".into()
}
fn default_factor() -> f64 {
    1.0
}

#[async_trait]
impl CanModelSource for JsonModelSource {
    async fn load(&self) -> Result<CanModel, Error> {
        load_json(&self.path).await
    }
}

pub async fn load_json(path: &Path) -> Result<CanModel, Error> {
    let content = tokio::fs::read_to_string(path).await?;
    if let Ok(config) = serde_json::from_str::<ExplicitConfig>(&content) {
        return explicit_model(config);
    }
    let legacy: Signals = serde_json::from_str(&content)?;
    Ok(legacy_model(legacy))
}

pub fn legacy_signals(model: &CanModel) -> Signals {
    let mut signals = Signals::default();
    for channel in &model.channels {
        let target = signals.channels.entry(channel.name.clone()).or_default();
        for message in &channel.messages {
            target.insert(
                message.name.clone(),
                message
                    .signals
                    .iter()
                    .map(|signal| (signal.name.clone(), signal.initial_value as u64))
                    .collect(),
            );
        }
    }
    signals
}

fn explicit_model(config: ExplicitConfig) -> Result<CanModel, Error> {
    let mut channels = Vec::new();
    for (channel_name, channel) in config.channels {
        let messages = channel
            .messages
            .into_iter()
            .map(|(name, message)| {
                let signals = message
                    .signals
                    .into_iter()
                    .map(|(signal_name, signal)| {
                        let byte_order = match signal.byte_order.as_str() {
                            "little" | "little_endian" => ByteOrder::LittleEndian,
                            "big" | "big_endian" => ByteOrder::BigEndian,
                            other => {
                                return Err(Error::InvalidModel(format!(
                                    "unsupported byte order {other}"
                                )));
                            }
                        };
                        Ok(SignalSpec {
                            name: signal_name,
                            start_bit: signal.start_bit,
                            bit_length: signal.bit_length,
                            byte_order,
                            is_signed: signal.is_signed,
                            factor: signal.factor,
                            offset: signal.offset,
                            minimum: signal.minimum,
                            maximum: signal.maximum,
                            initial_value: signal.initial_value,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(CanMessage {
                    name,
                    can_id: message.can_id,
                    is_extended: message.is_extended,
                    cycle_time: Duration::from_millis(message.cycle_time_ms),
                    signals,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        channels.push(CanChannel {
            name: channel_name,
            messages,
        });
    }
    let model = CanModel { channels };
    model.validate()?;
    Ok(model)
}

fn legacy_model(signals: Signals) -> CanModel {
    let channels = signals
        .channels
        .into_iter()
        .map(|(channel, messages)| CanChannel {
            name: channel,
            messages: messages
                .into_iter()
                .map(|(name, values)| {
                    let mut values = values.into_iter().collect::<Vec<_>>();
                    values.sort_by(|left, right| left.0.cmp(&right.0));
                    CanMessage {
                        can_id: name.to_id() as u32,
                        is_extended: false,
                        cycle_time: name.get_cycle_time(),
                        name,
                        signals: values
                            .into_iter()
                            .enumerate()
                            .map(|(index, (name, value))| SignalSpec {
                                name,
                                start_bit: u8::try_from(index.saturating_mul(8)).unwrap_or(64),
                                bit_length: 8,
                                byte_order: ByteOrder::LittleEndian,
                                is_signed: false,
                                factor: 1.0,
                                offset: 0.0,
                                minimum: None,
                                maximum: None,
                                initial_value: value as f64,
                            })
                            .collect(),
                    }
                })
                .collect(),
        })
        .collect();
    CanModel { channels }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn loads_legacy_json_into_explicit_model() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"{{"vcan1":{{"test1":{{"Speed":0}}}},"vcan2":{{}},"vcan3":{{}},"vcan4":{{}},"vcan5":{{}},"vcan6":{{}}}}"#).unwrap();
        let model = load_json(file.path()).await.unwrap();
        let message = model
            .channels
            .iter()
            .find_map(|channel| channel.messages.first())
            .unwrap();
        assert_eq!(message.can_id, 0x100);
        assert_eq!(message.signals[0].bit_length, 8);
    }

    #[tokio::test]
    async fn loads_explicit_json_into_the_same_canonical_model() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"{{"channels":{{"vcan1":{{"messages":{{"Status":{{"can_id":256,"cycle_time_ms":100,"signals":{{"Speed":{{"start_bit":0,"bit_length":8,"initial_value":42}}}}}}}}}}}}}}"#
        )
        .unwrap();
        let model = load_json(file.path()).await.unwrap();
        let message = &model.channels[0].messages[0];
        assert_eq!(model.channels[0].name, "vcan1");
        assert_eq!(message.can_id, 0x100);
        assert_eq!(message.signals[0].initial_value, 42.0);
    }
}
