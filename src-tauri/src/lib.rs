pub mod commands;
pub mod compare;
pub mod error;
pub mod excel;
pub mod models;
pub mod parser;

use commands::cancel_compare::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cancel_token = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { cancel_token })
        .invoke_handler(tauri::generate_handler![
            commands::inspect_file::inspect_file,
            commands::compare_files::compare_files,
            commands::cancel_compare::cancel_compare,
            commands::export_excel::export_excel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
