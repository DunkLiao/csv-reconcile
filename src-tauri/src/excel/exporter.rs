use crate::error::AppError;
use crate::models::compare_options::CompareOptions;
use crate::models::compare_result::CompareResult;
use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook};
use std::collections::HashMap;
use std::path::Path;

pub fn export_to_excel<P: AsRef<Path>>(
    output_path: P,
    options: &CompareOptions,
    result: &CompareResult,
) -> Result<(), AppError> {
    let mut workbook = Workbook::new();

    // Setup reusable formats
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x2563EB))
        .set_font_color(Color::RGB(0xFFFFFF))
        .set_border(FormatBorder::Thin);

    let summary_title_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xF1F5F9))
        .set_border(FormatBorder::Thin);

    let text_format = Format::new()
        .set_num_format("@")
        .set_border(FormatBorder::Thin);

    let number_format = Format::new().set_border(FormatBorder::Thin);

    let changed_format = Format::new()
        .set_background_color(Color::RGB(0xFEE2E2)) // Light red
        .set_font_color(Color::RGB(0x991B1B))
        .set_bold()
        .set_border(FormatBorder::Thin);

    let a_only_format = Format::new()
        .set_background_color(Color::RGB(0xFEF3C7)) // Light amber
        .set_font_color(Color::RGB(0x92400E))
        .set_bold()
        .set_border(FormatBorder::Thin);

    let b_only_format = Format::new()
        .set_background_color(Color::RGB(0xDBEAFE)) // Light blue
        .set_font_color(Color::RGB(0x1E40AF))
        .set_bold()
        .set_border(FormatBorder::Thin);

    // ==========================================
    // 1. Summary Sheet
    // ==========================================
    let sheet_summary = workbook.add_worksheet();
    sheet_summary
        .set_name("Summary")
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    sheet_summary
        .write_string_with_format(0, 0, "Item", &header_format)
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    sheet_summary
        .write_string_with_format(0, 1, "Result / Details", &header_format)
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    let summary_items = vec![
        ("File A", options.file_a_path.clone()),
        (
            "File A Encoding",
            options
                .file_a_parse_options
                .encoding
                .display_name()
                .to_string(),
        ),
        (
            "File A Delimiter",
            options.file_a_parse_options.delimiter.display_name(),
        ),
        ("File B", options.file_b_path.clone()),
        (
            "File B Encoding",
            options
                .file_b_parse_options
                .encoding
                .display_name()
                .to_string(),
        ),
        (
            "File B Delimiter",
            options.file_b_parse_options.delimiter.display_name(),
        ),
        ("Compare Mode", format!("{:?}", options.comparison_mode)),
        (
            "Key Columns",
            if result.key_columns.is_empty() {
                "None (Row-by-Row)".to_string()
            } else {
                result.key_columns.join(" / ")
            },
        ),
        ("Rows A", result.rows_a.to_string()),
        ("Rows B", result.rows_b.to_string()),
        ("Matched Records", result.matched_records.to_string()),
        ("Same Records", result.same_records.to_string()),
        ("Different Records", result.different_records.to_string()),
        ("A Only", result.a_only_records.to_string()),
        ("B Only", result.b_only_records.to_string()),
        ("Different Cells", result.different_cells.to_string()),
        ("Duplicate Keys A", result.duplicate_keys_a.to_string()),
        ("Duplicate Keys B", result.duplicate_keys_b.to_string()),
        (
            "Result",
            if result.identical {
                "IDENTICAL (完全相同)".to_string()
            } else {
                "DIFFERENT (內容不同)".to_string()
            },
        ),
        (
            "Compare Duration (ms)",
            format!("{} ms", result.duration_ms),
        ),
    ];

    for (row_idx, (item, val)) in summary_items.iter().enumerate() {
        let r = (row_idx + 1) as u32;
        sheet_summary
            .write_string_with_format(r, 0, *item, &summary_title_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
        sheet_summary
            .write_string_with_format(r, 1, val, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    }
    sheet_summary.set_column_width(0, 25).ok();
    sheet_summary.set_column_width(1, 60).ok();

    // ==========================================
    // 2. Differences Sheet
    // ==========================================
    let sheet_diff = workbook.add_worksheet();
    sheet_diff
        .set_name("Differences")
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    // Headers: [Key Column 1, Key Column 2, ..., Row A, Row B, Column, File A Value, File B Value, Type]
    let mut diff_headers = Vec::new();
    for key_col in &result.key_columns {
        diff_headers.push(key_col.clone());
    }
    diff_headers.push("Row A".to_string());
    diff_headers.push("Row B".to_string());
    diff_headers.push("Column".to_string());
    diff_headers.push("File A Value".to_string());
    diff_headers.push("File B Value".to_string());
    diff_headers.push("Type".to_string());

    for (col_idx, h) in diff_headers.iter().enumerate() {
        sheet_diff
            .write_string_with_format(0, col_idx as u16, h, &header_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
        sheet_diff.set_column_width(col_idx as u16, 18).ok();
    }

    let num_key_cols = result.key_columns.len();

    for (row_idx, diff) in result.differences.iter().enumerate() {
        let r = (row_idx + 1) as u32;

        // Map key values to respective columns
        let key_map: HashMap<&str, &str> = diff
            .key_values
            .iter()
            .map(|kv| (kv.column.as_str(), kv.value.as_str()))
            .collect();

        for (k_idx, key_col) in result.key_columns.iter().enumerate() {
            let val = key_map.get(key_col.as_str()).unwrap_or(&"");
            sheet_diff
                .write_string_with_format(r, k_idx as u16, *val, &text_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;
        }

        let base_col = num_key_cols as u16;

        // Row A
        if let Some(ra) = diff.row_a {
            sheet_diff
                .write_number_with_format(r, base_col, ra as f64, &number_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;
        } else {
            sheet_diff
                .write_string_with_format(r, base_col, "-", &text_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;
        }

        // Row B
        if let Some(rb) = diff.row_b {
            sheet_diff
                .write_number_with_format(r, base_col + 1, rb as f64, &number_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;
        } else {
            sheet_diff
                .write_string_with_format(r, base_col + 1, "-", &text_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;
        }

        // Column Name
        let col_name = diff.column_name.as_deref().unwrap_or("-");
        sheet_diff
            .write_string_with_format(r, base_col + 2, col_name, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;

        // File A Value (Always Text format)
        let val_a = diff.value_a.as_deref().unwrap_or("-");
        sheet_diff
            .write_string_with_format(r, base_col + 3, val_a, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;

        // File B Value (Always Text format)
        let val_b = diff.value_b.as_deref().unwrap_or("-");
        sheet_diff
            .write_string_with_format(r, base_col + 4, val_b, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;

        // Difference Type (With color highlight)
        let type_str = format!("{:?}", diff.difference_type);
        let type_fmt = match diff.difference_type {
            crate::models::difference::DifferenceType::ValueChanged => &changed_format,
            crate::models::difference::DifferenceType::AOnly => &a_only_format,
            crate::models::difference::DifferenceType::BOnly => &b_only_format,
            _ => &text_format,
        };
        sheet_diff
            .write_string_with_format(r, base_col + 5, &type_str, type_fmt)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    }

    sheet_diff.set_freeze_panes(1, 0).ok();
    if !result.differences.is_empty() {
        sheet_diff
            .autofilter(
                0,
                0,
                result.differences.len() as u32,
                (diff_headers.len() - 1) as u16,
            )
            .ok();
    }

    // ==========================================
    // 3. Column_Differences Sheet
    // ==========================================
    let sheet_col_diff = workbook.add_worksheet();
    sheet_col_diff
        .set_name("Column_Differences")
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    let col_headers = ["Column", "File A", "File B", "Status"];
    for (i, h) in col_headers.iter().enumerate() {
        sheet_col_diff
            .write_string_with_format(0, i as u16, *h, &header_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
        sheet_col_diff.set_column_width(i as u16, 20).ok();
    }

    for (row_idx, cd) in result.column_differences.iter().enumerate() {
        let r = (row_idx + 1) as u32;
        sheet_col_diff
            .write_string_with_format(r, 0, &cd.column, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
        sheet_col_diff
            .write_string_with_format(r, 1, if cd.file_a { "Yes" } else { "No" }, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
        sheet_col_diff
            .write_string_with_format(r, 2, if cd.file_b { "Yes" } else { "No" }, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
        sheet_col_diff
            .write_string_with_format(r, 3, &cd.status, &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    }
    sheet_col_diff.set_freeze_panes(1, 0).ok();

    // ==========================================
    // 4. Excluded_Columns Sheet
    // ==========================================
    let sheet_ex = workbook.add_worksheet();
    sheet_ex
        .set_name("Excluded_Columns")
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    sheet_ex
        .write_string_with_format(0, 0, "Column", &header_format)
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    sheet_ex.set_column_width(0, 30).ok();

    if result.excluded_columns.is_empty() {
        sheet_ex
            .write_string_with_format(1, 0, "(None)", &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    } else {
        for (i, col) in result.excluded_columns.iter().enumerate() {
            sheet_ex
                .write_string_with_format((i + 1) as u32, 0, col, &text_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;
        }
    }
    sheet_ex.set_freeze_panes(1, 0).ok();

    // ==========================================
    // 5. Duplicate_Keys Sheet
    // ==========================================
    let sheet_dup = workbook.add_worksheet();
    sheet_dup
        .set_name("Duplicate_Keys")
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    let mut dup_headers = vec!["Source".to_string()];
    for key_col in &result.key_columns {
        dup_headers.push(key_col.clone());
    }
    dup_headers.push("Count".to_string());
    dup_headers.push("Rows".to_string());

    for (i, h) in dup_headers.iter().enumerate() {
        sheet_dup
            .write_string_with_format(0, i as u16, h, &header_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
        sheet_dup.set_column_width(i as u16, 20).ok();
    }

    if result.duplicate_key_records.is_empty() {
        sheet_dup
            .write_string_with_format(1, 0, "No duplicate keys found.", &text_format)
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    } else {
        for (row_idx, dup) in result.duplicate_key_records.iter().enumerate() {
            let r = (row_idx + 1) as u32;
            sheet_dup
                .write_string_with_format(r, 0, &dup.source, &text_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;

            let key_map: HashMap<&str, &str> = dup
                .key_values
                .iter()
                .map(|kv| (kv.column.as_str(), kv.value.as_str()))
                .collect();

            for (k_idx, key_col) in result.key_columns.iter().enumerate() {
                let val = key_map.get(key_col.as_str()).unwrap_or(&"");
                sheet_dup
                    .write_string_with_format(r, (k_idx + 1) as u16, *val, &text_format)
                    .map_err(|e| AppError::ExportError(e.to_string()))?;
            }

            let base = (result.key_columns.len() + 1) as u16;
            sheet_dup
                .write_number_with_format(r, base, dup.count as f64, &number_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;

            let rows_str = dup
                .rows
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            sheet_dup
                .write_string_with_format(r, base + 1, &rows_str, &text_format)
                .map_err(|e| AppError::ExportError(e.to_string()))?;
        }
    }
    sheet_dup.set_freeze_panes(1, 0).ok();

    workbook
        .save(output_path.as_ref())
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    Ok(())
}
