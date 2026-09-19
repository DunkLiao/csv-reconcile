use crate::models::composite_key::CompositeKey;
use crate::models::difference::{DuplicateKeyRecord, KeyValue};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DuplicateDetector {
    /// key -> list of logical row numbers (1-indexed)
    key_rows: HashMap<CompositeKey, Vec<u64>>,
}

impl DuplicateDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records row number for key. Returns true if this is a duplicate occurrence (count > 1).
    pub fn record_key(&mut self, key: CompositeKey, row: u64) -> bool {
        let entry = self.key_rows.entry(key).or_default();
        entry.push(row);
        entry.len() > 1
    }

    pub fn is_duplicate(&self, key: &CompositeKey) -> bool {
        self.key_rows.get(key).map(|v| v.len() > 1).unwrap_or(false)
    }

    pub fn get_duplicate_records(
        &self,
        source: &str,
        key_columns: &[String],
    ) -> Vec<DuplicateKeyRecord> {
        let mut results = Vec::new();
        for (key, rows) in &self.key_rows {
            if rows.len() > 1 {
                let key_values = key_columns
                    .iter()
                    .zip(&key.0)
                    .map(|(col, val)| KeyValue {
                        column: col.clone(),
                        value: val.clone(),
                    })
                    .collect();

                results.push(DuplicateKeyRecord {
                    source: source.to_string(),
                    key_values,
                    count: rows.len(),
                    rows: rows.clone(),
                });
            }
        }
        // Sort by first row number for deterministic output
        results.sort_by_key(|r| r.rows.first().copied().unwrap_or(0));
        results
    }
}
