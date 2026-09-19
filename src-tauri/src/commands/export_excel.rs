use crate::error::AppError;
use crate::excel::export_to_excel as run_export_to_excel;
use crate::models::compare_options::CompareOptions;
use crate::models::compare_result::CompareResult;

#[tauri::command]
pub async fn export_excel(
    output_path: String,
    options: CompareOptions,
    result: CompareResult,
) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || run_export_to_excel(&output_path, &options, &result))
        .await
        .map_err(|e| AppError::General(e.to_string()))?
}
