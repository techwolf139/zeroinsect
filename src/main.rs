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
    }
}
