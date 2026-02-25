use crate::core;
use crate::db::{queries, schema::*};
use crate::state::AppState;

#[tauri::command]
pub async fn list_emails(
    state: tauri::State<'_, AppState>,
    mailbox_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Email>, String> {
    queries::get_emails(
        &state.db,
        &mailbox_id,
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<EmailDetail, String> {
    let password = state
        .bridge_password
        .lock()
        .await
        .clone()
        .ok_or_else(|| "No bridge password".to_string())?;

    let login_user = state.account_email.lock().await.clone();
    if login_user.is_empty() {
        return Err("No account email configured".to_string());
    }

    core::mail::get_email_with_fetch(
        &state.db,
        uid,
        &mailbox_id,
        &state.imap_host,
        state.imap_port,
        &login_user,
        &password,
    )
    .await
}

#[tauri::command]
pub async fn mark_read(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<(), String> {
    core::mail::mark_read(&state.db, uid, &mailbox_id).await
}

#[tauri::command]
pub async fn mark_unread(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<(), String> {
    core::mail::mark_unread(&state.db, uid, &mailbox_id).await
}

#[tauri::command]
pub async fn star_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<(), String> {
    core::mail::star(&state.db, uid, &mailbox_id).await
}

#[tauri::command]
pub async fn unstar_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<(), String> {
    core::mail::unstar(&state.db, uid, &mailbox_id).await
}

#[tauri::command]
pub async fn delete_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<(), String> {
    queries::delete_email(&state.db, uid, &mailbox_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn move_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    from_mailbox: String,
    to_mailbox: String,
) -> Result<(), String> {
    queries::move_email(&state.db, uid, &from_mailbox, &to_mailbox)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<(), String> {
    core::mail::archive(&state.db, uid, &mailbox_id).await
}
