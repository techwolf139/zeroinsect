#![allow(dead_code)]

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zeroinsect")]
#[command(about = "ROS2 execution layer with ZeroClaw integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(long)]
        workspace: Option<String>,
    },
    Introspect {
        #[arg(long, default_value = "json")]
        format: String,
    },
    Tools,
    ValidateSafety {
        #[arg(long)]
        config: Option<String>,
    },
    Discover {
        #[arg(long)]
        workspace: Option<String>,
    },
    Capability {
        #[command(subcommand)]
        subcommand: CapabilityCommands,
    },
}

#[derive(Subcommand)]
enum CapabilityCommands {
    Map {
        #[arg(long)]
        category: Option<String>,

        #[arg(long)]
        node: Option<String>,

        #[arg(long, default_value = "json")]
        format: String,
    },
    Plan {
        #[arg(long)]
        goal: String,

        #[arg(long)]
        from: Option<String>,
    },
    Graph {
        #[arg(long)]
        from: Option<String>,

        #[arg(long)]
        target: Option<String>,
    },
    List {
        #[arg(long)]
        category: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { workspace } => {
            println!("Starting ZeroInsect...");
            if let Some(ws) = workspace {
                println!("Workspace: {}", ws);
            }
        }
        Commands::Introspect { format } => {
            println!("Introspecting ROS system... (format: {})", format);
        }
        Commands::Tools => {
            println!("Available tools:");
            println!("  - get_robot_state: Get current robot state");
            println!("  - navigate_to_pose: Navigate robot to pose");
            println!("  - grasp_object: Execute grasp action");
        }
        Commands::ValidateSafety { config } => {
            println!("Validating safety config...");
            if let Some(c) = config {
                println!("Config file: {}", c);
            } else {
                println!("Using default safety config");
            }
        }
        Commands::Discover { workspace } => {
            println!("Discovering ROS capabilities...");
            if let Some(ws) = workspace {
                println!("Workspace: {}", ws);
            }
        }
        Commands::Capability { subcommand } => {
            handle_capability_command(subcommand);
        }
    }
}

fn handle_capability_command(cmd: CapabilityCommands) {
    use zeroinsect::capability_map::graph::CapabilityCategory;
    use zeroinsect::capability_map::{ActionPlanner, CapabilityClassifier, Goal};
    use zeroinsect::introspect::runtime::RosRuntimeIntrospector;

    match cmd {
        CapabilityCommands::Map {
            category,
            node,
            format,
        } => {
            println!("Building capability map...");

            let mut introspector = RosRuntimeIntrospector::new();
            let snapshot = match introspector.capture_snapshot() {
                Ok(s) => s,
                Err(e) => {
                    println!("Error capturing snapshot: {}", e);
                    return;
                }
            };

            let classifier = CapabilityClassifier::new();
            let cap_map = classifier.classify(&snapshot);

            println!("\n=== Capability Map ===");
            println!("Total nodes: {}", cap_map.node_count());
            println!("Total edges: {}", cap_map.edge_count());

            if let Some(cat) = category {
                let cat_enum = match cat.to_lowercase().as_str() {
                    "sensing" => Some(CapabilityCategory::Sensing),
                    "decision" => Some(CapabilityCategory::Decision),
                    "actuation" => Some(CapabilityCategory::Actuation),
                    _ => None,
                };

                if let Some(c) = cat_enum {
                    println!("\n--- {} Capabilities ---", cat);
                    for n in cap_map.nodes_by_category(&c) {
                        println!("  [{}] {}", format!("{:?}", n.ros_type), n.id);
                    }
                }
            } else {
                println!("\n--- Sensing ---");
                for n in cap_map.nodes_by_category(&CapabilityCategory::Sensing) {
                    println!("  [topic] {}", n.id);
                }

                println!("\n--- Decision ---");
                for n in cap_map.nodes_by_category(&CapabilityCategory::Decision) {
                    println!("  [service] {}", n.id);
                }

                println!("\n--- Actuation ---");
                for n in cap_map.nodes_by_category(&CapabilityCategory::Actuation) {
                    println!("  [{}] {}", format!("{:?}", n.ros_type), n.id);
                }
            }

            if let Some(n) = node {
                println!("\n--- Node: {} ---", n);
                if let Some(node_info) = cap_map.get_node(&n) {
                    println!("  Category: {:?}", node_info.category);
                    println!("  Type: {:?}", node_info.ros_type);
                    println!("  Description: {}", node_info.description);

                    if !node_info.preconditions.is_empty() {
                        println!("  Preconditions:");
                        for p in &node_info.preconditions {
                            println!("    - {}", p.capability_id);
                        }
                    }

                    if !node_info.effects.is_empty() {
                        println!("  Effects:");
                        for e in &node_info.effects {
                            println!("    - {} ({})", e.capability_id, e.relation);
                        }
                    }
                }
            }
        }

        CapabilityCommands::Plan { goal, from } => {
            println!("Planning action sequence for: {}", goal);

            let mut introspector = RosRuntimeIntrospector::new();
            let snapshot = match introspector.capture_snapshot() {
                Ok(s) => s,
                Err(e) => {
                    println!("Error capturing snapshot: {}", e);
                    return;
                }
            };

            let classifier = CapabilityClassifier::new();
            let cap_map = classifier.classify(&snapshot);

            let goal_obj = Goal::from_string(&goal);
            let planner = ActionPlanner::new(&cap_map).with_max_depth(10);
            let plan = planner.plan(&goal_obj);

            if plan.found {
                println!("\n=== Action Plan ===");
                println!("Goal: {}", plan.goal);
                println!("Total steps: {}", plan.steps.len());
                println!("Total cost: {}", plan.total_cost);

                for (i, step) in plan.steps.iter().enumerate() {
                    println!("\nStep {}: {}", i + 1, step.name);
                    println!("  Capability: {}", step.capability_id);
                    if !step.preconditions.is_empty() {
                        println!("  Preconditions: {:?}", step.preconditions);
                    }
                }
            } else {
                println!("\nNo plan found for goal: {}", goal);

                let suggestions = planner.find_capabilities_by_goal(&goal);
                if !suggestions.is_empty() {
                    println!("\nDid you mean:");
                    for s in suggestions.iter().take(5) {
                        println!("  - {}", s);
                    }
                }
            }
        }

        CapabilityCommands::Graph { from, target } => {
            println!("Querying causal graph...");

            let mut introspector = RosRuntimeIntrospector::new();
            let snapshot = match introspector.capture_snapshot() {
                Ok(s) => s,
                Err(e) => {
                    println!("Error capturing snapshot: {}", e);
                    return;
                }
            };

            let classifier = CapabilityClassifier::new();
            let cap_map = classifier.classify(&snapshot);

            if let Some(ref f) = from {
                println!("\n=== Edges from: {} ===", f);
                for edge in cap_map.get_edges_from(f) {
                    println!("  {} -> {} ({:?})", edge.from, edge.to, edge.relation);
                }
            }

            if let Some(ref t) = target {
                println!("\n=== Edges to: {} ===", t);
                for edge in cap_map.get_edges_to(t) {
                    println!("  {} -> {} ({:?})", edge.from, edge.to, edge.relation);
                }
            }

            if from.is_none() && target.is_none() {
                println!("\n=== All Causal Edges ===");
                for edge in cap_map.edges.iter().take(20) {
                    println!("  {} -> {} ({:?})", edge.from, edge.to, edge.relation);
                }
                if cap_map.edges.len() > 20 {
                    println!("  ... and {} more", cap_map.edges.len() - 20);
                }
            }
        }

        CapabilityCommands::List { category } => {
            println!("Listing capabilities...");

            let mut introspector = RosRuntimeIntrospector::new();
            let snapshot = match introspector.capture_snapshot() {
                Ok(s) => s,
                Err(e) => {
                    println!("Error capturing snapshot: {}", e);
                    return;
                }
            };

            let classifier = CapabilityClassifier::new();
            let cap_map = classifier.classify(&snapshot);

            let cat_enum = category
                .as_ref()
                .and_then(|c| match c.to_lowercase().as_str() {
                    "sensing" => Some(CapabilityCategory::Sensing),
                    "decision" => Some(CapabilityCategory::Decision),
                    "actuation" => Some(CapabilityCategory::Actuation),
                    _ => None,
                });

            if let Some(c) = cat_enum {
                println!("\n=== {} Capabilities ===", category.as_ref().unwrap());
                for n in cap_map.nodes_by_category(&c) {
                    println!("  {}", n.id);
                }
            } else {
                println!("\n=== Topics ===");
                for (name, topic) in &cap_map.topics {
                    println!("  {} [{}]", name, topic.type_name);
                }

                println!("\n=== Services ===");
                for (name, service) in &cap_map.services {
                    println!("  {} [{}]", name, service.type_name);
                }

                println!("\n=== Actions ===");
                for (name, action) in &cap_map.actions {
                    println!("  {} [{}]", name, action.type_name);
                }
            }
        }
    }
}
