use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityCategory {
    Sensing,
    Decision,
    Actuation,
    Unknown,
}

impl Default for CapabilityCategory {
    fn default() -> Self {
        CapabilityCategory::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RosCapabilityType {
    Node,
    Topic,
    Service,
    Action,
}

impl Default for RosCapabilityType {
    fn default() -> Self {
        RosCapabilityType::Node
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub capability_id: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub capability_id: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityNode {
    pub id: String,
    pub name: String,
    pub category: CapabilityCategory,
    pub ros_type: RosCapabilityType,
    pub node: String,
    pub description: String,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub ros_path: String,
}

impl CapabilityNode {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category: CapabilityCategory::default(),
            ros_type: RosCapabilityType::default(),
            node: String::new(),
            description: String::new(),
            preconditions: Vec::new(),
            effects: Vec::new(),
            ros_path: id.to_string(),
        }
    }

    pub fn with_category(mut self, category: CapabilityCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_ros_type(mut self, ros_type: RosCapabilityType) -> Self {
        self.ros_type = ros_type;
        self
    }

    pub fn with_node(mut self, node: &str) -> Self {
        self.node = node.to_string();
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_precondition(mut self, condition: Condition) -> Self {
        self.preconditions.push(condition);
        self
    }

    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CausalRelation {
    Enables,
    Produces,
    Consumes,
    Conflicts,
    Triggers,
}

impl Default for CausalRelation {
    fn default() -> Self {
        CausalRelation::Triggers
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub relation: CausalRelation,
    pub probability: f32,
}

impl CausalEdge {
    pub fn new(from: &str, to: &str, relation: CausalRelation) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            relation,
            probability: 1.0,
        }
    }

    pub fn with_probability(mut self, p: f32) -> Self {
        self.probability = p.clamp(0.0, 1.0);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCapability {
    pub name: String,
    pub type_name: String,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
    pub category: CapabilityCategory,
}

impl TopicCapability {
    pub fn new(name: &str, type_name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            publishers: Vec::new(),
            subscribers: Vec::new(),
            category: CapabilityCategory::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    pub name: String,
    pub type_name: String,
    pub provider_nodes: Vec<String>,
    pub category: CapabilityCategory,
}

impl ServiceCapability {
    pub fn new(name: &str, type_name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            provider_nodes: Vec::new(),
            category: CapabilityCategory::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCapability {
    pub name: String,
    pub type_name: String,
    pub server_nodes: Vec<String>,
    pub category: CapabilityCategory,
}

impl ActionCapability {
    pub fn new(name: &str, type_name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            server_nodes: Vec::new(),
            category: CapabilityCategory::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityMap {
    pub nodes: HashMap<String, CapabilityNode>,
    pub edges: Vec<CausalEdge>,
    pub topics: HashMap<String, TopicCapability>,
    pub services: HashMap<String, ServiceCapability>,
    pub actions: HashMap<String, ActionCapability>,
    pub version: String,
    pub timestamp: i64,
}

impl CapabilityMap {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            topics: HashMap::new(),
            services: HashMap::new(),
            actions: HashMap::new(),
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn add_node(&mut self, node: CapabilityNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn remove_node(&mut self, id: &str) -> Option<CapabilityNode> {
        self.nodes.remove(id)
    }

    pub fn get_node(&self, id: &str) -> Option<&CapabilityNode> {
        self.nodes.get(id)
    }

    pub fn add_edge(&mut self, edge: CausalEdge) {
        self.edges.push(edge);
    }

    pub fn remove_edge(&mut self, from: &str, to: &str) -> bool {
        let initial_len = self.edges.len();
        self.edges.retain(|e| !(e.from == from && e.to == to));
        self.edges.len() < initial_len
    }

    pub fn get_edges_from(&self, id: &str) -> Vec<&CausalEdge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    pub fn get_edges_to(&self, id: &str) -> Vec<&CausalEdge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }

    pub fn add_topic(&mut self, topic: TopicCapability) {
        self.topics.insert(topic.name.clone(), topic);
    }

    pub fn add_service(&mut self, service: ServiceCapability) {
        self.services.insert(service.name.clone(), service);
    }

    pub fn add_action(&mut self, action: ActionCapability) {
        self.actions.insert(action.name.clone(), action);
    }

    pub fn nodes_by_category(&self, category: &CapabilityCategory) -> Vec<&CapabilityNode> {
        self.nodes
            .values()
            .filter(|n| &n.category == category)
            .collect()
    }

    pub fn topics_by_category(&self, category: &CapabilityCategory) -> Vec<&TopicCapability> {
        self.topics
            .values()
            .filter(|t| &t.category == category)
            .collect()
    }

    pub fn services_by_category(&self, category: &CapabilityCategory) -> Vec<&ServiceCapability> {
        self.services
            .values()
            .filter(|s| &s.category == category)
            .collect()
    }

    pub fn actions_by_category(&self, category: &CapabilityCategory) -> Vec<&ActionCapability> {
        self.actions
            .values()
            .filter(|a| &a.category == category)
            .collect()
    }

    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();

        queue.push_back((from.to_string(), vec![from.to_string()]));
        visited.insert(from.to_string());

        while let Some((current, path)) = queue.pop_front() {
            if current == to {
                return Some(path);
            }

            for edge in self.get_edges_from(&current) {
                if !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    let mut new_path = path.clone();
                    new_path.push(edge.to.clone());
                    queue.push_back((edge.to.clone(), new_path));
                }
            }
        }

        None
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.topics.clear();
        self.services.clear();
        self.actions.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for CapabilityNode {
    fn default() -> Self {
        Self::new("", "")
    }
}

impl Default for CausalEdge {
    fn default() -> Self {
        Self::new("", "", CausalRelation::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = CapabilityNode::new("node1", "test_node")
            .with_category(CapabilityCategory::Sensing)
            .with_ros_type(RosCapabilityType::Topic)
            .with_description("Test node");

        assert_eq!(node.id, "node1");
        assert_eq!(node.category, CapabilityCategory::Sensing);
        assert_eq!(node.description, "Test node");
    }

    #[test]
    fn test_capability_map_basic() {
        let mut map = CapabilityMap::new();

        let node = CapabilityNode::new("node1", "test");
        map.add_node(node);

        assert_eq!(map.node_count(), 1);
        assert!(map.get_node("node1").is_some());
        assert!(map.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_edge_operations() {
        let mut map = CapabilityMap::new();

        let node1 = CapabilityNode::new("n1", "node1");
        let node2 = CapabilityNode::new("n2", "node2");
        map.add_node(node1);
        map.add_node(node2);

        let edge = CausalEdge::new("n1", "n2", CausalRelation::Produces);
        map.add_edge(edge);

        assert_eq!(map.edge_count(), 1);

        let edges_from = map.get_edges_from("n1");
        assert_eq!(edges_from.len(), 1);
        assert_eq!(edges_from[0].to, "n2");
    }

    #[test]
    fn test_find_path() {
        let mut map = CapabilityMap::new();

        map.add_node(CapabilityNode::new("a", "a"));
        map.add_node(CapabilityNode::new("b", "b"));
        map.add_node(CapabilityNode::new("c", "c"));

        map.add_edge(CausalEdge::new("a", "b", CausalRelation::Enables));
        map.add_edge(CausalEdge::new("b", "c", CausalRelation::Enables));

        let path = map.find_path("a", "c");
        assert!(path.is_some());
        assert_eq!(path.unwrap(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_category_filter() {
        let mut map = CapabilityMap::new();

        map.add_node(CapabilityNode::new("n1", "n1").with_category(CapabilityCategory::Sensing));
        map.add_node(CapabilityNode::new("n2", "n2").with_category(CapabilityCategory::Actuation));
        map.add_node(CapabilityNode::new("n3", "n3").with_category(CapabilityCategory::Sensing));

        let sensing = map.nodes_by_category(&CapabilityCategory::Sensing);
        assert_eq!(sensing.len(), 2);
    }
}
