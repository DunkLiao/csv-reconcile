use super::cancel_compare::AppState;
use crate::compare::{compare_key_based, compare_row_by_row};
use crate::error::AppError;
use crate::models::compare_options::{CompareOptions, ComparisonMode};
use crate::models::compare_result::CompareResult;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn compare_files(
    app: AppHandle,
    state: State<'_, AppState>,
    options: CompareOptions,
) -> Result<CompareResult, AppError> {
    let cancel_token = state.cancel_token.clone();
    cancel_token.store(false, Ordering::Relaxed);

    tokio::task::spawn_blocking(move || {
        let app_handle = app.clone();
        let progress_cb = move |payload| {
            let _ = app_handle.emit("compare-progress", payload);
        };

        match options.comparison_mode {
            ComparisonMode::KeyBased => compare_key_based(&options, cancel_token, progress_cb),
            ComparisonMode::RowByRow => compare_row_by_row(&options, cancel_token, progress_cb),
        }
    })
    .await
    .map_err(|e| AppError::General(e.to_string()))?
}
