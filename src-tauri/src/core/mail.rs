use crate::db::{queries, schema::*};
use sqlx::SqlitePool;

use super::imap;

pub async fn modify_flag(
    pool: &SqlitePool,
    uid: i64,
    mailbox_id: &str,
    add: Option<&str>,
    remove: Option<&str>,
) -> Result<(), String> {
    let detail = queries::get_email_detail(pool, uid, mailbox_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Email not found".to_string())?;

    let mut flags: Vec<String> =
        serde_json::from_str(detail.email.flags.as_deref().unwrap_or("[]")).unwrap_or_default();

    if let Some(flag) = add {
        if !flags.iter().any(|f| f == flag) {
            flags.push(flag.to_string());
        }
    }

    if let Some(flag) = remove {
        flags.retain(|f| f != flag);
    }

    let flags_json = serde_json::to_string(&flags).map_err(|e| e.to_string())?;
    queries::update_flags(pool, uid, mailbox_id, &flags_json)
        .await
        .map_err(|e| e.to_string())
}

pub async fn star(pool: &SqlitePool, uid: i64, mailbox_id: &str) -> Result<(), String> {
    modify_flag(pool, uid, mailbox_id, Some("\\Flagged"), None).await
}

pub async fn unstar(pool: &SqlitePool, uid: i64, mailbox_id: &str) -> Result<(), String> {
    modify_flag(pool, uid, mailbox_id, None, Some("\\Flagged")).await
}

pub async fn mark_read(pool: &SqlitePool, uid: i64, mailbox_id: &str) -> Result<(), String> {
    modify_flag(pool, uid, mailbox_id, Some("\\Seen"), None).await
}

pub async fn mark_unread(pool: &SqlitePool, uid: i64, mailbox_id: &str) -> Result<(), String> {
    modify_flag(pool, uid, mailbox_id, None, Some("\\Seen")).await
}

pub async fn archive(pool: &SqlitePool, uid: i64, mailbox_id: &str) -> Result<(), String> {
    let archive = queries::get_mailbox_by_name(pool, "Archive")
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Archive mailbox not found".to_string())?;

    queries::move_email(pool, uid, mailbox_id, &archive.id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_email_with_fetch(
    pool: &SqlitePool,
    uid: i64,
    mailbox_id: &str,
    imap_host: &str,
    imap_port: u16,
    email: &str,
    password: &str,
) -> Result<EmailDetail, String> {
    let detail = queries::get_email_detail(pool, uid, mailbox_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Email not found".to_string())?;

    if detail.html_body.is_some() || detail.text_body.is_some() {
        return Ok(detail);
    }

    let mb_name = mailbox_id
        .split(':')
        .nth(1)
        .unwrap_or(mailbox_id)
        .to_string();

    let mut session = imap::connect_imap(imap_host, imap_port, email, password).await?;
    let (html_body, text_body) = imap::fetch_email_body(&mut session, &mb_name, uid).await?;
    session.logout().await.ok();

    queries::upsert_email_body(
        pool,
        uid,
        mailbox_id,
        html_body.as_deref(),
        text_body.as_deref(),
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(EmailDetail {
        html_body,
        text_body,
        ..detail
    })
}
