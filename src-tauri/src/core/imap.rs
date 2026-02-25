use crate::db::{queries, schema::*};
use futures::TryStreamExt;
use mail_parser::MimeHeaders;
use sqlx::SqlitePool;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub type ImapSession = async_imap::Session<tokio_util::compat::Compat<TcpStream>>;

#[derive(Debug, Clone)]
pub enum SyncEvent {
    MailboxFound { name: String },
    SyncStart { mailbox: String, new_count: usize, last_uid: i64 },
    EmailSynced { uid: i64, sender: String, subject: String },
    SyncComplete { mailbox: String, count: usize },
}

pub async fn connect_imap(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
) -> Result<ImapSession, String> {
    let addr = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("TCP connect to {} failed: {}", addr, e))?;

    let tcp_compat = tcp.compat();
    let client = async_imap::Client::new(tcp_compat);

    let session = client
        .login(email, password)
        .await
        .map_err(|(e, _)| format!("IMAP login failed: {}", e))?;

    Ok(session)
}

pub async fn ensure_account(db: &SqlitePool, email: &str) -> Result<String, String> {
    let existing = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE email = ?")
        .bind(email)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(account) = existing {
        return Ok(account.id);
    }

    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO accounts (id, email, display_name) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(email)
        .bind(email)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id)
}

pub fn extract_header_fields(
    parsed: &Option<mail_parser::Message<'_>>,
) -> (
    Option<String>, // subject
    Option<String>, // sender_name
    Option<String>, // sender_email
    String,         // recipients JSON
    i64,            // date_utc
    i64,            // has_attachments
) {
    if let Some(parsed) = parsed {
        let subject = parsed.subject().map(|s| s.to_string());

        let (sname, semail) = parsed
            .from()
            .and_then(|from| from.first())
            .map(|addr| {
                (
                    addr.name().map(|n| n.to_string()),
                    addr.address().map(|a| a.to_string()),
                )
            })
            .unwrap_or((None, None));

        let recipients_str = parsed
            .to()
            .map(|list| {
                let addrs: Vec<String> = list
                    .iter()
                    .filter_map(|a| a.address().map(|s| s.to_string()))
                    .collect();
                serde_json::to_string(&addrs).unwrap_or_else(|_| "[]".to_string())
            })
            .unwrap_or_else(|| "[]".to_string());

        let date = parsed.date().map(|d| d.to_timestamp()).unwrap_or(0);

        let has_attachments = parsed
            .content_type()
            .map(|ct| ct.ctype() == "multipart" && ct.subtype().unwrap_or("") == "mixed")
            .unwrap_or(false);

        (
            subject,
            sname,
            semail,
            recipients_str,
            date,
            if has_attachments { 1 } else { 0 },
        )
    } else {
        (None, None, None, "[]".to_string(), 0, 0)
    }
}

pub async fn fetch_email_body(
    session: &mut ImapSession,
    mailbox_name: &str,
    uid: i64,
) -> Result<(Option<String>, Option<String>), String> {
    session
        .select(mailbox_name)
        .await
        .map_err(|e| format!("Failed to select {}: {}", mailbox_name, e))?;

    let messages: Vec<_> = session
        .uid_fetch(uid.to_string(), "(BODY.PEEK[])")
        .await
        .map_err(|e| format!("Fetch body failed: {}", e))?
        .try_collect()
        .await
        .map_err(|e| format!("Failed to collect body: {}", e))?;

    let msg = messages.first().ok_or("Message not found on server")?;
    let body_bytes = msg.body().unwrap_or(&[]);
    let parsed = mail_parser::MessageParser::default().parse(body_bytes);

    let html = parsed
        .as_ref()
        .and_then(|p| p.body_html(0))
        .map(|h| h.to_string());
    let text = parsed
        .as_ref()
        .and_then(|p| p.body_text(0))
        .map(|t| t.to_string());

    Ok((html, text))
}

pub async fn sync_all<F>(
    session: &mut ImapSession,
    pool: &SqlitePool,
    email: &str,
    on_event: &F,
) -> Result<Vec<(String, String)>, String>
where
    F: Fn(SyncEvent),
{
    let mailboxes: Vec<_> = session
        .list(Some(""), Some("*"))
        .await
        .map_err(|e| format!("IMAP list failed: {}", e))?
        .try_collect()
        .await
        .map_err(|e| format!("Failed to collect mailboxes: {}", e))?;

    let account_id = ensure_account(pool, email).await?;

    let mut mailbox_ids: Vec<(String, String)> = Vec::new();

    for mailbox in &mailboxes {
        let name = mailbox.name().to_string();
        let mb_id = format!("{}:{}", account_id, name);

        on_event(SyncEvent::MailboxFound { name: name.clone() });

        let mb = Mailbox {
            id: mb_id.clone(),
            account_id: account_id.clone(),
            name: name.clone(),
            path: name.clone(),
            uid_validity: None,
            last_synced_uid: Some(0),
            unread_count: Some(0),
        };

        queries::upsert_mailbox(pool, &mb)
            .await
            .map_err(|e| e.to_string())?;
        mailbox_ids.push((mb_id, name));
    }

    for (mb_id, mb_name) in &mailbox_ids {
        if let Err(e) = sync_mailbox(session, pool, mb_id, mb_name, on_event).await {
            log::error!("Error syncing mailbox {}: {}", mb_name, e);
        }
    }

    Ok(mailbox_ids)
}

pub async fn sync_mailbox<F>(
    session: &mut ImapSession,
    db: &SqlitePool,
    mailbox_id: &str,
    mailbox_name: &str,
    on_event: &F,
) -> Result<(), String>
where
    F: Fn(SyncEvent),
{
    let mailbox_status = session
        .select(mailbox_name)
        .await
        .map_err(|e| format!("Failed to select {}: {}", mailbox_name, e))?;

    let existing_mb = queries::get_mailbox_by_name(db, mailbox_name)
        .await
        .map_err(|e| e.to_string())?;

    let last_uid = existing_mb
        .as_ref()
        .and_then(|m| m.last_synced_uid)
        .unwrap_or(0);

    let fetch_range = if last_uid > 0 {
        format!("{}:*", last_uid + 1)
    } else {
        "1:*".to_string()
    };

    let uid_messages: Vec<_> = session
        .uid_fetch(&fetch_range, "(UID FLAGS ENVELOPE)")
        .await
        .map_err(|e| format!("UID fetch failed for {}: {}", mailbox_name, e))?
        .try_collect()
        .await
        .map_err(|e| format!("Failed to collect UIDs for {}: {}", mailbox_name, e))?;

    let new_uids: Vec<u32> = uid_messages
        .iter()
        .filter_map(|m| m.uid)
        .filter(|&uid| (uid as i64) > last_uid)
        .collect();

    on_event(SyncEvent::SyncStart {
        mailbox: mailbox_name.to_string(),
        new_count: new_uids.len(),
        last_uid,
    });

    if new_uids.is_empty() {
        let unread = mailbox_status.unseen.unwrap_or(0) as i64;
        sqlx::query("UPDATE mailboxes SET unread_count = ? WHERE id = ?")
            .bind(unread)
            .bind(mailbox_id)
            .execute(db)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    let mut max_uid: i64 = last_uid;
    let mut new_count = 0;

    for chunk in new_uids.chunks(20) {
        let uid_set = chunk
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let messages: Vec<_> = session
            .uid_fetch(&uid_set, "(UID FLAGS BODY.PEEK[])")
            .await
            .map_err(|e| format!("Fetch failed for {}: {}", mailbox_name, e))?
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect messages for {}: {}", mailbox_name, e))?;

        for msg in &messages {
            let uid = msg.uid.unwrap_or(0) as i64;
            if uid <= last_uid {
                continue;
            }
            if uid > max_uid {
                max_uid = uid;
            }

            let flags: Vec<String> = msg.flags().map(|f| format!("{:?}", f)).collect();
            let flags_json =
                serde_json::to_string(&flags).unwrap_or_else(|_| "[]".to_string());

            let body_bytes = msg.body().unwrap_or(&[]);
            let parsed = mail_parser::MessageParser::default().parse(body_bytes);

            let (subject, sender_name, sender_email_addr, recipients, date_utc, has_attachments) =
                extract_header_fields(&parsed);

            let preview = parsed.as_ref().and_then(|p| {
                p.body_text(0)
                    .map(|t| t.chars().take(200).collect::<String>())
            });

            on_event(SyncEvent::EmailSynced {
                uid,
                sender: sender_email_addr.clone().unwrap_or_default(),
                subject: subject.clone().unwrap_or_default(),
            });

            let email = Email {
                uid,
                mailbox_id: mailbox_id.to_string(),
                message_id: parsed
                    .as_ref()
                    .and_then(|p| p.message_id().map(|s| s.to_string())),
                subject,
                sender_name,
                sender_email: sender_email_addr,
                recipients: Some(recipients),
                date_utc,
                flags: Some(flags_json),
                has_attachments,
                preview,
                classification: None,
                classification_confidence: None,
            };

            if let Err(e) = queries::upsert_email(db, &email).await {
                log::error!("Failed to store email uid={}: {}", uid, e);
                continue;
            }

            let html = parsed
                .as_ref()
                .and_then(|p| p.body_html(0).map(|h| h.to_string()));
            let text = parsed
                .as_ref()
                .and_then(|p| p.body_text(0).map(|t| t.to_string()));

            if let Err(e) = queries::upsert_email_body(
                db,
                uid,
                mailbox_id,
                html.as_deref(),
                text.as_deref(),
                Some(body_bytes),
            )
            .await
            {
                log::error!("Failed to store email body uid={}: {}", uid, e);
            }

            new_count += 1;
        }
    }

    let unread = mailbox_status.unseen.unwrap_or(0) as i64;
    sqlx::query("UPDATE mailboxes SET unread_count = ? WHERE id = ?")
        .bind(unread)
        .bind(mailbox_id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    if max_uid > last_uid {
        sqlx::query("UPDATE mailboxes SET last_synced_uid = ? WHERE id = ?")
            .bind(max_uid)
            .bind(mailbox_id)
            .execute(db)
            .await
            .map_err(|e| e.to_string())?;
    }

    on_event(SyncEvent::SyncComplete {
        mailbox: mailbox_name.to_string(),
        count: new_count,
    });

    Ok(())
}
