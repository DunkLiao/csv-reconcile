use super::difference::{ColumnDifference, Difference, DuplicateKeyRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    pub identical: bool,
    pub rows_a: u64,
    pub rows_b: u64,

    pub key_columns: Vec<String>,
    pub compared_columns: Vec<String>,
    pub excluded_columns: Vec<String>,

    pub matched_records: u64,
    pub same_records: u64,
    pub different_records: u64,

    pub a_only_records: u64,
    pub b_only_records: u64,

    pub duplicate_keys_a: u64,
    pub duplicate_keys_b: u64,

    pub different_cells: u64,

    pub differences: Vec<Difference>,
    pub column_differences: Vec<ColumnDifference>,
    pub duplicate_key_records: Vec<DuplicateKeyRecord>,

    pub duration_ms: u64,
}
