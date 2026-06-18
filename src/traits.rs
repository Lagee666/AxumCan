use std::collections::HashMap;
use crate::can_sender::CanActor;
use crate::error::Error;

/// Minimal trait for hardware abstraction.
/// This allows the Registry to be decoupled from specific socket or builder logic.
pub trait CanHardware: Send + Sync + 'static {
    fn create_actor(
        &self,
        channel: &str,
        message_name: &str,
        signals: &HashMap<String, u64>,
    ) -> Result<CanActor, Error>;
}
