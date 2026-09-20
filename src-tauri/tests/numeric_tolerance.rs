use csv_reconcile_lib::compare::{compare_key_based, compare_row_by_row};
use csv_reconcile_lib::excel::export_to_excel;
use csv_reconcile_lib::models::{CompareOptions, CompareResult, ComparisonMode};
use serde_json::json;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(a: &str, b: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "csv_numeric_{}_{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("a.csv"), a).unwrap();
        std::fs::write(dir.join("b.csv"), b).unwrap();
        Self(dir)
    }

    fn options(&self, mode: &str, tolerance: Option<&str>) -> CompareOptions {
        let mut value = json!({
            "file_a_path": self.0.join("a.csv"),
            "file_b_path": self.0.join("b.csv"),
            "file_a_parse_options": {"encoding": "utf8", "delimiter": {"type": "Pipe"}},
            "file_b_parse_options": {"encoding": "utf8", "delimiter": {"type": "Pipe"}},
            "comparison_mode": mode,
            "key_columns": ["ID"],
            "excluded_columns": [],
            "trim_whitespace": false,
            "ignore_case": false
        });
        if let Some(tolerance) = tolerance {
            value["numeric_tolerance"] = json!(tolerance);
        }
        serde_json::from_value(value).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn compare(options: &CompareOptions) -> CompareResult {
    let cancel = Arc::new(AtomicBool::new(false));
    match options.comparison_mode {
        ComparisonMode::KeyBased => compare_key_based(options, cancel, |_| {}),
        ComparisonMode::RowByRow => compare_row_by_row(options, cancel, |_| {}),
    }
    .unwrap()
}

#[test]
fn both_modes_use_inclusive_exact_decimal_tolerance() {
    let fixture = Fixture::new(
        "ID|Value\n1|1\n2|1\n3|1\n4|-1.1\n5|9007199254740993\n6|0.000000000000000000000000000001\n",
        "ID|Value\n1|1.09\n2|1.1\n3|1.11\n4|-1\n5|9007199254740994\n6|0.000000000000000000000000000002\n",
    );
    for mode in ["key_based", "row_by_row"] {
        let result = compare(&fixture.options(mode, Some("0.1")));
        assert_eq!(result.same_records, 4, "{mode}");
        assert_eq!(result.different_records, 2);
        assert_eq!(result.different_cells, 2);
        assert_eq!(result.differences.len(), 2);
        assert!(!result.identical);
        let result = compare(&fixture.options(mode, Some("1e-30")));
        assert_eq!(result.same_records, 1);
    }
}

#[test]
fn missing_tolerance_defaults_to_numeric_equality() {
    let fixture = Fixture::new(
        "ID|Value\n1|001\n2|1.0\n3|1,234.50\n4|-1.2e3\n5|+.5\n6|-0\n",
        "ID|Value\n1|1\n2|1\n3|1234.5\n4|-1,200\n5|0.50\n6|0\n",
    );
    for mode in ["key_based", "row_by_row"] {
        let result = compare(&fixture.options(mode, None));
        assert!(result.identical, "{mode}: {:?}", result.differences);
        assert_eq!(result.same_records, 6);
        assert_eq!(result.different_cells, 0);
    }
}

#[test]
fn non_numeric_values_follow_text_rules_and_never_become_zero() {
    let fixture = Fixture::new(
        "ID|Value\n1|12,34\n2|10%\n3|$10\n4|(10)\n5|NaN\n6|Infinity\n7|\n8| 1 \n9|abc\n10|1_000\n",
        "ID|Value\n1|1234\n2|0.1\n3|10\n4|-10\n5|0\n6|0\n7|0\n8|1\n9|ABC\n10|1000\n",
    );
    for mode in ["key_based", "row_by_row"] {
        let mut options = fixture.options(mode, Some("10000"));
        assert_eq!(compare(&options).different_cells, 10);
        options.trim_whitespace = true;
        options.ignore_case = true;
        assert_eq!(compare(&options).different_cells, 8);
    }
}

#[test]
fn key_matching_is_exact_but_row_mode_compares_numeric_ids() {
    let fixture = Fixture::new("ID|Value\n001|10\n", "ID|Value\n1|10\n");
    let result = compare(&fixture.options("key_based", Some("100")));
    assert_eq!(result.a_only_records, 1);
    assert_eq!(result.b_only_records, 1);
    assert!(!result.identical);
    assert!(compare(&fixture.options("row_by_row", Some("0"))).identical);
}

#[test]
fn excluded_columns_do_not_affect_numeric_comparison() {
    let fixture = Fixture::new("ID|Value|Skip\n1|1|10\n", "ID|Value|Skip\n1|1.1|99\n");
    for mode in ["key_based", "row_by_row"] {
        let mut options = fixture.options(mode, Some("0.1"));
        options.excluded_columns = vec!["Skip".into()];
        assert!(compare(&options).identical);
    }
}

#[test]
fn invalid_tolerance_is_rejected_by_both_comparators() {
    let fixture = Fixture::new("ID|Value\n1|1\n", "ID|Value\n1|1\n");
    for mode in ["key_based", "row_by_row"] {
        for value in [
            "",
            " ",
            "-0.1",
            "NaN",
            "Infinity",
            "1,00",
            "10%",
            "1e999999999",
        ] {
            let options = fixture.options(mode, Some(value));
            let cancel = Arc::new(AtomicBool::new(false));
            let result = if mode == "key_based" {
                compare_key_based(&options, cancel, |_| {})
            } else {
                compare_row_by_row(&options, cancel, |_| {})
            };
            assert!(result.is_err(), "accepted {value:?} in {mode}");
        }
    }
}

#[test]
fn excel_records_tolerance_and_preserves_original_differences() {
    let fixture = Fixture::new("ID|Value\n1|001.00\n2|1\n", "ID|Value\n1|1.11\n2|1.1\n");
    let options = fixture.options("key_based", Some("0.1"));
    let result = compare(&options);
    assert_eq!(result.differences.len(), 1);
    assert_eq!(result.differences[0].value_a.as_deref(), Some("001.00"));
    let path = fixture.0.join("result.xlsx");
    export_to_excel(&path, &options, &result).unwrap();
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut strings = String::new();
    archive
        .by_name("xl/sharedStrings.xml")
        .unwrap()
        .read_to_string(&mut strings)
        .unwrap();
    assert!(strings.contains("Numeric Tolerance"));
    assert!(strings.contains("<t>0.1</t>"));
    assert!(strings.contains("001.00"));
}
