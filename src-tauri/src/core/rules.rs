use crate::db::schema::*;
use crate::rules::{actions, engine};
use sqlx::SqlitePool;

pub fn build_rule(
    name: String,
    conditions: String,
    actions: String,
    priority: i64,
) -> AutomationRule {
    let now = chrono::Utc::now().timestamp();
    AutomationRule {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        enabled: 1,
        priority,
        conditions,
        actions,
        created_at: Some(now),
        updated_at: Some(now),
    }
}

pub fn apply_updates(
    existing: AutomationRule,
    name: Option<String>,
    enabled: Option<bool>,
    conditions: Option<String>,
    actions: Option<String>,
    priority: Option<i64>,
) -> AutomationRule {
    AutomationRule {
        id: existing.id,
        name: name.unwrap_or(existing.name),
        enabled: enabled
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(existing.enabled),
        priority: priority.unwrap_or(existing.priority),
        conditions: conditions.unwrap_or(existing.conditions),
        actions: actions.unwrap_or(existing.actions),
        created_at: existing.created_at,
        updated_at: Some(chrono::Utc::now().timestamp()),
    }
}

pub async fn run_rules(
    pool: &SqlitePool,
    emails: &[Email],
    rules: &[AutomationRule],
    dry_run: bool,
) -> Result<Vec<(i64, Vec<RuleAction>)>, String> {
    let mut results = Vec::new();

    for email in emails {
        let matched_actions = engine::evaluate_rules(email, rules);
        if matched_actions.is_empty() {
            continue;
        }

        if !dry_run {
            actions::execute_actions(pool, email, &matched_actions).await?;
        }

        results.push((email.uid, matched_actions));
    }

    Ok(results)
}
