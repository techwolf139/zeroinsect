use crate::skill_executor::registry::{CapabilityRefType, RosSkill, SkillRegistry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRequest {
    pub skill_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub message: String,
    pub execution_time_ms: u64,
}

impl SkillResponse {
    pub fn success(result: serde_json::Value) -> Self {
        Self {
            success: true,
            result,
            message: String::new(),
            execution_time_ms: 0,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            result: serde_json::Value::Null,
            message: message.to_string(),
            execution_time_ms: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub struct SkillExecutor {
    registry: Arc<SkillRegistry>,
    mock_mode: bool,
}

impl SkillExecutor {
    pub fn new(registry: Arc<SkillRegistry>) -> Self {
        Self {
            registry,
            mock_mode: true,
        }
    }

    pub fn with_mock_mode(mut self, enabled: bool) -> Self {
        self.mock_mode = enabled;
        self
    }

    pub fn execute(&self, request: SkillRequest) -> SkillResponse {
        let start = std::time::Instant::now();

        let skill = match self.registry.get(&request.skill_name) {
            Some(s) => s,
            None => {
                return SkillResponse::error(&format!("Skill not found: {}", request.skill_name));
            }
        };

        if let Err(e) = self.validate_parameters(skill, &request.parameters) {
            return SkillResponse::error(&format!("Parameter validation failed: {}", e.message));
        }

        let result = match skill.capability.cap_type {
            CapabilityRefType::Topic => self.execute_topic(skill, &request.parameters),
            CapabilityRefType::Service => self.execute_service(skill, &request.parameters),
            CapabilityRefType::Action => self.execute_action(skill, &request.parameters),
            CapabilityRefType::Node => self.execute_node(skill, &request.parameters),
        };

        let elapsed = start.elapsed().as_millis() as u64;

        SkillResponse {
            success: true,
            result,
            message: format!("Executed {} successfully", skill.metadata.name),
            execution_time_ms: elapsed,
        }
    }

    fn validate_parameters(
        &self,
        skill: &RosSkill,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ValidationError> {
        for param_def in &skill.parameters {
            if param_def.required && !params.contains_key(&param_def.name) {
                if param_def.default.is_none() {
                    return Err(ValidationError {
                        field: param_def.name.clone(),
                        message: format!("Required parameter missing: {}", param_def.name),
                    });
                }
            }

            if let Some(value) = params.get(&param_def.name) {
                if !self.validate_param_type(value, &param_def.param_type) {
                    return Err(ValidationError {
                        field: param_def.name.clone(),
                        message: format!(
                            "Invalid type for {}: expected {}",
                            param_def.name, param_def.param_type
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_param_type(&self, value: &serde_json::Value, expected_type: &str) -> bool {
        match expected_type {
            "bool" => value.is_boolean(),
            "int" | "i32" | "i64" => value.is_i64() || value.is_u64(),
            "float" | "f32" | "f64" | "double" => value.is_number(),
            "string" | "str" => value.is_string(),
            "pose" | "point" | "vector3" => value.is_object(),
            _ => true,
        }
    }

    fn execute_topic(
        &self,
        skill: &RosSkill,
        params: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        if self.mock_mode {
            return serde_json::json!({
                "status": "mock",
                "topic": skill.capability.ros_name,
                "type": skill.capability.ros_type,
                "published": true,
                "params": params
            });
        }

        serde_json::json!({
            "status": "published",
            "topic": skill.capability.ros_name
        })
    }

    fn execute_service(
        &self,
        skill: &RosSkill,
        params: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        if self.mock_mode {
            return serde_json::json!({
                "status": "mock",
                "service": skill.capability.ros_name,
                "type": skill.capability.ros_type,
                "response": {
                    "success": true,
                    "message": "Mock service call successful"
                },
                "params": params
            });
        }

        serde_json::json!({
            "status": "called",
            "service": skill.capability.ros_name,
            "response": {
                "success": true
            }
        })
    }

    fn execute_action(
        &self,
        skill: &RosSkill,
        params: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        if self.mock_mode {
            return serde_json::json!({
                "status": "mock",
                "action": skill.capability.ros_name,
                "type": skill.capability.ros_type,
                "goal_status": "SUCCEEDED",
                "result": {
                    "success": true,
                    "message": "Mock action completed"
                },
                "params": params
            });
        }

        serde_json::json!({
            "status": "goal_sent",
            "action": skill.capability.ros_name,
            "goal_status": "ACCEPTED"
        })
    }

    fn execute_node(
        &self,
        skill: &RosSkill,
        params: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        if self.mock_mode {
            return serde_json::json!({
                "status": "mock",
                "node": skill.capability.ros_name,
                "executed": true,
                "params": params
            });
        }

        serde_json::json!({
            "status": "executed",
            "node": skill.capability.ros_name
        })
    }

    pub fn check_preconditions(
        &self,
        skill: &RosSkill,
        _available_capabilities: &[String],
    ) -> Result<(), String> {
        if skill.preconditions.is_empty() {
            return Ok(());
        }

        for precond in &skill.preconditions {
            if !_available_capabilities.contains(&precond.capability_id) {
                return Err(format!(
                    "Precondition not met: {} ({})",
                    precond.capability_id, precond.description
                ));
            }
        }

        Ok(())
    }

    pub fn list_skills(&self) -> Vec<String> {
        self.registry
            .list_all()
            .iter()
            .map(|s| s.metadata.name.clone())
            .collect()
    }

    pub fn get_skill(&self, name: &str) -> Option<RosSkill> {
        self.registry.get(name).cloned()
    }
}

impl Default for SkillExecutor {
    fn default() -> Self {
        Self::new(Arc::new(SkillRegistry::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_executor::registry::{CapabilityRef, CapabilityRefType, ParameterDef};

    fn create_test_executor() -> (SkillExecutor, Arc<SkillRegistry>) {
        let mut registry = SkillRegistry::new();

        let skill = RosSkill::new("test_topic")
            .with_capability(CapabilityRef {
                cap_type: CapabilityRefType::Topic,
                ros_name: "/test/topic".to_string(),
                ros_type: "std_msgs/String".to_string(),
                node: "/test/node".to_string(),
            })
            .with_parameter(ParameterDef {
                name: "data".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
                description: "Test data".to_string(),
            });

        registry.register(skill);

        let executor = SkillExecutor::new(Arc::new(registry));
        (executor, Arc::new(SkillRegistry::new()))
    }

    #[test]
    fn test_execute_success() {
        let (executor, _) = create_test_executor();

        let mut params = HashMap::new();
        params.insert("data".to_string(), serde_json::json!("test_value"));

        let request = SkillRequest {
            skill_name: "test_topic".to_string(),
            parameters: params,
            timeout_ms: None,
        };

        let response = executor.execute(request);

        assert!(response.success);
    }

    #[test]
    fn test_execute_not_found() {
        let (executor, _) = create_test_executor();

        let request = SkillRequest {
            skill_name: "nonexistent".to_string(),
            parameters: HashMap::new(),
            timeout_ms: None,
        };

        let response = executor.execute(request);

        assert!(!response.success);
        assert!(response.message.contains("not found"));
    }

    #[test]
    fn test_validate_required_param() {
        let (executor, _) = create_test_executor();

        let skill = executor.get_skill("test_topic").unwrap();
        let params = HashMap::new();

        let result = executor.validate_parameters(&skill, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_optional_param() {
        let mut registry = SkillRegistry::new();

        let skill = RosSkill::new("optional_param_skill")
            .with_capability(CapabilityRef {
                cap_type: CapabilityRefType::Topic,
                ros_name: "/test/optional".to_string(),
                ros_type: "std_msgs/String".to_string(),
                node: "/test/node".to_string(),
            })
            .with_parameter(ParameterDef {
                name: "optional_data".to_string(),
                param_type: "string".to_string(),
                required: false,
                default: Some(serde_json::json!("default")),
                description: "Optional data".to_string(),
            });

        registry.register(skill);

        let executor = SkillExecutor::new(Arc::new(registry));

        let skill = executor.get_skill("optional_param_skill").unwrap();
        let params = HashMap::new();

        let result = executor.validate_parameters(&skill, &params);
        assert!(result.is_ok());
    }
}
