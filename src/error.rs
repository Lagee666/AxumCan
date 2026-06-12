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
}
