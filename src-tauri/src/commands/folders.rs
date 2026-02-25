use crate::core;
use crate::db::{queries, schema::*};
use crate::state::AppState;

#[tauri::command]
pub async fn list_mailboxes(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Mailbox>, String> {
    queries::get_mailboxes(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_folder(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<Mailbox, String> {
    core::folders::create_folder(&state.db, &name).await
}
