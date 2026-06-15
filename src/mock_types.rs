use std::time::Duration;
use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::traits::MessageTrait;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MockMessage(pub String);

impl FromStr for MockMessage {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MockMessage(s.to_string()))
    }
}

impl MessageTrait for MockMessage {
    fn raw_id(&self) -> u32 {
        match self.0.as_str() {
            "test1" => 0x100,
            "test2" => 0x200,
            "test3" => 0x300,
            "test4" => 0x400,
            "test5" => 0x500,
            _ => 0,
        }
    }

    fn cycle_time(&self) -> Duration {
        match self.0.as_str() {
            "test1" => Duration::from_millis(100),
            "test2" => Duration::from_millis(200),
            "test3" => Duration::from_millis(300),
            "test4" => Duration::from_millis(400),
            "test5" => Duration::from_millis(500),
            _ => Duration::from_secs(10000),
        }
    }
}

impl fmt::Display for MockMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MockSignal(pub String);

impl FromStr for MockSignal {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MockSignal(s.to_string()))
    }
}

impl fmt::Display for MockSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
