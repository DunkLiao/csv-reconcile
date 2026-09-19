pub mod delimited_parser;
pub mod delimiter;
pub mod encoding;
pub mod reader;

pub use delimited_parser::{
    inspect_file, open_delimited_reader, resolve_parse_settings, validate_headers,
};
pub use delimiter::detect_delimiter;
pub use encoding::{detect_bytes_encoding, detect_file_encoding, get_encoding_for_option};
pub use reader::DecodeReader;
