use crate::state::AppState;
use tauri_plugin_keyring::KeyringExt;

const KEYRING_SERVICE: &str = "protonmail-client";
const KEYRING_USER: &str = "bridge-password";
const KEYRING_EMAIL: &str = "account-email";

#[tauri::command]
pub async fn set_bridge_password(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    password: String,
    email: String,
) -> Result<(), String> {
    // Save to macOS Keychain
    app.keyring()
        .set_password(KEYRING_SERVICE, KEYRING_USER, &password)
        .map_err(|e| format!("Failed to save password to Keychain: {}", e))?;

    app.keyring()
        .set_password(KEYRING_SERVICE, KEYRING_EMAIL, &email)
        .map_err(|e| format!("Failed to save email to Keychain: {}", e))?;

    // Update in-memory state
    {
        let mut pw = state.bridge_password.lock().await;
        *pw = Some(password);
    }
    {
        let mut em = state.account_email.lock().await;
        *em = email;
    }

    log::info!("Credentials saved to Keychain, IMAP sync will connect on next cycle");
    Ok(())
}

#[tauri::command]
pub async fn get_connection_status(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let pw = state.bridge_password.lock().await;
    let em = state.account_email.lock().await;
    Ok(pw.is_some() && !em.is_empty())
}

#[tauri::command]
pub async fn get_account_email(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let email = state.account_email.lock().await;
    Ok(email.clone())
}
