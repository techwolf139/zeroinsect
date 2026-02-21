#![allow(dead_code)]

use crate::tools::robot_tools::RobotTool;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn RobotTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: RobotTool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn RobotTool>> {
        self.tools.get(name)
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
