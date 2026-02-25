use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub bridge_host: Option<String>,
    pub bridge_imap_port: Option<i64>,
    pub bridge_smtp_port: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mailbox {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub path: String,
    pub uid_validity: Option<i64>,
    pub last_synced_uid: Option<i64>,
    pub unread_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Email {
    pub uid: i64,
    pub mailbox_id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub sender_name: Option<String>,
    pub sender_email: Option<String>,
    pub recipients: Option<String>,
    pub date_utc: i64,
    pub flags: Option<String>,
    pub has_attachments: i64,
    pub preview: Option<String>,
    pub classification: Option<String>,
    pub classification_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailBody {
    pub uid: i64,
    pub mailbox_id: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDetail {
    #[serde(flatten)]
    pub email: Email,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attachment {
    pub id: String,
    pub email_uid: i64,
    pub mailbox_id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub stored_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationRule {
    pub id: String,
    pub name: String,
    pub enabled: i64,
    pub priority: i64,
    pub conditions: String,
    pub actions: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String,
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub label: String,
    pub confidence: f64,
    pub tier: String,
}
