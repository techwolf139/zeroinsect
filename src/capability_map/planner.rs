use crate::capability_map::graph::{CapabilityMap, CapabilityNode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub description: String,
    pub target_capability: Option<String>,
    pub target_category: Option<String>,
    pub constraints: Vec<GoalConstraint>,
}

impl Goal {
    pub fn from_string(s: &str) -> Self {
        let lower = s.to_lowercase();

        let target_capability = if lower.contains("nav")
            || lower.contains("移动")
            || lower.contains("move")
            || lower.contains("go to")
        {
            Some("/cmd_vel".to_string())
        } else if lower.contains("grasp") || lower.contains("抓取") || lower.contains("抓") {
            Some("/gripper".to_string())
        } else {
            None
        };

        let target_category =
            if lower.contains("感知") || lower.contains("sense") || lower.contains("sensor") {
                Some("sensing".to_string())
            } else if lower.contains("移动")
                || lower.contains("move")
                || lower.contains("执行")
                || lower.contains("act")
            {
                Some("actuation".to_string())
            } else if lower.contains("规划") || lower.contains("plan") || lower.contains("决定")
            {
                Some("decision".to_string())
            } else {
                None
            };

        Self {
            description: s.to_string(),
            target_capability,
            target_category,
            constraints: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalConstraint {
    pub capability_id: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    pub name: String,
    pub capability_id: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub preconditions: Vec<String>,
    pub expected_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub goal: String,
    pub steps: Vec<ActionStep>,
    pub total_cost: f32,
    pub found: bool,
}

impl ActionPlan {
    pub fn empty(goal: &str) -> Self {
        Self {
            goal: goal.to_string(),
            steps: Vec::new(),
            total_cost: 0.0,
            found: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SearchState {
    node_id: String,
    depth: usize,
    path: Vec<String>,
}

pub struct ActionPlanner {
    capability_map: CapabilityMap,
    max_depth: usize,
    use_a_star: bool,
}

impl ActionPlanner {
    pub fn new(capability_map: &CapabilityMap) -> Self {
        Self {
            capability_map: capability_map.clone(),
            max_depth: 10,
            use_a_star: false,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_a_star(mut self, enabled: bool) -> Self {
        self.use_a_star = enabled;
        self
    }

    pub fn plan(&self, goal: &Goal) -> ActionPlan {
        if let Some(target) = &goal.target_capability {
            return self.plan_to_capability(target);
        }

        if let Some(category) = &goal.target_category {
            return self.plan_to_category(category);
        }

        ActionPlan::empty(&goal.description)
    }

    fn plan_to_capability(&self, target: &str) -> ActionPlan {
        if self.use_a_star {
            self.a_star_search(target)
        } else {
            self.bfs_search(target)
        }
    }

    fn plan_to_category(&self, category: &str) -> ActionPlan {
        let target_nodes: Vec<&CapabilityNode> = self
            .capability_map
            .nodes
            .values()
            .filter(|n| {
                format!("{:?}", n.category)
                    .to_lowercase()
                    .contains(category)
            })
            .collect();

        if target_nodes.is_empty() {
            return ActionPlan::empty(category);
        }

        let mut best_plan = ActionPlan::empty(category);

        for node in target_nodes {
            let plan = self.plan_to_capability(&node.id);
            if plan.found && (!best_plan.found || plan.total_cost < best_plan.total_cost) {
                best_plan = plan;
            }
        }

        best_plan
    }

    fn bfs_search(&self, target: &str) -> ActionPlan {
        if !self.capability_map.nodes.contains_key(target) {
            return ActionPlan::empty(target);
        }

        let actuation_nodes: Vec<&CapabilityNode> = self
            .capability_map
            .nodes
            .values()
            .filter(|n| n.category == crate::capability_map::graph::CapabilityCategory::Actuation)
            .collect();

        let mut best_plan = ActionPlan::empty(target);

        for start_node in actuation_nodes {
            if let Some(path) = self.capability_map.find_path(&start_node.id, target) {
                let steps = self.path_to_steps(&path);
                let cost = steps.len() as f32;

                if !best_plan.found || cost < best_plan.total_cost {
                    best_plan = ActionPlan {
                        goal: target.to_string(),
                        steps,
                        total_cost: cost,
                        found: true,
                    };
                }
            }
        }

        best_plan
    }

    fn a_star_search(&self, target: &str) -> ActionPlan {
        if !self.capability_map.nodes.contains_key(target) {
            return ActionPlan::empty(target);
        }

        let actuation_nodes: Vec<&CapabilityNode> = self
            .capability_map
            .nodes
            .values()
            .filter(|n| n.category == crate::capability_map::graph::CapabilityCategory::Actuation)
            .collect();

        let mut best_plan = ActionPlan::empty(target);
        let mut open_set: VecDeque<SearchState> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();

        for start_node in actuation_nodes {
            open_set.push_back(SearchState {
                node_id: start_node.id.clone(),
                depth: 0,
                path: vec![start_node.id.clone()],
            });
        }

        while let Some(state) = open_set.pop_front() {
            if state.depth > self.max_depth {
                continue;
            }

            if state.node_id == target {
                let steps = self.path_to_steps(&state.path);
                let cost = state.depth as f32;

                if !best_plan.found || cost < best_plan.total_cost {
                    best_plan = ActionPlan {
                        goal: target.to_string(),
                        steps,
                        total_cost: cost,
                        found: true,
                    };
                }
                continue;
            }

            if visited.contains(&state.node_id) {
                continue;
            }
            visited.insert(state.node_id.clone());

            for edge in self.capability_map.get_edges_from(&state.node_id) {
                let mut new_path = state.path.clone();
                new_path.push(edge.to.clone());

                open_set.push_back(SearchState {
                    node_id: edge.to.clone(),
                    depth: state.depth + 1,
                    path: new_path,
                });
            }
        }

        best_plan
    }

    fn path_to_steps(&self, path: &[String]) -> Vec<ActionStep> {
        path.iter()
            .filter_map(|id| {
                self.capability_map.nodes.get(id).map(|node| {
                    let mut params = HashMap::new();
                    params.insert("target".to_string(), serde_json::json!(node.id));

                    ActionStep {
                        name: node.description.clone(),
                        capability_id: node.id.clone(),
                        parameters: params,
                        preconditions: node
                            .preconditions
                            .iter()
                            .map(|c| c.capability_id.clone())
                            .collect(),
                        expected_effects: node
                            .effects
                            .iter()
                            .map(|e| e.capability_id.clone())
                            .collect(),
                    }
                })
            })
            .collect()
    }

    pub fn find_capabilities_by_goal(&self, goal: &str) -> Vec<String> {
        let goal_lower = goal.to_lowercase();
        let mut results = Vec::new();

        for node in self.capability_map.nodes.values() {
            if node.name.to_lowercase().contains(&goal_lower)
                || node.description.to_lowercase().contains(&goal_lower)
            {
                results.push(node.id.clone());
            }
        }

        results
    }

    pub fn get_action_sequence(&self, from: &str, to: &str) -> Option<Vec<ActionStep>> {
        self.capability_map
            .find_path(from, to)
            .map(|path| self.path_to_steps(&path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_map::graph::{CapabilityCategory, RosCapabilityType};

    fn create_test_map() -> CapabilityMap {
        let mut map = CapabilityMap::new();

        map.add_node(
            CapabilityNode::new("/nav2/navigation", "navigation")
                .with_category(CapabilityCategory::Actuation)
                .with_ros_type(RosCapabilityType::Action),
        );

        map.add_node(
            CapabilityNode::new("/cmd_vel", "cmd_vel")
                .with_category(CapabilityCategory::Actuation)
                .with_ros_type(RosCapabilityType::Topic),
        );

        map.add_node(
            CapabilityNode::new("/scan", "scan")
                .with_category(CapabilityCategory::Sensing)
                .with_ros_type(RosCapabilityType::Topic),
        );

        map.add_node(
            CapabilityNode::new("/nav2/amcl", "amcl")
                .with_category(CapabilityCategory::Decision)
                .with_ros_type(RosCapabilityType::Node),
        );

        map.add_edge(CausalEdge::new(
            "/nav2/navigation",
            "/cmd_vel",
            CausalRelation::Produces,
        ));
        map.add_edge(CausalEdge::new(
            "/scan",
            "/nav2/amcl",
            CausalRelation::Produces,
        ));
        map.add_edge(CausalEdge::new(
            "/nav2/amcl",
            "/nav2/navigation",
            CausalRelation::Enables,
        ));

        map
    }

    #[test]
    fn test_bfs_plan() {
        let map = create_test_map();
        let planner = ActionPlanner::new(&map).with_max_depth(5);

        let goal = Goal::from_string("move to target");
        let plan = planner.plan(&goal);

        assert!(plan.found);
    }

    #[test]
    fn test_find_capabilities() {
        let map = create_test_map();
        let planner = ActionPlanner::new(&map);

        let results = planner.find_capabilities_by_goal("navigation");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_goal_parsing() {
        let goal = Goal::from_string("move to kitchen");
        assert!(goal.target_capability.is_some());
    }
}
