use crate::core;
use crate::state::AppState;

#[tauri::command]
pub async fn send_email(
    state: tauri::State<'_, AppState>,
    to: String,
    subject: String,
    body: String,
    is_html: Option<bool>,
) -> Result<(), String> {
    let from = state.account_email.lock().await.clone();
    if from.is_empty() {
        return Err("No account email configured yet — wait for first sync".to_string());
    }

    let transport = core::compose::build_smtp_transport(&state.smtp_host, state.smtp_port);
    core::compose::send_email(&transport, &from, &to, subject, body, is_html.unwrap_or(false))
        .await
}

#[tauri::command]
pub async fn reply_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
    body: String,
) -> Result<(), String> {
    let from = state.account_email.lock().await.clone();
    if from.is_empty() {
        return Err("No account email configured yet — wait for first sync".to_string());
    }

    let transport = core::compose::build_smtp_transport(&state.smtp_host, state.smtp_port);
    core::compose::reply_email(&state.db, &transport, &from, uid, &mailbox_id, body).await
}

#[tauri::command]
pub async fn forward_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
    to: String,
) -> Result<(), String> {
    let from = state.account_email.lock().await.clone();
    if from.is_empty() {
        return Err("No account email configured yet — wait for first sync".to_string());
    }

    let transport = core::compose::build_smtp_transport(&state.smtp_host, state.smtp_port);
    core::compose::forward_email(&state.db, &transport, &from, uid, &mailbox_id, &to).await
}
