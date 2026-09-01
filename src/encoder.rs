use crate::{
    error::Error,
    model::{ByteOrder, CanFrame, CanMessage, SignalSpec},
};

pub fn encode_message(
    message: &CanMessage,
    values: &std::collections::HashMap<String, f64>,
) -> Result<CanFrame, Error> {
    message.validate("<runtime>")?;
    let mut data = vec![0u8; 8];
    for signal in &message.signals {
        let value = values
            .get(&signal.name)
            .copied()
            .unwrap_or(signal.initial_value);
        encode_signal(signal, value, &mut data)?;
    }
    CanFrame::new(message.can_id, message.is_extended, data)
}

pub fn encode_signal(signal: &SignalSpec, value: f64, data: &mut [u8]) -> Result<(), Error> {
    if !value.is_finite()
        || signal.minimum.is_some_and(|minimum| value < minimum)
        || signal.maximum.is_some_and(|maximum| value > maximum)
    {
        return Err(Error::InvalidValue(signal.name.clone(), value));
    }
    let raw_float = (value - signal.offset) / signal.factor;
    if !raw_float.is_finite() || raw_float.fract() != 0.0 {
        return Err(Error::InvalidValue(signal.name.clone(), value));
    }
    let raw = if signal.is_signed {
        let minimum = -(1i128 << (signal.bit_length - 1));
        let maximum = (1i128 << (signal.bit_length - 1)) - 1;
        let raw = raw_float as i128;
        if raw < minimum || raw > maximum {
            return Err(Error::InvalidValue(signal.name.clone(), value));
        }
        if raw < 0 {
            ((1i128 << signal.bit_length) + raw) as u128
        } else {
            raw as u128
        }
    } else {
        let maximum = (1u128 << signal.bit_length) - 1;
        let raw = raw_float as i128;
        if raw < 0 || raw as u128 > maximum {
            return Err(Error::InvalidValue(signal.name.clone(), value));
        }
        raw as u128
    };

    for (offset, bit_position) in signal.bit_positions().into_iter().enumerate() {
        let raw_bit = match signal.byte_order {
            ByteOrder::LittleEndian => offset,
            ByteOrder::BigEndian => signal.bit_length as usize - offset - 1,
        };
        let bit = ((raw >> raw_bit) & 1) as u8;
        let byte = &mut data[(bit_position / 8) as usize];
        let mask = 1u8 << (bit_position % 8);
        if bit == 1 {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn message(signal: SignalSpec) -> CanMessage {
        CanMessage {
            name: "Status".into(),
            can_id: 0x100,
            is_extended: false,
            cycle_time: Duration::from_millis(100),
            signals: vec![signal],
        }
    }

    fn signal(start_bit: u8, bit_length: u8) -> SignalSpec {
        SignalSpec {
            name: "Speed".into(),
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
    fn encodes_two_byte_little_endian_signal() {
        let frame = encode_message(
            &message(signal(0, 16)),
            &[("Speed".into(), 0x1234 as f64)].into(),
        )
        .unwrap();
        assert_eq!(frame.data, vec![0x34, 0x12, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn encodes_scaled_value() {
        let mut signal = signal(0, 16);
        signal.factor = 0.01;
        let frame = encode_message(&message(signal), &[("Speed".into(), 12.34)].into()).unwrap();
        assert_eq!(frame.data[..2], [0xd2, 0x04]);
    }

    #[test]
    fn encodes_signed_negative_value() {
        let mut signal = signal(0, 8);
        signal.is_signed = true;
        let frame = encode_message(&message(signal), &[("Speed".into(), -1.0)].into()).unwrap();
        assert_eq!(frame.data[0], 0xff);
    }

    #[test]
    fn encodes_dbc_big_endian_signal() {
        let mut signal = signal(7, 16);
        signal.byte_order = ByteOrder::BigEndian;
        let frame =
            encode_message(&message(signal), &[("Speed".into(), 0x1234 as f64)].into()).unwrap();
        assert_eq!(frame.data[..2], [0x12, 0x34]);
    }
}
