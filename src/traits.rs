use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use socketcan::CanFdFrame;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;
use std::time::Duration;

/// Trait representing a CAN Signal label.
/// It is a blanket trait over standard Rust types like String or Enums.
pub trait SignalTrait:
    Clone + Eq + Hash + Send + Sync + Display + FromStr + Serialize + DeserializeOwned + Debug + 'static
{
}

impl<T> SignalTrait for T where
    T: Clone
        + Eq
        + Hash
        + Send
        + Sync
        + Display
        + FromStr
        + Serialize
        + DeserializeOwned
        + Debug
        + 'static
{
}

/// Trait representing a CAN Message label.
/// It is a blanket trait over standard Rust types like String or Enums.
pub trait MessageTrait:
    Clone + Eq + Hash + Send + Sync + Display + FromStr + Serialize + DeserializeOwned + Debug + 'static
{
}

impl<T> MessageTrait for T where
    T: Clone
        + Eq
        + Hash
        + Send
        + Sync
        + Display
        + FromStr
        + Serialize
        + DeserializeOwned
        + Debug
        + 'static
{
}

/// Trait for the low-level CAN socket utilities.
#[async_trait]
pub trait SocketUtilsTrait<Signal: SignalTrait>: Send + Sync + 'static {
    fn create_frame(&self, signal_map: HashMap<Signal, f64>) -> CanFdFrame;
    async fn send_can(&self, frame: CanFdFrame);
}

/// Trait for building CAN actors.
/// Production users implement this to inject real hardware support.
pub trait CanBuilderTrait<Signal: SignalTrait, Message: MessageTrait>:
    Default + Send + 'static
{
    fn set_interface(self, interface: &str) -> Self;
    fn set_message_label(self, label: Message) -> Self;
    fn build(self) -> Result<crate::can_sender::CanActor<Signal>, crate::error::Error>;
}
