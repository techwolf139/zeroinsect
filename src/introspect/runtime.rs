#![allow(dead_code)]

use crate::introspect::types::*;

pub struct RosRuntimeIntrospector {
    #[cfg(feature = "ros2")]
    node: Option<r2r::Node>,
}

impl RosRuntimeIntrospector {
    pub fn new() -> Self {
        #[cfg(feature = "ros2")]
        {
            let node = r2r::Node::builder()
                .name("zeroinsect_introspector")
                .build()
                .ok();
            Self { node }
        }
        #[cfg(not(feature = "ros2"))]
        {
            Self {}
        }
    }

    pub fn is_connected(&self) -> bool {
        #[cfg(feature = "ros2")]
        return self.node.is_some();
        #[cfg(not(feature = "ros2"))]
        false
    }
}

impl Default for RosRuntimeIntrospector {
    fn default() -> Self {
        Self::new()
    }
}
