use crate::error::AppError;
use crate::models::parse_options::EncodingOption;
use encoding_rs::{Encoding, BIG5, UTF_16BE, UTF_16LE, UTF_8};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct DetectedEncoding {
    pub encoding_option: EncodingOption,
    pub encoding: &'static Encoding,
    pub has_bom: bool,
    pub bom_len: usize,
}

pub fn detect_file_encoding<P: AsRef<Path>>(path: P) -> Result<DetectedEncoding, AppError> {
    let mut file = File::open(path.as_ref()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::FileNotFound(path.as_ref().display().to_string())
        } else {
            AppError::FileAccessError(path.as_ref().display().to_string())
        }
    })?;

    // Read up to 64KB for detection
    let mut buffer = vec![0u8; 65536];
    let n = file
        .read(&mut buffer)
        .map_err(|e| AppError::FileAccessError(e.to_string()))?;
    buffer.truncate(n);

    detect_bytes_encoding(&buffer)
}

pub fn detect_bytes_encoding(bytes: &[u8]) -> Result<DetectedEncoding, AppError> {
    // 1. Check BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(DetectedEncoding {
            encoding_option: EncodingOption::Utf8Bom,
            encoding: UTF_8,
            has_bom: true,
            bom_len: 3,
        });
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Ok(DetectedEncoding {
            encoding_option: EncodingOption::Utf16Le,
            encoding: UTF_16LE,
            has_bom: true,
            bom_len: 2,
        });
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Ok(DetectedEncoding {
            encoding_option: EncodingOption::Utf16Be,
            encoding: UTF_16BE,
            has_bom: true,
            bom_len: 2,
        });
    }

    if bytes.is_empty() {
        return Ok(DetectedEncoding {
            encoding_option: EncodingOption::Utf8,
            encoding: UTF_8,
            has_bom: false,
            bom_len: 0,
        });
    }

    // 2. UTF-8 Validation
    if std::str::from_utf8(bytes).is_ok() {
        return Ok(DetectedEncoding {
            encoding_option: EncodingOption::Utf8,
            encoding: UTF_8,
            has_bom: false,
            bom_len: 0,
        });
    }

    // 3. chardetng detection
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let (detected, is_confident) = detector.guess_assess(None, true);

    // If detector guesses Big5 or CP950 or is uncertain, fallback or return CP950
    if detected == BIG5 || !is_confident {
        // Test if bytes decode cleanly as Big5/CP950
        let (cow, _, had_errors) = BIG5.decode(bytes);
        if !had_errors && !cow.is_empty() {
            return Ok(DetectedEncoding {
                encoding_option: EncodingOption::Cp950,
                encoding: BIG5,
                has_bom: false,
                bom_len: 0,
            });
        }
    }

    if is_confident {
        let opt = match detected.name() {
            "UTF-8" => EncodingOption::Utf8,
            "Big5" => EncodingOption::Cp950, // encoding_rs BIG5 is CP950
            "windows-1252" => EncodingOption::Utf8, // Often pure ASCII mistaken
            _ => EncodingOption::Cp950,
        };
        return Ok(DetectedEncoding {
            encoding_option: opt,
            encoding: detected,
            has_bom: false,
            bom_len: 0,
        });
    }

    // Fallback default for Traditional Chinese Windows environment
    Ok(DetectedEncoding {
        encoding_option: EncodingOption::Cp950,
        encoding: BIG5,
        has_bom: false,
        bom_len: 0,
    })
}

pub fn get_encoding_for_option(option: &EncodingOption) -> (&'static Encoding, bool, usize) {
    match option {
        EncodingOption::Auto => (UTF_8, false, 0), // Default placeholder
        EncodingOption::Utf8 => (UTF_8, false, 0),
        EncodingOption::Utf8Bom => (UTF_8, true, 3),
        EncodingOption::Big5 | EncodingOption::Cp950 => (BIG5, false, 0),
        EncodingOption::Utf16Le => (UTF_16LE, false, 0),
        EncodingOption::Utf16Be => (UTF_16BE, false, 0),
    }
}
