use crate::db::schema::*;

/// Tier 3: Claude API-based classification
pub async fn classify_with_llm(detail: &EmailDetail) -> Result<Classification, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?;

    let subject = detail
        .email
        .subject
        .as_deref()
        .unwrap_or("(no subject)");
    let sender = detail.email.sender_email.as_deref().unwrap_or("unknown");
    let body_preview = detail
        .text_body
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(500)
        .collect::<String>();

    let prompt = format!(
        "Classify this email into exactly one category. Respond with ONLY the category name.\n\
         Categories: personal, work, newsletter, transaction, social, promotion, calendar, spam, other\n\n\
         From: {}\nSubject: {}\nBody preview: {}",
        sender, subject, body_preview
    );

    let request_body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 50,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Claude API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Claude API error {}: {}", status, body));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

    let label = response_json["content"][0]["text"]
        .as_str()
        .unwrap_or("other")
        .trim()
        .to_lowercase();

    Ok(Classification {
        label,
        confidence: 0.90,
        tier: "llm".to_string(),
    })
}
