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
    Skill {
        #[command(subcommand)]
        subcommand: SkillCommands,
    },
    Mqtt {
        #[arg(long, default_value = "127.0.0.1:1883")]
        addr: String,
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

#[derive(Subcommand)]
enum SkillCommands {
    List {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        source: Option<String>,
    },
    Info {
        #[arg(long)]
        name: String,
    },
    Call {
        #[arg(long)]
        name: String,

        #[arg(long)]
        params: Option<String>,
    },
    Execute {
        #[arg(long)]
        goal: String,
    },
    Discover {
        #[arg(long)]
        source: Option<String>,
    },
    Link {
        #[arg(long)]
        path: String,
    },
    Unlink {
        #[arg(long)]
        name: String,
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
        Commands::Skill { subcommand } => {
            handle_skill_command(subcommand);
        }
        Commands::Mqtt { addr } => {
            run_mqtt_server(&addr);
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
            format: _,
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

        CapabilityCommands::Plan { goal, from: _ } => {
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

fn handle_skill_command(cmd: SkillCommands) {
    use std::collections::HashMap;
    use std::sync::Arc;
    use zeroinsect::capability_map::CapabilityClassifier;
    use zeroinsect::introspect::runtime::RosRuntimeIntrospector;
    use zeroinsect::skill_executor::{SkillCategory, SkillExecutor, SkillLoader, SkillRegistry};

    match cmd {
        SkillCommands::List { category, source } => {
            use zeroinsect::skill_executor::{DiscoveredSkill, SkillDiscovery, SkillSource};

            match source.as_deref() {
                Some("ros") | None => {
                    println!("Loading skills from ROS system...");

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

                    let loader = SkillLoader::new();
                    let skills = loader.generate_from_capability_map(&cap_map);

                    let mut registry = SkillRegistry::new();
                    for skill in skills {
                        registry.register(skill);
                    }

                    println!("\n=== ROS Skills ===");

                    if let Some(cat) = category {
                        let cat_enum = match cat.to_lowercase().as_str() {
                            "motion" => Some(SkillCategory::Motion),
                            "perception" => Some(SkillCategory::Perception),
                            "manipulation" => Some(SkillCategory::Manipulation),
                            "navigation" => Some(SkillCategory::Navigation),
                            _ => None,
                        };

                        if let Some(c) = cat_enum {
                            let skills = registry.list_by_category(&c);
                            for skill in skills {
                                println!(
                                    "  ✦ {} - {}",
                                    skill.metadata.name, skill.metadata.description
                                );
                            }
                        }
                    } else {
                        for skill in registry.list_all() {
                            println!(
                                "  ✦ {} - {}",
                                skill.metadata.name, skill.metadata.description
                            );
                        }
                    }
                }
                Some("opencode") => {
                    let discovery = SkillDiscovery::new();
                    let skills = discovery.discover_from_opencode();
                    print_discovered_skills("OpenCode", &skills, category.as_deref());
                }
                Some("zeroclaw") | Some("config") => {
                    let discovery = SkillDiscovery::new();
                    let skills = discovery.discover_from_config();
                    print_discovered_skills("ZeroClaw/Config", &skills, category.as_deref());
                }
                Some("local") => {
                    let discovery = SkillDiscovery::new();
                    let local_skills = std::path::PathBuf::from("./skills");
                    let skills = if local_skills.exists() {
                        discovery.scan_skill_directory(&local_skills, SkillSource::Local)
                    } else {
                        vec![]
                    };
                    print_discovered_skills("Local", &skills, category.as_deref());
                }
                Some("linked") => {
                    let discovery = SkillDiscovery::new();
                    let skills = discovery.list_linked_skills();
                    print_discovered_skills("Linked", &skills, category.as_deref());
                }
                Some("all") => {
                    let discovery = SkillDiscovery::new();
                    let all_skills = discovery.discover_from_standard_locations();

                    let mut by_source: std::collections::HashMap<String, Vec<DiscoveredSkill>> =
                        std::collections::HashMap::new();
                    for skill in &all_skills {
                        let key = format!("{:?}", skill.source);
                        by_source
                            .entry(key)
                            .or_insert_with(Vec::new)
                            .push(skill.clone());
                    }

                    for (source_name, skills) in by_source {
                        print_discovered_skills(&source_name, &skills, category.as_deref());
                    }

                    println!("\n=== ROS Skills ===");
                    let mut introspector = RosRuntimeIntrospector::new();
                    if let Ok(snapshot) = introspector.capture_snapshot() {
                        let classifier = CapabilityClassifier::new();
                        let cap_map = classifier.classify(&snapshot);
                        let loader = SkillLoader::new();
                        let skills = loader.generate_from_capability_map(&cap_map);

                        let mut registry = SkillRegistry::new();
                        for skill in skills {
                            registry.register(skill);
                        }

                        if let Some(cat) = category {
                            let cat_enum = match cat.to_lowercase().as_str() {
                                "motion" => Some(SkillCategory::Motion),
                                "perception" => Some(SkillCategory::Perception),
                                "manipulation" => Some(SkillCategory::Manipulation),
                                "navigation" => Some(SkillCategory::Navigation),
                                _ => None,
                            };

                            if let Some(c) = cat_enum {
                                let skills = registry.list_by_category(&c);
                                for skill in skills {
                                    println!(
                                        "  ✦ {} - {}",
                                        skill.metadata.name, skill.metadata.description
                                    );
                                }
                            }
                        } else {
                            for skill in registry.list_all() {
                                println!(
                                    "  ✦ {} - {}",
                                    skill.metadata.name, skill.metadata.description
                                );
                            }
                        }
                    }
                }
                Some(unknown) => {
                    println!("Unknown source: {}", unknown);
                    println!("Available sources: ros, opencode, zeroclaw, local, linked, all");
                }
            }
        }

        SkillCommands::Info { name } => {
            println!("Getting skill info: {}", name);

            let mut introspector = RosRuntimeIntrospector::new();
            if let Ok(snapshot) = introspector.capture_snapshot() {
                let classifier = CapabilityClassifier::new();
                let cap_map = classifier.classify(&snapshot);
                let loader = SkillLoader::new();
                let skills = loader.generate_from_capability_map(&cap_map);

                let mut registry = SkillRegistry::new();
                for skill in skills {
                    registry.register(skill);
                }

                if let Some(skill) = registry.get(&name) {
                    println!("\n=== Skill: {} ===", skill.metadata.name);
                    println!("Description: {}", skill.metadata.description);
                    println!("Category: {}", skill.metadata.category);
                    println!("Capability: {:?}", skill.capability.cap_type);
                    println!("ROS Name: {}", skill.capability.ros_name);
                    println!("ROS Type: {}", skill.capability.ros_type);

                    if !skill.parameters.is_empty() {
                        println!("\nParameters:");
                        for param in &skill.parameters {
                            let req = if param.required { " (required)" } else { "" };
                            println!("  - {}{}: {}", param.name, req, param.param_type);
                        }
                    }

                    if !skill.returns.is_empty() {
                        println!("\nReturns:");
                        for ret in &skill.returns {
                            println!("  - {}: {}", ret.name, ret.return_type);
                        }
                    }
                } else {
                    println!("Skill not found: {}", name);
                }
            }
        }

        SkillCommands::Call { name, params } => {
            println!("Calling skill: {}", name);

            let mut introspector = RosRuntimeIntrospector::new();
            if let Ok(snapshot) = introspector.capture_snapshot() {
                let classifier = CapabilityClassifier::new();
                let cap_map = classifier.classify(&snapshot);
                let loader = SkillLoader::new();
                let skills = loader.generate_from_capability_map(&cap_map);

                let mut registry = SkillRegistry::new();
                for skill in skills {
                    registry.register(skill);
                }

                let executor = SkillExecutor::new(Arc::new(registry)).with_mock_mode(true);

                let mut param_map = HashMap::new();
                if let Some(p) = params {
                    for pair in p.split(',') {
                        let parts: Vec<&str> = pair.split('=').collect();
                        if parts.len() == 2 {
                            param_map.insert(parts[0].to_string(), serde_json::json!(parts[1]));
                        }
                    }
                }

                let request = zeroinsect::skill_executor::SkillRequest {
                    skill_name: name,
                    parameters: param_map,
                    timeout_ms: Some(5000),
                };

                let response = executor.execute(request);

                if response.success {
                    println!("\n✓ Success!");
                    println!("Result: {}", response.result);
                    println!("Execution time: {}ms", response.execution_time_ms);
                } else {
                    println!("\n✗ Error: {}", response.message);
                }
            }
        }

        SkillCommands::Execute { goal } => {
            println!("Executing goal: {}", goal);
            println!("\n(Note: LLM integration not implemented yet.)");
            println!("Use 'skill call' to execute specific skills.");
        }

        SkillCommands::Discover { source } => {
            use zeroinsect::skill_executor::SkillDiscovery;

            let discovery = SkillDiscovery::new();

            println!("Discovering skills...\n");

            if let Some(s) = source {
                match s.as_str() {
                    "opencode" => {
                        let skills = discovery.discover_from_opencode();
                        println!("=== OpenCode Skills ===");
                        for skill in skills {
                            println!("  ✦ {} ({})", skill.name, skill.path.display());
                        }
                    }
                    "config" => {
                        let skills = discovery.discover_from_config();
                        println!("=== Config/Agent Skills ===");
                        for skill in skills {
                            println!("  ✦ {} ({})", skill.name, skill.path.display());
                        }
                    }
                    _ => {
                        println!("Unknown source: {}. Use 'opencode' or 'config'", s);
                    }
                }
            } else {
                let skills = discovery.discover_from_standard_locations();
                println!("=== Discovered Skills ===");

                let mut by_source: std::collections::HashMap<String, Vec<String>> =
                    std::collections::HashMap::new();
                for skill in &skills {
                    let key = format!("{:?}", skill.source);
                    by_source
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(skill.name.clone());
                }

                for (source, names) in by_source {
                    println!("\n[{}]", source);
                    for name in names {
                        println!("  ✦ {}", name);
                    }
                }

                println!("\nTotal: {} skills found", skills.len());
            }
        }

        SkillCommands::Link { path } => {
            use zeroinsect::skill_executor::SkillDiscovery;

            let discovery = SkillDiscovery::new();
            let source = std::path::PathBuf::from(&path);
            let target = discovery.get_linked_skills_dir();

            match discovery.link_skill(&source, &target) {
                Ok(linked) => {
                    println!("✓ Successfully linked skill!");
                    println!("  Source: {}", path);
                    println!("  Target: {}", linked.display());
                }
                Err(e) => {
                    println!("✗ Error: {}", e);
                }
            }
        }

        SkillCommands::Unlink { name } => {
            use zeroinsect::skill_executor::SkillDiscovery;

            let discovery = SkillDiscovery::new();
            let skills_dir = discovery.get_linked_skills_dir();

            match discovery.unlink_skill(&name, &skills_dir) {
                Ok(_) => {
                    println!("✓ Successfully unlinked skill: {}", name);
                }
                Err(e) => {
                    println!("✗ Error: {}", e);
                }
            }
        }
    }
}

fn print_discovered_skills(
    source_name: &str,
    skills: &[zeroinsect::skill_executor::DiscoveredSkill],
    category_filter: Option<&str>,
) {
    use zeroinsect::skill_executor::SkillCategory;

    if skills.is_empty() {
        return;
    }

    println!("\n=== {} Skills ===", source_name);

    if let Some(cat) = category_filter {
        let cat_enum = match cat.to_lowercase().as_str() {
            "motion" => Some(SkillCategory::Motion),
            "perception" => Some(SkillCategory::Perception),
            "manipulation" => Some(SkillCategory::Manipulation),
            "navigation" => Some(SkillCategory::Navigation),
            _ => None,
        };

        if let Some(_c) = cat_enum {
            for skill in skills {
                let has_toml = if skill.has_skill_toml { " [TOML]" } else { "" };
                let has_md = if skill.has_skill_md { " [MD]" } else { "" };
                println!("  ✦ {}{}{}", skill.name, has_toml, has_md);
            }
        }
    } else {
        for skill in skills {
            let has_toml = if skill.has_skill_toml { " [TOML]" } else { "" };
            let has_md = if skill.has_skill_md { " [MD]" } else { "" };
            println!("  ✦ {}{}{}", skill.name, has_toml, has_md);
        }
    }
}

fn run_mqtt_server(addr: &str) {
    use zeroinsect::broker::server::BrokerServer;
    use zeroinsect::storage::kv_store::KvStore;
    use tokio;

    println!("Starting MQTT Broker...");
    println!("Address: {}", addr);
    
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    rt.block_on(async {
        let store = match KvStore::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to initialize storage: {}", e);
                return;
            }
        };
        
        let server = BrokerServer::new(store);
        
        if let Err(e) = server.start(addr).await {
            eprintln!("Server error: {}", e);
        }
    });
}
