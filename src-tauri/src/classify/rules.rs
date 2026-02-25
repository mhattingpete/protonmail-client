use crate::db::schema::*;

/// Tier 1: Heuristic-based classification using simple rules
pub fn classify_heuristic(detail: &EmailDetail) -> Option<Classification> {
    let subject = detail
        .email
        .subject
        .as_deref()
        .unwrap_or("")
        .to_lowercase();
    let sender = detail
        .email
        .sender_email
        .as_deref()
        .unwrap_or("")
        .to_lowercase();
    let body = detail
        .text_body
        .as_deref()
        .unwrap_or("")
        .to_lowercase();

    // Newsletter detection
    if sender.contains("newsletter")
        || sender.contains("noreply")
        || sender.contains("no-reply")
        || subject.contains("unsubscribe")
        || body.contains("unsubscribe")
    {
        return Some(Classification {
            label: "newsletter".to_string(),
            confidence: 0.85,
            tier: "heuristic".to_string(),
        });
    }

    // Transaction/receipt detection
    if subject.contains("receipt")
        || subject.contains("order confirmation")
        || subject.contains("invoice")
        || subject.contains("payment")
        || subject.contains("transaction")
    {
        return Some(Classification {
            label: "transaction".to_string(),
            confidence: 0.80,
            tier: "heuristic".to_string(),
        });
    }

    // Social notification detection
    if sender.contains("facebook")
        || sender.contains("twitter")
        || sender.contains("linkedin")
        || sender.contains("instagram")
        || sender.contains("github")
    {
        return Some(Classification {
            label: "social".to_string(),
            confidence: 0.80,
            tier: "heuristic".to_string(),
        });
    }

    // Promotion/marketing detection
    if subject.contains("sale")
        || subject.contains("% off")
        || subject.contains("discount")
        || subject.contains("limited time")
        || subject.contains("free shipping")
    {
        return Some(Classification {
            label: "promotion".to_string(),
            confidence: 0.75,
            tier: "heuristic".to_string(),
        });
    }

    // Calendar/meeting detection
    if subject.contains("invitation")
        || subject.contains("meeting")
        || subject.contains("calendar")
        || subject.contains("rsvp")
    {
        return Some(Classification {
            label: "calendar".to_string(),
            confidence: 0.80,
            tier: "heuristic".to_string(),
        });
    }

    None
}
