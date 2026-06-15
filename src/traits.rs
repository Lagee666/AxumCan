use std::collections::HashMap;
use std::fmt::{Display, Debug};
use std::hash::Hash;
use std::str::FromStr;
use std::time::Duration;
use async_trait::async_trait;
use socketcan::CanFdFrame;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Trait representing a CAN Signal label.
/// It must be hashable and convertible to/from strings for WebSocket communication.
pub trait SignalTrait:
    Clone + Eq + Hash + Send + Sync + Display + FromStr + Debug + Serialize + DeserializeOwned + 'static
{
}

/// Blanket implementation for types that satisfy the requirements.
impl<T: Clone + Eq + Hash + Send + Sync + Display + FromStr + Debug + Serialize + DeserializeOwned + 'static>
    SignalTrait for T
{
}

/// Trait representing a CAN Message label.
/// It must provide its CAN ID and its expected cycle time.
pub trait MessageTrait: Clone + Send + Sync + FromStr + Debug + 'static {
    fn raw_id(&self) -> u32;
    fn cycle_time(&self) -> Duration;
}

/// Trait for the low-level CAN socket utilities.
#[async_trait]
pub trait SocketUtilsTrait<S: SignalTrait>: Send + Sync + 'static {
    fn create_frame(&self, signal_map: HashMap<S, f64>) -> CanFdFrame;
    async fn send_can(&self, frame: CanFdFrame);
}

/// Trait for building CAN actors.
pub trait CanBuilderTrait<S: SignalTrait, M: MessageTrait>: Default {
    fn set_interface(self, interface: &str) -> Self;
    fn set_message_label(self, label: M) -> Self;
    fn build(self) -> Result<crate::can_sender::CanActor<S>, crate::error::Error>;
}
