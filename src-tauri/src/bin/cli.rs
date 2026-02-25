use clap::{Parser, Subcommand};
use protonmail_client_lib::classify;
use protonmail_client_lib::core;
use protonmail_client_lib::db::{self, queries};
use sqlx::SqlitePool;

const IMAP_HOST: &str = "127.0.0.1";
const IMAP_PORT: u16 = 1143;
const SMTP_HOST: &str = "127.0.0.1";
const SMTP_PORT: u16 = 1025;
const KEYRING_SERVICE: &str = "protonmail-client";
const KEYRING_USER: &str = "bridge-password";
const KEYRING_EMAIL: &str = "account-email";

#[derive(Parser)]
#[command(name = "pm-cli", about = "ProtonMail Client CLI diagnostic tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test IMAP login using stored credentials
    Connect,
    /// Store credentials to macOS Keychain
    Setup {
        email: String,
        password: String,
    },
    /// Run full IMAP sync to SQLite
    Sync,
    /// Show connection status and DB stats
    Status,
    /// List all mailboxes with unread counts
    Mailboxes,
    /// List emails in a mailbox
    List {
        #[arg(long, default_value = "INBOX")]
        mailbox: String,
        #[arg(long, default_value_t = 50)]
        limit: i64,
    },
    /// Read full email body
    Read {
        uid: i64,
        mailbox_id: String,
    },
    /// Star an email
    Star {
        uid: i64,
        mailbox_id: String,
    },
    /// Unstar an email
    Unstar {
        uid: i64,
        mailbox_id: String,
    },
    /// Mark email as read
    MarkRead {
        uid: i64,
        mailbox_id: String,
    },
    /// Mark email as unread
    MarkUnread {
        uid: i64,
        mailbox_id: String,
    },
    /// Delete an email
    Delete {
        uid: i64,
        mailbox_id: String,
    },
    /// Move an email between mailboxes
    Move {
        uid: i64,
        from_mailbox: String,
        to_mailbox: String,
    },
    /// Archive an email
    Archive {
        uid: i64,
        mailbox_id: String,
    },
    /// Send an email
    Send {
        #[arg(long)]
        to: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        body: String,
    },
    /// Reply to an email
    Reply {
        uid: i64,
        mailbox_id: String,
        #[arg(long)]
        body: String,
    },
    /// Forward an email
    Forward {
        uid: i64,
        mailbox_id: String,
        #[arg(long)]
        to: String,
    },
    /// Search emails by subject, sender, or body
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: i64,
    },
    /// Create a new mailbox/folder
    CreateFolder {
        name: String,
    },
    /// Manage automation rules
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },
    /// Run classification on an email
    Classify {
        uid: i64,
        mailbox_id: String,
    },
    /// Run automation rules against emails in a mailbox
    RunRules {
        #[arg(long, default_value = "INBOX")]
        mailbox: String,
        #[arg(long, default_value_t = 50)]
        limit: i64,
        /// Preview matched rules without executing actions
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum RulesAction {
    /// List all automation rules
    List,
    /// Create a new rule
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        conditions: String,
        #[arg(long)]
        actions: String,
        #[arg(long)]
        priority: Option<i64>,
    },
    /// Update an existing rule
    Update {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        enabled: Option<bool>,
        #[arg(long)]
        conditions: Option<String>,
        #[arg(long)]
        actions: Option<String>,
        #[arg(long)]
        priority: Option<i64>,
    },
    /// Delete a rule
    Delete {
        id: String,
    },
}

// --- Keychain helpers (macOS security CLI) ---

fn keychain_get(service: &str, account: &str) -> Option<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", service, "-a", account, "-w"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn keychain_set(service: &str, account: &str, password: &str) -> Result<(), String> {
    std::process::Command::new("security")
        .args(["delete-generic-password", "-s", service, "-a", account])
        .output()
        .ok();

    let output = std::process::Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            service,
            "-a",
            account,
            "-w",
            password,
        ])
        .output()
        .map_err(|e| format!("Failed to run security command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Keychain write failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn load_credentials() -> Result<(String, String), String> {
    let email = keychain_get(KEYRING_SERVICE, KEYRING_EMAIL)
        .ok_or("No email in Keychain. Run: pm-cli setup <email> <password>")?;
    let password = keychain_get(KEYRING_SERVICE, KEYRING_USER)
        .ok_or("No password in Keychain. Run: pm-cli setup <email> <password>")?;
    Ok((email, password))
}

// --- DB helpers ---

fn db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!(
        "{}/Library/Application Support/com.protonmail.client/protonmail.db",
        home
    )
}

async fn open_db() -> Result<SqlitePool, String> {
    let path = db_path();
    let dir = std::path::Path::new(&path).parent().unwrap();
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    let url = format!("sqlite:{}?mode=rwc", path);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(|e| format!("Failed to open DB: {}", e))?;

    db::init_db(&pool)
        .await
        .map_err(|e| format!("Failed to init DB: {}", e))?;

    Ok(pool)
}

// --- Main ---

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Setup { email, password } => {
            keychain_set(KEYRING_SERVICE, KEYRING_EMAIL, &email)?;
            keychain_set(KEYRING_SERVICE, KEYRING_USER, &password)?;
            println!("Credentials saved to Keychain for {}", email);
            Ok(())
        }

        Commands::Connect => {
            let (email, password) = load_credentials()?;
            println!("Connecting to {}:{} as {}...", IMAP_HOST, IMAP_PORT, email);
            let mut session =
                core::imap::connect_imap(IMAP_HOST, IMAP_PORT, &email, &password).await?;
            println!("Login successful!");

            let caps = session
                .capabilities()
                .await
                .map_err(|e| format!("Capabilities failed: {}", e))?;
            println!(
                "Server capabilities: {}",
                caps.iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            session.logout().await.ok();
            Ok(())
        }

        Commands::Sync => {
            let (email, password) = load_credentials()?;
            let pool = open_db().await?;

            println!("Connecting to IMAP...");
            let mut session =
                core::imap::connect_imap(IMAP_HOST, IMAP_PORT, &email, &password).await?;

            let on_event = |event: core::imap::SyncEvent| match &event {
                core::imap::SyncEvent::MailboxFound { name } => {
                    println!("  - {}", name);
                }
                core::imap::SyncEvent::SyncStart {
                    mailbox,
                    new_count,
                    last_uid,
                } => {
                    println!(
                        "  {} — {} new messages (last_uid={})",
                        mailbox, new_count, last_uid
                    );
                }
                core::imap::SyncEvent::EmailSynced {
                    uid,
                    sender,
                    subject,
                } => {
                    println!("    [{}] {} — {}", uid, sender, subject);
                }
                core::imap::SyncEvent::SyncComplete { mailbox, count } => {
                    if *count > 0 {
                        println!("  Synced {} new emails in {}", count, mailbox);
                    }
                }
            };

            println!("Found mailboxes:");
            let _mailbox_ids =
                core::imap::sync_all(&mut session, &pool, &email, &on_event).await?;

            session.logout().await.ok();

            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails")
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            println!("\nSync complete. Total emails in DB: {}", total.0);
            Ok(())
        }

        Commands::Status => {
            let pool = open_db().await?;
            let (email, _) = load_credentials().unwrap_or(("(not set)".into(), String::new()));

            println!("Account: {}", email);
            println!("DB path: {}", db_path());

            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails")
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            println!("Total emails: {}", total.0);

            let bodies: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM email_bodies")
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            println!("Cached bodies: {}", bodies.0);

            let mailboxes = queries::get_mailboxes(&pool)
                .await
                .map_err(|e| e.to_string())?;
            println!("Mailboxes: {}", mailboxes.len());
            for mb in &mailboxes {
                let count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM emails WHERE mailbox_id = ?")
                        .bind(&mb.id)
                        .fetch_one(&pool)
                        .await
                        .map_err(|e| e.to_string())?;
                println!(
                    "  {} — {} emails, {} unread (last_uid={})",
                    mb.name,
                    count.0,
                    mb.unread_count.unwrap_or(0),
                    mb.last_synced_uid.unwrap_or(0),
                );
            }

            let rules_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM automation_rules")
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            println!("Automation rules: {}", rules_count.0);

            Ok(())
        }

        Commands::Mailboxes => {
            let pool = open_db().await?;
            let mailboxes = queries::get_mailboxes(&pool)
                .await
                .map_err(|e| e.to_string())?;

            if mailboxes.is_empty() {
                println!("No mailboxes found. Run 'pm-cli sync' first.");
                return Ok(());
            }

            println!("{:<30} {:>8} {:>10}", "MAILBOX", "UNREAD", "LAST_UID");
            println!("{}", "-".repeat(50));
            for mb in &mailboxes {
                println!(
                    "{:<30} {:>8} {:>10}",
                    mb.name,
                    mb.unread_count.unwrap_or(0),
                    mb.last_synced_uid.unwrap_or(0),
                );
            }
            Ok(())
        }

        Commands::List { mailbox, limit } => {
            let pool = open_db().await?;

            let mailboxes = queries::get_mailboxes(&pool)
                .await
                .map_err(|e| e.to_string())?;

            let mb = mailboxes
                .iter()
                .find(|m| m.name.eq_ignore_ascii_case(&mailbox))
                .ok_or_else(|| {
                    let names: Vec<_> = mailboxes.iter().map(|m| m.name.as_str()).collect();
                    format!(
                        "Mailbox '{}' not found. Available: {}",
                        mailbox,
                        names.join(", ")
                    )
                })?;

            let emails = queries::get_emails(&pool, &mb.id, limit, 0)
                .await
                .map_err(|e| e.to_string())?;

            if emails.is_empty() {
                println!("No emails in {}.", mailbox);
                return Ok(());
            }

            println!(
                "{:>6}  {:<20}  {:<40}  {}",
                "UID", "FROM", "SUBJECT", "DATE"
            );
            println!("{}", "-".repeat(90));

            for email in &emails {
                let date = chrono::DateTime::from_timestamp(email.date_utc, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "?".to_string());

                let from = email
                    .sender_email
                    .as_deref()
                    .unwrap_or("?")
                    .chars()
                    .take(20)
                    .collect::<String>();

                let subject = email
                    .subject
                    .as_deref()
                    .unwrap_or("(no subject)")
                    .chars()
                    .take(40)
                    .collect::<String>();

                let flags = email.flags.as_deref().unwrap_or("[]");
                let seen = if flags.contains("Seen") { " " } else { "*" };

                println!(
                    "{:>6}{} {:<20}  {:<40}  {}",
                    email.uid, seen, from, subject, date
                );
            }

            println!("\n{} emails shown (mailbox_id: {})", emails.len(), mb.id);
            Ok(())
        }

        Commands::Read { uid, mailbox_id } => {
            let pool = open_db().await?;

            // Try fetching with IMAP fallback if body not cached
            let creds = load_credentials();
            let detail = if let Ok((email, password)) = &creds {
                core::mail::get_email_with_fetch(
                    &pool,
                    uid,
                    &mailbox_id,
                    IMAP_HOST,
                    IMAP_PORT,
                    email,
                    password,
                )
                .await?
            } else {
                queries::get_email_detail(&pool, uid, &mailbox_id)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Email uid={} not found in {}", uid, mailbox_id))?
            };

            println!("UID:     {}", detail.email.uid);
            println!(
                "From:    {} <{}>",
                detail.email.sender_name.as_deref().unwrap_or(""),
                detail.email.sender_email.as_deref().unwrap_or("?")
            );
            println!(
                "To:      {}",
                detail.email.recipients.as_deref().unwrap_or("?")
            );
            println!(
                "Subject: {}",
                detail.email.subject.as_deref().unwrap_or("(no subject)")
            );
            let date = chrono::DateTime::from_timestamp(detail.email.date_utc, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "?".to_string());
            println!("Date:    {}", date);
            println!(
                "Flags:   {}",
                detail.email.flags.as_deref().unwrap_or("[]")
            );
            println!("{}", "-".repeat(60));

            if let Some(text) = &detail.text_body {
                println!("{}", text);
            } else if let Some(html) = &detail.html_body {
                println!("[HTML body — {} chars]", html.len());
                println!("{}", html);
            } else {
                println!("[No body cached. Run 'pm-cli sync' or configure credentials.]");
            }
            Ok(())
        }

        Commands::Star { uid, mailbox_id } => {
            let pool = open_db().await?;
            core::mail::star(&pool, uid, &mailbox_id).await?;
            println!("Starred email uid={}", uid);
            Ok(())
        }

        Commands::Unstar { uid, mailbox_id } => {
            let pool = open_db().await?;
            core::mail::unstar(&pool, uid, &mailbox_id).await?;
            println!("Unstarred email uid={}", uid);
            Ok(())
        }

        Commands::MarkRead { uid, mailbox_id } => {
            let pool = open_db().await?;
            core::mail::mark_read(&pool, uid, &mailbox_id).await?;
            println!("Marked email uid={} as read", uid);
            Ok(())
        }

        Commands::MarkUnread { uid, mailbox_id } => {
            let pool = open_db().await?;
            core::mail::mark_unread(&pool, uid, &mailbox_id).await?;
            println!("Marked email uid={} as unread", uid);
            Ok(())
        }

        Commands::Delete { uid, mailbox_id } => {
            let pool = open_db().await?;
            queries::delete_email(&pool, uid, &mailbox_id)
                .await
                .map_err(|e| e.to_string())?;
            println!("Deleted email uid={} from {}", uid, mailbox_id);
            Ok(())
        }

        Commands::Move {
            uid,
            from_mailbox,
            to_mailbox,
        } => {
            let pool = open_db().await?;
            queries::move_email(&pool, uid, &from_mailbox, &to_mailbox)
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "Moved email uid={} from {} to {}",
                uid, from_mailbox, to_mailbox
            );
            Ok(())
        }

        Commands::Archive { uid, mailbox_id } => {
            let pool = open_db().await?;
            core::mail::archive(&pool, uid, &mailbox_id).await?;
            println!("Archived email uid={}", uid);
            Ok(())
        }

        Commands::Send { to, subject, body } => {
            let (email, _) = load_credentials()?;
            let transport = core::compose::build_smtp_transport(SMTP_HOST, SMTP_PORT);
            core::compose::send_email(&transport, &email, &to, subject, body, false).await?;
            println!("Email sent from {}", email);
            Ok(())
        }

        Commands::Reply {
            uid,
            mailbox_id,
            body,
        } => {
            let (email, _) = load_credentials()?;
            let pool = open_db().await?;
            let transport = core::compose::build_smtp_transport(SMTP_HOST, SMTP_PORT);
            core::compose::reply_email(&pool, &transport, &email, uid, &mailbox_id, body).await?;
            println!("Reply sent from {}", email);
            Ok(())
        }

        Commands::Forward {
            uid,
            mailbox_id,
            to,
        } => {
            let (email, _) = load_credentials()?;
            let pool = open_db().await?;
            let transport = core::compose::build_smtp_transport(SMTP_HOST, SMTP_PORT);
            core::compose::forward_email(&pool, &transport, &email, uid, &mailbox_id, &to).await?;
            println!("Forwarded from {}", email);
            Ok(())
        }

        Commands::Search { query, limit } => {
            let pool = open_db().await?;
            let emails = queries::search_emails_fts(&pool, &query, limit)
                .await
                .map_err(|e| e.to_string())?;

            if emails.is_empty() {
                println!("No results for '{}'", query);
                return Ok(());
            }

            println!("Found {} results for '{}':", emails.len(), query);
            println!(
                "{:>6}  {:<20}  {:<40}  {}",
                "UID", "FROM", "SUBJECT", "DATE"
            );
            println!("{}", "-".repeat(90));

            for email in &emails {
                let date = chrono::DateTime::from_timestamp(email.date_utc, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "?".to_string());

                println!(
                    "{:>6}  {:<20}  {:<40}  {}",
                    email.uid,
                    email
                        .sender_email
                        .as_deref()
                        .unwrap_or("?")
                        .chars()
                        .take(20)
                        .collect::<String>(),
                    email
                        .subject
                        .as_deref()
                        .unwrap_or("(no subject)")
                        .chars()
                        .take(40)
                        .collect::<String>(),
                    date,
                );
            }
            Ok(())
        }

        Commands::CreateFolder { name } => {
            let pool = open_db().await?;
            let mailbox = core::folders::create_folder(&pool, &name).await?;
            println!("Created folder '{}' (id: {})", name, mailbox.id);
            Ok(())
        }

        Commands::Rules { action } => {
            let pool = open_db().await?;
            match action {
                RulesAction::List => {
                    let all_rules = queries::get_rules(&pool)
                        .await
                        .map_err(|e| e.to_string())?;

                    if all_rules.is_empty() {
                        println!("No automation rules configured.");
                        return Ok(());
                    }

                    for rule in &all_rules {
                        println!(
                            "[{}] {} (priority={}, enabled={})",
                            rule.id,
                            rule.name,
                            rule.priority,
                            rule.enabled == 1,
                        );
                        println!("  Conditions: {}", rule.conditions);
                        println!("  Actions:    {}", rule.actions);
                        println!();
                    }
                    Ok(())
                }

                RulesAction::Create {
                    name,
                    conditions,
                    actions,
                    priority,
                } => {
                    let rule = core::rules::build_rule(
                        name.clone(),
                        conditions,
                        actions,
                        priority.unwrap_or(0),
                    );

                    queries::insert_rule(&pool, &rule)
                        .await
                        .map_err(|e| e.to_string())?;
                    println!("Created rule '{}' (id: {})", name, rule.id);
                    Ok(())
                }

                RulesAction::Update {
                    id,
                    name,
                    enabled,
                    conditions,
                    actions,
                    priority,
                } => {
                    let existing = queries::get_rule(&pool, &id)
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or_else(|| format!("Rule '{}' not found", id))?;

                    let updated = core::rules::apply_updates(
                        existing, name, enabled, conditions, actions, priority,
                    );

                    queries::update_rule(&pool, &updated)
                        .await
                        .map_err(|e| e.to_string())?;
                    println!("Updated rule '{}'", updated.name);
                    Ok(())
                }

                RulesAction::Delete { id } => {
                    let deleted = queries::delete_rule(&pool, &id)
                        .await
                        .map_err(|e| e.to_string())?;

                    if deleted {
                        println!("Deleted rule '{}'", id);
                    } else {
                        println!("Rule '{}' not found", id);
                    }
                    Ok(())
                }
            }
        }

        Commands::Classify { uid, mailbox_id } => {
            let pool = open_db().await?;
            let (email, password) = load_credentials()?;

            let detail = core::mail::get_email_with_fetch(
                &pool,
                uid,
                &mailbox_id,
                IMAP_HOST,
                IMAP_PORT,
                &email,
                &password,
            )
            .await?;

            println!(
                "Classifying: {}",
                detail.email.subject.as_deref().unwrap_or("(no subject)")
            );

            let result = classify::classify_email_cascade(&detail).await?;

            queries::update_classification(
                &pool,
                uid,
                &mailbox_id,
                &result.label,
                result.confidence,
            )
            .await
            .map_err(|e| e.to_string())?;

            println!("Classification: {}", result.label);
            println!("Confidence:     {:.0}%", result.confidence * 100.0);
            println!("Tier:           {}", result.tier);
            Ok(())
        }

        Commands::RunRules {
            mailbox,
            limit,
            dry_run,
        } => {
            let pool = open_db().await?;

            let all_rules = queries::get_rules(&pool)
                .await
                .map_err(|e| e.to_string())?;

            let enabled_count = all_rules.iter().filter(|r| r.enabled == 1).count();
            if enabled_count == 0 {
                println!("No enabled rules. Create one with: pm-cli rules create ...");
                return Ok(());
            }

            println!("Loaded {} enabled rule(s)", enabled_count);

            let mailboxes = queries::get_mailboxes(&pool)
                .await
                .map_err(|e| e.to_string())?;

            let mb = mailboxes
                .iter()
                .find(|m| m.name.eq_ignore_ascii_case(&mailbox))
                .ok_or_else(|| {
                    let names: Vec<_> = mailboxes.iter().map(|m| m.name.as_str()).collect();
                    format!(
                        "Mailbox '{}' not found. Available: {}",
                        mailbox,
                        names.join(", ")
                    )
                })?;

            let emails = queries::get_emails(&pool, &mb.id, limit, 0)
                .await
                .map_err(|e| e.to_string())?;

            if emails.is_empty() {
                println!("No emails in {}.", mailbox);
                return Ok(());
            }

            if dry_run {
                println!("DRY RUN — previewing matches (no actions executed)\n");
            }

            let results =
                core::rules::run_rules(&pool, &emails, &all_rules, dry_run).await?;

            for (uid, matched_actions) in &results {
                let email = emails.iter().find(|e| e.uid == *uid).unwrap();
                println!(
                    "  [uid={}] {} — {}",
                    email.uid,
                    email.sender_email.as_deref().unwrap_or("?"),
                    email.subject.as_deref().unwrap_or("(no subject)"),
                );
                for action in matched_actions {
                    println!("    -> {} : {}", action.action_type, action.value);
                }
            }

            if dry_run {
                println!(
                    "\n{} of {} emails matched rules (dry run, nothing changed)",
                    results.len(),
                    emails.len()
                );
            } else {
                println!(
                    "\nExecuted rules on {} of {} emails",
                    results.len(),
                    emails.len()
                );
            }

            Ok(())
        }
    }
}
