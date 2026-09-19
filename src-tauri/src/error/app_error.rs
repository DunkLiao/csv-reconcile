use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("找不到檔案：{0}")]
    FileNotFound(String),

    #[error("無法讀取檔案：{0}。請確認檔案是否存在，以及目前帳號是否具有讀取權限。")]
    FileAccessError(String),

    #[error("無法可靠判斷文字編碼。請手動指定：UTF-8 / Big5 / CP950 / UTF-16 LE / UTF-16 BE")]
    EncodingError(String),

    #[error("無法可靠判斷欄位分隔符號。請手動指定 Delimiter。")]
    DelimiterError(String),

    #[error("檔案格式異常。Record: {record_index}，預期欄位: {expected}，實際欄位: {actual}")]
    InvalidRecord {
        record_index: u64,
        expected: usize,
        actual: usize,
    },

    #[error("發現重複欄位名稱：{0}。請修正來源檔案後重新執行。")]
    DuplicateHeader(String),

    #[error("Key 欄位不存在於檔案：{0}")]
    MissingKeyColumn(String),

    #[error("比對未指定任何 Key 欄位")]
    NoKeySpecified,

    #[error("比對未找到任何共同欄位")]
    NoCommonColumns,

    #[error("Excel 報告無法儲存：{0}。請確認輸出資料夾權限、同名檔案是否已開啟或磁碟空間。")]
    ExportError(String),

    #[error("比較已由使用者取消")]
    Cancelled,

    #[error("系統錯誤：{0}")]
    General(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileAccessError(err.to_string())
    }
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> Self {
        AppError::General(err.to_string())
    }
}
