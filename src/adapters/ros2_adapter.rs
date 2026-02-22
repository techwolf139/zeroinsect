//! ROS2 Adapter - Connects ROS2 Humble to MQTT Bridge
//!
//! This adapter discovers ROS2 capabilities and publishes them to the Bridge Hub
//! via MQTT, enabling cross-ROS communication.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub ros_version: String,
    pub device_id: String,
    pub topics: Vec<TopicInfo>,
    pub services: Vec<ServiceInfo>,
    pub actions: Vec<ActionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub name: String,
    pub msg_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub srv_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub name: String,
    pub action_type: String,
}

/// Adapter for ROS2 to MQTT Bridge
pub struct Ros2Adapter {
    device_id: String,
    mqtt_broker: String,
}

impl Ros2Adapter {
    pub fn new(device_id: &str, mqtt_broker: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            mqtt_broker: mqtt_broker.to_string(),
        }
    }

    /// Create a capability manifest from current ROS2 system
    /// Note: This requires r2r runtime - returns empty manifest if not available
    pub fn create_manifest(&self) -> CapabilityManifest {
        CapabilityManifest {
            ros_version: "humble".to_string(),
            device_id: self.device_id.clone(),
            topics: vec![],
            services: vec![],
            actions: vec![],
        }
    }

    /// Get the MQTT topic for publishing capabilities
    pub fn capability_topic(&self) -> String {
        format!("bridge/capabilities/{}", self.device_id)
    }
}

#[cfg(feature = "ros2")]
mod r2r_impl {
    use super::*;
    use r2r::R2RResult;

    impl Ros2Adapter {
        /// Discover actual ROS2 capabilities using r2r
        pub fn discover_capabilities(&self, node: &r2r::Node) -> R2RResult<CapabilityManifest> {
            let topics: Vec<TopicInfo> = node
                .topics()
                .into_iter()
                .map(|(name, msg_type)| TopicInfo { name, msg_type })
                .collect();

            let services: Vec<ServiceInfo> = node
                .services()
                .into_iter()
                .map(|(name, srv_type)| ServiceInfo { name, srv_type })
                .collect();

            Ok(CapabilityManifest {
                ros_version: "humble".to_string(),
                device_id: self.device_id.clone(),
                topics,
                services,
                actions: vec![], // TODO: Discover actions
            })
        }
    }
}
