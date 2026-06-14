use std::time::Duration;

pub trait MessageUtils {
    fn to_id(&self) -> u16;
    fn get_cycle_time(&self) -> Duration;
}

pub type MockMessage = String;

impl MessageUtils for MockMessage {
    fn to_id(&self) -> u16 {
        let value = self.as_str();
        match value {
            "test1" => 0x100,
            "test2" => 0x200,
            "test3" => 0x300,
            "test4" => 0x400,
            "test5" => 0x500,
            _ => 0,
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
            _ => Duration::from_secs(10000),
        }
    }
}
