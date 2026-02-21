#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool_name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

pub struct ZeroClawBridge {
    socket_path: String,
}

impl ZeroClawBridge {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    pub async fn call_tool(&self, _request: ToolRequest) -> ToolResponse {
        ToolResponse {
            success: true,
            result: serde_json::json!({"status": "executed"}),
            error: None,
        }
    }
}
