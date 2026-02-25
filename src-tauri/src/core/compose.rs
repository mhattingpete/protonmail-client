use crate::db::queries;
use lettre::message::header::ContentType;
use lettre::transport::smtp::client::Tls;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::SqlitePool;

pub fn build_smtp_transport(
    host: &str,
    port: u16,
) -> AsyncSmtpTransport<Tokio1Executor> {
    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
        .port(port)
        .tls(Tls::None)
        .build()
}

pub async fn send_email(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    to: &str,
    subject: String,
    body: String,
    is_html: bool,
) -> Result<(), String> {
    let content_type = if is_html {
        ContentType::TEXT_HTML
    } else {
        ContentType::TEXT_PLAIN
    };

    let message = Message::builder()
        .from(
            from.parse()
                .map_err(|e: lettre::address::AddressError| e.to_string())?,
        )
        .to(to
            .parse()
            .map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject(subject)
        .header(content_type)
        .body(body)
        .map_err(|e| e.to_string())?;

    transport
        .send(message)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn reply_email(
    pool: &SqlitePool,
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    uid: i64,
    mailbox_id: &str,
    body: String,
) -> Result<(), String> {
    let detail = queries::get_email_detail(pool, uid, mailbox_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Email not found".to_string())?;

    let to = detail
        .email
        .sender_email
        .as_deref()
        .ok_or_else(|| "No sender email".to_string())?;

    let subject = format!(
        "Re: {}",
        detail.email.subject.as_deref().unwrap_or("(no subject)")
    );

    let mut builder = Message::builder()
        .from(
            from.parse()
                .map_err(|e: lettre::address::AddressError| e.to_string())?,
        )
        .to(to
            .parse()
            .map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject(subject);

    if let Some(ref msg_id) = detail.email.message_id {
        builder = builder.in_reply_to(msg_id.clone());
    }

    let message = builder
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| e.to_string())?;

    transport
        .send(message)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn forward_email(
    pool: &SqlitePool,
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    uid: i64,
    mailbox_id: &str,
    to: &str,
) -> Result<(), String> {
    let detail = queries::get_email_detail(pool, uid, mailbox_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Email not found".to_string())?;

    let subject = format!(
        "Fwd: {}",
        detail.email.subject.as_deref().unwrap_or("(no subject)")
    );

    let original_body = detail
        .text_body
        .or(detail.html_body)
        .unwrap_or_default();

    let forward_body = format!(
        "\n\n---------- Forwarded message ----------\nFrom: {} <{}>\nSubject: {}\n\n{}",
        detail.email.sender_name.as_deref().unwrap_or(""),
        detail.email.sender_email.as_deref().unwrap_or(""),
        detail.email.subject.as_deref().unwrap_or("(no subject)"),
        original_body,
    );

    let message = Message::builder()
        .from(
            from.parse()
                .map_err(|e: lettre::address::AddressError| e.to_string())?,
        )
        .to(to
            .parse()
            .map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(forward_body)
        .map_err(|e| e.to_string())?;

    transport
        .send(message)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
