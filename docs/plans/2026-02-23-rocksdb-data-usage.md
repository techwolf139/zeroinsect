# RocksDB Data Lake 数据使用模式

> 本文档描述 Local Cognition Device Management System 中 RocksDB 的数据存储架构和使用模式。

## 1. 系统架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Data Lake (RocksDB)                              │
├──────────────────┬──────────────────┬──────────────────┬─────────────────┤
│  device_state    │   sensor_data    │ knowledge_graph  │   analytics    │
│   (设备状态)      │  (传感器时序)     │   (知识图谱)     │   (分析结果)    │
└──────────────────┴──────────────────┴──────────────────┴─────────────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │   Block Cache      │
                         │   (128MB LRU)     │
                         └─────────────────────┘
```

## 2. Column Families

| CF 名称 | 用途 | 数据类型 |
|---------|------|---------|
| `device_state` | 设备状态历史 | DeviceState |
| `sensor_data` | 传感器时序数据 | SensorData |
| `knowledge_graph` | 知识图谱节点/边 | KnowledgeNode, KnowledgeEdge, DeviceKnowledge |
| `analytics` | 分析结果缓存 | AnalyticsResult |

## 3. Key-Value 设计模式

### 3.1 二进制 Key 编码 (device_state / sensor_data)

使用 `KeyBuilder` 生成二进制 Key：

```
┌────────────────────────────────────────────────────────────────┐
│ Key Format: device_id + 0x00 + tag + 0x00 + timestamp (BigEndian) │
├────────────────────────────────────────────────────────────────┤
│ 示例: "robot-arm-01\0state\0\x00\x00\x01\x8a\x9c\xd0"           │
│        └────┘└┘└─────┘└┘└────────────────────┘                 │
│        ID    分隔  标签  分隔        8字节 BigEndian 时间戳       │
└────────────────────────────────────────────────────────────────┘
```

**为什么用 BigEndian?**
- RocksDB 按字节字典序排序
- BigEndian 确保时间戳 `100` 排在 `200` 前面
- 小端序会导致时间逆序

### 3.2 字符串 Key (knowledge_graph / analytics)

```rust
// 知识图谱节点
key = format!("node_{}", node.node_id)

// 知识图谱边  
key = format!("edge_{}", edge.edge_id)

// 设备知识
key = format!("knowledge_{}", device_id)

// 分析结果
key = format!("analyze_{}_{}", device_id, time_window)
```

### 3.3 Key 前缀扫描

```rust
// 获取某设备所有状态
prefix = KeyBuilder::build_prefix(device_id, "state")
// 结果: "robot-arm-01\0state\0"

// 获取某设备某传感器所有数据
prefix = KeyBuilder::build_prefix(device_id, "temperature")
// 结果: "robot-arm-01\0temperature\0"
```

## 4. 数据模型

### 4.1 DeviceState (设备状态)

```rust
struct DeviceState {
    device_id: String,      // 设备标识: "robot-arm-01"
    timestamp: u64,         // Unix 时间戳
    status: DeviceStatus,    // Online/Offline/Error/Idle
    cpu_usage: f32,         // CPU 使用率 (0-100%)
    memory_usage: f32,       // 内存使用率 (0-100%)
    temperature: f32,        // 温度 (°C)
    last_command: Option<String>, // 上次执行的命令
}
```

### 4.2 SensorData (传感器数据)

```rust
struct SensorData {
    device_id: String,      // 设备标识
    sensor_type: String,    // 传感器类型: "joint_position", "torque", "battery"
    values: Vec<f32>,       // 时序数据数组
    timestamp: u64,        // Unix 时间戳
}
```

### 4.3 KnowledgeNode (知识图谱节点)

```rust
struct KnowledgeNode {
    node_id: String,
    device_id: String,
    node_type: NodeType,    // Device/Sensor/Actuator/Controller/Gateway
    properties: HashMap<String, String>,
    metadata: HashMap<String, String>,
}
```

### 4.4 KnowledgeEdge (知识图谱边)

```rust
struct KnowledgeEdge {
    edge_id: String,
    from_node: String,
    to_node: String,
    relation: RelationType,  // DependsOn/Produces/Consumes/Controls/Monitors
    weight: f32,
    properties: HashMap<String, String>,
}
```

### 4.5 AnalyticsResult (分析结果)

```rust
struct AnalyticsResult {
    device_id: String,
    time_window: String,    // "1h", "24h", "7d"
    metrics: HashMap<String, f32>,
    trends: Vec<TrendPoint>,
    anomalies: Vec<Anomaly>,
}
```

## 5. 核心操作

### 5.1 设备状态操作

| 操作 | 方法 | Key 模式 |
|------|------|----------|
| 存储状态 | `store_device_state()` | `KeyBuilder::build(id, "state", ts)` |
| 获取最新 | `get_latest_device_state()` | 前缀扫描 |
| 范围查询 | `get_device_states_range()` | 前缀扫描 + 时间过滤 |
| 删除 | `delete_device_state()` | 精确 Key |

### 5.2 传感器数据操作

| 操作 | 方法 | Key 模式 |
|------|------|----------|
| 存储 | `store_sensor_data()` | `KeyBuilder::build(id, type, ts)` |
| 点查 | `get_sensor_data()` | 精确 Key |
| 范围查询 | `get_sensor_data_range()` | 前缀扫描 |
| 最新 | `get_latest_sensor_data()` | 前缀扫描 |

### 5.3 批量写入

```rust
// 批量存储设备状态 (ROS 高频数据)
pub fn store_device_states_batch(&self, states: &[DeviceState]) -> Result<()>

// 批量存储传感器数据
pub fn store_sensor_data_batch(&self, data_list: &[SensorData]) -> Result<()>
```

使用 `WriteBatch` 减少 I/O 开销。

## 6. 访问模式分析

### 6.1 读操作

```
┌─────────────────────────────────────────────────────────────┐
│  Prefix Scan (前缀扫描)                                      │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 1. 构建前缀: KeyBuilder::build_prefix("robot-arm-01", "state")│
│  │ 2. RocksDB iterator: prefix_iterator_cf(cf, prefix)  │    │
│  │ 3. 内存过滤: timestamp >= start && timestamp <= end   │    │
│  │ 4. 返回排序结果                                       │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

**特点:**
- 前缀扫描利用 RocksDB 内部排序
- 时间范围在内存中过滤
- 适合中小数据量 (< 10k 条)

### 6.2 写操作

```
┌─────────────────────────────────────────────────────────────┐
│  Write Path                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 1. 序列化: serde_json::to_vec()                     │    │
│  │ 2. 构建 Key: KeyBuilder::build()                    │    │
│  │ 3. 写入: db.put_cf(cf, key, value)                │    │
│  │ 4. 刷盘: RocksDB 后台自动刷盘                        │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 批量写入

```
┌─────────────────────────────────────────────────────────────┐
│  Batch Write (WriteBatch)                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 1. 创建 WriteBatch                                   │    │
│  │ 2. 循环: batch.put_cf(key, value)                  │    │
│  │ 3. 提交: db.write(batch) ← 单次 I/O                │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## 7. 性能优化

### 7.1 Block Cache

```rust
let mut block_opts = BlockBasedOptions::default();
block_opts.set_block_cache(&Cache::new_lru_cache(128 * 1024 * 1024)); // 128MB
opts.set_block_based_table_factory(&block_opts);
```

- LRU 缓存最近访问的 Data Block
- 适合读多写少场景
- 边缘设备推荐 128MB

### 7.2 并行配置

```rust
opts.increase_parallelism(4);      // 4 线程压缩
opts.set_max_open_files(10000);     // 最多打开文件数
opts.set_keep_log_file_num(10);    // 保留日志文件数
```

### 7.3 压缩

```rust
opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
```

- LZ4 压缩: 平衡压缩速度与压缩比
- 适合边缘设备

## 8. 实际业务数据示例

### 8.1 robot-arm-01 (机械臂)

| 传感器类型 | 用途 | 典型值 |
|-----------|------|--------|
| `joint_position` | 关节角度 | [30, 45, 60, 45, 30] |
| `joint_velocity` | 关节速度 | [15, 25, 30, 25, 15] |
| `torque` | 扭矩 | [2.5, 3.2, 4.1, 3.8, 2.9] |
| `gripper` | 夹爪开合 | [0, 0, 0, 100, 100] |

### 8.2 agv-01 (自动导引车)

| 传感器类型 | 用途 | 典型值 |
|-----------|------|--------|
| `position_x` | X 坐标 | [0, 2, 4, 6, 8, 10] |
| `position_y` | Y 坐标 | [0, 0, 0, 0, 0, 1] |
| `velocity` | 速度 | [0.5, 0.5, 0.5, 0.5, 0.5] |
| `battery` | 电池电量 | [100, 95, 90, 85, 80] |
| `brush_rpm` | 刷子转速 | [0, 300, 300, 300, 0] |
| `cleaning_coverage` | 清洁覆盖率 | [0, 5, 12, 20, 30, 42] |

## 9. HTTP API

```
POST   /api/device/:device_id/state          # 存储设备状态
GET    /api/device/:device_id/state           # 获取最新状态
GET    /api/device/:device_id/state/range    # 范围查询
POST   /api/device/:device_id/sensor/:type   # 存储传感器数据
GET    /api/device/:device_id/sensor/:type   # 获取传感器数据
GET    /health                                # 健康检查
```

## 10. 限制与注意事项

### 10.1 TTL 支持

- RocksDB Rust crate 0.22/0.23 不支持 TTL feature
- 需要应用层实现数据过期清理

### 10.2 索引

- 知识图谱查询使用全扫描，无二级索引
- 未来可添加 `kg_index` CF 做设备→节点映射

### 10.3 数据膨胀

- 每次状态更新生成新 Key (不覆盖)
- 需要定期清理历史数据

## 11. 依赖版本

```toml
rocksdb = "0.22"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
byteorder = "1.4"
```
