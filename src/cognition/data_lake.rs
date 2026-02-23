use crate::cognition::types::*;
use anyhow::Result;
use byteorder::{BigEndian, WriteBytesExt};
// TTL support shim: provide a TTL-enabled DB wrapper when the ttl feature is enabled.
#[cfg(feature = "ttl")]
mod ttlshim {
    use rocksdb::{Options, DB};
    use std::path::Path;
    pub struct DBWithTTL(DB);
    impl std::ops::Deref for DBWithTTL {
        type Target = DB;
        fn deref(&self) -> &DB {
            &self.0
        }
    }
    impl DBWithTTL {
        pub fn open_cf_with_ttl(
            opts: &Options,
            path: &Path,
            cfs: Vec<&str>,
            _ttl: i32,
        ) -> anyhow::Result<DBWithTTL> {
            // Fallback to normal open; TTL is to be handled by RocksDB when a concrete API is available.
            let db = DB::open_cf(opts, path, cfs)?;
            Ok(DBWithTTL(db))
        }
    }
}
// Unconditional imports for core RocksDB types; TTL path is feature-gated below
use rocksdb::{ColumnFamily, Options, WriteBatch, DB};
#[cfg(feature = "ttl")]
use ttlshim::DBWithTTL;
#[cfg(feature = "ttl")]
type RocksDBHandle = DBWithTTL;
#[cfg(not(feature = "ttl"))]
type RocksDBHandle = DB;
use rocksdb::{BlockBasedOptions, Cache};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

/// KeyBuilder constructs binary keys for RocksDB storage
/// Format: device_id + 0x00 + tag + 0x00 + timestamp (BigEndian)
pub struct KeyBuilder;

impl KeyBuilder {
    /// Build binary key: device_id + 0x00 + tag + 0x00 + timestamp (BigEndian)
    pub fn build(device_id: &str, tag: &str, timestamp: u64) -> Vec<u8> {
        let mut key = Vec::new();
        key.extend_from_slice(device_id.as_bytes());
        key.push(0x00);
        key.extend_from_slice(tag.as_bytes());
        key.push(0x00);
        key.write_u64::<BigEndian>(timestamp).unwrap();
        key
    }

    /// Build prefix for scanning
    pub fn build_prefix(device_id: &str, tag: &str) -> Vec<u8> {
        let mut prefix = Vec::new();
        prefix.extend_from_slice(device_id.as_bytes());
        prefix.push(0x00);
        prefix.extend_from_slice(tag.as_bytes());
        prefix.push(0x00);
        prefix
    }

    /// Build range keys for precise boundary control
    pub fn build_range_keys(
        device_id: &str,
        tag: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> (Vec<u8>, Vec<u8>) {
        (
            Self::build(device_id, tag, start_ts),
            Self::build(device_id, tag, end_ts),
        )
    }
}

const DB_PATH: &str = "./data_lake";

const CF_DEVICE_STATE: &str = "device_state";
const CF_SENSOR_DATA: &str = "sensor_data";
const CF_KNOWLEDGE_GRAPH: &str = "knowledge_graph";
const CF_ANALYTICS: &str = "analytics";

// TTL (time-to-live) for RocksDB entries in seconds. Adjust as needed for your environment.
// This value is kept small enough for tests; adjust in production as appropriate.
#[cfg(feature = "ttl")]
const TTL_SECONDS: i32 = 7 * 24 * 60 * 60; // 1 week
#[cfg(not(feature = "ttl"))]
const TTL_SECONDS: i32 = 0;

#[derive(Clone)]
pub struct DataLake {
    db: Arc<RocksDBHandle>,
}

impl DataLake {
    pub fn new() -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_keep_log_file_num(10);
        opts.increase_parallelism(4);
        opts.set_max_open_files(10000);

        let cfs = vec![
            CF_DEVICE_STATE,
            CF_SENSOR_DATA,
            CF_KNOWLEDGE_GRAPH,
            CF_ANALYTICS,
        ];
        // Configure RocksDB block cache for improved performance
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_cache(&Cache::new_lru_cache(128 * 1024 * 1024));
        opts.set_block_based_table_factory(&block_opts);
        // Configure RocksDB block cache for improved performance
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_cache(&Cache::new_lru_cache(128 * 1024 * 1024));
        opts.set_block_based_table_factory(&block_opts);
        let db = {
            #[cfg(feature = "ttl")]
            {
                ttlshim::DBWithTTL::open_cf_with_ttl(
                    &opts,
                    Path::new(DB_PATH),
                    cfs.clone(),
                    TTL_SECONDS,
                )?
            }
            #[cfg(not(feature = "ttl"))]
            {
                DB::open_cf(&opts, DB_PATH, cfs)?
            }
        };
        Ok(Self { db: Arc::new(db) })
    }

    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_keep_log_file_num(10);
        opts.increase_parallelism(4);
        opts.set_max_open_files(10000);

        let cfs = vec![
            CF_DEVICE_STATE,
            CF_SENSOR_DATA,
            CF_KNOWLEDGE_GRAPH,
            CF_ANALYTICS,
        ];
        let db = {
            #[cfg(feature = "ttl")]
            {
                ttlshim::DBWithTTL::open_cf_with_ttl(
                    &opts,
                    path.as_ref(),
                    cfs.clone(),
                    TTL_SECONDS,
                )?
            }
            #[cfg(not(feature = "ttl"))]
            {
                DB::open_cf(&opts, path.as_ref(), cfs)?
            }
        };
        Ok(Self { db: Arc::new(db) })
    }

    fn cf_device_state(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_DEVICE_STATE).unwrap()
    }

    fn cf_sensor_data(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_SENSOR_DATA).unwrap()
    }

    fn cf_knowledge_graph(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_KNOWLEDGE_GRAPH).unwrap()
    }

    fn cf_analytics(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_ANALYTICS).unwrap()
    }

    pub fn store_device_state(&self, state: &DeviceState) -> Result<()> {
        let key = KeyBuilder::build(&state.device_id, "state", state.timestamp);
        let value = serde_json::to_vec(state)?;
        self.db.put_cf(self.cf_device_state(), key, value)?;
        Ok(())
    }

    pub fn get_device_state(&self, device_id: &str, timestamp: u64) -> Result<Option<DeviceState>> {
        let key = KeyBuilder::build(device_id, "state", timestamp);
        match self.db.get_cf(self.cf_device_state(), key)? {
            Some(value) => {
                let state: DeviceState = serde_json::from_slice(&value)?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    pub fn get_latest_device_state(&self, device_id: &str) -> Result<Option<DeviceState>> {
        let prefix = KeyBuilder::build_prefix(device_id, "state");
        let mut iter = self.db.prefix_iterator_cf(self.cf_device_state(), &prefix);

        if let Some(item) = iter.next() {
            let (_, value) = item?;
            let state: DeviceState = serde_json::from_slice(&value)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    pub fn get_device_states_range(
        &self,
        device_id: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> Result<Vec<DeviceState>> {
        let prefix = KeyBuilder::build_prefix(device_id, "state");
        let mut states = Vec::new();

        let iter = self.db.prefix_iterator_cf(self.cf_device_state(), &prefix);
        for item in iter {
            let (_, value) = item?;
            let state: DeviceState = serde_json::from_slice(&value)?;
            if state.timestamp >= start_ts && state.timestamp <= end_ts {
                states.push(state);
            }
        }

        states.sort_by_key(|s| s.timestamp);
        Ok(states)
    }

    pub fn store_sensor_data(&self, data: &SensorData) -> Result<()> {
        let key = KeyBuilder::build(&data.device_id, &data.sensor_type, data.timestamp);
        let value = serde_json::to_vec(data)?;
        self.db.put_cf(self.cf_sensor_data(), key, value)?;
        Ok(())
    }

    pub fn get_sensor_data(
        &self,
        device_id: &str,
        sensor_type: &str,
        timestamp: u64,
    ) -> Result<Option<SensorData>> {
        let key = KeyBuilder::build(device_id, sensor_type, timestamp);
        match self.db.get_cf(self.cf_sensor_data(), key)? {
            Some(value) => {
                let data: SensorData = serde_json::from_slice(&value)?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    pub fn get_sensor_data_range(
        &self,
        device_id: &str,
        sensor_type: Option<&str>,
        start_ts: u64,
        end_ts: u64,
    ) -> Result<Vec<SensorData>> {
        let tag = sensor_type.unwrap_or("");
        let prefix = KeyBuilder::build_prefix(device_id, tag);
        let mut data_list = Vec::new();

        let iter = self.db.prefix_iterator_cf(self.cf_sensor_data(), &prefix);
        for item in iter {
            let (_, value) = item?;
            let data: SensorData = serde_json::from_slice(&value)?;

            let matches_type = sensor_type.map(|t| data.sensor_type == t).unwrap_or(true);

            if matches_type && data.timestamp >= start_ts && data.timestamp <= end_ts {
                data_list.push(data);
            }
        }

        data_list.sort_by_key(|d| d.timestamp);
        Ok(data_list)
    }

    pub fn get_latest_sensor_data(
        &self,
        device_id: &str,
        sensor_type: &str,
    ) -> Result<Option<SensorData>> {
        let prefix = KeyBuilder::build_prefix(device_id, sensor_type);
        let mut iter = self.db.prefix_iterator_cf(self.cf_sensor_data(), &prefix);

        while let Some(item) = iter.next() {
            let (_, value) = item?;
            let data: SensorData = serde_json::from_slice(&value)?;
            if data.sensor_type == sensor_type {
                return Ok(Some(data));
            }
        }

        Ok(None)
    }

    pub fn store_knowledge_node(&self, node: &KnowledgeNode) -> Result<()> {
        let key = format!("node_{}", node.node_id);
        let value = serde_json::to_vec(node)?;
        self.db.put_cf(self.cf_knowledge_graph(), key, value)?;
        Ok(())
    }

    pub fn get_knowledge_node(&self, node_id: &str) -> Result<Option<KnowledgeNode>> {
        let key = format!("node_{}", node_id);
        match self.db.get_cf(self.cf_knowledge_graph(), key)? {
            Some(value) => {
                let node: KnowledgeNode = serde_json::from_slice(&value)?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    pub fn get_nodes_by_device(&self, device_id: &str) -> Result<Vec<KnowledgeNode>> {
        let prefix = "node_";
        let mut nodes = Vec::new();

        let iter = self
            .db
            .prefix_iterator_cf(self.cf_knowledge_graph(), prefix.as_bytes());
        for item in iter {
            let (_, value) = item?;
            let node: KnowledgeNode = serde_json::from_slice(&value)?;
            if node.device_id == device_id {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    pub fn store_knowledge_edge(&self, edge: &KnowledgeEdge) -> Result<()> {
        let key = format!("edge_{}", edge.edge_id);
        let value = serde_json::to_vec(edge)?;
        self.db.put_cf(self.cf_knowledge_graph(), key, value)?;
        Ok(())
    }

    pub fn get_knowledge_edge(&self, edge_id: &str) -> Result<Option<KnowledgeEdge>> {
        let key = format!("edge_{}", edge_id);
        match self.db.get_cf(self.cf_knowledge_graph(), key)? {
            Some(value) => {
                let edge: KnowledgeEdge = serde_json::from_slice(&value)?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    pub fn get_edges_from_node(&self, node_id: &str) -> Result<Vec<KnowledgeEdge>> {
        let prefix = "edge_";
        let mut edges = Vec::new();

        let iter = self
            .db
            .prefix_iterator_cf(self.cf_knowledge_graph(), prefix.as_bytes());
        for item in iter {
            let (_, value) = item?;
            let edge: KnowledgeEdge = serde_json::from_slice(&value)?;
            if edge.from_node == node_id {
                edges.push(edge);
            }
        }

        Ok(edges)
    }

    pub fn get_edges_to_node(&self, node_id: &str) -> Result<Vec<KnowledgeEdge>> {
        let prefix = "edge_";
        let mut edges = Vec::new();

        let iter = self
            .db
            .prefix_iterator_cf(self.cf_knowledge_graph(), prefix.as_bytes());
        for item in iter {
            let (_, value) = item?;
            let edge: KnowledgeEdge = serde_json::from_slice(&value)?;
            if edge.to_node == node_id {
                edges.push(edge);
            }
        }

        Ok(edges)
    }

    pub fn store_device_knowledge(&self, knowledge: &DeviceKnowledge) -> Result<()> {
        let key = format!("knowledge_{}", knowledge.device_id);
        let value = serde_json::to_vec(knowledge)?;
        self.db.put_cf(self.cf_knowledge_graph(), key, value)?;
        Ok(())
    }

    pub fn get_device_knowledge(&self, device_id: &str) -> Result<Option<DeviceKnowledge>> {
        let key = format!("knowledge_{}", device_id);
        match self.db.get_cf(self.cf_knowledge_graph(), key)? {
            Some(value) => {
                let knowledge: DeviceKnowledge = serde_json::from_slice(&value)?;
                Ok(Some(knowledge))
            }
            None => Ok(None),
        }
    }

    pub fn store_analytics(&self, analytics: &AnalyticsResult) -> Result<()> {
        let key = format!("analyze_{}_{}", analytics.device_id, analytics.time_window);
        let value = serde_json::to_vec(analytics)?;
        self.db.put_cf(self.cf_analytics(), key, value)?;
        Ok(())
    }

    pub fn get_analytics(
        &self,
        device_id: &str,
        time_window: &str,
    ) -> Result<Option<AnalyticsResult>> {
        let key = format!("analyze_{}_{}", device_id, time_window);
        match self.db.get_cf(self.cf_analytics(), key)? {
            Some(value) => {
                let analytics: AnalyticsResult = serde_json::from_slice(&value)?;
                Ok(Some(analytics))
            }
            None => Ok(None),
        }
    }

    pub fn delete_device_state(&self, device_id: &str, timestamp: u64) -> Result<()> {
        let key = format!("device_{}_state_{}", device_id, timestamp);
        self.db.delete_cf(self.cf_device_state(), key)?;
        Ok(())
    }

    pub fn delete_sensor_data(&self, device_id: &str, timestamp: u64) -> Result<()> {
        let key = format!("sensor_{}_{}", device_id, timestamp);
        self.db.delete_cf(self.cf_sensor_data(), key)?;
        Ok(())
    }

    pub fn store_device_states_batch(&self, states: &[DeviceState]) -> Result<()> {
        let mut batch = WriteBatch::default();
        for state in states {
            let key = KeyBuilder::build(&state.device_id, "state", state.timestamp);
            let value = serde_json::to_vec(state)?;
            batch.put_cf(self.cf_device_state(), key, value);
        }
        self.db.write(batch)?;
        Ok(())
    }

    pub fn store_sensor_data_batch(&self, data_list: &[SensorData]) -> Result<()> {
        let mut batch = WriteBatch::default();
        for data in data_list {
            let key = KeyBuilder::build(&data.device_id, &data.sensor_type, data.timestamp);
            let value = serde_json::to_vec(data)?;
            batch.put_cf(self.cf_sensor_data(), key, value);
        }
        self.db.write(batch)?;
        Ok(())
    }
}

impl Default for DataLake {
    fn default() -> Self {
        Self::new().expect("Failed to create DataLake")
    }
}
