use crate::core;
use crate::db::{queries, schema::*};
use crate::state::AppState;

#[tauri::command]
pub async fn list_rules(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AutomationRule>, String> {
    queries::get_rules(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_rule(
    state: tauri::State<'_, AppState>,
    name: String,
    conditions: String,
    actions: String,
    priority: Option<i64>,
) -> Result<AutomationRule, String> {
    let rule = core::rules::build_rule(name, conditions, actions, priority.unwrap_or(0));

    queries::insert_rule(&state.db, &rule)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rule)
}

#[tauri::command]
pub async fn update_rule(
    state: tauri::State<'_, AppState>,
    id: String,
    name: Option<String>,
    conditions: Option<String>,
    actions: Option<String>,
    enabled: Option<i64>,
    priority: Option<i64>,
) -> Result<AutomationRule, String> {
    let existing = queries::get_rule(&state.db, &id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Rule not found".to_string())?;

    let enabled_bool = enabled.map(|e| e != 0);
    let updated = core::rules::apply_updates(existing, name, enabled_bool, conditions, actions, priority);

    queries::update_rule(&state.db, &updated)
        .await
        .map_err(|e| e.to_string())?;

    Ok(updated)
}

#[tauri::command]
pub async fn delete_rule(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    queries::delete_rule(&state.db, &id)
        .await
        .map_err(|e| e.to_string())
}
