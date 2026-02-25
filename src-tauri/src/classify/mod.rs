pub mod llm;
pub mod model;
pub mod rules;

use crate::db::schema::*;

/// Classification cascade: tier 1 (heuristics) -> tier 2 (ONNX placeholder) -> tier 3 (Claude API)
pub async fn classify_email_cascade(detail: &EmailDetail) -> Result<Classification, String> {
    // Tier 1: Heuristic rules
    if let Some(result) = rules::classify_heuristic(detail) {
        return Ok(result);
    }

    // Tier 2: Local ML model (placeholder)
    if let Some(result) = model::classify_local(detail) {
        return Ok(result);
    }

    // Tier 3: Claude API
    llm::classify_with_llm(detail).await
}
