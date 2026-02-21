#![allow(dead_code)]

use crate::tools::robot_tools::RobotTool;

pub struct NavigateTool;

impl NavigateTool {
    pub fn new() -> Self {
        Self
    }
}

impl RobotTool for NavigateTool {
    fn name(&self) -> &str {
        "navigate_to_pose"
    }

    fn description(&self) -> &str {
        "Navigate robot to specified pose"
    }

    fn supported_topics(&self) -> Vec<&'static str> {
        vec!["/navigate_to_pose", "/goal_pose"]
    }
}

pub struct GraspTool;

impl GraspTool {
    pub fn new() -> Self {
        Self
    }
}

impl RobotTool for GraspTool {
    fn name(&self) -> &str {
        "grasp_object"
    }

    fn description(&self) -> &str {
        "Execute grasp动作"
    }

    fn supported_topics(&self) -> Vec<&'static str> {
        vec!["/grasp", "/gripper_cmd"]
    }
}
