use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyValue {
    pub column: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DifferenceType {
    ValueChanged,
    AOnly,
    BOnly,
    ColumnAOnly,
    ColumnBOnly,
    DuplicateKeyA,
    DuplicateKeyB,
    EmptyKeyA,
    EmptyKeyB,
    IncompleteKeyA,
    IncompleteKeyB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    pub key_values: Vec<KeyValue>,
    pub row_a: Option<u64>,
    pub row_b: Option<u64>,
    pub column_name: Option<String>,
    pub value_a: Option<String>,
    pub value_b: Option<String>,
    pub difference_type: DifferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateKeyRecord {
    pub source: String,
    pub key_values: Vec<KeyValue>,
    pub count: usize,
    pub rows: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDifference {
    pub column: String,
    pub file_a: bool,
    pub file_b: bool,
    pub status: String,
}
