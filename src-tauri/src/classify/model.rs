use crate::db::schema::*;

/// Tier 2: Local ML model classification (ONNX placeholder)
///
/// This is a placeholder for future ONNX Runtime integration.
/// When implemented, it would load a trained text classifier model
/// and run inference locally for fast, private classification.
pub fn classify_local(_detail: &EmailDetail) -> Option<Classification> {
    // Placeholder: always returns None to fall through to tier 3
    // Future implementation would:
    // 1. Load ONNX model from app data directory
    // 2. Tokenize email subject + body
    // 3. Run inference
    // 4. Return classification if confidence > threshold
    None
}
