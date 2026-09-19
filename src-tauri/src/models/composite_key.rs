use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompositeKey(pub Vec<String>);

impl CompositeKey {
    pub fn new(values: Vec<String>) -> Self {
        Self(values)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|v| v.is_empty())
    }

    pub fn is_incomplete(&self) -> bool {
        !self.is_empty() && self.0.iter().any(|v| v.is_empty())
    }

    pub fn to_display_string(&self) -> String {
        self.0.join(" | ")
    }
}
