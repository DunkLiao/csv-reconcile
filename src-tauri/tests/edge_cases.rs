//! 邊界與生成檔案測試：以程式即時產生各類編碼 / 分隔符 / CSV 特殊情況，
//! 補足 `test-data/` 靜態檔案未涵蓋的情境。
//!
//! 涵蓋：
//! - 編碼：UTF-16 LE/BE (BOM)、CP950、UTF-8 (含/不含 BOM) 交叉比對
//! - 分隔符：Tab、Colon、自訂字元之自動偵測與手動指定
//! - CSV：CRLF/LF、引號表頭、僅表頭、空檔
//! - Key：複合 Key 順序無關、雙方重複 Key、B 檔重複 Key、重複 Key 值不同
//! - 排除欄位與 Key 併用
//! - 效能：20,000 列自我比對
//! - 取消：Cancel 旗標中止
//! - Excel：前導零以文字保存

use csv_reconcile_lib::compare::{compare_key_based, compare_row_by_row};
use csv_reconcile_lib::excel::export_to_excel;
use csv_reconcile_lib::models::{
    CompareOptions, ComparisonMode, DelimiterOption, DifferenceType, EncodingOption, ParseOptions,
};
use csv_reconcile_lib::parser::{
    detect_delimiter, detect_file_encoding, inspect_file, open_delimited_reader,
};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// ============================================================
// Helpers
// ============================================================

fn temp_dir(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("csv_reconcile_edge_{}_{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn utf8(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

fn utf16le_bom(s: &str) -> Vec<u8> {
    // 注意：encoding_rs 的 encode() 對 UTF-16 會輸出 UTF-8，故以 encode_utf16 手動編碼
    let mut v = vec![0xFF, 0xFE];
    for u in s.encode_utf16() {
        v.extend_from_slice(&u.to_le_bytes());
    }
    v
}

fn utf16be_bom(s: &str) -> Vec<u8> {
    let mut v = vec![0xFE, 0xFF];
    for u in s.encode_utf16() {
        v.extend_from_slice(&u.to_be_bytes());
    }
    v
}

fn cp950(s: &str) -> Vec<u8> {
    encoding_rs::BIG5.encode(s).0.into_owned()
}

fn parse_opts(encoding: EncodingOption, delimiter: DelimiterOption) -> ParseOptions {
    ParseOptions {
        encoding,
        delimiter,
    }
}

#[allow(clippy::too_many_arguments)]
fn opts_paths(
    a: &Path,
    b: &Path,
    a_enc: EncodingOption,
    a_delim: DelimiterOption,
    b_enc: EncodingOption,
    b_delim: DelimiterOption,
    mode: ComparisonMode,
    keys: &[&str],
    excluded: &[&str],
) -> CompareOptions {
    CompareOptions {
        file_a_path: a.to_string_lossy().to_string(),
        file_b_path: b.to_string_lossy().to_string(),
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

fn key_compare(opts: &CompareOptions) -> csv_reconcile_lib::models::CompareResult {
    compare_key_based(opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap()
}

fn svec(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

// ============================================================
// 編碼
// ============================================================

#[test]
fn enc_utf16be_bom_detected_and_compared() {
    let dir = temp_dir("utf16be");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, utf16be_bom("ID,Name\n001,王小明\n002,陳小華\n")).unwrap();
    std::fs::write(&b, utf16be_bom("Name,ID\n王小明,001\n陳小華,002\n")).unwrap();

    let detected = detect_file_encoding(&a).unwrap();
    assert_eq!(detected.encoding_option, EncodingOption::Utf16Be);
    assert!(detected.has_bom);

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf16Be,
        DelimiterOption::Comma,
        EncodingOption::Utf16Be,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.matched_records, 2);
    assert_eq!(r.same_records, 2);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn enc_cp950_vs_utf16le_cross_compare() {
    let dir = temp_dir("cp950_utf16");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, cp950("客戶編號,分行名稱,金額\n001,台北分行,50000\n")).unwrap();
    std::fs::write(
        &b,
        utf16le_bom("客戶編號,分行名稱,金額\n001,台北分行,50000\n"),
    )
    .unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Cp950,
        DelimiterOption::Comma,
        EncodingOption::Utf16Le,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["客戶編號"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.matched_records, 1);
    assert_eq!(r.same_records, 1);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn enc_utf8_with_and_without_bom_cross_compare() {
    let dir = temp_dir("utf8_bom");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    let mut with_bom = vec![0xEF, 0xBB, 0xBF];
    with_bom.extend_from_slice(&utf8("ID,Name\n001,Alice\n"));
    std::fs::write(&a, with_bom).unwrap();
    std::fs::write(&b, utf8("ID,Name\n001,Alice\n")).unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8Bom,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.matched_records, 1);
    assert!(r.identical);

    // 表頭不含 BOM 殘留
    let fi = inspect_file(
        &a,
        &parse_opts(EncodingOption::Utf8Bom, DelimiterOption::Comma),
    )
    .unwrap();
    assert_eq!(fi.headers[0], "ID");

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// 分隔符
// ============================================================

#[test]
fn delim_tab_and_colon_auto_detection() {
    assert_eq!(
        detect_delimiter("ID\tName\tAmt\n1\tA\t10\n2\tB\t20\n").unwrap(),
        '\t'
    );
    assert_eq!(
        detect_delimiter("ID:Name:Amt\n1:A:10\n2:B:20\n").unwrap(),
        ':'
    );

    let dir = temp_dir("tab_colon");
    let tab = dir.join("tab.csv");
    std::fs::write(&tab, "ID\tName\n1\tAlice\n2\tBob\n").unwrap();
    let fi = inspect_file(&tab, &ParseOptions::default()).unwrap();
    assert_eq!(fi.delimiter_char, '\t');
    assert_eq!(fi.headers, svec(&["ID", "Name"]));
    assert_eq!(fi.row_count, Some(2));

    let colon = dir.join("colon.csv");
    std::fs::write(&colon, "ID:Name\n1:Alice\n2:Bob\n").unwrap();
    let fi = inspect_file(&colon, &ParseOptions::default()).unwrap();
    assert_eq!(fi.delimiter_char, ':');
    assert_eq!(fi.headers, svec(&["ID", "Name"]));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn delim_custom_manual_and_pipe_vs_custom() {
    let dir = temp_dir("custom_delim");
    let a = dir.join("a.csv");
    let b = dir.join("b.txt");
    std::fs::write(&a, "ID|Name\n1|Alice\n2|Bob\n").unwrap();
    std::fs::write(&b, "ID~Name\n1~Alice\n2~Bob\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Pipe,
        EncodingOption::Utf8,
        DelimiterOption::Custom('~'),
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.matched_records, 2);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// CSV 邊界
// ============================================================

#[test]
fn csv_crlf_vs_lf_is_identical() {
    let dir = temp_dir("crlf");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, b"ID,Name\r\n1,Alice\r\n2,Bob\r\n").unwrap();
    std::fs::write(&b, b"ID,Name\n1,Alice\n2,Bob\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.matched_records, 2);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn csv_quoted_headers_are_unquoted() {
    let dir = temp_dir("quoted_header");
    let a = dir.join("a.csv");
    std::fs::write(&a, "\"ID\",\"Name\"\n\"1\",\"Alice\"\n").unwrap();

    let fi = inspect_file(&a, &ParseOptions::default()).unwrap();
    assert_eq!(fi.headers, svec(&["ID", "Name"]));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn csv_header_only_is_identical() {
    let dir = temp_dir("header_only");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Name\n").unwrap();
    std::fs::write(&b, "ID,Name\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.rows_a, 0);
    assert_eq!(r.rows_b, 0);
    assert_eq!(r.matched_records, 0);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn csv_empty_file_has_no_common_columns() {
    let dir = temp_dir("empty");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "").unwrap();
    std::fs::write(&b, "").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let err = compare_key_based(&opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap_err();
    assert!(matches!(
        err,
        csv_reconcile_lib::error::AppError::NoCommonColumns
    ));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn csv_field_with_crlf_inside_quotes() {
    let dir = temp_dir("crlf_in_quotes");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Note\n001,\"line1\r\nline2\"\n").unwrap();
    std::fs::write(&b, "ID,Note\n001,\"line1\r\nline2\"\n").unwrap();

    let fi = inspect_file(&a, &ParseOptions::default()).unwrap();
    assert_eq!(fi.row_count, Some(1));

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// Key 邏輯
// ============================================================

#[test]
fn key_composite_order_independent_matches() {
    let dir = temp_dir("composite_order");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "K1,K2,V\nA,B,1\nC,D,2\n").unwrap();
    std::fs::write(&b, "K2,K1,V\nB,A,1\nD,C,2\n").unwrap();

    // 刻意以相反順序提供 key 欄位
    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["K2", "K1"],
        &[],
    );
    let r = key_compare(&opts);
    // key 欄位順序會依 A 檔表頭重新排列 -> K1,K2
    assert_eq!(r.key_columns, svec(&["K1", "K2"]));
    assert_eq!(r.matched_records, 2);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn key_duplicate_on_both_sides_not_paired() {
    let dir = temp_dir("dup_both");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,N\n001,A\n001,B\n002,C\n").unwrap();
    std::fs::write(&b, "ID,N\n001,A\n001,X\n002,C\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.duplicate_keys_a, 1);
    assert_eq!(r.duplicate_keys_b, 1);
    assert_eq!(r.matched_records, 1); // 只有 002 可配對
    assert_eq!(r.same_records, 1);
    assert_eq!(r.a_only_records, 0);
    assert_eq!(r.b_only_records, 0);
    assert!(!r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn key_duplicate_in_b_only_second_is_skipped() {
    let dir = temp_dir("dup_b_only");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,N\n001,A\n").unwrap();
    std::fs::write(&b, "ID,N\n001,A\n001,B\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.duplicate_keys_b, 1);
    assert_eq!(r.matched_records, 1);
    assert_eq!(r.same_records, 1);
    assert_eq!(r.b_only_records, 0);
    assert!(!r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn key_duplicate_with_different_values_no_pairing() {
    let dir = temp_dir("dup_values");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,N\n001,Alice\n001,Bob\n").unwrap();
    std::fs::write(&b, "ID,N\n001,Alice\n001,Carol\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.duplicate_keys_a, 1);
    assert_eq!(r.duplicate_keys_b, 1);
    assert_eq!(r.matched_records, 0);
    assert_eq!(r.different_cells, 0);
    assert_eq!(r.a_only_records, 0);
    assert_eq!(r.b_only_records, 0);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn key_excluded_column_still_used_as_key() {
    let dir = temp_dir("exclude_key");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    // 故意讓 ID 欄位值不同，但 ID 為 Key，仍應以 ID 配對
    std::fs::write(&a, "ID,Name,Val\n001,A,1\n").unwrap();
    std::fs::write(&b, "ID,Name,Val\n001,A,2\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &["ID"],
    );
    let r = key_compare(&opts);
    assert_eq!(r.matched_records, 1);
    assert_eq!(r.compared_columns, svec(&["Name", "Val"]));
    assert_eq!(r.different_cells, 1); // 只有 Val 不同
    assert!(r
        .differences
        .iter()
        .all(|d| d.column_name.as_deref() != Some("ID")));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn key_multiple_column_changes_reported() {
    let dir = temp_dir("multi_change");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,A,B,C\n001,x,y,z\n").unwrap();
    std::fs::write(&b, "ID,A,B,C\n001,x,y2,z3\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.different_records, 1);
    assert_eq!(r.different_cells, 2);
    let changed: std::collections::HashSet<String> = r
        .differences
        .iter()
        .filter(|d| d.difference_type == DifferenceType::ValueChanged)
        .filter_map(|d| d.column_name.clone())
        .collect();
    assert_eq!(
        changed,
        std::collections::HashSet::from(["B".to_string(), "C".to_string()])
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn key_value_changed_a_and_b_only_combined() {
    let dir = temp_dir("mixed");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,V\n001,1\n002,2\n003,3\n").unwrap();
    std::fs::write(&b, "ID,V\n001,9\n002,2\n004,4\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.matched_records, 2);
    assert_eq!(r.same_records, 1);
    assert_eq!(r.different_records, 1);
    assert_eq!(r.a_only_records, 1);
    assert_eq!(r.b_only_records, 1);
    assert_eq!(r.different_cells, 1);
    assert!(!r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// 效能 / 取消
// ============================================================

fn make_large_csv(rows: usize) -> String {
    let mut s = String::with_capacity(rows * 24);
    s.push_str("ID,Name,Amount\n");
    for i in 0..rows {
        s.push_str(&format!("{:06},{},{}\n", i, "Name", i * 2));
    }
    s
}

#[test]
fn perf_20k_rows_self_compare_identical() {
    let dir = temp_dir("perf");
    let a = dir.join("a.csv");
    let content = make_large_csv(20_000);
    std::fs::write(&a, &content).unwrap();

    let opts = opts_paths(
        &a,
        &a,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);
    assert_eq!(r.rows_a, 20_000);
    assert_eq!(r.rows_b, 20_000);
    assert_eq!(r.matched_records, 20_000);
    assert_eq!(r.same_records, 20_000);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cancel_flag_aborts_large_compare() {
    let dir = temp_dir("cancel");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    let content = make_large_csv(6_000);
    std::fs::write(&a, &content).unwrap();
    std::fs::write(&b, &content).unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    // 事先將取消旗標設為 true，讀到第 5000 列時應中止
    let err = compare_key_based(&opts, Arc::new(AtomicBool::new(true)), |_| {}).unwrap_err();
    assert!(matches!(err, csv_reconcile_lib::error::AppError::Cancelled));

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// Excel 前導零
// ============================================================

#[test]
fn excel_preserves_leading_zero_text() {
    let dir = temp_dir("excel_leading_zero");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Amt\n001,1000\n").unwrap();
    std::fs::write(&b, "ID,Amt\n001,2500\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        &["ID"],
        &[],
    );
    let r = key_compare(&opts);

    let out = dir.join("result.xlsx");
    export_to_excel(&out, &opts, &r).unwrap();

    // 解壓 xlsx，串接所有字串內容，確認前導零 "001" 以文字形式存在
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut content = String::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let name = entry.name().to_string();
        if name.ends_with("sharedStrings.xml") || name.contains("worksheets/sheet") {
            let mut s = String::new();
            entry.read_to_string(&mut s).unwrap();
            content.push_str(&s);
        }
    }
    assert!(
        content.contains("001"),
        "expected leading-zero value 001 to be preserved as text"
    );

    let _ = std::fs::remove_dir_all(dir);
}

// ============================================================
// Row-by-Row 與 Key-based 基本行為對照
// ============================================================

#[test]
fn row_by_row_column_reorder_neutralized() {
    let dir = temp_dir("row_reorder_cols");
    let a = dir.join("a.csv");
    let b = dir.join("b.csv");
    std::fs::write(&a, "ID,Name,Amt\n001,Alice,100\n002,Bob,200\n").unwrap();
    // 欄位順序不同、內容相同、列序相同
    std::fs::write(&b, "Name,Amt,ID\nAlice,100,001\nBob,200,002\n").unwrap();

    let opts = opts_paths(
        &a,
        &b,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        ComparisonMode::RowByRow,
        &[],
        &[],
    );
    let r = compare_row_by_row(&opts, Arc::new(AtomicBool::new(false)), |_| {}).unwrap();
    assert_eq!(r.matched_records, 2);
    assert_eq!(r.same_records, 2);
    assert!(r.identical);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn reader_streams_decoded_content() {
    let dir = temp_dir("reader_stream");
    let a = dir.join("a.csv");
    std::fs::write(&a, cp950("欄一,欄二\n值一,值二\n")).unwrap();

    let (enc, _, skip) = csv_reconcile_lib::parser::get_encoding_for_option(&EncodingOption::Cp950);
    let mut reader = open_delimited_reader(&a, enc, skip, b',').unwrap();
    let headers: Vec<String> = reader.headers().unwrap().iter().map(String::from).collect();
    assert_eq!(headers, svec(&["欄一", "欄二"]));

    let _ = std::fs::remove_dir_all(dir);
}
