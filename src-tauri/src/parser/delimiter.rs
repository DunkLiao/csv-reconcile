use crate::error::AppError;
use std::io::Cursor;

pub const CANDIDATE_DELIMITERS: &[u8] = &[b',', b';', b'\t', b'|', b':'];

pub fn detect_delimiter(sample_utf8_text: &str) -> Result<char, AppError> {
    if sample_utf8_text.trim().is_empty() {
        return Ok(',');
    }

    let mut best_delimiter = ',';
    let mut best_score = -1000.0;

    for &delim_byte in CANDIDATE_DELIMITERS {
        let delim_char = delim_byte as char;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(delim_byte)
            .has_headers(false)
            .flexible(true)
            .from_reader(Cursor::new(sample_utf8_text.as_bytes()));

        let mut record_field_counts = Vec::new();
        let mut total_records = 0;
        let mut parse_errors = 0;

        for result in rdr.records() {
            total_records += 1;
            if total_records > 100 {
                break;
            }
            match result {
                Ok(record) => {
                    record_field_counts.push(record.len());
                }
                Err(_) => {
                    parse_errors += 1;
                }
            }
        }

        if record_field_counts.is_empty() {
            continue;
        }

        let first_count = record_field_counts[0];
        if first_count <= 1 {
            // Delimiter produced only 1 column, likely not the delimiter
            continue;
        }

        // Check consistency: how many lines match the first line's column count
        let matching_counts = record_field_counts
            .iter()
            .filter(|&&c| c == first_count)
            .count();
        let consistency_ratio = matching_counts as f64 / record_field_counts.len() as f64;

        // Score formulation:
        // + (columns - 1) * 10.0
        // + consistency_ratio * 100.0
        // - parse_errors * 20.0
        let score =
            (first_count as f64) * 5.0 + (consistency_ratio * 100.0) - (parse_errors as f64 * 25.0);

        if score > best_score {
            best_score = score;
            best_delimiter = delim_char;
        }
    }

    Ok(best_delimiter)
}
