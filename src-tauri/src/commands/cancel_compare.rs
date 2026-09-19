use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;

pub struct AppState {
    pub cancel_token: Arc<AtomicBool>,
}

#[tauri::command]
pub async fn cancel_compare(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_token.store(true, Ordering::Relaxed);
    Ok(())
}
