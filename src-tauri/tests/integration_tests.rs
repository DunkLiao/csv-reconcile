use csv_reconcile_lib::compare::{compare_key_based, compare_row_by_row};
use csv_reconcile_lib::excel::export_to_excel;
use csv_reconcile_lib::models::compare_options::{CompareOptions, ComparisonMode};
use csv_reconcile_lib::models::difference::DifferenceType;
use csv_reconcile_lib::models::parse_options::{DelimiterOption, EncodingOption, ParseOptions};
use csv_reconcile_lib::parser::delimiter::detect_delimiter;
use csv_reconcile_lib::parser::encoding::detect_bytes_encoding;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[test]
fn test_dialog_capability_allows_file_selection_and_export() {
    let capability_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("capabilities")
        .join("default.json");
    let capability = std::fs::read_to_string(&capability_path).unwrap();

    assert!(capability.contains("dialog:default"));
}

#[test]
fn test_encoding_bom_detection() {
    let utf8_bom = [0xEF, 0xBB, 0xBF, b'H', b'e', b'l', b'l', b'o'];
    let detected = detect_bytes_encoding(&utf8_bom).unwrap();
    assert_eq!(detected.encoding_option, EncodingOption::Utf8Bom);
    assert!(detected.has_bom);

    let utf16le = [0xFF, 0xFE, b'H', 0x00, b'i', 0x00];
    let detected16 = detect_bytes_encoding(&utf16le).unwrap();
    assert_eq!(detected16.encoding_option, EncodingOption::Utf16Le);
    assert!(detected16.has_bom);
}

#[test]
fn test_delimiter_detection() {
    let comma_sample = "ID,Name,Amount\n1,Alice,100\n2,Bob,200\n3,Charlie,300";
    assert_eq!(detect_delimiter(comma_sample).unwrap(), ',');

    let pipe_sample = "ID|Name|Amount\n1|Alice|100\n2|Bob|200\n3|Charlie|300";
    assert_eq!(detect_delimiter(pipe_sample).unwrap(), '|');

    let tab_sample = "ID\tName\tAmount\n1\tAlice\t100\n2\tBob\t200";
    assert_eq!(detect_delimiter(tab_sample).unwrap(), '\t');

    let semicolon_sample = "ID;Name;Amount\n1;Alice;100\n2;Bob;200";
    assert_eq!(detect_delimiter(semicolon_sample).unwrap(), ';');
}

#[test]
fn test_spec_section_71_integration_case() {
    let temp_dir = std::env::temp_dir().join("csv_compare_test_spec71");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let file_a_path = temp_dir.join("file_a.csv");
    let file_b_path = temp_dir.join("file_b.txt");

    // File A: Comma separated, CP950 / UTF-8
    let content_a = "\"客戶編號\",\"交易日期\",\"交易序號\",\"金額\",\"更新時間\"\n\
\"001\",\"2026/09/18\",\"01\",\"1000\",\"08:00\"\n\
\"001\",\"2026/09/18\",\"02\",\"2000\",\"08:00\"\n\
\"002\",\"2026/09/18\",\"01\",\"3000\",\"08:00\"\n";

    // File B: Pipe separated, different column order and different row order
    let content_b = "\"交易日期\"|\"交易序號\"|\"客戶編號\"|\"金額\"|\"更新時間\"\n\
\"2026/09/18\"|\"01\"|\"002\"|\"3000\"|\"09:30\"\n\
\"2026/09/18\"|\"01\"|\"001\"|\"1000\"|\"09:30\"\n\
\"2026/09/18\"|\"02\"|\"001\"|\"2500\"|\"09:30\"\n";

    File::create(&file_a_path)
        .unwrap()
        .write_all(content_a.as_bytes())
        .unwrap();
    File::create(&file_b_path)
        .unwrap()
        .write_all(content_b.as_bytes())
        .unwrap();

    let options = CompareOptions {
        file_a_path: file_a_path.to_string_lossy().to_string(),
        file_b_path: file_b_path.to_string_lossy().to_string(),
        file_a_parse_options: ParseOptions {
            encoding: EncodingOption::Utf8,
            delimiter: DelimiterOption::Comma,
        },
        file_b_parse_options: ParseOptions {
            encoding: EncodingOption::Utf8,
            delimiter: DelimiterOption::Pipe,
        },
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec![
            "客戶編號".to_string(),
            "交易日期".to_string(),
            "交易序號".to_string(),
        ],
        excluded_columns: vec!["更新時間".to_string()],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = compare_key_based(&options, cancel_token, |_| {}).unwrap();

    assert_eq!(result.rows_a, 3);
    assert_eq!(result.rows_b, 3);
    assert_eq!(result.matched_records, 3);
    assert_eq!(result.same_records, 2);
    assert_eq!(result.different_records, 1);
    assert_eq!(result.a_only_records, 0);
    assert_eq!(result.b_only_records, 0);
    assert_eq!(result.different_cells, 1);
    assert_eq!(result.duplicate_keys_a, 0);
    assert_eq!(result.duplicate_keys_b, 0);
    assert!(!result.identical);

    // Verify the single difference is Amount 2000 vs 2500
    assert_eq!(result.differences.len(), 1);
    let diff = &result.differences[0];
    assert_eq!(diff.difference_type, DifferenceType::ValueChanged);
    assert_eq!(diff.column_name.as_deref(), Some("金額"));
    assert_eq!(diff.value_a.as_deref(), Some("2000"));
    assert_eq!(diff.value_b.as_deref(), Some("2500"));

    // Verify Excel export
    let excel_path = temp_dir.join("test_result.xlsx");
    export_to_excel(&excel_path, &options, &result).unwrap();
    assert!(excel_path.exists());
    assert!(excel_path.metadata().unwrap().len() > 1000);

    // Cleanup
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_row_by_row_compare() {
    let temp_dir = std::env::temp_dir().join("csv_compare_test_row_by_row");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let file_a_path = temp_dir.join("file_a.csv");
    let file_b_path = temp_dir.join("file_b.csv");

    let content_a = "ID,Name,Amount\n001,Alice,100\n002,Bob,200\n003,Charlie,300\n";
    let content_b = "Name,Amount,ID\nAlice,100,001\nBob,250,002\n"; // Different order, Bob has 250, missing 003

    File::create(&file_a_path)
        .unwrap()
        .write_all(content_a.as_bytes())
        .unwrap();
    File::create(&file_b_path)
        .unwrap()
        .write_all(content_b.as_bytes())
        .unwrap();

    let options = CompareOptions {
        file_a_path: file_a_path.to_string_lossy().to_string(),
        file_b_path: file_b_path.to_string_lossy().to_string(),
        file_a_parse_options: ParseOptions::default(),
        file_b_parse_options: ParseOptions::default(),
        comparison_mode: ComparisonMode::RowByRow,
        key_columns: vec![],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = compare_row_by_row(&options, cancel_token, |_| {}).unwrap();

    assert_eq!(result.rows_a, 3);
    assert_eq!(result.rows_b, 2);
    assert_eq!(result.matched_records, 2);
    assert_eq!(result.same_records, 1);
    assert_eq!(result.different_records, 1);
    assert_eq!(result.a_only_records, 1);
    assert_eq!(result.b_only_records, 0);

    // Cleanup
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_quoted_field_with_newline_and_delimiter() {
    let temp_dir = std::env::temp_dir().join("csv_compare_test_quoted");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let file_a_path = temp_dir.join("file_a.csv");
    let file_b_path = temp_dir.join("file_b.csv");

    // CSV with embedded comma, newline, and quotes
    let content_a = "\"ID\",\"Address\",\"Note\"\n\
\"001\",\"Taipei, Da'an\",\"Customer said \"\"pay today\"\"\"\n\
\"002\",\"Kaohsiung\",\"Line 1\nLine 2\"\n";

    let content_b = "\"ID\",\"Address\",\"Note\"\n\
\"001\",\"Taipei, Da'an\",\"Customer said \"\"pay today\"\"\"\n\
\"002\",\"Kaohsiung\",\"Line 1\nLine 2\"\n";

    File::create(&file_a_path)
        .unwrap()
        .write_all(content_a.as_bytes())
        .unwrap();
    File::create(&file_b_path)
        .unwrap()
        .write_all(content_b.as_bytes())
        .unwrap();

    let options = CompareOptions {
        file_a_path: file_a_path.to_string_lossy().to_string(),
        file_b_path: file_b_path.to_string_lossy().to_string(),
        file_a_parse_options: ParseOptions::default(),
        file_b_parse_options: ParseOptions::default(),
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["ID".to_string()],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = compare_key_based(&options, cancel_token, |_| {}).unwrap();

    assert_eq!(result.rows_a, 2);
    assert_eq!(result.rows_b, 2);
    assert!(result.identical);
    assert_eq!(result.different_records, 0);

    // Cleanup
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_duplicate_key_detection() {
    let temp_dir = std::env::temp_dir().join("csv_compare_test_dups");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let file_a_path = temp_dir.join("file_a.csv");
    let file_b_path = temp_dir.join("file_b.csv");

    // File A has duplicate key "001"
    let content_a = "ID,Name\n001,Alice\n001,AliceDuplicate\n002,Bob\n";
    let content_b = "ID,Name\n001,Alice\n002,Bob\n";

    File::create(&file_a_path)
        .unwrap()
        .write_all(content_a.as_bytes())
        .unwrap();
    File::create(&file_b_path)
        .unwrap()
        .write_all(content_b.as_bytes())
        .unwrap();

    let options = CompareOptions {
        file_a_path: file_a_path.to_string_lossy().to_string(),
        file_b_path: file_b_path.to_string_lossy().to_string(),
        file_a_parse_options: ParseOptions::default(),
        file_b_parse_options: ParseOptions::default(),
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["ID".to_string()],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = compare_key_based(&options, cancel_token, |_| {}).unwrap();

    assert_eq!(result.duplicate_keys_a, 1);
    assert_eq!(result.duplicate_key_records.len(), 1);
    assert_eq!(result.duplicate_key_records[0].source, "File A");
    assert_eq!(result.duplicate_key_records[0].count, 2);
    assert_eq!(result.duplicate_key_records[0].rows, vec![1, 2]);

    // Cleanup
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_big5_encoding_and_generate_samples() {
    let big5_a_text = "\"客戶編號\",\"分行名稱\",\"金額\"\r\n\"001\",\"台北分行\",\"50000\"\r\n\"002\",\"高雄分行\",\"12000\"\r\n";
    let big5_b_text = "\"客戶編號\",\"分行名稱\",\"金額\"\r\n\"001\",\"台北分行\",\"55000\"\r\n\"002\",\"高雄分行\",\"12000\"\r\n";

    let (encoded_a, _, _) = encoding_rs::BIG5.encode(big5_a_text);
    let (encoded_b, _, _) = encoding_rs::BIG5.encode(big5_b_text);

    // Save to sample_data directory for acceptance test
    let sample_dir = std::path::Path::new("..").join("sample_data");
    if sample_dir.exists() {
        let _ = std::fs::write(sample_dir.join("big5_file_a.csv"), &*encoded_a);
        let _ = std::fs::write(sample_dir.join("big5_file_b.csv"), &*encoded_b);
    }

    let temp_dir = std::env::temp_dir().join("csv_compare_test_big5");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_a_path = temp_dir.join("big5_a.csv");
    let file_b_path = temp_dir.join("big5_b.csv");
    std::fs::write(&file_a_path, &*encoded_a).unwrap();
    std::fs::write(&file_b_path, &*encoded_b).unwrap();

    let options = CompareOptions {
        file_a_path: file_a_path.to_string_lossy().to_string(),
        file_b_path: file_b_path.to_string_lossy().to_string(),
        file_a_parse_options: ParseOptions {
            encoding: EncodingOption::Cp950,
            delimiter: DelimiterOption::Comma,
        },
        file_b_parse_options: ParseOptions {
            encoding: EncodingOption::Cp950,
            delimiter: DelimiterOption::Comma,
        },
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: vec!["客戶編號".to_string()],
        excluded_columns: vec![],
        trim_whitespace: false,
        ignore_case: false,
        numeric_tolerance: "0".into(),
    };

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = compare_key_based(&options, cancel_token, |_| {}).unwrap();

    assert_eq!(result.rows_a, 2);
    assert_eq!(result.rows_b, 2);
    assert_eq!(result.matched_records, 2);
    assert_eq!(result.different_records, 1);
    assert_eq!(result.differences.len(), 1);
    assert_eq!(result.differences[0].column_name.as_deref(), Some("金額"));
    assert_eq!(result.differences[0].value_a.as_deref(), Some("50000"));
    assert_eq!(result.differences[0].value_b.as_deref(), Some("55000"));

    let _ = std::fs::remove_dir_all(temp_dir);
}
