use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error due to {0}")]
    Io(#[from] std::io::Error),
    #[error("serde json error due to {0}")]
    Serde(#[from] serde_json::Error),
    #[error("fmt error due to {0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("message label is none")]
    MessageLabelNone,
    #[error("not support can frame type")]
    NotSupportCanFrameType,
    #[error("invalid CAN model: {0}")]
    InvalidModel(String),
    #[error("invalid value for signal {0}: {1}")]
    InvalidValue(String, f64),
    #[error("invalid CAN frame: {0}")]
    InvalidFrame(String),
    #[error("transport error: {0}")]
    Transport(String),
}
