use crate::db::{queries, schema::*};
use sqlx::SqlitePool;

pub async fn execute_actions(
    pool: &SqlitePool,
    email: &Email,
    actions: &[RuleAction],
) -> Result<(), String> {
    for action in actions {
        if let Err(e) = execute_action(pool, email, action).await {
            log::error!(
                "Failed to execute action {:?} on email uid={}: {}",
                action.action_type,
                email.uid,
                e
            );
        }
    }
    Ok(())
}

async fn execute_action(
    pool: &SqlitePool,
    email: &Email,
    action: &RuleAction,
) -> Result<(), String> {
    match action.action_type.as_str() {
        "move" => {
            let target = queries::get_mailbox_by_name(pool, &action.value)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Target mailbox '{}' not found", action.value))?;

            queries::move_email(pool, email.uid, &email.mailbox_id, &target.id)
                .await
                .map_err(|e| e.to_string())
        }
        "label" | "classify" => {
            queries::update_classification(pool, email.uid, &email.mailbox_id, &action.value, 1.0)
                .await
                .map_err(|e| e.to_string())
        }
        "mark_read" => {
            let mut flags: Vec<String> =
                serde_json::from_str(email.flags.as_deref().unwrap_or("[]"))
                    .unwrap_or_default();
            if !flags.iter().any(|f| f == "\\Seen") {
                flags.push("\\Seen".to_string());
            }
            let flags_json = serde_json::to_string(&flags).map_err(|e| e.to_string())?;
            queries::update_flags(pool, email.uid, &email.mailbox_id, &flags_json)
                .await
                .map_err(|e| e.to_string())
        }
        "star" => {
            let mut flags: Vec<String> =
                serde_json::from_str(email.flags.as_deref().unwrap_or("[]"))
                    .unwrap_or_default();
            if !flags.iter().any(|f| f == "\\Flagged") {
                flags.push("\\Flagged".to_string());
            }
            let flags_json = serde_json::to_string(&flags).map_err(|e| e.to_string())?;
            queries::update_flags(pool, email.uid, &email.mailbox_id, &flags_json)
                .await
                .map_err(|e| e.to_string())
        }
        "delete" => queries::delete_email(pool, email.uid, &email.mailbox_id)
            .await
            .map_err(|e| e.to_string()),
        _ => {
            log::warn!("Unknown action type: {}", action.action_type);
            Ok(())
        }
    }
}
