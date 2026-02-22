use crate::capability_map::graph::{
    ActionCapability, CapabilityCategory, CapabilityMap, CapabilityNode, CausalEdge,
    CausalRelation, ServiceCapability, TopicCapability,
};
use crate::introspect::types::snapshot::RosSnapshot;

pub struct CapabilityClassifier;

impl CapabilityClassifier {
    pub fn new() -> Self {
        Self
    }

    pub fn classify(&self, snapshot: &RosSnapshot) -> CapabilityMap {
        let mut map = CapabilityMap::new();

        for (name, topic) in &snapshot.topics {
            let category = self.classify_topic(name, &topic.type_name);

            let mut topic_cap = TopicCapability::new(name, &topic.type_name);
            topic_cap.publishers = topic.publishers.clone();
            topic_cap.subscribers = topic.subscribers.clone();
            topic_cap.category = category.clone();
            map.add_topic(topic_cap);

            let node = self.topic_to_node(
                name,
                &topic.type_name,
                &category,
                &topic.publishers,
                &topic.subscribers,
            );
            map.add_node(node);
        }

        for (name, service) in &snapshot.services {
            let category = self.classify_service(name, &service.type_name);

            let mut service_cap = ServiceCapability::new(name, &service.type_name);
            service_cap.provider_nodes = service.provider_nodes.clone();
            service_cap.category = category.clone();
            map.add_service(service_cap);

            let node =
                self.service_to_node(name, &service.type_name, &category, &service.provider_nodes);
            map.add_node(node);
        }

        for (name, action) in &snapshot.actions {
            let category = CapabilityCategory::Actuation;

            let mut action_cap = ActionCapability::new(name, &action.type_name);
            action_cap.server_nodes = action.server_nodes.clone();
            action_cap.category = category.clone();
            map.add_action(action_cap);

            let node = self.action_to_node(name, &action.type_name, &action.server_nodes);
            map.add_node(node);
        }

        for (name, node_info) in &snapshot.nodes {
            let category = self.classify_node(
                name,
                &node_info.publishers,
                &node_info.subscribers,
                &node_info.services,
            );

            let mut node = CapabilityNode::new(name, &node_info.name);
            node.category = category;
            node.ros_type = crate::capability_map::graph::RosCapabilityType::Node;
            node.node = name.clone();
            node.description = format!("ROS node: {}", node_info.name);

            if let Some(exec_) = &node_info.executable {
                node.description
                    .push_str(&format!(", executable: {}", exec_));
            }

            map.add_node(node);
        }

        self.infer_causal_edges(&mut map);

        map
    }

    fn classify_topic(&self, name: &str, _type_name: &str) -> CapabilityCategory {
        let lower = name.to_lowercase();

        if lower.contains("scan")
            || lower.contains("laser")
            || lower.contains("sensor")
            || lower.contains("camera")
            || lower.contains("imu")
            || lower.contains("odom")
            || lower.contains("tf")
            || lower.contains("joint_states")
            || lower.contains("battery")
            || lower.contains("gps")
            || lower.contains("depth")
        {
            CapabilityCategory::Sensing
        } else if lower.contains("cmd")
            || lower.contains("velocity")
            || lower.contains("twist")
            || lower.contains("trajectory")
            || lower.contains("goal")
            || lower.contains("path")
            || lower.contains("gripper")
            || lower.contains("grasp")
            || lower.contains("move")
        {
            CapabilityCategory::Actuation
        } else if lower.contains("state") || lower.contains("status") || lower.contains("feedback")
        {
            CapabilityCategory::Sensing
        } else {
            CapabilityCategory::Unknown
        }
    }

    fn classify_service(&self, name: &str, _type_name: &str) -> CapabilityCategory {
        let lower = name.to_lowercase();

        if lower.contains("get")
            || lower.contains("query")
            || lower.contains("check")
            || lower.contains("localize")
            || lower.contains("estimate")
        {
            CapabilityCategory::Sensing
        } else if lower.contains("set")
            || lower.contains("control")
            || lower.contains("move")
            || lower.contains("navigate")
            || lower.contains("grasp")
            || lower.contains("execute")
        {
            CapabilityCategory::Actuation
        } else if lower.contains("plan")
            || lower.contains("compute")
            || lower.contains("decide")
            || lower.contains("optimize")
        {
            CapabilityCategory::Decision
        } else {
            CapabilityCategory::Decision
        }
    }

    fn classify_node(
        &self,
        name: &str,
        publishers: &[String],
        subscribers: &[String],
        services: &[String],
    ) -> CapabilityCategory {
        let lower = name.to_lowercase();

        if lower.contains("sensor")
            || lower.contains("camera")
            || lower.contains("laser")
            || lower.contains("imu")
            || lower.contains("gps")
        {
            CapabilityCategory::Sensing
        } else if lower.contains("navigation")
            || lower.contains("planner")
            || lower.contains("controller")
            || lower.contains("driver")
            || lower.contains("actuator")
        {
            CapabilityCategory::Actuation
        } else if lower.contains("fusion")
            || lower.contains("estimator")
            || lower.contains("localization")
        {
            CapabilityCategory::Decision
        } else if publishers.is_empty() && subscribers.is_empty() && services.is_empty() {
            CapabilityCategory::Unknown
        } else {
            CapabilityCategory::Sensing
        }
    }

    fn topic_to_node(
        &self,
        name: &str,
        type_name: &str,
        category: &CapabilityCategory,
        publishers: &[String],
        subscribers: &[String],
    ) -> CapabilityNode {
        let mut node = CapabilityNode::new(name, name);
        node.ros_type = crate::capability_map::graph::RosCapabilityType::Topic;
        node.category = category.clone();
        node.description = format!("ROS topic: {} [{}]", name, type_name);

        for pub_ in publishers {
            node.effects.push(crate::capability_map::graph::Effect {
                capability_id: pub_.clone(),
                relation: "publishes".to_string(),
            });
        }

        for sub in subscribers {
            node.preconditions
                .push(crate::capability_map::graph::Condition {
                    capability_id: sub.clone(),
                    description: format!("Subscribes to {}", sub),
                    required: false,
                });
        }

        node
    }

    fn service_to_node(
        &self,
        name: &str,
        type_name: &str,
        category: &CapabilityCategory,
        providers: &[String],
    ) -> CapabilityNode {
        let mut node = CapabilityNode::new(name, name);
        node.ros_type = crate::capability_map::graph::RosCapabilityType::Service;
        node.category = category.clone();
        node.description = format!("ROS service: {} [{}]", name, type_name);
        node.node = providers.first().cloned().unwrap_or_default();

        for provider in providers {
            node.effects.push(crate::capability_map::graph::Effect {
                capability_id: provider.clone(),
                relation: "provided_by".to_string(),
            });
        }

        node
    }

    fn action_to_node(&self, name: &str, type_name: &str, servers: &[String]) -> CapabilityNode {
        let mut node = CapabilityNode::new(name, name);
        node.ros_type = crate::capability_map::graph::RosCapabilityType::Action;
        node.category = CapabilityCategory::Actuation;
        node.description = format!("ROS action: {} [{}]", name, type_name);
        node.node = servers.first().cloned().unwrap_or_default();

        for server in servers {
            node.effects.push(crate::capability_map::graph::Effect {
                capability_id: server.clone(),
                relation: "server".to_string(),
            });
        }

        node
    }

    fn infer_causal_edges(&self, map: &mut CapabilityMap) {
        let mut edges_to_add = Vec::new();

        for (_topic_name, topic) in &map.topics {
            for pub_node in &topic.publishers {
                for sub_node in &topic.subscribers {
                    let edge = CausalEdge::new(pub_node, sub_node, CausalRelation::Produces);
                    edges_to_add.push(edge);
                }
            }
        }

        for (_service_name, service) in &map.services {
            for provider in &service.provider_nodes {
                let edge = CausalEdge::new(provider, _service_name, CausalRelation::Enables);
                edges_to_add.push(edge);
            }
        }

        for (_action_name, action) in &map.actions {
            for server in &action.server_nodes {
                let edge = CausalEdge::new(_action_name, server, CausalRelation::Triggers);
                edges_to_add.push(edge);
            }
        }

        for node in map.nodes.values() {
            for effect in &node.effects {
                if map.nodes.contains_key(&effect.capability_id) {
                    let edge = CausalEdge::new(
                        &node.id,
                        &effect.capability_id,
                        match effect.relation.as_str() {
                            "publishes" => CausalRelation::Produces,
                            "subscribes_to" => CausalRelation::Consumes,
                            _ => CausalRelation::Triggers,
                        },
                    );
                    edges_to_add.push(edge);
                }
            }
        }

        for edge in edges_to_add {
            map.add_edge(edge);
        }
    }
}

impl Default for CapabilityClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_sensing_topic() {
        let classifier = CapabilityClassifier::new();

        assert_eq!(
            classifier.classify_topic("/scan", "LaserScan"),
            CapabilityCategory::Sensing
        );
        assert_eq!(
            classifier.classify_topic("/odom", "Odometry"),
            CapabilityCategory::Sensing
        );
        assert_eq!(
            classifier.classify_topic("/camera/image", "Image"),
            CapabilityCategory::Sensing
        );
    }

    #[test]
    fn test_classify_actuation_topic() {
        let classifier = CapabilityClassifier::new();

        assert_eq!(
            classifier.classify_topic("/cmd_vel", "Twist"),
            CapabilityCategory::Actuation
        );
        assert_eq!(
            classifier.classify_topic("/gripper/command", "Float64"),
            CapabilityCategory::Actuation
        );
    }

    #[test]
    fn test_classify_service() {
        let classifier = CapabilityClassifier::new();

        assert_eq!(
            classifier.classify_service("/get_state", "GetState"),
            CapabilityCategory::Sensing
        );
        assert_eq!(
            classifier.classify_service("/navigate_to_pose", "NavigateToPose"),
            CapabilityCategory::Actuation
        );
    }
}
