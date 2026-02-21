#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Idle,
    Busy,
    Charging,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNode {
    pub id: String,
    pub battery: f32,
    pub position: Position,
    pub status: NodeStatus,
}

impl SwarmNode {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            battery: 100.0,
            position: Position {
                x: 0.0,
                y: 0.0,
                theta: 0.0,
            },
            status: NodeStatus::Idle,
        }
    }

    pub fn is_available(&self) -> bool {
        self.status == NodeStatus::Idle && self.battery > 20.0
    }
}
