CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    bridge_host TEXT DEFAULT '127.0.0.1',
    bridge_imap_port INTEGER DEFAULT 1143,
    bridge_smtp_port INTEGER DEFAULT 1025
);

CREATE TABLE IF NOT EXISTS mailboxes (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    uid_validity INTEGER,
    last_synced_uid INTEGER DEFAULT 0,
    unread_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS emails (
    uid INTEGER NOT NULL,
    mailbox_id TEXT NOT NULL REFERENCES mailboxes(id),
    message_id TEXT,
    subject TEXT,
    sender_name TEXT,
    sender_email TEXT,
    recipients TEXT,
    date_utc INTEGER NOT NULL,
    flags TEXT DEFAULT '[]',
    has_attachments INTEGER DEFAULT 0,
    preview TEXT,
    classification TEXT,
    classification_confidence REAL,
    PRIMARY KEY (uid, mailbox_id)
);

CREATE TABLE IF NOT EXISTS email_bodies (
    uid INTEGER NOT NULL,
    mailbox_id TEXT NOT NULL,
    html_body TEXT,
    text_body TEXT,
    raw_rfc822 BLOB,
    PRIMARY KEY (uid, mailbox_id)
);

CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    email_uid INTEGER NOT NULL,
    mailbox_id TEXT NOT NULL,
    filename TEXT,
    content_type TEXT,
    size_bytes INTEGER,
    stored_path TEXT
);

CREATE TABLE IF NOT EXISTS automation_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    priority INTEGER DEFAULT 0,
    conditions TEXT NOT NULL,
    actions TEXT NOT NULL,
    created_at INTEGER,
    updated_at INTEGER
);

CREATE VIRTUAL TABLE IF NOT EXISTS email_fts USING fts5(
    subject, sender_name, sender_email, text_body,
    content='emails',
    tokenize='porter unicode61'
);
