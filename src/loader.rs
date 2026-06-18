use std::path::Path;
use crate::{
    error::Error,
    registry::Registry,
    signals::Signals,
    traits::CanHardware,
};

pub async fn setup_registry(
    registry: &mut Registry,
    hardware: &dyn CanHardware,
    config_path: &Path,
) -> Result<(), Error> {
    let signals = Signals::init(config_path).await?;
    registry.set_initial_signals(signals.clone());

    // Loop through all channels and register actors
    let channels = [
        ("vcan1", &signals.vcan1),
        ("vcan2", &signals.vcan2),
        ("vcan3", &signals.vcan3),
        ("vcan4", &signals.vcan4),
        ("vcan5", &signals.vcan5),
        ("vcan6", &signals.vcan6),
    ];

    for (channel_name, messages) in channels {
        for (msg_name, signal_map) in messages {
            let actor = hardware.create_actor(channel_name, msg_name, signal_map)?;
            
            // Map each signal in the message to this actor in the registry
            for signal_name in signal_map.keys() {
                registry.add_actor(signal_name.clone(), actor.clone());
            }
        }
    }

    Ok(())
}
