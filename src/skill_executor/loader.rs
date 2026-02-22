use crate::capability_map::graph::{CapabilityCategory, CapabilityMap};
use crate::skill_executor::registry::{
    CapabilityRef, CapabilityRefType, Condition, Effect, ParameterDef, ReturnDef, RosSkill,
    SkillCategory,
};
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

pub struct SkillLoader;

impl SkillLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load_from_yaml(&self, path: &Path) -> Result<RosSkill> {
        let content = std::fs::read_to_string(path)?;
        self.parse_yaml(&content)
    }

    pub fn load_from_json(&self, path: &Path) -> Result<RosSkill> {
        let content = std::fs::read_to_string(path)?;
        let skill: RosSkill = serde_json::from_str(&content)?;
        Ok(skill)
    }

    pub fn parse_yaml(&self, content: &str) -> Result<RosSkill> {
        if let Ok(skill) = serde_yaml::from_str::<SkillYaml>(content) {
            return Ok(self.convert_from_yaml(skill));
        }
        Err(anyhow::anyhow!("Failed to parse YAML"))
    }

    fn convert_from_yaml(&self, yaml: SkillYaml) -> RosSkill {
        let mut skill = RosSkill::new(&yaml.skill.name);

        skill.metadata.version = yaml.skill.version;
        skill.metadata.description = yaml.skill.description;
        skill.metadata.category = match yaml.skill.category.to_lowercase().as_str() {
            "motion" => SkillCategory::Motion,
            "perception" => SkillCategory::Perception,
            "manipulation" => SkillCategory::Manipulation,
            "navigation" => SkillCategory::Navigation,
            "communication" => SkillCategory::Communication,
            "planning" => SkillCategory::Planning,
            _ => SkillCategory::Custom(yaml.skill.category),
        };
        skill.metadata.tags = yaml.skill.tags;

        if let Some(cap) = yaml.capability {
            skill.capability = CapabilityRef {
                cap_type: match cap.cap_type.as_str() {
                    "node" => CapabilityRefType::Node,
                    "topic" => CapabilityRefType::Topic,
                    "service" => CapabilityRefType::Service,
                    "action" => CapabilityRefType::Action,
                    _ => CapabilityRefType::Action,
                },
                ros_name: cap.ros_name,
                ros_type: cap.ros_type,
                node: cap.node,
            };
        }

        for param in yaml.parameters.unwrap_or_default() {
            skill.parameters.push(ParameterDef {
                name: param.name,
                param_type: param.param_type,
                required: param.required,
                default: param.default,
                description: param.description,
            });
        }

        for ret in yaml.returns.unwrap_or_default() {
            skill.returns.push(ReturnDef {
                name: ret.name,
                return_type: ret.return_type,
                description: ret.description,
            });
        }

        for pre in yaml.preconditions.unwrap_or_default() {
            skill.preconditions.push(Condition {
                capability_id: pre.capability,
                cap_type: match pre.cap_type.as_str() {
                    "node" => CapabilityRefType::Node,
                    "topic" => CapabilityRefType::Topic,
                    "service" => CapabilityRefType::Service,
                    "action" => CapabilityRefType::Action,
                    _ => CapabilityRefType::Topic,
                },
                description: pre.description,
            });
        }

        for eff in yaml.effects.unwrap_or_default() {
            skill.effects.push(Effect {
                capability_id: eff.capability,
                cap_type: match eff.cap_type.as_str() {
                    "node" => CapabilityRefType::Node,
                    "topic" => CapabilityRefType::Topic,
                    "service" => CapabilityRefType::Service,
                    "action" => CapabilityRefType::Action,
                    _ => CapabilityRefType::Topic,
                },
                description: eff.description,
            });
        }

        skill
    }

    pub fn generate_from_capability_map(&self, cap_map: &CapabilityMap) -> Vec<RosSkill> {
        let mut skills = Vec::new();

        for (name, topic) in &cap_map.topics {
            let category = self.classify_capability(&topic.category);
            let name_clean = name.trim_start_matches('/').replace('/', "_");
            let skill_name = format!("ros2_topic_{}", name_clean);

            let mut skill = RosSkill::new(&skill_name)
                .with_category(category)
                .with_description(&format!("ROS topic: {} [{}]", name, topic.type_name))
                .with_capability(CapabilityRef {
                    cap_type: CapabilityRefType::Topic,
                    ros_name: name.clone(),
                    ros_type: topic.type_name.clone(),
                    node: topic.publishers.first().cloned().unwrap_or_default(),
                })
                .with_return(ReturnDef {
                    name: "data".to_string(),
                    return_type: topic.type_name.clone(),
                    description: "Topic message data".to_string(),
                });

            if !topic.publishers.is_empty() {
                skill = skill.with_effect(Effect {
                    capability_id: name.clone(),
                    cap_type: CapabilityRefType::Topic,
                    description: "Publishes data".to_string(),
                });
            }

            skills.push(skill);
        }

        for (name, service) in &cap_map.services {
            let category = self.classify_capability(&service.category);
            let name_clean = name.trim_start_matches('/').replace('/', "_");
            let skill_name = format!("ros2_service_{}", name_clean);

            let skill = RosSkill::new(&skill_name)
                .with_category(category)
                .with_description(&format!("ROS service: {} [{}]", name, service.type_name))
                .with_capability(CapabilityRef {
                    cap_type: CapabilityRefType::Service,
                    ros_name: name.clone(),
                    ros_type: service.type_name.clone(),
                    node: service.provider_nodes.first().cloned().unwrap_or_default(),
                })
                .with_return(ReturnDef {
                    name: "response".to_string(),
                    return_type: service.type_name.clone(),
                    description: "Service response".to_string(),
                })
                .with_return(ReturnDef {
                    name: "success".to_string(),
                    return_type: "bool".to_string(),
                    description: "Whether call succeeded".to_string(),
                });

            skills.push(skill);
        }

        for (name, action) in &cap_map.actions {
            let category = self.classify_capability(&action.category);
            let name_clean = name.trim_start_matches('/').replace('/', "_");
            let skill_name = format!("ros2_action_{}", name_clean);

            let skill = RosSkill::new(&skill_name)
                .with_category(category)
                .with_description(&format!("ROS action: {} [{}]", name, action.type_name))
                .with_capability(CapabilityRef {
                    cap_type: CapabilityRefType::Action,
                    ros_name: name.clone(),
                    ros_type: action.type_name.clone(),
                    node: action.server_nodes.first().cloned().unwrap_or_default(),
                })
                .with_parameter(ParameterDef {
                    name: "goal".to_string(),
                    param_type: action.type_name.clone(),
                    required: true,
                    default: None,
                    description: "Action goal".to_string(),
                })
                .with_return(ReturnDef {
                    name: "result".to_string(),
                    return_type: format!("{}Result", action.type_name),
                    description: "Action result".to_string(),
                })
                .with_return(ReturnDef {
                    name: "success".to_string(),
                    return_type: "bool".to_string(),
                    description: "Whether goal succeeded".to_string(),
                });

            skills.push(skill);
        }

        for (name, node_info) in &cap_map.nodes {
            let category = self.classify_capability(&node_info.category);
            let name_clean = name.trim_start_matches('/').replace('/', "_");
            let skill_name = format!("ros2_node_{}", name_clean);

            let mut skill = RosSkill::new(&skill_name)
                .with_category(category)
                .with_description(&node_info.description)
                .with_capability(CapabilityRef {
                    cap_type: CapabilityRefType::Node,
                    ros_name: name.clone(),
                    ros_type: "rcl_node".to_string(),
                    node: name.clone(),
                });

            for effect in &node_info.effects {
                skill = skill.with_effect(Effect {
                    capability_id: effect.capability_id.clone(),
                    cap_type: CapabilityRefType::Topic,
                    description: effect.relation.clone(),
                });
            }

            for precond in &node_info.preconditions {
                skill = skill.with_precondition(Condition {
                    capability_id: precond.capability_id.clone(),
                    cap_type: CapabilityRefType::Topic,
                    description: precond.description.clone(),
                });
            }

            skills.push(skill);
        }

        skills
    }

    fn classify_capability(&self, cat: &CapabilityCategory) -> SkillCategory {
        match cat {
            CapabilityCategory::Sensing => SkillCategory::Perception,
            CapabilityCategory::Decision => SkillCategory::Planning,
            CapabilityCategory::Actuation => SkillCategory::Motion,
            CapabilityCategory::Unknown => SkillCategory::Custom("unknown".to_string()),
        }
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct SkillYaml {
    skill: SkillMetaYaml,
    capability: Option<CapabilityYaml>,
    parameters: Option<Vec<ParameterYaml>>,
    returns: Option<Vec<ReturnYaml>>,
    preconditions: Option<Vec<PreconditionYaml>>,
    effects: Option<Vec<EffectYaml>>,
}

#[derive(Debug, Deserialize)]
struct SkillMetaYaml {
    name: String,
    version: String,
    description: String,
    category: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CapabilityYaml {
    #[serde(rename = "type")]
    cap_type: String,
    ros_name: String,
    ros_type: String,
    node: String,
}

#[derive(Debug, Deserialize)]
struct ParameterYaml {
    name: String,
    #[serde(rename = "type")]
    param_type: String,
    required: bool,
    default: Option<serde_json::Value>,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ReturnYaml {
    name: String,
    #[serde(rename = "type")]
    return_type: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct PreconditionYaml {
    capability: String,
    #[serde(rename = "type")]
    cap_type: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct EffectYaml {
    capability: String,
    #[serde(rename = "type")]
    cap_type: String,
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_from_capability_map() {
        let mut cap_map = CapabilityMap::new();

        cap_map.add_topic(crate::capability_map::graph::TopicCapability {
            name: "/scan".to_string(),
            type_name: "sensor_msgs/LaserScan".to_string(),
            publishers: vec!["/rplidar".to_string()],
            subscribers: vec![],
            category: CapabilityCategory::Sensing,
        });

        let loader = SkillLoader::new();
        let skills = loader.generate_from_capability_map(&cap_map);

        assert!(!skills.is_empty());
        let nav_skill = skills
            .iter()
            .find(|s| s.metadata.name.contains("scan"))
            .unwrap();
        assert_eq!(nav_skill.metadata.category, SkillCategory::Perception);
    }
}
