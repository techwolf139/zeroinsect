#![allow(dead_code)]

use crate::ros2::types::TwistCommand;
use crate::safety::config::{SafetyConfig, ValidationResult};

pub struct SafetyGateway {
    config: SafetyConfig,
}

impl SafetyGateway {
    pub fn new(config: SafetyConfig) -> Self {
        Self { config }
    }

    pub fn from_default() -> Self {
        Self {
            config: SafetyConfig::default(),
        }
    }

    pub fn validate_command(&self, cmd: &TwistCommand) -> ValidationResult {
        let linear_vel = (cmd.linear.x.powi(2) + cmd.linear.y.powi(2)).sqrt();

        if let ValidationResult::Rejected(reason) = self.config.validate_velocity(linear_vel) {
            return ValidationResult::Rejected(reason);
        }

        if let ValidationResult::Rejected(reason) = self.config.validate_velocity(cmd.angular.z) {
            return ValidationResult::Rejected(reason);
        }

        ValidationResult::Approved
    }

    pub fn get_config(&self) -> &SafetyConfig {
        &self.config
    }
}
