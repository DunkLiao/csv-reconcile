use crate::error::AppError;
use crate::models::file_info::FileInfo;
use crate::models::parse_options::ParseOptions;
use crate::parser::delimited_parser::inspect_file as run_inspect_file;

#[tauri::command]
pub async fn inspect_file(
    path: String,
    options: Option<ParseOptions>,
) -> Result<FileInfo, AppError> {
    tokio::task::spawn_blocking(move || {
        let opts = options.unwrap_or_default();
        run_inspect_file(&path, &opts)
    })
    .await
    .map_err(|e| AppError::General(e.to_string()))?
}
