use sqlx::SqlitePool;

use super::schema::*;

pub async fn get_emails(
    pool: &SqlitePool,
    mailbox_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Email>, sqlx::Error> {
    sqlx::query_as::<_, Email>(
        "SELECT * FROM emails WHERE mailbox_id = ? ORDER BY date_utc DESC LIMIT ? OFFSET ?",
    )
    .bind(mailbox_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_email_detail(
    pool: &SqlitePool,
    uid: i64,
    mailbox_id: &str,
) -> Result<Option<EmailDetail>, sqlx::Error> {
    let email = sqlx::query_as::<_, Email>(
        "SELECT * FROM emails WHERE uid = ? AND mailbox_id = ?",
    )
    .bind(uid)
    .bind(mailbox_id)
    .fetch_optional(pool)
    .await?;

    match email {
        Some(email) => {
            let body = sqlx::query_as::<_, EmailBody>(
                "SELECT uid, mailbox_id, html_body, text_body FROM email_bodies WHERE uid = ? AND mailbox_id = ?",
            )
            .bind(uid)
            .bind(mailbox_id)
            .fetch_optional(pool)
            .await?;

            Ok(Some(EmailDetail {
                email,
                html_body: body.as_ref().and_then(|b| b.html_body.clone()),
                text_body: body.as_ref().and_then(|b| b.text_body.clone()),
            }))
        }
        None => Ok(None),
    }
}

pub async fn update_flags(
    pool: &SqlitePool,
    uid: i64,
    mailbox_id: &str,
    flags: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE emails SET flags = ? WHERE uid = ? AND mailbox_id = ?")
        .bind(flags)
        .bind(uid)
        .bind(mailbox_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_email(
    pool: &SqlitePool,
    uid: i64,
    mailbox_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM email_bodies WHERE uid = ? AND mailbox_id = ?")
        .bind(uid)
        .bind(mailbox_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM attachments WHERE email_uid = ? AND mailbox_id = ?")
        .bind(uid)
        .bind(mailbox_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM emails WHERE uid = ? AND mailbox_id = ?")
        .bind(uid)
        .bind(mailbox_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn move_email(
    pool: &SqlitePool,
    uid: i64,
    from_mailbox: &str,
    to_mailbox: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE emails SET mailbox_id = ? WHERE uid = ? AND mailbox_id = ?")
        .bind(to_mailbox)
        .bind(uid)
        .bind(from_mailbox)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE email_bodies SET mailbox_id = ? WHERE uid = ? AND mailbox_id = ?")
        .bind(to_mailbox)
        .bind(uid)
        .bind(from_mailbox)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_mailboxes(pool: &SqlitePool) -> Result<Vec<Mailbox>, sqlx::Error> {
    sqlx::query_as::<_, Mailbox>("SELECT * FROM mailboxes ORDER BY name")
        .fetch_all(pool)
        .await
}

pub async fn get_mailbox_by_name(
    pool: &SqlitePool,
    name: &str,
) -> Result<Option<Mailbox>, sqlx::Error> {
    sqlx::query_as::<_, Mailbox>("SELECT * FROM mailboxes WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await
}

pub async fn upsert_mailbox(
    pool: &SqlitePool,
    mailbox: &Mailbox,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO mailboxes (id, account_id, name, path, uid_validity, last_synced_uid, unread_count)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           path = excluded.path,
           uid_validity = COALESCE(excluded.uid_validity, mailboxes.uid_validity),
           unread_count = COALESCE(excluded.unread_count, mailboxes.unread_count)",
    )
    .bind(&mailbox.id)
    .bind(&mailbox.account_id)
    .bind(&mailbox.name)
    .bind(&mailbox.path)
    .bind(mailbox.uid_validity)
    .bind(mailbox.last_synced_uid)
    .bind(mailbox.unread_count)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_email(pool: &SqlitePool, email: &Email) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO emails (uid, mailbox_id, message_id, subject, sender_name, sender_email, recipients, date_utc, flags, has_attachments, preview, classification, classification_confidence)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(uid, mailbox_id) DO UPDATE SET
           flags = excluded.flags,
           classification = excluded.classification,
           classification_confidence = excluded.classification_confidence",
    )
    .bind(email.uid)
    .bind(&email.mailbox_id)
    .bind(&email.message_id)
    .bind(&email.subject)
    .bind(&email.sender_name)
    .bind(&email.sender_email)
    .bind(&email.recipients)
    .bind(email.date_utc)
    .bind(&email.flags)
    .bind(email.has_attachments)
    .bind(&email.preview)
    .bind(&email.classification)
    .bind(email.classification_confidence)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_email_body(
    pool: &SqlitePool,
    uid: i64,
    mailbox_id: &str,
    html_body: Option<&str>,
    text_body: Option<&str>,
    raw_rfc822: Option<&[u8]>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO email_bodies (uid, mailbox_id, html_body, text_body, raw_rfc822)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(uid, mailbox_id) DO UPDATE SET
           html_body = excluded.html_body,
           text_body = excluded.text_body,
           raw_rfc822 = excluded.raw_rfc822",
    )
    .bind(uid)
    .bind(mailbox_id)
    .bind(html_body)
    .bind(text_body)
    .bind(raw_rfc822)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn search_emails_fts(
    pool: &SqlitePool,
    query: &str,
    limit: i64,
) -> Result<Vec<Email>, sqlx::Error> {
    sqlx::query_as::<_, Email>(
        "SELECT e.* FROM emails e
         WHERE e.subject LIKE '%' || ? || '%'
            OR e.sender_name LIKE '%' || ? || '%'
            OR e.sender_email LIKE '%' || ? || '%'
         ORDER BY e.date_utc DESC
         LIMIT ?",
    )
    .bind(query)
    .bind(query)
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_rules(pool: &SqlitePool) -> Result<Vec<AutomationRule>, sqlx::Error> {
    sqlx::query_as::<_, AutomationRule>(
        "SELECT * FROM automation_rules ORDER BY priority DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_rule(pool: &SqlitePool, id: &str) -> Result<Option<AutomationRule>, sqlx::Error> {
    sqlx::query_as::<_, AutomationRule>("SELECT * FROM automation_rules WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_rule(pool: &SqlitePool, rule: &AutomationRule) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO automation_rules (id, name, enabled, priority, conditions, actions, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&rule.id)
    .bind(&rule.name)
    .bind(rule.enabled)
    .bind(rule.priority)
    .bind(&rule.conditions)
    .bind(&rule.actions)
    .bind(rule.created_at)
    .bind(rule.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_rule(pool: &SqlitePool, rule: &AutomationRule) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE automation_rules SET name = ?, enabled = ?, priority = ?, conditions = ?, actions = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&rule.name)
    .bind(rule.enabled)
    .bind(rule.priority)
    .bind(&rule.conditions)
    .bind(&rule.actions)
    .bind(rule.updated_at)
    .bind(&rule.id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_rule(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM automation_rules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_classification(
    pool: &SqlitePool,
    uid: i64,
    mailbox_id: &str,
    classification: &str,
    confidence: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE emails SET classification = ?, classification_confidence = ? WHERE uid = ? AND mailbox_id = ?",
    )
    .bind(classification)
    .bind(confidence)
    .bind(uid)
    .bind(mailbox_id)
    .execute(pool)
    .await?;
    Ok(())
}
