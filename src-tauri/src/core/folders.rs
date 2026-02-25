use crate::db::{queries, schema::*};
use sqlx::SqlitePool;

pub async fn create_folder(pool: &SqlitePool, name: &str) -> Result<Mailbox, String> {
    let id = uuid::Uuid::new_v4().to_string();

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No account configured".to_string())?;

    let mailbox = Mailbox {
        id,
        account_id: account.id,
        name: name.to_string(),
        path: name.to_string(),
        uid_validity: None,
        last_synced_uid: Some(0),
        unread_count: Some(0),
    };

    queries::upsert_mailbox(pool, &mailbox)
        .await
        .map_err(|e| e.to_string())?;

    Ok(mailbox)
}
