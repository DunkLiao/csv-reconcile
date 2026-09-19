use crate::error::AppError;
use crate::models::composite_key::CompositeKey;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KeyBuilder {
    /// Ordered key column names (follows File A header order)
    pub key_columns: Vec<String>,
    /// Column indices in this specific file's headers
    indices: Vec<usize>,
}

impl KeyBuilder {
    pub fn new(file_headers: &[String], ordered_key_columns: &[String]) -> Result<Self, AppError> {
        let header_map: HashMap<&str, usize> = file_headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.as_str(), i))
            .collect();

        let mut indices = Vec::with_capacity(ordered_key_columns.len());
        for key in ordered_key_columns {
            match header_map.get(key.as_str()) {
                Some(&idx) => indices.push(idx),
                None => return Err(AppError::MissingKeyColumn(key.clone())),
            }
        }

        Ok(Self {
            key_columns: ordered_key_columns.to_vec(),
            indices,
        })
    }

    pub fn build_key(&self, record: &csv::StringRecord) -> CompositeKey {
        let values: Vec<String> = self
            .indices
            .iter()
            .map(|&idx| record.get(idx).unwrap_or("").to_string())
            .collect();
        CompositeKey::new(values)
    }
}
