#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    pub points: Vec<Point2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub max_linear_velocity: f64,
    pub max_angular_velocity: f64,
    pub max_force: f64,
    pub workspace_boundary: Vec<Point2D>,
    pub forbidden_zones: Vec<Polygon>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationResult {
    Approved,
    Rejected(String),
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_linear_velocity: 1.0,
            max_angular_velocity: 1.0,
            max_force: 100.0,
            workspace_boundary: vec![
                Point2D { x: -10.0, y: -10.0 },
                Point2D { x: 10.0, y: -10.0 },
                Point2D { x: 10.0, y: 10.0 },
                Point2D { x: -10.0, y: 10.0 },
            ],
            forbidden_zones: vec![],
        }
    }
}

impl SafetyConfig {
    pub fn validate_velocity(&self, linear: f64) -> ValidationResult {
        if linear.abs() > self.max_linear_velocity {
            ValidationResult::Rejected(format!(
                "Exceeds max velocity {} m/s",
                self.max_linear_velocity
            ))
        } else {
            ValidationResult::Approved
        }
    }

    pub fn validate_workspace(&self, x: f64, y: f64) -> ValidationResult {
        if x < -10.0 || x > 10.0 || y < -10.0 || y > 10.0 {
            ValidationResult::Rejected("Outside workspace boundary".to_string())
        } else {
            ValidationResult::Approved
        }
    }
}
