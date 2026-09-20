use super::parse_options::ParseOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonMode {
    KeyBased,
    RowByRow,
}

impl Default for ComparisonMode {
    fn default() -> Self {
        Self::KeyBased
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareOptions {
    pub file_a_path: String,
    pub file_b_path: String,
    pub file_a_parse_options: ParseOptions,
    pub file_b_parse_options: ParseOptions,
    pub comparison_mode: ComparisonMode,
    pub key_columns: Vec<String>,
    pub excluded_columns: Vec<String>,
    pub trim_whitespace: bool,
    pub ignore_case: bool,
    #[serde(default = "default_numeric_tolerance")]
    pub numeric_tolerance: String,
}

fn default_numeric_tolerance() -> String {
    "0".to_string()
}
