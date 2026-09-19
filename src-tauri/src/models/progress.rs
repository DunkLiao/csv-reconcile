use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPayload {
    pub stage: String,
    pub processed_a: u64,
    pub processed_b: u64,
    pub total_a: Option<u64>,
    pub total_b: Option<u64>,
    pub percent: Option<u8>,
    pub message: String,
}
