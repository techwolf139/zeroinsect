use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillCategory {
    Motion,
    Perception,
    Manipulation,
    Navigation,
    Communication,
    Planning,
    Custom(String),
}

impl Hash for SkillCategory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SkillCategory::Motion => 0.hash(state),
            SkillCategory::Perception => 1.hash(state),
            SkillCategory::Manipulation => 2.hash(state),
            SkillCategory::Navigation => 3.hash(state),
            SkillCategory::Communication => 4.hash(state),
            SkillCategory::Planning => 5.hash(state),
            SkillCategory::Custom(s) => (6, s).hash(state),
        }
    }
}

impl Default for SkillCategory {
    fn default() -> Self {
        SkillCategory::Custom("unknown".to_string())
    }
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillCategory::Motion => write!(f, "motion"),
            SkillCategory::Perception => write!(f, "perception"),
            SkillCategory::Manipulation => write!(f, "manipulation"),
            SkillCategory::Navigation => write!(f, "navigation"),
            SkillCategory::Communication => write!(f, "communication"),
            SkillCategory::Planning => write!(f, "planning"),
            SkillCategory::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: SkillCategory,
    pub tags: Vec<String>,
}

impl SkillMetadata {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            category: SkillCategory::default(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityRefType {
    Node,
    Topic,
    Service,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRef {
    pub cap_type: CapabilityRefType,
    pub ros_name: String,
    pub ros_type: String,
    pub node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnDef {
    pub name: String,
    pub return_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub capability_id: String,
    pub cap_type: CapabilityRefType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub capability_id: String,
    pub cap_type: CapabilityRefType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosSkill {
    pub metadata: SkillMetadata,
    pub capability: CapabilityRef,
    pub parameters: Vec<ParameterDef>,
    pub returns: Vec<ReturnDef>,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
}

impl RosSkill {
    pub fn new(name: &str) -> Self {
        Self {
            metadata: SkillMetadata::new(name),
            capability: CapabilityRef {
                cap_type: CapabilityRefType::Action,
                ros_name: String::new(),
                ros_type: String::new(),
                node: String::new(),
            },
            parameters: Vec::new(),
            returns: Vec::new(),
            preconditions: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.metadata.description = desc.to_string();
        self
    }

    pub fn with_category(mut self, category: SkillCategory) -> Self {
        self.metadata.category = category;
        self
    }

    pub fn with_capability(mut self, cap: CapabilityRef) -> Self {
        self.capability = cap;
        self
    }

    pub fn with_parameter(mut self, param: ParameterDef) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn with_return(mut self, ret: ReturnDef) -> Self {
        self.returns.push(ret);
        self
    }

    pub fn with_precondition(mut self, cond: Condition) -> Self {
        self.preconditions.push(cond);
        self
    }

    pub fn with_effect(mut self, eff: Effect) -> Self {
        self.effects.push(eff);
        self
    }

    pub fn get_param(&self, name: &str) -> Option<&ParameterDef> {
        self.parameters.iter().find(|p| p.name == name)
    }
}

pub struct SkillRegistry {
    skills: HashMap<String, RosSkill>,
    category_index: HashMap<SkillCategory, Vec<String>>,
    tag_index: HashMap<String, Vec<String>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            category_index: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: RosSkill) {
        let name = skill.metadata.name.clone();

        self.skills.insert(name.clone(), skill.clone());

        let category = skill.metadata.category.clone();
        self.category_index
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());

        for tag in &skill.metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());
        }
    }

    pub fn get(&self, name: &str) -> Option<&RosSkill> {
        self.skills.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut RosSkill> {
        self.skills.get_mut(name)
    }

    pub fn list_all(&self) -> Vec<&RosSkill> {
        self.skills.values().collect()
    }

    pub fn list_by_category(&self, category: &SkillCategory) -> Vec<&RosSkill> {
        self.category_index
            .get(category)
            .map(|names| names.iter().filter_map(|n| self.skills.get(n)).collect())
            .unwrap_or_default()
    }

    pub fn list_by_tag(&self, tag: &str) -> Vec<&RosSkill> {
        self.tag_index
            .get(tag)
            .map(|names| names.iter().filter_map(|n| self.skills.get(n)).collect())
            .unwrap_or_default()
    }

    pub fn search(&self, query: &str) -> Vec<&RosSkill> {
        let query_lower = query.to_lowercase();
        self.skills
            .values()
            .filter(|s| {
                s.metadata.name.to_lowercase().contains(&query_lower)
                    || s.metadata.description.to_lowercase().contains(&query_lower)
                    || s.metadata
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    pub fn remove(&mut self, name: &str) -> Option<RosSkill> {
        if let Some(skill) = self.skills.remove(name) {
            for (category, names) in &mut self.category_index {
                names.retain(|n| n != name);
            }
            for (tag, names) in &mut self.tag_index {
                names.retain(|n| n != name);
            }
            Some(skill)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.skills.clear();
        self.category_index.clear();
        self.tag_index.clear();
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let skill = RosSkill::new("test_skill")
            .with_description("Test skill description")
            .with_category(SkillCategory::Navigation);

        assert_eq!(skill.metadata.name, "test_skill");
        assert_eq!(skill.metadata.description, "Test skill description");
    }

    #[test]
    fn test_registry_register_get() {
        let mut registry = SkillRegistry::new();

        let skill = RosSkill::new("nav_skill").with_category(SkillCategory::Navigation);
        registry.register(skill);

        assert!(registry.get("nav_skill").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_by_category() {
        let mut registry = SkillRegistry::new();

        registry.register(RosSkill::new("nav1").with_category(SkillCategory::Navigation));
        registry.register(RosSkill::new("nav2").with_category(SkillCategory::Navigation));
        registry.register(RosSkill::new("perc1").with_category(SkillCategory::Perception));

        let nav_skills = registry.list_by_category(&SkillCategory::Navigation);
        assert_eq!(nav_skills.len(), 2);
    }

    #[test]
    fn test_registry_search() {
        let mut registry = SkillRegistry::new();

        registry.register(
            RosSkill::new("navigate_to_pose").with_description("Navigate robot to target pose"),
        );
        registry.register(RosSkill::new("grasp_object").with_description("Grasp an object"));

        let results = registry.search("navigate");
        assert_eq!(results.len(), 1);
    }
}
