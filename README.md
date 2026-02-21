# ROS Capability Discovery System

Automatic ROS2 system capability discovery through runtime introspection and static analysis. Generates structured capability data for ZeroClaw tool integration.

## Features

- **Runtime Introspection**: Discover ROS nodes, topics, services, and actions via DDS
- **Static Analysis**: Parse ROS2 launch files (.launch.xml, .launch.py)
- **Capability Cards**: Generate tool-ready capability definitions
- **Caching**: Multi-tier caching with memory, disk, and TTL expiry

## Modules

### Runtime Introspection (`src/introspect/runtime.rs`)

```rust
let mut introspector = RosRuntimeIntrospector::new();
let snapshot = introspector.capture_snapshot().unwrap();

for (name, node) in snapshot.nodes {
    println!("Node: {}", name);
}
```

### Launch File Parsing (`src/introspect/launch.rs`)

```rust
let launch = parse_launch_file("robot.launch.xml").unwrap();
for node in launch.nodes {
    println!("Package: {}, Exec: {}", node.package, node.executable);
}
```

### Capability Cards (`src/introspect/types.rs`)

```rust
use introspect::types::capability::{topic_to_publisher_card, service_to_card};

let card = topic_to_publisher_card(&topic);
println!("Tool: {}", card.call_template.tool_name);
```

### Cache Manager (`src/introspect/cache.rs`)

```rust
let mut cache = CacheManager::new()
    .with_disk(PathBuf::from(".ros_cache"))
    .with_ttl(Duration::from_secs(300));

cache.set("snapshot".to_string(), data, None);
if let Some(cached) = cache.get("snapshot") {
    // use cached data
}
```

## Data Structures

| Type | Description |
|------|-------------|
| `RosSnapshot` | Atomic snapshot of all ROS system state |
| `SnapshotDiff` | Change detection between snapshots |
| `CapabilityCard` | Tool-ready capability definition |
| `LaunchFile` | Parsed launch file with nodes, params, remaps |

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              Data Collection Layer                 │
├──────────────┬──────────────────┬──────────────────┤
│   Runtime    │  Launch Files    │   Source Code    │
│  (DDS/rcl)  │  (quick-xml)     │   (AST)          │
└──────┬───────┴────────┬─────────┴────────┬─────────┘
       │                │                  │
       ▼                ▼                  ▼
┌─────────────────────────────────────────────────────┐
│              Capability Aggregation                 │
│  - Node capability merging                         │
│  - Interface type inference                        │
│  - Call template generation                        │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│              Output Layer                            │
│  - ros_capabilities.json (tools)                   │
│  - ros_snapshot.json (system state)                 │
│  - ros_topics.json / services / actions            │
└─────────────────────────────────────────────────────┘
```

## Consistency Guarantees

- **Atomic Snapshots**: All data captured within 100ms window
- **Version Tracking**: Each snapshot increments version
- **Change Detection**: Diff between snapshots identifies additions/removals
- **Checksum**: SHA256 hash validates snapshot integrity

## Testing

```bash
cargo test introspect
```

## License

MIT
