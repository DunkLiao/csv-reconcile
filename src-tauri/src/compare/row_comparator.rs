use crate::compare::value_comparator::{compare_values, parse_tolerance};
use crate::error::AppError;
use crate::models::compare_options::CompareOptions;
use crate::models::compare_result::CompareResult;
use crate::models::difference::{ColumnDifference, Difference, DifferenceType};
use crate::models::progress::ProgressPayload;
use crate::parser::delimited_parser::{open_delimited_reader, resolve_parse_settings};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub fn compare_row_by_row<F>(
    options: &CompareOptions,
    cancel_token: Arc<AtomicBool>,
    mut progress_callback: F,
) -> Result<CompareResult, AppError>
where
    F: FnMut(ProgressPayload),
{
    let start_time = Instant::now();
    let tolerance = parse_tolerance(&options.numeric_tolerance)?;

    // 1. Open readers
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

    let common_columns: Vec<String> = headers_a
        .iter()
        .filter(|h| set_b.contains(h.as_str()))
        .cloned()
        .collect();

    if common_columns.is_empty() {
        return Err(AppError::NoCommonColumns);
    }

    let excluded_set: HashSet<&str> = options
        .excluded_columns
        .iter()
        .map(|s| s.as_str())
        .collect();
    let compared_columns: Vec<String> = common_columns
        .iter()
        .filter(|col| !excluded_set.contains(col.as_str()))
        .cloned()
        .collect();

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

    let mut differences = Vec::new();
    let mut row_num = 0u64;
    let mut matched_records = 0u64;
    let mut same_records = 0u64;
    let mut different_records = 0u64;
    let mut different_cells = 0u64;
    let mut a_only_records = 0u64;
    let mut b_only_records = 0u64;

    let mut record_a = csv::StringRecord::new();
    let mut record_b = csv::StringRecord::new();

    progress_callback(ProgressPayload {
        stage: "依資料列順序比對中...".to_string(),
        processed_a: 0,
        processed_b: 0,
        total_a: None,
        total_b: None,
        percent: Some(10),
        message: "正在逐列比對...".to_string(),
    });

    loop {
        row_num += 1;

        if row_num % 5000 == 0 {
            if cancel_token.load(Ordering::Relaxed) {
                return Err(AppError::Cancelled);
            }
            progress_callback(ProgressPayload {
                stage: "依資料列順序比對中...".to_string(),
                processed_a: row_num,
                processed_b: row_num,
                total_a: None,
                total_b: None,
                percent: None,
                message: format!("已比對至第 {} 列", row_num),
            });
        }

        let has_a = rdr_a.read_record(&mut record_a)?;
        let has_b = rdr_b.read_record(&mut record_b)?;

        if !has_a && !has_b {
            break;
        }

        if has_a && has_b {
            matched_records += 1;
            let mut row_has_diff = false;

            for col in &compared_columns {
                let idx_a = header_idx_a.get(col).copied().unwrap_or(usize::MAX);
                let idx_b = header_idx_b.get(col).copied().unwrap_or(usize::MAX);

                let val_a = if idx_a < record_a.len() {
                    record_a.get(idx_a).unwrap_or("")
                } else {
                    ""
                };
                let val_b = if idx_b < record_b.len() {
                    record_b.get(idx_b).unwrap_or("")
                } else {
                    ""
                };

                if compare_values(
                    val_a,
                    val_b,
                    options.trim_whitespace,
                    options.ignore_case,
                    &tolerance,
                ) {
                    row_has_diff = true;
                    different_cells += 1;
                    differences.push(Difference {
                        key_values: vec![],
                        row_a: Some(row_num),
                        row_b: Some(row_num),
                        column_name: Some(col.clone()),
                        value_a: Some(val_a.to_string()),
                        value_b: Some(val_b.to_string()),
                        difference_type: DifferenceType::ValueChanged,
                    });
                }
            }

            if row_has_diff {
                different_records += 1;
            } else {
                same_records += 1;
            }
        } else if has_a {
            a_only_records += 1;
            differences.push(Difference {
                key_values: vec![],
                row_a: Some(row_num),
                row_b: None,
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::AOnly,
            });
        } else {
            b_only_records += 1;
            differences.push(Difference {
                key_values: vec![],
                row_a: None,
                row_b: Some(row_num),
                column_name: None,
                value_a: None,
                value_b: None,
                difference_type: DifferenceType::BOnly,
            });
        }
    }

    let rows_a = matched_records + a_only_records;
    let rows_b = matched_records + b_only_records;
    let identical = differences.is_empty();
    let duration_ms = start_time.elapsed().as_millis() as u64;

    progress_callback(ProgressPayload {
        stage: "完成".to_string(),
        processed_a: rows_a,
        processed_b: rows_b,
        total_a: Some(rows_a),
        total_b: Some(rows_b),
        percent: Some(100),
        message: "比對已完成！".to_string(),
    });

    Ok(CompareResult {
        identical,
        rows_a,
        rows_b,
        key_columns: vec![],
        compared_columns,
        excluded_columns: options.excluded_columns.clone(),
        matched_records,
        same_records,
        different_records,
        a_only_records,
        b_only_records,
        duplicate_keys_a: 0,
        duplicate_keys_b: 0,
        different_cells,
        differences,
        column_differences,
        duplicate_key_records: vec![],
        duration_ms,
    })
}
