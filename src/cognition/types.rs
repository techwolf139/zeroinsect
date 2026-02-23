use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Error,
    Idle,
}

impl Default for DeviceStatus {
    fn default() -> Self {
        DeviceStatus::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub device_id: String,
    pub timestamp: u64,
    pub status: DeviceStatus,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub temperature: f32,
    pub last_command: Option<String>,
}

impl DeviceState {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: DeviceStatus::Idle,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            temperature: 0.0,
            last_command: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub device_id: String,
    pub sensor_type: String,
    pub values: Vec<f32>,
    pub timestamp: u64,
}

impl SensorData {
    pub fn new(device_id: String, sensor_type: String, values: Vec<f32>) -> Self {
        Self {
            device_id,
            sensor_type,
            values,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceKnowledge {
    pub device_id: String,
    pub capabilities: Vec<Capability>,
    pub relationships: Vec<Relationship>,
    pub last_analysis: Option<AnalysisResult>,
}

impl DeviceKnowledge {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            capabilities: Vec::new(),
            relationships: Vec::new(),
            last_analysis: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub category: CapabilityCategory,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CapabilityCategory {
    Sensing,
    Decision,
    Actuation,
    Communication,
    Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target_device_id: String,
    pub relation_type: RelationType,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    DependsOn,
    Produces,
    Consumes,
    Controls,
    Monitors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub timestamp: u64,
    pub analysis_type: String,
    pub result: HashMap<String, String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub node_id: String,
    pub device_id: String,
    pub node_type: NodeType,
    pub properties: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Device,
    Sensor,
    Actuator,
    Controller,
    Gateway,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub edge_id: String,
    pub from_node: String,
    pub to_node: String,
    pub relation: RelationType,
    pub weight: f32,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub device_id: String,
    pub time_window: String,
    pub metrics: HashMap<String, f32>,
    pub trends: Vec<TrendPoint>,
    pub anomalies: Vec<Anomaly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub timestamp: u64,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub timestamp: u64,
    pub anomaly_type: String,
    pub severity: f32,
    pub description: String,
}
