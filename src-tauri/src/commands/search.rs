use crate::db::{queries, schema::*};
use crate::state::AppState;

#[tauri::command]
pub async fn search_emails(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<Email>, String> {
    queries::search_emails_fts(&state.db, &query, limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())
}
