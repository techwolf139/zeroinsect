#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolError {
    Ros2Error(String),
    Timeout(String),
    NotConnected,
}

pub trait RobotTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn supported_topics(&self) -> Vec<&'static str>;
}

pub struct RobotStateTool;

impl RobotStateTool {
    pub fn new() -> Self {
        Self
    }
}

impl RobotTool for RobotStateTool {
    fn name(&self) -> &str {
        "get_robot_state"
    }

    fn description(&self) -> &str {
        "Get current robot state (position, velocity, battery)"
    }

    fn supported_topics(&self) -> Vec<&'static str> {
        vec!["/odom", "/battery_state", "/joint_states"]
    }
}
