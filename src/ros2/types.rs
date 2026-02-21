use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwistCommand {
    pub linear: Vector3,
    pub angular: Vector3,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    pub position: Vector3,
    pub orientation: Quaternion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotState {
    pub pose: Pose,
    pub velocity: TwistCommand,
    pub battery_level: f32,
    pub timestamp: i64,
}

impl Default for RobotState {
    fn default() -> Self {
        Self {
            pose: Pose {
                position: Vector3::default(),
                orientation: Quaternion::default(),
            },
            velocity: TwistCommand {
                linear: Vector3::default(),
                angular: Vector3::default(),
            },
            battery_level: 100.0,
            timestamp: 0,
        }
    }
}
