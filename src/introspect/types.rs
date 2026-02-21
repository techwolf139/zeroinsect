#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod node {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NodeInfo {
        pub name: String,
        pub namespace: String,
        pub full_name: String,
        pub executable: Option<String>,
        pub publishers: Vec<String>,
        pub subscribers: Vec<String>,
        pub services: Vec<String>,
        pub actions: Vec<String>,
    }

    impl NodeInfo {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                namespace: String::new(),
                full_name: name.to_string(),
                executable: None,
                publishers: Vec::new(),
                subscribers: Vec::new(),
                services: Vec::new(),
                actions: Vec::new(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct NodeGraph {
        pub nodes: Vec<NodeInfo>,
        pub edges: Vec<GraphEdge>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GraphEdge {
        pub from: String,
        pub to: String,
        pub topic: String,
    }

    impl NodeGraph {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add_node(&mut self, node: NodeInfo) {
            self.nodes.push(node);
        }

        pub fn add_edge(&mut self, edge: GraphEdge) {
            self.edges.push(edge);
        }
    }
}

pub mod topic {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TopicInfo {
        pub name: String,
        pub type_name: String,
        pub publishers: Vec<String>,
        pub subscribers: Vec<String>,
        pub qos: Option<QosProfile>,
    }

    impl TopicInfo {
        pub fn new(name: &str, type_name: &str) -> Self {
            Self {
                name: name.to_string(),
                type_name: type_name.to_string(),
                publishers: Vec::new(),
                subscribers: Vec::new(),
                qos: None,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct QosProfile {
        pub reliability: String,
        pub durability: String,
        pub history: String,
        pub depth: u32,
    }

    impl Default for QosProfile {
        fn default() -> Self {
            Self {
                reliability: "RELIABLE".to_string(),
                durability: "VOLATILE".to_string(),
                history: "KEEP_LAST".to_string(),
                depth: 10,
            }
        }
    }
}

pub mod service {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ServiceInfo {
        pub name: String,
        pub type_name: String,
        pub provider_nodes: Vec<String>,
    }

    impl ServiceInfo {
        pub fn new(name: &str, type_name: &str) -> Self {
            Self {
                name: name.to_string(),
                type_name: type_name.to_string(),
                provider_nodes: Vec::new(),
            }
        }
    }
}

pub mod action {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ActionInfo {
        pub name: String,
        pub type_name: String,
        pub server_nodes: Vec<String>,
        pub client_nodes: Vec<String>,
    }

    impl ActionInfo {
        pub fn new(name: &str, type_name: &str) -> Self {
            Self {
                name: name.to_string(),
                type_name: type_name.to_string(),
                server_nodes: Vec::new(),
                client_nodes: Vec::new(),
            }
        }
    }
}

pub mod snapshot {
    use super::*;
    use chrono::Utc;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RosSnapshot {
        pub version: u64,
        pub timestamp: i64,
        pub nodes: HashMap<String, node::NodeInfo>,
        pub topics: HashMap<String, topic::TopicInfo>,
        pub services: HashMap<String, service::ServiceInfo>,
        pub actions: HashMap<String, action::ActionInfo>,
        pub checksum: String,
    }

    impl RosSnapshot {
        pub fn new() -> Self {
            Self {
                version: 0,
                timestamp: Utc::now().timestamp(),
                nodes: HashMap::new(),
                topics: HashMap::new(),
                services: HashMap::new(),
                actions: HashMap::new(),
                checksum: String::new(),
            }
        }

        pub fn compute_checksum(&self) -> String {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            self.version.hash(&mut hasher);
            self.timestamp.hash(&mut hasher);
            self.nodes.len().hash(&mut hasher);
            self.topics.len().hash(&mut hasher);
            self.services.len().hash(&mut hasher);
            self.actions.len().hash(&mut hasher);

            format!("{:016x}", hasher.finish())
        }

        pub fn diff(&self, other: &RosSnapshot) -> SnapshotDiff {
            let mut added_nodes = Vec::new();
            let mut removed_nodes = Vec::new();

            for (name, node) in &other.nodes {
                if !self.nodes.contains_key(name) {
                    added_nodes.push(node.clone());
                }
            }

            for name in self.nodes.keys() {
                if !other.nodes.contains_key(name) {
                    removed_nodes.push(name.clone());
                }
            }

            let mut added_topics = Vec::new();
            let mut removed_topics = Vec::new();

            for (name, topic) in &other.topics {
                if !self.topics.contains_key(name) {
                    added_topics.push(topic.clone());
                }
            }

            for name in self.topics.keys() {
                if !other.topics.contains_key(name) {
                    removed_topics.push(name.clone());
                }
            }

            let mut added_services = Vec::new();
            let mut removed_services = Vec::new();

            for (name, svc) in &other.services {
                if !self.services.contains_key(name) {
                    added_services.push(svc.clone());
                }
            }

            for name in self.services.keys() {
                if !other.services.contains_key(name) {
                    removed_services.push(name.clone());
                }
            }

            SnapshotDiff {
                added_nodes,
                removed_nodes,
                added_topics,
                removed_topics,
                added_services,
                removed_services,
            }
        }
    }

    impl Default for RosSnapshot {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct SnapshotDiff {
        pub added_nodes: Vec<node::NodeInfo>,
        pub removed_nodes: Vec<String>,
        pub added_topics: Vec<topic::TopicInfo>,
        pub removed_topics: Vec<String>,
        pub added_services: Vec<service::ServiceInfo>,
        pub removed_services: Vec<String>,
    }
}

pub mod cache {
    use super::*;
    use snapshot::RosSnapshot;
    use std::fs;
    use std::time::{Duration, Instant};

    pub struct CacheManager {
        memory_cache: Option<RosSnapshot>,
        disk_cache_path: PathBuf,
        ttl: Duration,
        last_fetch: Option<Instant>,
    }

    impl CacheManager {
        pub fn new(cache_dir: &str, ttl_seconds: u64) -> Self {
            let path = PathBuf::from(cache_dir).join("ros_snapshot.json");
            Self {
                memory_cache: None,
                disk_cache_path: path,
                ttl: Duration::from_secs(ttl_seconds),
                last_fetch: None,
            }
        }

        pub fn get(&self) -> Option<&RosSnapshot> {
            self.memory_cache.as_ref()
        }

        pub fn set_memory(&mut self, snapshot: RosSnapshot) {
            self.memory_cache = Some(snapshot);
            self.last_fetch = Some(Instant::now());
        }

        pub fn load_disk(&self) -> Option<RosSnapshot> {
            if self.disk_cache_path.exists() {
                let content = fs::read_to_string(&self.disk_cache_path).ok()?;
                serde_json::from_str(&content).ok()
            } else {
                None
            }
        }

        pub fn save_disk(&self, snapshot: &RosSnapshot) -> Result<(), String> {
            if let Some(parent) = self.disk_cache_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = serde_json::to_string_pretty(snapshot).map_err(|e| e.to_string())?;
            fs::write(&self.disk_cache_path, content).map_err(|e| e.to_string())
        }

        pub fn is_expired(&self) -> bool {
            match self.last_fetch {
                Some(time) => time.elapsed() > self.ttl,
                None => true,
            }
        }

        pub fn clear(&mut self) {
            self.memory_cache = None;
            self.last_fetch = None;
        }
    }

    impl Default for CacheManager {
        fn default() -> Self {
            Self::new("/tmp/zeroinsect", 30)
        }
    }
}

pub mod capability {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CapabilityCard {
        pub name: String,
        pub capability_type: CapabilityType,
        pub ros_type: String,
        pub description: String,
        pub parameters: Vec<FieldSchema>,
        pub call_template: CallTemplate,
        pub confidence: f32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum CapabilityType {
        TopicPublisher,
        TopicSubscriber,
        Service,
        Action,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FieldSchema {
        pub name: String,
        pub field_type: String,
        pub required: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CallTemplate {
        pub tool_name: String,
        pub args_template: serde_json::Value,
    }

    impl CapabilityCard {
        pub fn new(name: &str, capability_type: CapabilityType, ros_type: &str) -> Self {
            Self {
                name: name.to_string(),
                capability_type,
                ros_type: ros_type.to_string(),
                description: String::new(),
                parameters: Vec::new(),
                call_template: CallTemplate {
                    tool_name: String::new(),
                    args_template: serde_json::json!({}),
                },
                confidence: 1.0,
            }
        }
    }
}
