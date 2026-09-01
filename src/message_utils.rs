use std::time::Duration;

pub trait MessageProvider {
    fn to_id(&self) -> u16;
    fn get_cycle_time(&self) -> Duration;
}

pub type MockMessage = String;

impl MessageProvider for MockMessage {
    fn to_id(&self) -> u16 {
        let value = self.as_str();
        match value {
            "test1" => 0x100,
            "test2" => 0x200,
            "test3" => 0x300,
            "test4" => 0x400,
            "test5" => 0x500,
            _ => {
                // If it starts with "test" followed by a number
                if let Some(suffix) = value.strip_prefix("test")
                    && let Ok(num) = suffix.parse::<u16>()
                    && num > 0
                    && num <= 7
                {
                    return num * 0x100;
                }

                // Fallback: Deterministic DJB2 hash mapped to standard CAN ID range 1..=2047 (0x7FF)
                let mut hash: u32 = 5381;
                for b in value.bytes() {
                    hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u32);
                }
                (hash % 2047) as u16 + 1
            }
        }
    }

    fn get_cycle_time(&self) -> Duration {
        let value = self.as_str();
        match value {
            "test1" => Duration::from_millis(100),
            "test2" => Duration::from_millis(200),
            "test3" => Duration::from_millis(300),
            "test4" => Duration::from_millis(400),
            "test5" => Duration::from_millis(500),
            _ => {
                // If it starts with "test" followed by a number
                if let Some(suffix) = value.strip_prefix("test")
                    && let Ok(num) = suffix.parse::<u64>()
                    && num > 0
                    && let Some(milliseconds) = num.checked_mul(100)
                {
                    return Duration::from_millis(milliseconds);
                }

                // Try to parse suffix like "_200ms" or "200ms"
                if let Some(idx) = value.find("ms") {
                    let prefix = &value[..idx];
                    let digits: String = prefix
                        .chars()
                        .rev()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if !digits.is_empty() {
                        let val_str: String = digits.chars().rev().collect();
                        if let Ok(ms) = val_str.parse::<u64>() {
                            return Duration::from_millis(ms);
                        }
                    }
                }

                // Default cycle time for standard messages
                Duration::from_millis(100)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_message_provider() {
        // Test compatibility
        assert_eq!(String::from("test1").to_id(), 0x100);
        assert_eq!(
            String::from("test1").get_cycle_time(),
            Duration::from_millis(100)
        );

        assert_eq!(String::from("test5").to_id(), 0x500);
        assert_eq!(
            String::from("test5").get_cycle_time(),
            Duration::from_millis(500)
        );

        // Test extra test message patterns
        assert_eq!(String::from("test7").to_id(), 0x700);
        assert_eq!(
            String::from("test7").get_cycle_time(),
            Duration::from_millis(700)
        );

        // Test dynamic cycle parsing from name suffix
        assert_eq!(
            String::from("engine_state_250ms").get_cycle_time(),
            Duration::from_millis(250)
        );
        assert_eq!(
            String::from("sensor_readings_50ms").get_cycle_time(),
            Duration::from_millis(50)
        );

        // Test dynamic ID hashing fallback
        let id_custom = String::from("custom_can_msg").to_id();
        assert!(id_custom > 0 && id_custom <= 2047);
        // Deterministic hash check
        assert_eq!(id_custom, String::from("custom_can_msg").to_id());
    }
}
