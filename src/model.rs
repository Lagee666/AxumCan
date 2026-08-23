use std::{collections::HashSet, time::Duration};

use crate::error::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanFrame {
    pub id: u32,
    pub is_extended: bool,
    pub data: Vec<u8>,
}

impl CanFrame {
    pub fn new(id: u32, is_extended: bool, data: Vec<u8>) -> Result<Self, Error> {
        let max_id = if is_extended { 0x1fff_ffff } else { 0x7ff };
        if id > max_id {
            return Err(Error::InvalidFrame(format!(
                "CAN ID 0x{id:x} is out of range"
            )));
        }
        if data.len() > 8 {
            return Err(Error::InvalidFrame(format!(
                "classic CAN payload has {} bytes, maximum is 8",
                data.len()
            )));
        }
        Ok(Self {
            id,
            is_extended,
            data,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanModel {
    pub channels: Vec<CanChannel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanChannel {
    pub name: String,
    pub messages: Vec<CanMessage>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanMessage {
    pub name: String,
    pub can_id: u32,
    pub is_extended: bool,
    pub cycle_time: Duration,
    pub signals: Vec<SignalSpec>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignalSpec {
    pub name: String,
    pub start_bit: u8,
    pub bit_length: u8,
    pub byte_order: ByteOrder,
    pub is_signed: bool,
    pub factor: f64,
    pub offset: f64,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub initial_value: f64,
}

impl CanModel {
    pub fn validate(&self) -> Result<(), Error> {
        if self.channels.is_empty() {
            return Err(Error::InvalidModel(
                "at least one CAN channel is required".into(),
            ));
        }
        for channel in &self.channels {
            if channel.name.trim().is_empty() {
                return Err(Error::InvalidModel("channel name cannot be empty".into()));
            }
            for message in &channel.messages {
                message.validate(&channel.name)?;
            }
        }
        Ok(())
    }
}

impl CanMessage {
    pub fn validate(&self, channel: &str) -> Result<(), Error> {
        if self.name.trim().is_empty() {
            return Err(Error::InvalidModel(format!(
                "message on channel {channel} has an empty name"
            )));
        }
        let max_id = if self.is_extended { 0x1fff_ffff } else { 0x7ff };
        if self.can_id > max_id {
            return Err(Error::InvalidModel(format!(
                "message {} on channel {} has invalid CAN ID 0x{:x}",
                self.name, channel, self.can_id
            )));
        }
        if self.cycle_time.is_zero() {
            return Err(Error::InvalidModel(format!(
                "message {} on channel {} has a zero cycle time",
                self.name, channel
            )));
        }
        if self.signals.is_empty() {
            return Err(Error::InvalidModel(format!(
                "message {} on channel {} has no signals",
                self.name, channel
            )));
        }

        let mut occupied = [false; 64];
        let mut names = HashSet::new();
        for signal in &self.signals {
            signal.validate(&self.name, channel)?;
            if !names.insert(&signal.name) {
                return Err(Error::InvalidModel(format!(
                    "message {} on channel {} contains duplicate signal {}",
                    self.name, channel, signal.name
                )));
            }
            for bit in signal.bit_positions() {
                let slot = &mut occupied[bit as usize];
                if *slot {
                    return Err(Error::InvalidModel(format!(
                        "signal {} overlaps another signal in message {} on channel {}",
                        signal.name, self.name, channel
                    )));
                }
                *slot = true;
            }
        }
        Ok(())
    }
}

impl SignalSpec {
    pub fn validate(&self, message: &str, channel: &str) -> Result<(), Error> {
        if self.name.trim().is_empty() {
            return Err(Error::InvalidModel(format!(
                "message {message} on channel {channel} has an empty signal name"
            )));
        }
        if self.bit_length == 0 || self.bit_length > 64 {
            return Err(Error::InvalidModel(format!(
                "signal {} has invalid bit length {}",
                self.name, self.bit_length
            )));
        }
        if self.bit_positions().iter().any(|bit| *bit >= 64) {
            return Err(Error::InvalidModel(format!(
                "signal {} is outside the 8-byte classic CAN payload",
                self.name
            )));
        }
        if !self.factor.is_finite() || self.factor == 0.0 {
            return Err(Error::InvalidModel(format!(
                "signal {} has an invalid factor",
                self.name
            )));
        }
        if !self.offset.is_finite() || !self.initial_value.is_finite() {
            return Err(Error::InvalidModel(format!(
                "signal {} has a non-finite value",
                self.name
            )));
        }
        if let (Some(minimum), Some(maximum)) = (self.minimum, self.maximum)
            && (!minimum.is_finite() || !maximum.is_finite() || minimum > maximum)
        {
            return Err(Error::InvalidModel(format!(
                "signal {} has invalid limits",
                self.name
            )));
        }
        if self
            .minimum
            .is_some_and(|minimum| self.initial_value < minimum)
            || self
                .maximum
                .is_some_and(|maximum| self.initial_value > maximum)
        {
            return Err(Error::InvalidModel(format!(
                "signal {} initial value is outside its limits",
                self.name
            )));
        }
        let raw = (self.initial_value - self.offset) / self.factor;
        if !raw.is_finite() || raw.fract() != 0.0 {
            return Err(Error::InvalidModel(format!(
                "signal {} initial value cannot be represented as an integer raw value",
                self.name
            )));
        }
        if self.is_signed {
            let minimum = -(1i128 << (self.bit_length - 1));
            let maximum = (1i128 << (self.bit_length - 1)) - 1;
            let raw = raw as i128;
            if raw < minimum || raw > maximum {
                return Err(Error::InvalidModel(format!(
                    "signal {} initial value is outside its signed bit range",
                    self.name
                )));
            }
        } else if raw < 0.0 || raw > ((1u128 << self.bit_length) - 1) as f64 {
            return Err(Error::InvalidModel(format!(
                "signal {} initial value is outside its unsigned bit range",
                self.name
            )));
        }
        Ok(())
    }

    pub fn bit_positions(&self) -> Vec<u8> {
        match self.byte_order {
            ByteOrder::LittleEndian => (0..self.bit_length)
                .map(|offset| self.start_bit.saturating_add(offset))
                .collect(),
            ByteOrder::BigEndian => {
                let mut position = self.start_bit;
                let mut positions = Vec::with_capacity(self.bit_length as usize);
                for _ in 0..self.bit_length {
                    positions.push(position);
                    position = if position.is_multiple_of(8) {
                        position.saturating_add(15)
                    } else {
                        position.saturating_sub(1)
                    };
                }
                positions
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(name: &str, start_bit: u8, bit_length: u8) -> SignalSpec {
        SignalSpec {
            name: name.into(),
            start_bit,
            bit_length,
            byte_order: ByteOrder::LittleEndian,
            is_signed: false,
            factor: 1.0,
            offset: 0.0,
            minimum: None,
            maximum: None,
            initial_value: 0.0,
        }
    }

    #[test]
    fn rejects_overlapping_signals() {
        let message = CanMessage {
            name: "Status".into(),
            can_id: 0x100,
            is_extended: false,
            cycle_time: Duration::from_millis(100),
            signals: vec![signal("A", 0, 8), signal("B", 4, 8)],
        };
        assert!(message.validate("vcan0").is_err());
    }

    #[test]
    fn uses_dbc_big_endian_bit_walk() {
        let mut value = signal("A", 7, 16);
        value.byte_order = ByteOrder::BigEndian;
        assert_eq!(
            value.bit_positions(),
            vec![7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8]
        );
    }
}
