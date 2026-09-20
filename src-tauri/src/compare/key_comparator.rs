use crate::compare::duplicate_detector::DuplicateDetector;
use crate::compare::key_builder::KeyBuilder;
use crate::compare::value_comparator::{compare_values, parse_tolerance};
use crate::error::AppError;
use crate::models::compare_options::CompareOptions;
use crate::models::compare_result::CompareResult;
use crate::models::composite_key::CompositeKey;
use crate::models::difference::{ColumnDifference, Difference, DifferenceType, KeyValue};
use crate::models::progress::ProgressPayload;
use crate::parser::delimited_parser::{open_delimited_reader, resolve_parse_settings};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct StoredRecordA {
    pub row: u64,
    pub values_by_col: HashMap<String, String>,
    pub key: CompositeKey,
    pub matched: bool,
    pub is_duplicate: bool,
}

pub fn compare_key_based<F>(
    options: &CompareOptions,
    cancel_token: Arc<AtomicBool>,
    mut progress_callback: F,
) -> Result<CompareResult, AppError>
where
    F: FnMut(ProgressPayload),
{
    let start_time = Instant::now();
    let tolerance = parse_tolerance(&options.numeric_tolerance)?;

    // 1. Open readers for File A and File B
    let (enc_a, skip_a, delim_a) =
        resolve_parse_settings(&options.file_a_path, &options.file_a_parse_options)?;
    let mut rdr_a = open_delimited_reader(&options.file_a_path, enc_a, skip_a, delim_a)?;

    let (enc_b, skip_b, delim_b) =
        resolve_parse_settings(&options.file_b_path, &options.file_b_parse_options)?;
    let mut rdr_b = open_delimited_reader(&options.file_b_path, enc_b, skip_b, delim_b)?;

    // 2. Read headers
    let headers_a: Vec<String> = rdr_a.headers()?.iter().map(|s| s.to_string()).collect();
    let headers_b: Vec<String> = rdr_b.headers()?.iter().map(|s| s.to_string()).collect();

    let set_a: HashSet<&str> = headers_a.iter().map(|s| s.as_str()).collect();
    let set_b: HashSet<&str> = headers_b.iter().map(|s| s.as_str()).collect();

    // Column differences
    let mut column_differences = Vec::new();
    let mut all_unique_headers = Vec::new();
    for h in &headers_a {
        if !all_unique_headers.contains(h) {
            all_unique_headers.push(h.clone());
        }
    }
    for h in &headers_b {
        if !all_unique_headers.contains(h) {
            all_unique_headers.push(h.clone());
        }
    }

    for col in &all_unique_headers {
        let in_a = set_a.contains(col.as_str());
        let in_b = set_b.contains(col.as_str());
        let status = if in_a && in_b {
            "SAME"
        } else if in_a {
            "A_ONLY"
        } else {
            "B_ONLY"
        };
        column_differences.push(ColumnDifference {
            column: col.clone(),
            file_a: in_a,
            file_b: in_b,
            status: status.to_string(),
        });
    }

    // Common columns
    let common_columns: Vec<String> = headers_a
        .iter()
        .filter(|h| set_b.contains(h.as_str()))
        .cloned()
        .collect();

    if common_columns.is_empty() {
        return Err(AppError::NoCommonColumns);
    }

    // Determine ordered key columns (preserving File A header order)
    let ordered_key_columns: Vec<String> = headers_a
        .iter()
        .filter(|h| options.key_columns.contains(h))
        .cloned()
        .collect();

    if ordered_key_columns.is_empty() {
        return Err(AppError::NoKeySpecified);
    }

    // Compared columns = common columns minus excluded columns and minus key columns
    let excluded_set: HashSet<&str> = options
        .excluded_columns
        .iter()
        .map(|s| s.as_str())
        .collect();
    let key_set: HashSet<&str> = ordered_key_columns.iter().map(|s| s.as_str()).collect();

    let compared_columns: Vec<String> = common_columns
        .iter()
        .filter(|col| !key_set.contains(col.as_str()) && !excluded_set.contains(col.as_str()))
        .cloned()
        .collect();

    let key_builder_a = KeyBuilder::new(&headers_a, &ordered_key_columns)?;
    let key_builder_b = KeyBuilder::new(&headers_b, &ordered_key_columns)?;

    let mut dup_detector_a = DuplicateDetector::new();
    let mut dup_detector_b = DuplicateDetector::new();

    let mut differences: Vec<Difference> = Vec::new();
    let mut index_a: HashMap<CompositeKey, StoredRecordA> = HashMap::new();

    // Map column names to index in File A and B
    let header_idx_a: HashMap<String, usize> = headers_a
        .iter()
        .enumerate()
        .map(|(i, h)| (h.clone(), i))
        .collect();
    let header_idx_b: HashMap<String, usize> = headers_b
        .iter()
        .enumerate()
        .map(|(i, h)| (h.clone(), i))
        .collect();

    // 3. Scan File A and build index
    progress_callback(ProgressPayload {
        stage: "讀取檔案 A 並建立索引...".to_string(),
        processed_a: 0,
        processed_b: 0,
        total_a: None,
        total_b: None,
        percent: Some(10),
        message: "正在載入檔案 A...".to_string(),
    });

    let mut row_a = 0u64;
    let mut raw_record = csv::StringRecord::new();

    while rdr_a.read_record(&mut raw_record)? {
        row_a += 1;

        if row_a % 5000 == 0 {
            if cancel_token.load(Ordering::Relaxed) {
                return Err(AppError::Cancelled);
            }
            progress_callback(ProgressPayload {
                stage: "讀取檔案 A 並建立索引...".to_string(),
                processed_a: row_a,
                processed_b: 0,
                total_a: None,
                total_b: None,
                percent: Some(25),
                message: format!("已讀取檔案 A 共 {} 筆資料", row_a),
            });
        }

        let key = key_builder_a.build_key(&raw_record);
        let key_vals: Vec<KeyValue> = ordered_key_columns
            .iter()
            .zip(&key.0)
            .map(|(c, v)| KeyValue {
                column: c.clone(),
                value: v.clone(),
            })
            .collect();

        // 空 Key / 不完整 Key 只回報一次，且不參與配對、重複偵測與 A Only 統計
        if key.is_empty() {
            differences.push(Difference {
                key_values: key_vals.clone(),
                row_a: Some(row_a),
                row_b: None,
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::EmptyKeyA,
            });
            continue;
        }
        if key.is_incomplete() {
            differences.push(Difference {
                key_values: key_vals.clone(),
                row_a: Some(row_a),
                row_b: None,
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::IncompleteKeyA,
            });
            continue;
        }

        let is_dup = dup_detector_a.record_key(key.clone(), row_a);
        if is_dup {
            if let Some(existing) = index_a.get_mut(&key) {
                existing.is_duplicate = true;
            }
            continue;
        }

        let mut values_by_col = HashMap::with_capacity(compared_columns.len());
        for col in &compared_columns {
            if let Some(&idx) = header_idx_a.get(col) {
                values_by_col.insert(col.clone(), raw_record.get(idx).unwrap_or("").to_string());
            }
        }

        index_a.insert(
            key.clone(),
            StoredRecordA {
                row: row_a,
                values_by_col,
                key,
                matched: false,
                is_duplicate: false,
            },
        );
    }

    // 4. Stream File B and compare
    progress_callback(ProgressPayload {
        stage: "串流比對檔案 B...".to_string(),
        processed_a: row_a,
        processed_b: 0,
        total_a: Some(row_a),
        total_b: None,
        percent: Some(50),
        message: "正在比對檔案 B...".to_string(),
    });

    let mut row_b = 0u64;
    let mut matched_records = 0u64;
    let mut same_records = 0u64;
    let mut different_records = 0u64;
    let mut different_cells = 0u64;
    let mut b_only_records = 0u64;

    while rdr_b.read_record(&mut raw_record)? {
        row_b += 1;

        if row_b % 5000 == 0 {
            if cancel_token.load(Ordering::Relaxed) {
                return Err(AppError::Cancelled);
            }
            progress_callback(ProgressPayload {
                stage: "串流比對檔案 B...".to_string(),
                processed_a: row_a,
                processed_b: row_b,
                total_a: Some(row_a),
                total_b: None,
                percent: Some(75),
                message: format!("已比對檔案 B 共 {} 筆資料", row_b),
            });
        }

        let key = key_builder_b.build_key(&raw_record);
        let key_vals: Vec<KeyValue> = ordered_key_columns
            .iter()
            .zip(&key.0)
            .map(|(c, v)| KeyValue {
                column: c.clone(),
                value: v.clone(),
            })
            .collect();

        // 空 Key / 不完整 Key 只回報一次，且不參與配對、重複偵測與 B Only 統計
        if key.is_empty() {
            differences.push(Difference {
                key_values: key_vals.clone(),
                row_a: None,
                row_b: Some(row_b),
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::EmptyKeyB,
            });
            continue;
        }
        if key.is_incomplete() {
            differences.push(Difference {
                key_values: key_vals.clone(),
                row_a: None,
                row_b: Some(row_b),
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::IncompleteKeyB,
            });
            continue;
        }

        let is_dup_b = dup_detector_b.record_key(key.clone(), row_b);
        if is_dup_b {
            continue;
        }

        // Look up key in File A index
        if let Some(record_a) = index_a.get_mut(&key) {
            if record_a.is_duplicate {
                // Key is duplicate in File A, cannot pair uniquely
                continue;
            }

            record_a.matched = true;
            matched_records += 1;

            let mut record_has_diff = false;

            for col in &compared_columns {
                let val_a = record_a
                    .values_by_col
                    .get(col)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let b_idx = header_idx_b.get(col).copied().unwrap_or(usize::MAX);
                let val_b = if b_idx < raw_record.len() {
                    raw_record.get(b_idx).unwrap_or("")
                } else {
                    ""
                };

                let is_different = compare_values(
                    val_a,
                    val_b,
                    options.trim_whitespace,
                    options.ignore_case,
                    &tolerance,
                );

                if is_different {
                    record_has_diff = true;
                    different_cells += 1;
                    differences.push(Difference {
                        key_values: key_vals.clone(),
                        row_a: Some(record_a.row),
                        row_b: Some(row_b),
                        column_name: Some(col.clone()),
                        value_a: Some(val_a.to_string()),
                        value_b: Some(val_b.to_string()),
                        difference_type: DifferenceType::ValueChanged,
                    });
                }
            }

            if record_has_diff {
                different_records += 1;
            } else {
                same_records += 1;
            }
        } else {
            // Not found in A => B_ONLY
            b_only_records += 1;
            differences.push(Difference {
                key_values: key_vals.clone(),
                row_a: None,
                row_b: Some(row_b),
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::BOnly,
            });
        }
    }

    // 5. Identify A_ONLY records (unmatched and non-duplicate)
    let mut a_only_records = 0u64;
    for (_key, record_a) in index_a.iter() {
        if !record_a.matched && !record_a.is_duplicate {
            a_only_records += 1;
            let key_vals: Vec<KeyValue> = ordered_key_columns
                .iter()
                .zip(&record_a.key.0)
                .map(|(c, v)| KeyValue {
                    column: c.clone(),
                    value: v.clone(),
                })
                .collect();

            differences.push(Difference {
                key_values: key_vals,
                row_a: Some(record_a.row),
                row_b: None,
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::AOnly,
            });
        }
    }

    // 6. Gather Duplicate Key Records
    let duplicate_key_records_a =
        dup_detector_a.get_duplicate_records("File A", &ordered_key_columns);
    let duplicate_key_records_b =
        dup_detector_b.get_duplicate_records("File B", &ordered_key_columns);

    let dup_count_a = duplicate_key_records_a.len() as u64;
    let dup_count_b = duplicate_key_records_b.len() as u64;

    for dup in &duplicate_key_records_a {
        differences.push(Difference {
            key_values: dup.key_values.clone(),
            row_a: dup.rows.first().copied(),
            row_b: None,
            column_name: None,
            value_a: None,
            value_b: None,
            difference_type: DifferenceType::DuplicateKeyA,
        });
    }

    for dup in &duplicate_key_records_b {
        differences.push(Difference {
            key_values: dup.key_values.clone(),
            row_a: None,
            row_b: dup.rows.first().copied(),
            column_name: None,
            value_a: None,
            value_b: None,
            difference_type: DifferenceType::DuplicateKeyB,
        });
    }

    let mut duplicate_key_records = duplicate_key_records_a;
    duplicate_key_records.extend(duplicate_key_records_b);

    // Sort differences: group by row_a, row_b for clean display
    differences.sort_by(|a, b| {
        let ra = a.row_a.or(a.row_b).unwrap_or(0);
        let rb = b.row_a.or(b.row_b).unwrap_or(0);
        ra.cmp(&rb)
    });

    let identical = differences.is_empty()
        && dup_count_a == 0
        && dup_count_b == 0
        && a_only_records == 0
        && b_only_records == 0
        && different_records == 0;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    progress_callback(ProgressPayload {
        stage: "完成".to_string(),
        processed_a: row_a,
        processed_b: row_b,
        total_a: Some(row_a),
        total_b: Some(row_b),
        percent: Some(100),
        message: "比對已完成！".to_string(),
    });

    Ok(CompareResult {
        identical,
        rows_a: row_a,
        rows_b: row_b,
        key_columns: ordered_key_columns,
        compared_columns,
        excluded_columns: options.excluded_columns.clone(),
        matched_records,
        same_records,
        different_records,
        a_only_records,
        b_only_records,
        duplicate_keys_a: dup_count_a,
        duplicate_keys_b: dup_count_b,
        different_cells,
        differences,
        column_differences,
        duplicate_key_records,
        duration_ms,
    })
}
