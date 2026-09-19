use crate::error::AppError;
use crate::models::file_info::FileInfo;
use crate::models::parse_options::{DelimiterOption, EncodingOption, ParseOptions};
use crate::parser::delimiter::detect_delimiter;
use crate::parser::encoding::{detect_file_encoding, get_encoding_for_option};
use crate::parser::reader::DecodeReader;
use encoding_rs::Encoding;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn open_delimited_reader<P: AsRef<Path>>(
    path: P,
    encoding: &'static Encoding,
    skip_bytes: usize,
    delimiter: u8,
) -> Result<csv::Reader<DecodeReader<BufReader<File>>>, AppError> {
    let file = File::open(path.as_ref()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::FileNotFound(path.as_ref().display().to_string())
        } else {
            AppError::FileAccessError(path.as_ref().display().to_string())
        }
    })?;

    let buf_reader = BufReader::with_capacity(65536, file);
    let decode_reader = DecodeReader::new(buf_reader, encoding, skip_bytes)
        .map_err(|e| AppError::FileAccessError(e.to_string()))?;

    let rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::None)
        .from_reader(decode_reader);

    Ok(rdr)
}

pub fn resolve_parse_settings<P: AsRef<Path>>(
    path: P,
    options: &ParseOptions,
) -> Result<(&'static Encoding, usize, u8), AppError> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(AppError::FileNotFound(path_ref.display().to_string()));
    }

    let (encoding, skip_bytes) = match options.encoding {
        EncodingOption::Auto => {
            let detected = detect_file_encoding(path_ref)?;
            (
                detected.encoding,
                if detected.has_bom {
                    detected.bom_len
                } else {
                    0
                },
            )
        }
        _ => {
            let (encoding, has_bom, bom_len) = get_encoding_for_option(&options.encoding);
            (encoding, if has_bom { bom_len } else { 0 })
        }
    };

    let delimiter_char = match &options.delimiter {
        DelimiterOption::Auto => {
            let mut file =
                File::open(path_ref).map_err(|e| AppError::FileAccessError(e.to_string()))?;
            let mut sample_bytes = vec![0u8; 65536];
            let n = file
                .read(&mut sample_bytes)
                .map_err(|e| AppError::FileAccessError(e.to_string()))?;
            sample_bytes.truncate(n);
            let slice_to_decode = if skip_bytes < sample_bytes.len() {
                &sample_bytes[skip_bytes..]
            } else {
                &sample_bytes[..]
            };
            let (cow, _, _) = encoding.decode(slice_to_decode);
            detect_delimiter(&cow)?
        }
        manual => manual.to_char().unwrap_or(','),
    };

    let delimiter = if delimiter_char.is_ascii() {
        delimiter_char as u8
    } else {
        b','
    };

    Ok((encoding, skip_bytes, delimiter))
}

pub fn validate_headers(headers: &[String]) -> Result<(), AppError> {
    let mut seen = HashSet::new();
    for header in headers {
        if !seen.insert(header.clone()) {
            return Err(AppError::DuplicateHeader(header.clone()));
        }
    }
    Ok(())
}

pub fn inspect_file<P: AsRef<Path>>(path: P, options: &ParseOptions) -> Result<FileInfo, AppError> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(AppError::FileNotFound(path_ref.display().to_string()));
    }

    // 1. Determine encoding and delimiter using the same settings as compare.
    let (encoding, skip_bytes, delim_byte) = resolve_parse_settings(path_ref, options)?;
    let encoding_name = match options.encoding {
        EncodingOption::Auto => detect_file_encoding(path_ref)?
            .encoding_option
            .display_name()
            .to_string(),
        _ => options.encoding.display_name().to_string(),
    };

    let delimiter_char = if delim_byte == b'\t' {
        '\t'
    } else {
        delim_byte as char
    };
    let delimiter_name = match delimiter_char {
        ',' => "Comma (,)".to_string(),
        ';' => "Semicolon (;)".to_string(),
        '\t' => "Tab (\\t)".to_string(),
        '|' => "Pipe (|)".to_string(),
        ':' => "Colon (:)".to_string(),
        c => format!("Custom ({})", c),
    };

    // 2. Read headers and count rows
    let mut rdr = open_delimited_reader(path_ref, encoding, skip_bytes, delim_byte)?;
    let raw_headers = rdr
        .headers()
        .map_err(|e| AppError::General(format!("無法讀取 Header: {}", e)))?
        .clone();

    let headers: Vec<String> = raw_headers.iter().map(|s| s.to_string()).collect();
    validate_headers(&headers)?;

    // Fast estimate or read records count
    let mut row_count = 0u64;
    let mut record = csv::StringRecord::new();
    while rdr
        .read_record(&mut record)
        .map_err(|e| AppError::General(e.to_string()))?
    {
        row_count += 1;
    }

    Ok(FileInfo {
        path: path_ref.display().to_string(),
        encoding: encoding_name,
        delimiter: delimiter_name,
        delimiter_char,
        headers,
        row_count: Some(row_count),
    })
}
