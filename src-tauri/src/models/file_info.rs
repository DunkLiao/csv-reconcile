use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub encoding: String,
    pub delimiter: String,
    pub delimiter_char: char,
    pub headers: Vec<String>,
    pub row_count: Option<u64>,
}
