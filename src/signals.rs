use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Signals {
    pub vcan1: HashMap<String, HashMap<String, u64>>,
    pub vcan2: HashMap<String, HashMap<String, u64>>,
    pub vcan3: HashMap<String, HashMap<String, u64>>,
    pub vcan4: HashMap<String, HashMap<String, u64>>,
    pub vcan5: HashMap<String, HashMap<String, u64>>,
    pub vcan6: HashMap<String, HashMap<String, u64>>,
}

impl Signals {
    pub async fn init(path: &Path) -> Result<Self, Error> {
        let content = tokio::fs::read_to_string(path).await?;
        serde_json::from_str(&content).map_err(|err| err.into())
    }
}
