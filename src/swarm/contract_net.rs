#![allow(dead_code)]

use crate::swarm::state::NodeStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub min_battery: f32,
    pub needed_capabilities: Vec<String>,
    pub weight_estimate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBid {
    pub node_id: String,
    pub cost: f64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnnouncement {
    pub task_id: String,
    pub description: String,
    pub requirements: TaskRequirements,
    pub deadline: i64,
}

pub struct ContractNet {
    node_id: String,
}

impl ContractNet {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
        }
    }

    pub fn announce_task(&self, announcement: TaskAnnouncement) -> TaskAnnouncement {
        announcement
    }

    pub fn evaluate_bid(&self, bid: &TaskBid, requirements: &TaskRequirements) -> bool {
        bid.cost > 0.0
            && requirements
                .needed_capabilities
                .iter()
                .all(|cap| bid.capabilities.contains(cap))
    }

    pub fn select_winner<'a>(&self, bids: &'a [TaskBid]) -> Option<&'a TaskBid> {
        bids.iter()
            .min_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap())
    }
}
