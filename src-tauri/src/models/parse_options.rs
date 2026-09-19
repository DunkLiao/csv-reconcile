use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EncodingOption {
    Auto,
    Utf8,
    Utf8Bom,
    Big5,
    Cp950,
    Utf16Le,
    Utf16Be,
}

impl Default for EncodingOption {
    fn default() -> Self {
        Self::Auto
    }
}

impl EncodingOption {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Big5 => "Big5",
            Self::Cp950 => "CP950",
            Self::Utf16Le => "UTF-16 LE",
            Self::Utf16Be => "UTF-16 BE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum DelimiterOption {
    Auto,
    Comma,
    Semicolon,
    Tab,
    Pipe,
    Colon,
    Custom(char),
}

impl Default for DelimiterOption {
    fn default() -> Self {
        Self::Auto
    }
}

impl DelimiterOption {
    pub fn display_name(&self) -> String {
        match self {
            Self::Auto => "Auto".to_string(),
            Self::Comma => "Comma (,)".to_string(),
            Self::Semicolon => "Semicolon (;)".to_string(),
            Self::Tab => "Tab (\\t)".to_string(),
            Self::Pipe => "Pipe (|)".to_string(),
            Self::Colon => "Colon (:)".to_string(),
            Self::Custom(c) => format!("Custom ({})", c),
        }
    }

    pub fn to_char(&self) -> Option<char> {
        match self {
            Self::Auto => None,
            Self::Comma => Some(','),
            Self::Semicolon => Some(';'),
            Self::Tab => Some('\t'),
            Self::Pipe => Some('|'),
            Self::Colon => Some(':'),
            Self::Custom(c) => Some(*c),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParseOptions {
    pub encoding: EncodingOption,
    pub delimiter: DelimiterOption,
}
