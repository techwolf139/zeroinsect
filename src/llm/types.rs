use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub intent: String,
    pub parameters: serde_json::Value,
    pub confidence: f32,
    pub needs_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
