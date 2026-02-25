pub mod classify;
mod commands;
pub mod core;
pub mod db;
mod hydroxide;
pub mod rules;
pub mod state;
mod sync;

use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

const KEYRING_SERVICE: &str = "protonmail-client";
const KEYRING_USER: &str = "bridge-password";
const KEYRING_EMAIL: &str = "account-email";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_keyring::init())
        .setup(|app| {
            use tauri::Manager;
            use tauri_plugin_keyring::KeyringExt;

            // Start hydroxide if not already running
            hydroxide::ensure_running();

            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("protonmail.db");
            let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

            let pool = tauri::async_runtime::block_on(async {
                sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&db_url)
                    .await
            })
            .map_err(|e| format!("Failed to create SQLite pool: {}", e))?;

            // Try to load credentials from macOS Keychain
            let saved_password = app
                .keyring()
                .get_password(KEYRING_SERVICE, KEYRING_USER)
                .ok()
                .flatten();

            let saved_email = app
                .keyring()
                .get_password(KEYRING_SERVICE, KEYRING_EMAIL)
                .ok()
                .flatten()
                .unwrap_or_default();

            if saved_password.is_some() && !saved_email.is_empty() {
                log::info!("Credentials loaded from Keychain for {}", saved_email);
            } else {
                log::info!("No credentials in Keychain, awaiting user input");
            }

            let state = AppState {
                db: pool.clone(),
                imap_host: "127.0.0.1".to_string(),
                imap_port: 1143,
                smtp_host: "127.0.0.1".to_string(),
                smtp_port: 1025,
                account_email: Arc::new(Mutex::new(saved_email)),
                bridge_password: Arc::new(Mutex::new(saved_password)),
            };

            app.manage(state);

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::init_db(&pool).await {
                    log::error!("Failed to initialize database: {}", e);
                    return;
                }
                log::info!("Database initialized successfully");
                sync::start_sync(handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::mail::list_emails,
            commands::mail::get_email,
            commands::mail::mark_read,
            commands::mail::mark_unread,
            commands::mail::star_email,
            commands::mail::unstar_email,
            commands::mail::delete_email,
            commands::mail::move_email,
            commands::mail::archive_email,
            commands::compose::send_email,
            commands::compose::reply_email,
            commands::compose::forward_email,
            commands::folders::list_mailboxes,
            commands::folders::create_folder,
            commands::search::search_emails,
            commands::rules::list_rules,
            commands::rules::create_rule,
            commands::rules::update_rule,
            commands::rules::delete_rule,
            commands::classify::classify_email,
            commands::account::set_bridge_password,
            commands::account::get_connection_status,
            commands::account::get_account_email,
        ])
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::Exit = event {
                hydroxide::stop_if_we_started();
            }
        });
}
