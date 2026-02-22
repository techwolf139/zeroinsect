use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TopicTree {
    root: Arc<RwLock<Node>>,
}

#[derive(Debug, Clone)]
struct Node {
    children: HashMap<String, Node>,
    subscribers: HashSet<String>,
    is_wildcard: bool,
}

impl Node {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            subscribers: HashSet::new(),
            is_wildcard: false,
        }
    }
}

impl TopicTree {
    pub fn new() -> Self {
        Self {
            root: Arc::new(RwLock::new(Node::new())),
        }
    }

    pub fn insert(&self, topic: &str, client_id: &str) {
        let parts: Vec<&str> = topic.split('/').collect();
        let mut node = self.root.write();

        Self::insert_recursive(&mut node, &parts, client_id);
    }

    fn insert_recursive(node: &mut Node, parts: &[&str], client_id: &str) {
        if parts.is_empty() {
            node.subscribers.insert(client_id.to_string());
            return;
        }

        let part = parts[0];
        let child = node
            .children
            .entry(part.to_string())
            .or_insert_with(Node::new);

        if part == "+" || part == "#" {
            child.is_wildcard = true;
        }

        Self::insert_recursive(child, &parts[1..], client_id);
    }

    pub fn remove(&self, topic: &str, client_id: &str) {
        let parts: Vec<&str> = topic.split('/').collect();
        let mut node = self.root.write();

        Self::remove_recursive(&mut node, &parts, client_id);
    }

    fn remove_recursive(node: &mut Node, parts: &[&str], client_id: &str) {
        if parts.is_empty() {
            node.subscribers.remove(client_id);
            return;
        }

        if let Some(child) = node.children.get_mut(parts[0]) {
            Self::remove_recursive(child, &parts[1..], client_id);
        }
    }

    pub fn matches(&self, topic: &str) -> HashSet<String> {
        let parts: Vec<&str> = topic.split('/').collect();
        let node = self.root.read();
        let mut matches = HashSet::new();

        Self::match_recursive(&node, &parts, &mut matches);
        matches
    }

    fn match_recursive(node: &Node, parts: &[&str], matches: &mut HashSet<String>) {
        for subscriber in &node.subscribers {
            matches.insert(subscriber.clone());
        }

        if parts.is_empty() {
            return;
        }

        let part = parts[0];

        if let Some(child) = node.children.get(part) {
            Self::match_recursive(child, &parts[1..], matches);
        }

        if let Some(wildcard) = node.children.get("+") {
            Self::match_recursive(wildcard, &parts[1..], matches);
        }

        if let Some(multi_wildcard) = node.children.get("#") {
            for i in 0..=parts.len() {
                Self::match_recursive(multi_wildcard, &parts[i..], matches);
            }
        }
    }

    pub fn get_subscribers(&self, topic: &str) -> Vec<String> {
        self.matches(topic).into_iter().collect()
    }
}

impl Default for TopicTree {
    fn default() -> Self {
        Self::new()
    }
}
