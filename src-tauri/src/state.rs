use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: SqlitePool,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    /// Discovered at runtime from the IMAP login response or set during setup.
    pub account_email: Arc<Mutex<String>>,
    pub bridge_password: Arc<Mutex<Option<String>>>,
}
