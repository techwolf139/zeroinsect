use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscovery {
    pub name: String,
    pub type_name: String,
    pub provider_node: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDiscovery {
    pub name: String,
    pub type_name: String,
    pub provider_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCard {
    pub name: String,
    pub type_: CapabilityType,
    pub inputs: Vec<FieldSchema>,
    pub outputs: Vec<FieldSchema>,
    pub description: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CapabilityType {
    Topic,
    Service,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

impl CapabilityCard {
    pub fn new(name: &str, type_: CapabilityType) -> Self {
        Self {
            name: name.to_string(),
            type_,
            inputs: Vec::new(),
            outputs: Vec::new(),
            description: String::new(),
            confidence: 1.0,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<FieldSchema>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<FieldSchema>) -> Self {
        self.outputs = outputs;
        self
    }
}
