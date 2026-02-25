use crate::core::imap::{self, ImapSession, SyncEvent};
use sqlx::SqlitePool;
use tauri::Emitter;

pub async fn sync_loop(
    app_handle: &tauri::AppHandle,
    db: &SqlitePool,
    host: &str,
    port: u16,
    email: &str,
    password: &str,
) -> Result<(), String> {
    let mut session = imap::connect_imap(host, port, email, password).await?;

    let emit_handle = app_handle.clone();
    let on_event = move |event: SyncEvent| {
        match &event {
            SyncEvent::SyncComplete { mailbox, count } => {
                if *count > 0 {
                    log::info!("Synced {} new emails in {}", count, mailbox);
                    emit_handle.emit("emails-updated", ()).ok();
                }
            }
            SyncEvent::SyncStart { mailbox, new_count, last_uid } => {
                log::info!(
                    "Syncing {} — {} new messages (last_uid={})",
                    mailbox, new_count, last_uid
                );
            }
            _ => {}
        }
    };

    let mailbox_ids = imap::sync_all(&mut session, db, email, &on_event).await?;

    app_handle
        .emit("emails-updated", ())
        .map_err(|e| e.to_string())?;

    // IDLE loop on INBOX for real-time notifications
    let inbox_entry = mailbox_ids
        .iter()
        .find(|(_, name)| name.eq_ignore_ascii_case("INBOX"));

    if let Some((inbox_id, _)) = inbox_entry {
        let inbox_id = inbox_id.clone();
        idle_loop(session, db, app_handle, &inbox_id, host, port, email, password, &on_event).await?;
    } else {
        poll_loop(session, db, app_handle, &mailbox_ids, &on_event).await?;
    }

    Ok(())
}

async fn idle_loop<F>(
    mut session: ImapSession,
    db: &SqlitePool,
    app_handle: &tauri::AppHandle,
    inbox_id: &str,
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    on_event: &F,
) -> Result<(), String>
where
    F: Fn(SyncEvent),
{
    session
        .select("INBOX")
        .await
        .map_err(|e| format!("Failed to select INBOX for IDLE: {}", e))?;

    loop {
        let mut handle = session.idle();

        tokio::select! {
            result = handle.init() => {
                match result {
                    Ok(_) => log::info!("IDLE notification received"),
                    Err(e) => log::warn!("IDLE error: {}", e),
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                log::debug!("IDLE timeout, re-syncing");
            }
        }

        session = match handle.done().await {
            Ok(s) => s,
            Err(e) => {
                log::error!("IDLE done failed: {}, reconnecting", e);
                imap::connect_imap(host, port, email, password).await?
            }
        };

        if let Err(e) = imap::sync_mailbox(&mut session, db, inbox_id, "INBOX", on_event).await {
            log::error!("Error syncing INBOX after IDLE: {}", e);
        }

        app_handle.emit("emails-updated", ()).ok();
    }
}

async fn poll_loop<F>(
    mut session: ImapSession,
    db: &SqlitePool,
    app_handle: &tauri::AppHandle,
    mailbox_ids: &[(String, String)],
    on_event: &F,
) -> Result<(), String>
where
    F: Fn(SyncEvent),
{
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;

        for (mb_id, mb_name) in mailbox_ids {
            if let Err(e) = imap::sync_mailbox(&mut session, db, mb_id, mb_name, on_event).await {
                log::error!("Poll sync error for {}: {}", mb_name, e);
            }
        }

        app_handle.emit("emails-updated", ()).ok();
    }
}
