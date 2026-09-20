use crate::error::AppError;
use bigdecimal::BigDecimal;
use regex::Regex;
use std::str::FromStr;
use std::sync::OnceLock;

// Bound expansion of scientific notation before exact decimal subtraction.
// Keep these limits in sync with src/lib/numericTolerance.ts.
const MAX_NUMERIC_LENGTH: usize = 4096;
const MAX_EXPONENT: i32 = 4096;

fn parse_numeric(value: &str) -> Option<BigDecimal> {
    static FORMAT: OnceLock<Regex> = OnceLock::new();
    if value.len() > MAX_NUMERIC_LENGTH {
        return None;
    }
    let format = FORMAT.get_or_init(|| {
        Regex::new(r"\A[+-]?(?:(?:[0-9]+|[0-9]{1,3}(?:,[0-9]{3})+)(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?\z")
            .expect("valid numeric format")
    });
    if !format.is_match(value) {
        return None;
    }
    if let Some((_, exponent)) = value.split_once(['e', 'E']) {
        let exponent = exponent.parse::<i32>().ok()?;
        if !(-MAX_EXPONENT..=MAX_EXPONENT).contains(&exponent) {
            return None;
        }
    }
    BigDecimal::from_str(&value.replace(',', "")).ok()
}

pub fn parse_tolerance(value: &str) -> Result<BigDecimal, AppError> {
    parse_numeric(value)
        .filter(|number| number >= &BigDecimal::from(0))
        .ok_or_else(|| {
            AppError::General(
                "數值誤差容許值必須為非負數，最多 4096 字元，科學記號指數須介於 -4096 至 4096。"
                    .into(),
            )
        })
}

/// Returns true when the values differ under the configured rules.
pub fn compare_values(
    a: &str,
    b: &str,
    trim: bool,
    ignore_case: bool,
    tolerance: &BigDecimal,
) -> bool {
    let a = if trim { a.trim() } else { a };
    let b = if trim { b.trim() } else { b };
    if a == b {
        return false;
    }
    if let (Some(a), Some(b)) = (parse_numeric(a), parse_numeric(b)) {
        return (a - b).abs() > *tolerance;
    }
    if ignore_case {
        a.to_lowercase() != b.to_lowercase()
    } else {
        a != b
    }
}
