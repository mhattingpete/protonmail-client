use crate::classify;
use crate::db::{queries, schema::*};
use crate::state::AppState;

#[tauri::command]
pub async fn classify_email(
    state: tauri::State<'_, AppState>,
    uid: i64,
    mailbox_id: String,
) -> Result<Classification, String> {
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

    let detail = crate::core::mail::get_email_with_fetch(
        &state.db,
        uid,
        &mailbox_id,
        &state.imap_host,
        state.imap_port,
        &login_user,
        &password,
    )
    .await?;

    let result = classify::classify_email_cascade(&detail).await?;

    queries::update_classification(
        &state.db,
        uid,
        &mailbox_id,
        &result.label,
        result.confidence,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}
