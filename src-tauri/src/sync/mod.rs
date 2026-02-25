pub mod imap_sync;

use tauri::Manager;

use crate::state::AppState;

pub async fn start_sync(app_handle: tauri::AppHandle) {
    let state: tauri::State<'_, AppState> = app_handle.state();
    let db = state.db.clone();
    let host = state.imap_host.clone();
    let port = state.imap_port;
    let email_lock = state.account_email.clone();
    let password_lock = state.bridge_password.clone();

    loop {
        let password = {
            let guard = password_lock.lock().await;
            guard.clone()
        };

        let Some(password) = password else {
            log::info!("No bridge password configured, waiting...");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            continue;
        };

        let login_user = {
            let email = email_lock.lock().await;
            if email.is_empty() {
                log::warn!("No account email configured, waiting...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            email.clone()
        };

        match imap_sync::sync_loop(&app_handle, &db, &host, port, &login_user, &password).await {
            Ok(()) => {
                log::info!("IMAP sync loop ended normally");
                // After successful sync, store the account email we discovered
                let account = sqlx::query_as::<_, crate::db::schema::Account>(
                    "SELECT * FROM accounts LIMIT 1",
                )
                .fetch_optional(&db)
                .await;
                if let Ok(Some(account)) = account {
                    let mut email = email_lock.lock().await;
                    if email.is_empty() {
                        *email = account.email;
                    }
                }
            }
            Err(e) => {
                log::error!("IMAP sync error: {}, retrying in 30s", e);
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        }
    }
}
