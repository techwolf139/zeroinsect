#![allow(dead_code)]

use crate::introspect::types::action::ActionInfo;
use crate::introspect::types::node::NodeInfo;
use crate::introspect::types::service::ServiceInfo;
use crate::introspect::types::snapshot::{RosSnapshot, SnapshotDiff};
use crate::introspect::types::topic::{QosProfile, TopicInfo};
use chrono::Utc;
use std::collections::HashMap;

pub struct RosRuntimeIntrospector {
    version_counter: u64,
}

impl RosRuntimeIntrospector {
    pub fn new() -> Self {
        Self { version_counter: 0 }
    }

    pub fn list_nodes(&self) -> Result<HashMap<String, NodeInfo>, String> {
        #[cfg(feature = "ros2")]
        {
            Err("ROS2 runtime not available".to_string())
        }
        #[cfg(not(feature = "ros2"))]
        {
            Ok(self.mock_nodes())
        }
    }

    pub fn list_topics(&self) -> Result<HashMap<String, TopicInfo>, String> {
        #[cfg(feature = "ros2")]
        {
            Err("ROS2 runtime not available".to_string())
        }
        #[cfg(not(feature = "ros2"))]
        {
            Ok(self.mock_topics())
        }
    }

    pub fn list_services(&self) -> Result<HashMap<String, ServiceInfo>, String> {
        #[cfg(feature = "ros2")]
        {
            Err("ROS2 runtime not available".to_string())
        }
        #[cfg(not(feature = "ros2"))]
        {
            Ok(self.mock_services())
        }
    }

    pub fn list_actions(&self) -> Result<HashMap<String, ActionInfo>, String> {
        #[cfg(feature = "ros2")]
        {
            Err("ROS2 runtime not available".to_string())
        }
        #[cfg(not(feature = "ros2"))]
        {
            Ok(self.mock_actions())
        }
    }

    pub fn capture_snapshot(&mut self) -> Result<RosSnapshot, String> {
        self.version_counter += 1;

        let nodes = self.list_nodes()?;
        let topics = self.list_topics()?;
        let services = self.list_services()?;
        let actions = self.list_actions()?;

        let mut snapshot = RosSnapshot {
            version: self.version_counter,
            timestamp: Utc::now().timestamp(),
            nodes,
            topics,
            services,
            actions,
            checksum: String::new(),
        };

        snapshot.checksum = snapshot.compute_checksum();

        Ok(snapshot)
    }

    pub fn diff(&self, old: &RosSnapshot, new: &RosSnapshot) -> SnapshotDiff {
        old.diff(new)
    }

    fn mock_nodes(&self) -> HashMap<String, NodeInfo> {
        let mut nodes = HashMap::new();

        let mut node = NodeInfo::new("robot_state_publisher");
        node.namespace = "/robot".to_string();
        node.full_name = "/robot/robot_state_publisher".to_string();
        node.executable = Some("robot_state_publisher".to_string());
        node.publishers = vec!["/robot/joint_states".to_string(), "/robot/odom".to_string()];
        nodes.insert(node.full_name.clone(), node);

        let mut node2 = NodeInfo::new("navigation");
        node2.namespace = "/nav2".to_string();
        node2.full_name = "/nav2/navigation".to_string();
        node2.executable = Some("nav2_controller".to_string());
        node2.services = vec!["/nav2/navigate_to_pose".to_string()];
        nodes.insert(node2.full_name.clone(), node2);

        nodes
    }

    fn mock_topics(&self) -> HashMap<String, TopicInfo> {
        let mut topics = HashMap::new();

        let mut scan = TopicInfo::new("/scan", "sensor_msgs/msg/LaserScan");
        scan.publishers = vec!["/robot/rplidar_node".to_string()];
        scan.subscribers = vec!["/nav2/slam_toolbox".to_string()];
        scan.qos = Some(QosProfile::default());
        topics.insert(scan.name.clone(), scan);

        let mut cmd_vel = TopicInfo::new("/cmd_vel", "geometry_msgs/msg/Twist");
        cmd_vel.publishers = vec!["/nav2/navigation".to_string()];
        cmd_vel.subscribers = vec!["/robot/diff_drive_controller".to_string()];
        topics.insert(cmd_vel.name.clone(), cmd_vel);

        topics
    }

    fn mock_services(&self) -> HashMap<String, ServiceInfo> {
        let mut services = HashMap::new();

        let mut nav = ServiceInfo::new("/nav2/navigate_to_pose", "nav2_msgs/srv/NavigateToPose");
        nav.provider_nodes = vec!["/nav2/navigation".to_string()];
        services.insert(nav.name.clone(), nav);

        let mut localize = ServiceInfo::new("/localization/reset", "std_srvs/srv/Empty");
        localize.provider_nodes = vec!["/nav2/amcl".to_string()];
        services.insert(localize.name.clone(), localize);

        services
    }

    fn mock_actions(&self) -> HashMap<String, ActionInfo> {
        let mut actions = HashMap::new();

        let mut nav = ActionInfo::new("/nav2/navigate_to_pose", "nav2_msgs/action/NavigateToPose");
        nav.server_nodes = vec!["/nav2/navigation".to_string()];
        actions.insert(nav.name.clone(), nav);

        actions
    }
}

impl Default for RosRuntimeIntrospector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_capture() {
        let mut introspector = RosRuntimeIntrospector::new();
        let snapshot = introspector.capture_snapshot().unwrap();

        assert!(snapshot.version > 0);
        assert!(!snapshot.checksum.is_empty());
        assert!(!snapshot.nodes.is_empty());
    }

    #[test]
    fn test_snapshot_diff() {
        let mut introspector = RosRuntimeIntrospector::new();

        let old = introspector.capture_snapshot().unwrap();
        let new = introspector.capture_snapshot().unwrap();

        let diff = introspector.diff(&old, &new);

        assert!(diff.added_nodes.is_empty());
        assert!(diff.removed_nodes.is_empty());
    }
}
