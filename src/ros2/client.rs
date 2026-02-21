#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

#[cfg(feature = "ros2")]
use std::sync::Arc;

#[cfg(feature = "ros2")]
use parking_lot::RwLock;

#[cfg(feature = "ros2")]
use r2r::Node;

pub struct Ros2Client {
    #[cfg(feature = "ros2")]
    node: Option<Arc<RwLock<Node>>>,
    publishers: HashMap<String, ()>,
    subscribers: HashMap<String, ()>,
}

impl Ros2Client {
    pub fn new(node_name: &str) -> Result<Self, String> {
        #[cfg(feature = "ros2")]
        {
            let node = Node::builder()
                .name(node_name)
                .build()
                .map_err(|e| format!("Failed to create node: {}", e))?;
            Ok(Self {
                node: Some(Arc::new(RwLock::new(node))),
                publishers: HashMap::new(),
                subscribers: HashMap::new(),
            })
        }
        #[cfg(not(feature = "ros2"))]
        {
            let _ = node_name;
            Ok(Self {
                publishers: HashMap::new(),
                subscribers: HashMap::new(),
            })
        }
    }

    pub fn is_connected(&self) -> bool {
        #[cfg(feature = "ros2")]
        return self.node.is_some();
        #[cfg(not(feature = "ros2"))]
        false
    }
}
