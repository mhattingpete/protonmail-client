use crate::db::schema::*;

pub fn evaluate_rules(email: &Email, rules: &[AutomationRule]) -> Vec<RuleAction> {
    let mut matched_actions: Vec<RuleAction> = Vec::new();

    let mut sorted_rules: Vec<&AutomationRule> = rules.iter().filter(|r| r.enabled == 1).collect();
    sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

    for rule in sorted_rules {
        let conditions: Vec<RuleCondition> =
            serde_json::from_str(&rule.conditions).unwrap_or_default();
        let actions: Vec<RuleAction> =
            serde_json::from_str(&rule.actions).unwrap_or_default();

        if conditions_match(email, &conditions) {
            matched_actions.extend(actions);
        }
    }

    matched_actions
}

fn conditions_match(email: &Email, conditions: &[RuleCondition]) -> bool {
    conditions.iter().all(|c| condition_matches(email, c))
}

fn condition_matches(email: &Email, condition: &RuleCondition) -> bool {
    let field_value = match condition.field.as_str() {
        "subject" => email.subject.as_deref().unwrap_or(""),
        "sender_email" => email.sender_email.as_deref().unwrap_or(""),
        "sender_name" => email.sender_name.as_deref().unwrap_or(""),
        "recipients" => email.recipients.as_deref().unwrap_or(""),
        _ => return false,
    };

    match condition.operator.as_str() {
        "contains" => field_value
            .to_lowercase()
            .contains(&condition.value.to_lowercase()),
        "equals" => field_value.eq_ignore_ascii_case(&condition.value),
        "starts_with" => field_value
            .to_lowercase()
            .starts_with(&condition.value.to_lowercase()),
        "ends_with" => field_value
            .to_lowercase()
            .ends_with(&condition.value.to_lowercase()),
        "not_contains" => !field_value
            .to_lowercase()
            .contains(&condition.value.to_lowercase()),
        _ => false,
    }
}
