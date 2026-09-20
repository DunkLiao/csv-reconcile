//! 全面整合測試：直接使用專案 `test-data/` 目錄中的實際檔案。
//!
//! 覆蓋 SPEC §70 驗收條目與 §71 最終整合案例，包含：
//! - 混合編碼：UTF-8、UTF-8 BOM、CP950(Big5)、UTF-16 LE(BOM)
//! - 分隔符：逗號、分號、管道、自訂 `^`
//! - 欄序調換、列序調換、複合 Key、排除欄位
//! - Value Changed / A Only / B Only / Column A Only / Column B Only
//! - 重複 Key、空 Key 偵測
//! - 引號內含分隔符、換行、逃脫引號（RFC 4180）
//! - 編碼與分隔符自動偵測
//! - Excel 五頁籤匯出
//! - 錯誤處理：Missing Key Column / No Common Columns / Duplicate Header

use csv_reconcile_lib::compare::{compare_key_based, compare_row_by_row};
use csv_reconcile_lib::excel::export_to_excel;
use csv_reconcile_lib::models::{
    CompareOptions, CompareResult, ComparisonMode, DelimiterOption, DifferenceType, EncodingOption,
    FileInfo, ParseOptions,
};
use csv_reconcile_lib::parser::{detect_file_encoding, inspect_file};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// ============================================================
// Helpers
// ============================================================

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test-data")
}

fn data_path(name: &str) -> String {
    data_dir().join(name).to_string_lossy().to_string()
}

fn parse_opts(encoding: EncodingOption, delimiter: DelimiterOption) -> ParseOptions {
    ParseOptions {
        encoding,
        delimiter,
    }
}

#[allow(clippy::too_many_arguments)]
fn options(
    a: &str,
    b: &str,
    a_enc: EncodingOption,
    a_delim: DelimiterOption,
    b_enc: EncodingOption,
    b_delim: DelimiterOption,
    mode: ComparisonMode,
    keys: &[&str],
    excluded: &[&str],
) -> CompareOptions {
    CompareOptions {
        file_a_path: data_path(a),
        file_b_path: data_path(b),
        file_a_parse_options: parse_opts(a_enc, a_delim),
        file_b_parse_options: parse_opts(b_enc, b_delim),
        comparison_mode: mode,
        key_columns: keys.iter().map(|s| s.to_string()).collect(),
        excluded_columns: excluded.iter().map(|s| s.to_string()).collect(),
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    }
}

fn key_compare(opts: &CompareOptions) -> CompareResult {
    compare_key_based(opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap()
}

fn row_compare(opts: &CompareOptions) -> CompareResult {
    compare_row_by_row(opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap()
}

fn inspect(name: &str, opts: ParseOptions) -> FileInfo {
    inspect_file(data_path(name), &opts).unwrap()
}

fn svec(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn col_status(r: &CompareResult, name: &str) -> Option<String> {
    r.column_differences
        .iter()
        .find(|c| c.column == name)
        .map(|c| c.status.clone())
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("csv_reconcile_td_{}_{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ============================================================
// TC-1  完全一致（欄序/列序調換，UTF-8 comma vs pipe）
// ============================================================

#[test]
fn tc01_same_pair_is_identical_key_based() {
    let opts = options(
        "01_same_A.csv",
        "01_same_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Pipe,
        ComparisonMode::KeyBased,
        &["客戶編號", "交易日期", "交易序號"],
        &[],
    );
    let r = key_compare(&opts);

    assert_eq!(r.rows_a, 3);
    assert_eq!(r.rows_b, 3);
    assert_eq!(r.matched_records, 3);
    assert_eq!(r.same_records, 3);
    assert_eq!(r.different_records, 0);
    assert_eq!(r.a_only_records, 0);
    assert_eq!(r.b_only_records, 0);
    assert_eq!(r.different_cells, 0);
    assert_eq!(r.duplicate_keys_a, 0);
    assert_eq!(r.duplicate_keys_b, 0);
    assert!(r.differences.is_empty());
    assert!(r.identical);

    // Key 欄位依 A 檔表頭順序；比較欄位 = 共同欄位減 Key
    assert_eq!(r.key_columns, svec(&["客戶編號", "交易日期", "交易序號"]));
    assert_eq!(r.compared_columns, svec(&["金額", "更新時間"]));
    assert!(r.column_differences.iter().all(|c| c.status == "SAME"));
}

#[test]
fn tc01_same_pair_row_by_row_treats_reordering_as_diff() {
    // Row-by-Row 模式：欄序仍依名稱對應，但列序不同即產生差異
    let opts = options(
        "01_same_A.csv",
        "01_same_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Pipe,
        ComparisonMode::RowByRow,
        &[],
        &[],
    );
    let r = row_compare(&opts);

    assert_eq!(r.rows_a, 3);
    assert_eq!(r.rows_b, 3);
    assert_eq!(r.matched_records, 3);
    assert_eq!(r.same_records, 1);
    assert_eq!(r.different_records, 2);
    assert_eq!(r.different_cells, 4);
    assert!(!r.identical);
    // Row-by-Row 沒有 Key
    assert!(r.key_columns.is_empty());
    assert_eq!(
        r.compared_columns,
        svec(&["客戶編號", "交易日期", "交易序號", "金額", "更新時間"])
    );
}

// ============================================================
// TC-2  內容變更（CP950 comma vs UTF-8 semicolon + 排除欄位）
// ============================================================

#[test]
fn tc02_value_changed_cp950_and_semicolon_with_excluded_column() {
    let opts = options(
        "02_change_A.csv",
        "02_change_B.csv",
        EncodingOption::Cp950,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Semicolon,
        ComparisonMode::KeyBased,
        &["客戶編號", "交易日期", "交易序號"],
        &["更新時間"],
    );
    let r = key_compare(&opts);

    assert_eq!(r.rows_a, 4);
    assert_eq!(r.rows_b, 4);
    assert_eq!(r.matched_records, 4);
    assert_eq!(r.same_records, 3);
    assert_eq!(r.different_records, 1);
    assert_eq!(r.a_only_records, 0);
    assert_eq!(r.b_only_records, 0);
    assert_eq!(r.different_cells, 1);
    assert!(!r.identical);
    assert_eq!(r.compared_columns, svec(&["金額"]));
    assert_eq!(r.excluded_columns, svec(&["更新時間"]));

    let d = r
        .differences
        .iter()
        .find(|d| d.difference_type == DifferenceType::ValueChanged)
        .expect("should contain one VALUE_CHANGED difference");
    assert_eq!(d.column_name.as_deref(), Some("金額"));
    assert_eq!(d.value_a.as_deref(), Some("2000"));
    assert_eq!(d.value_b.as_deref(), Some("2500"));

    let keys: HashSet<(String, String)> = d
        .key_values
        .iter()
        .map(|k| (k.column.clone(), k.value.clone()))
        .collect();
    assert!(keys.contains(&("客戶編號".to_string(), "001".to_string())));
    assert!(keys.contains(&("交易日期".to_string(), "2026/09/18".to_string())));
    assert!(keys.contains(&("交易序號".to_string(), "02".to_string())));

    // 更新時間已被排除，不得出現在任何差異中
    assert!(r
        .differences
        .iter()
        .all(|d| d.column_name.as_deref() != Some("更新時間")));
}

#[test]
fn tc02_without_excluding_timestamp_more_cells_differ() {
    let opts = options(
        "02_change_A.csv",
        "02_change_B.csv",
        EncodingOption::Cp950,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Semicolon,
        ComparisonMode::KeyBased,
        &["客戶編號", "交易日期", "交易序號"],
        &[],
    );
    let r = key_compare(&opts);

    assert_eq!(r.matched_records, 4);
    assert_eq!(r.different_records, 4);
    // 4 筆更新時間 (08:00 -> 09:30) + 1 筆金額
    assert_eq!(r.different_cells, 5);
    assert_eq!(r.compared_columns, svec(&["金額", "更新時間"]));
    assert!(!r.identical);
}

// ============================================================
// TC-3  重複 Key + 空 Key（UTF-8 comma vs pipe）
// ============================================================

#[test]
fn tc03_duplicate_key_and_empty_key_detected() {
    let opts = options(
        "03_dup_A.csv",
        "03_dup_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Pipe,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);

    assert_eq!(r.rows_a, 5);
    assert_eq!(r.rows_b, 3);
    assert_eq!(r.matched_records, 2);
    assert_eq!(r.same_records, 2);
    assert_eq!(r.different_records, 0);
    assert_eq!(r.b_only_records, 0);
    assert_eq!(r.duplicate_keys_a, 1);
    assert_eq!(r.duplicate_keys_b, 0);
    assert!(!r.identical);

    // 重複 Key 明細：001 出現於第 1、2 列
    assert_eq!(r.duplicate_key_records.len(), 1);
    let dup = &r.duplicate_key_records[0];
    assert_eq!(dup.source, "File A");
    assert_eq!(dup.count, 2);
    assert_eq!(dup.rows, vec![1, 2]);
    assert_eq!(dup.key_values.len(), 1);
    assert_eq!(dup.key_values[0].column, "ID");
    assert_eq!(dup.key_values[0].value, "001");

    // 重複 Key 出現在差異清單
    assert_eq!(
        r.differences
            .iter()
            .filter(|d| d.difference_type == DifferenceType::DuplicateKeyA)
            .count(),
        1
    );

    // 空 Key 必須被偵測
    assert!(r
        .differences
        .iter()
        .any(|d| d.difference_type == DifferenceType::EmptyKeyA));

    // 空 Key 列只回報 EmptyKeyA，不得再被計入 A Only
    assert_eq!(r.a_only_records, 0);
    assert!(!r
        .differences
        .iter()
        .any(|d| d.difference_type == DifferenceType::AOnly));

    // B 檔的 001 因 A 檔重複而無法配對，不應被當成 B Only
    assert!(r
        .differences
        .iter()
        .all(|d| d.difference_type != DifferenceType::BOnly));

    // 差異總數 = 1 筆 EmptyKeyA + 1 筆 DuplicateKeyA
    assert_eq!(r.differences.len(), 2);
}

#[test]
fn tc03_empty_key_in_b_is_not_counted_as_b_only() {
    let dir = temp_dir("empty_key_b");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Name\n001,Alice\n").unwrap();
    // B 檔含一筆空 ID 的資料列
    std::fs::write(&b, "ID,Name\n001,Alice\n,X\n").unwrap();

    let opts = CompareOptions {
        file_a_path: a.to_string_lossy().to_string(),
        file_b_path: b.to_string_lossy().to_string(),
        file_a_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        file_b_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["ID".to_string()],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };
    let r = key_compare(&opts);

    assert_eq!(r.matched_records, 1);
    assert_eq!(r.same_records, 1);
    assert_eq!(r.b_only_records, 0);
    assert_eq!(r.a_only_records, 0);
    assert!(r
        .differences
        .iter()
        .any(|d| d.difference_type == DifferenceType::EmptyKeyB));
    assert!(!r
        .differences
        .iter()
        .any(|d| d.difference_type == DifferenceType::BOnly));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tc03_incomplete_composite_key_is_not_counted_as_set_difference() {
    let dir = temp_dir("incomplete_key");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    // 複合 Key = 客戶 + 日期；A 的日期為空 -> IncompleteKeyA
    std::fs::write(&a, "客戶,日期,金額\n001,,100\n").unwrap();
    std::fs::write(&b, "客戶,日期,金額\n001,2026/09/18,100\n").unwrap();

    let opts = CompareOptions {
        file_a_path: a.to_string_lossy().to_string(),
        file_b_path: b.to_string_lossy().to_string(),
        file_a_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        file_b_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["客戶".to_string(), "日期".to_string()],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };
    let r = key_compare(&opts);

    // 不完整 Key 不參與配對，因此不會產生 A Only
    assert_eq!(r.a_only_records, 0);
    assert!(!r
        .differences
        .iter()
        .any(|d| d.difference_type == DifferenceType::AOnly));
    assert!(r
        .differences
        .iter()
        .any(|d| d.difference_type == DifferenceType::IncompleteKeyA));
    // B 檔的完整 Key 找不到對應 -> B Only
    assert_eq!(r.b_only_records, 1);

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// TC-4  A Only / B Only / 欄位差異（UTF-8 comma 雙檔）
// ============================================================

#[test]
fn tc04_a_only_b_only_and_column_differences() {
    let opts = options(
        "04_only_A.csv",
        "04_only_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);

    assert_eq!(r.rows_a, 5);
    assert_eq!(r.rows_b, 3);
    assert_eq!(r.matched_records, 2);
    assert_eq!(r.same_records, 2);
    assert_eq!(r.different_records, 0);
    assert_eq!(r.a_only_records, 3);
    assert_eq!(r.b_only_records, 1);
    assert!(!r.identical);
    assert_eq!(r.compared_columns, svec(&["姓名"]));

    // 欄位層級差異
    assert_eq!(col_status(&r, "ID").as_deref(), Some("SAME"));
    assert_eq!(col_status(&r, "姓名").as_deref(), Some("SAME"));
    assert_eq!(col_status(&r, "金額").as_deref(), Some("A_ONLY"));
    assert_eq!(col_status(&r, "日期").as_deref(), Some("B_ONLY"));

    let a_only_keys: HashSet<String> = r
        .differences
        .iter()
        .filter(|d| d.difference_type == DifferenceType::AOnly)
        .map(|d| d.key_values[0].value.clone())
        .collect();
    assert_eq!(
        a_only_keys,
        HashSet::from(["003".to_string(), "004".to_string(), "005".to_string()])
    );

    let b_only_keys: HashSet<String> = r
        .differences
        .iter()
        .filter(|d| d.difference_type == DifferenceType::BOnly)
        .map(|d| d.key_values[0].value.clone())
        .collect();
    assert_eq!(b_only_keys, HashSet::from(["006".to_string()]));
}

// ============================================================
// TC-5  引號/換行/逃脫引號 + UTF-16 LE + 自訂分隔符 ^
// ============================================================

#[test]
fn tc05_quoted_fields_utf16le_custom_delimiter_identical() {
    let opts = options(
        "05_quoted_A.csv",
        "05_custom_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf16Le,
        DelimiterOption::Custom('^'),
        ComparisonMode::KeyBased,
        &["編號"],
        &[],
    );
    let r = key_compare(&opts);

    assert_eq!(r.rows_a, 3);
    assert_eq!(r.rows_b, 3);
    assert_eq!(r.matched_records, 3);
    assert_eq!(r.same_records, 3);
    assert_eq!(r.different_records, 0);
    assert_eq!(r.different_cells, 0);
    assert!(r.identical);
    assert_eq!(r.compared_columns, svec(&["備註", "地址"]));
}

#[test]
fn tc05_parses_embedded_delimiter_newline_and_escaped_quotefields() {
    // 以 UTF-16 LE + 自訂分隔符手動解析，驗證引號欄位內容正確
    let fi = inspect(
        "05_custom_B.csv",
        parse_opts(EncodingOption::Utf16Le, DelimiterOption::Custom('^')),
    );
    assert_eq!(fi.headers, svec(&["編號", "備註", "地址"]));
    assert_eq!(fi.row_count, Some(3));
    assert_eq!(fi.delimiter_char, '^');

    // 直接讀取文字內容，確認換行與逃脫引號被解析為單一欄位內容
    let (enc, _, skip) =
        csv_reconcile_lib::parser::get_encoding_for_option(&EncodingOption::Utf16Le);
    let mut reader = csv_reconcile_lib::parser::open_delimited_reader(
        data_path("05_custom_B.csv"),
        enc,
        skip,
        b'^',
    )
    .unwrap();
    let mut records = reader.records();
    let first = records.next().unwrap().unwrap();
    assert_eq!(first.get(1).unwrap(), "含,分隔符號,備註"); // 引號內含逗號
    let second = records.next().unwrap().unwrap();
    assert_eq!(second.get(1).unwrap(), "第一行\n第二行"); // 引號內含換行
    let third = records.next().unwrap().unwrap();
    assert_eq!(third.get(1).unwrap(), "客戶說\"今天會付款\""); // 逃脫引號
}

// ============================================================
// TC-6  自動偵測（編碼 + 分隔符）
// ============================================================

#[test]
fn tc06_detect_file_encoding_for_all_test_data() {
    let cases = [
        ("01_same_A.csv", EncodingOption::Utf8, false),
        ("01_same_B.csv", EncodingOption::Utf8, false),
        ("02_change_B.csv", EncodingOption::Utf8, false),
        ("03_dup_A.csv", EncodingOption::Utf8, false),
        ("04_only_A.csv", EncodingOption::Utf8, false),
        ("02_change_A.csv", EncodingOption::Cp950, false),
        ("05_custom_B.csv", EncodingOption::Utf16Le, true),
    ];
    for (file, expected, has_bom) in cases {
        let detected = detect_file_encoding(data_path(file)).unwrap();
        assert_eq!(
            detected.encoding_option, expected,
            "unexpected encoding for {file}"
        );
        assert_eq!(detected.has_bom, has_bom, "BOM flag mismatch for {file}");
    }

    let bom = detect_file_encoding(data_path("海葬資料1150319.csv")).unwrap();
    assert_eq!(bom.encoding_option, EncodingOption::Utf8Bom);
    assert!(bom.has_bom);
}

#[test]
fn tc06_auto_inspect_utf8_comma_and_pipe() {
    let fi = inspect("01_same_A.csv", ParseOptions::default());
    assert_eq!(fi.encoding, "UTF-8");
    assert_eq!(fi.delimiter_char, ',');
    assert_eq!(fi.row_count, Some(3));
    assert_eq!(
        fi.headers,
        svec(&["客戶編號", "交易日期", "交易序號", "金額", "更新時間"])
    );

    let fi = inspect("01_same_B.csv", ParseOptions::default());
    assert_eq!(fi.encoding, "UTF-8");
    assert_eq!(fi.delimiter_char, '|');
    assert_eq!(fi.row_count, Some(3));
    assert_eq!(
        fi.headers,
        svec(&["交易日期", "交易序號", "客戶編號", "金額", "更新時間"])
    );
}

#[test]
fn tc06_auto_compare_uses_detected_delimiter_for_each_file() {
    let opts = options(
        "01_same_A.csv",
        "01_same_B.csv",
        EncodingOption::Auto,
        DelimiterOption::Auto,
        EncodingOption::Auto,
        DelimiterOption::Auto,
        ComparisonMode::KeyBased,
        &["客戶編號", "交易日期", "交易序號"],
        &[],
    );

    let r = key_compare(&opts);

    assert!(r.identical);
    assert_eq!(r.rows_a, 3);
    assert_eq!(r.rows_b, 3);
    assert_eq!(r.matched_records, 3);
    assert_eq!(r.same_records, 3);
}

#[test]
fn tc06_auto_inspect_cp950_and_semicolon() {
    let fi = inspect("02_change_A.csv", ParseOptions::default());
    assert!(
        fi.encoding == "CP950" || fi.encoding == "Big5",
        "unexpected auto encoding: {}",
        fi.encoding
    );
    assert_eq!(fi.delimiter_char, ',');
    assert_eq!(fi.row_count, Some(4));

    let fi = inspect("02_change_B.csv", ParseOptions::default());
    assert_eq!(fi.encoding, "UTF-8");
    assert_eq!(fi.delimiter_char, ';');
    assert_eq!(fi.row_count, Some(4));
}

#[test]
fn tc06_auto_inspect_utf16le_bom() {
    let fi = inspect("05_custom_B.csv", ParseOptions::default());
    assert_eq!(fi.encoding, "UTF-16 LE");
    // BOM 已被移除：表頭第一個欄位不以 U+FEFF 開頭
    assert!(!fi.headers[0].starts_with('\u{FEFF}'));
    // 自訂分隔符 `^` 不在自動偵測候選集合 (, ; tab | :) 中，
    // 因此整個表頭列會被視為單一欄位，這正是需手動指定分隔符的情境。
    assert_eq!(fi.delimiter_char, ',');
    assert_eq!(fi.headers.len(), 1);
    assert!(fi.headers[0].contains('^'));
    // 因換行未被視為引號內換行，002 的兩行被拆成兩筆 -> 4 筆
    assert_eq!(fi.row_count, Some(4));
}

#[test]
fn tc06_manual_delimiter_override_for_custom_char() {
    // 自訂分隔符 ^ 無法自動偵測，需手動指定
    let fi = inspect(
        "05_custom_B.csv",
        parse_opts(EncodingOption::Utf16Le, DelimiterOption::Custom('^')),
    );
    assert_eq!(fi.delimiter_char, '^');
    assert_eq!(fi.headers, svec(&["編號", "備註", "地址"]));
    assert_eq!(fi.row_count, Some(3));

    // 05_quoted_A 自動偵測應為逗號
    let fi = inspect("05_quoted_A.csv", ParseOptions::default());
    assert_eq!(fi.delimiter_char, ',');
    assert_eq!(fi.headers, svec(&["編號", "備註", "地址"]));
    assert_eq!(fi.row_count, Some(3));
}

// ============================================================
// TC-7  真實資料：UTF-8 BOM 海葬資料
// ============================================================

#[test]
fn tc07_haizang_utf8bom_inspect_and_self_compare() {
    let name = "海葬資料1150319.csv";

    let fi = inspect(name, ParseOptions::default());
    assert_eq!(fi.encoding, "UTF-8 BOM");
    assert_eq!(fi.delimiter_char, ',');
    assert_eq!(fi.row_count, Some(23));
    assert_eq!(
        fi.headers,
        svec(&["序號", "民國年度", "西元年度", "人數", "單位"])
    );

    // 自我比對：完全相同
    let opts = options(
        name,
        name,
        EncodingOption::Utf8Bom,
        DelimiterOption::Comma,
        EncodingOption::Utf8Bom,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["序號"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.rows_a, 23);
    assert_eq!(r.rows_b, 23);
    assert_eq!(r.matched_records, 23);
    assert_eq!(r.same_records, 23);
    assert_eq!(r.different_cells, 0);
    assert!(r.identical);
    assert_eq!(
        r.compared_columns,
        svec(&["民國年度", "西元年度", "人數", "單位"])
    );
}

// ============================================================
// TC-8  Excel 匯出（五頁籤）
// ============================================================

#[test]
fn tc08_excel_export_has_five_sheets() {
    let opts = options(
        "02_change_A.csv",
        "02_change_B.csv",
        EncodingOption::Cp950,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Semicolon,
        ComparisonMode::KeyBased,
        &["客戶編號", "交易日期", "交易序號"],
        &["更新時間"],
    );
    let r = key_compare(&opts);

    let out = std::env::temp_dir().join(format!(
        "csv_reconcile_td_export_{}.xlsx",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&out);
    export_to_excel(&out, &opts, &r).unwrap();

    assert!(out.exists());
    assert!(out.metadata().unwrap().len() > 1000);

    // 解壓 xlsx 檢查五個工作表名稱
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut workbook = String::new();
    archive
        .by_name("xl/workbook.xml")
        .expect("xlsx should contain xl/workbook.xml")
        .read_to_string(&mut workbook)
        .unwrap();

    for sheet in [
        "Summary",
        "Differences",
        "Column_Differences",
        "Excluded_Columns",
        "Duplicate_Keys",
    ] {
        assert!(workbook.contains(sheet), "missing sheet: {sheet}");
    }

    let _ = std::fs::remove_file(&out);
}

// ============================================================
// TC-9  錯誤處理
// ============================================================

#[test]
fn tc09_missing_key_column_in_file_b() {
    // 金額 只存在於 A 檔，指定為 Key 時 B 檔應報 MissingKeyColumn
    let opts = options(
        "04_only_A.csv",
        "04_only_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["金額"],
        &[],
    );
    let err = compare_key_based(&opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap_err();
    match err {
        csv_reconcile_lib::error::AppError::MissingKeyColumn(col) => assert_eq!(col, "金額"),
        other => panic!("expected MissingKeyColumn, got {other:?}"),
    }
}

#[test]
fn tc09_no_key_specified() {
    let opts = options(
        "01_same_A.csv",
        "01_same_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Pipe,
        ComparisonMode::KeyBased,
        &[],
        &[],
    );
    let err = compare_key_based(&opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap_err();
    assert!(matches!(
        err,
        csv_reconcile_lib::error::AppError::NoKeySpecified
    ));
}

#[test]
fn tc09_no_common_columns() {
    let dir = temp_dir("no_common");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Name\n1,Alice\n").unwrap();
    std::fs::write(&b, "編號,名稱\n1,Alice\n").unwrap();

    let opts = CompareOptions {
        file_a_path: a.to_string_lossy().to_string(),
        file_b_path: b.to_string_lossy().to_string(),
        file_a_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        file_b_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["ID".to_string()],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };
    let err = compare_key_based(&opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap_err();
    assert!(matches!(
        err,
        csv_reconcile_lib::error::AppError::NoCommonColumns
    ));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tc09_duplicate_header_detected() {
    let dir = temp_dir("dup_header");
    let a = dir.join("dup.csv");
    std::fs::write(&a, "ID,ID,Name\n1,2,Alice\n").unwrap();

    let err = inspect_file(&a, &ParseOptions::default()).unwrap_err();
    match err {
        csv_reconcile_lib::error::AppError::DuplicateHeader(h) => assert_eq!(h, "ID"),
        other => panic!("expected DuplicateHeader, got {other:?}"),
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tc09_missing_file_error() {
    let opts = options(
        "does_not_exist_A.csv",
        "01_same_B.csv",
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Pipe,
        ComparisonMode::KeyBased,
        &["客戶編號"],
        &[],
    );
    let err = compare_key_based(&opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap_err();
    assert!(matches!(
        err,
        csv_reconcile_lib::error::AppError::FileNotFound(_)
    ));
}

// ============================================================
// TC-10  比較選項：Trim / Ignore Case / 前導零
// ============================================================

#[test]
fn tc10_trim_whitespace_and_ignore_case() {
    let dir = temp_dir("trim_case");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Name\n001, Alice \n").unwrap();
    std::fs::write(&b, "ID,Name\n001,alice\n").unwrap();

    let base = |trim: bool, ignore_case: bool| CompareOptions {
        file_a_path: a.to_string_lossy().to_string(),
        file_b_path: b.to_string_lossy().to_string(),
        file_a_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        file_b_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["ID".to_string()],
        excluded_columns: vec![],
        trim_whitespace: trim,
        ignore_case: ignore_case,
        numeric_tolerance: "0".into(),
    };

    // 預設：嚴格比對 -> 不同
    let r = key_compare(&base(false, false));
    assert_eq!(r.different_records, 1);
    assert!(!r.identical);

    // 啟用 trim + ignore_case -> 相同
    let r = key_compare(&base(true, true));
    assert_eq!(r.different_records, 0);
    assert_eq!(r.same_records, 1);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tc10_preserves_leading_zero_key() {
    let dir = temp_dir("leading_zero");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Amt\n001,100\n").unwrap();
    std::fs::write(&b, "ID,Amt\n1,100\n").unwrap(); // 不同 Key：001 vs 1

    let opts = CompareOptions {
        file_a_path: a.to_string_lossy().to_string(),
        file_b_path: b.to_string_lossy().to_string(),
        file_a_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        file_b_parse_options: parse_opts(EncodingOption::Utf8, DelimiterOption::Comma),
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["ID".to_string()],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };
    let r = key_compare(&opts);
    // Key 為純字串比對，前導零保留 -> 001 與 1 視為不同 Key
    assert_eq!(r.a_only_records, 1);
    assert_eq!(r.b_only_records, 1);
    assert_eq!(r.matched_records, 0);
    assert!(!r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// TC-11  進度回呼 / 取消旗標
// ============================================================

#[test]
fn tc11_progress_callback_reaches_100_percent() {
    use std::sync::Mutex;

    let opts = options(
        "02_change_A.csv",
        "02_change_B.csv",
        EncodingOption::Cp950,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Semicolon,
        ComparisonMode::KeyBased,
        &["客戶編號", "交易日期", "交易序號"],
        &["更新時間"],
    );

    let seen = Mutex::new(Vec::new());
    let result = compare_key_based(&opts, Arc::new(AtomicBool::new(false)), |p| {
        seen.lock().unwrap().push(p.percent);
    })
    .unwrap();
    assert_eq!(result.matched_records, 4);

    let percents = seen.into_inner().unwrap();
    assert!(!percents.is_empty());
    assert!(percents.iter().any(|p| *p == Some(100)));
}
