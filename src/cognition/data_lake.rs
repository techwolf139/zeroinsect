use crate::cognition::types::*;
use anyhow::Result;
use rocksdb::{ColumnFamily, Options, DB};
use std::path::Path;
use std::sync::Arc;

const DB_PATH: &str = "./data_lake";

const CF_DEVICE_STATE: &str = "device_state";
const CF_SENSOR_DATA: &str = "sensor_data";
const CF_KNOWLEDGE_GRAPH: &str = "knowledge_graph";
const CF_ANALYTICS: &str = "analytics";

#[derive(Clone)]
pub struct DataLake {
    db: Arc<DB>,
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

        let db = DB::open_cf(&opts, DB_PATH, cfs)?;
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

        let db = DB::open_cf(&opts, path.as_ref(), cfs)?;
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
        let key = format!("device_{}_state_{}", state.device_id, state.timestamp);
        let value = serde_json::to_vec(state)?;
        self.db.put_cf(self.cf_device_state(), key, value)?;
        Ok(())
    }

    pub fn get_device_state(&self, device_id: &str, timestamp: u64) -> Result<Option<DeviceState>> {
        let key = format!("device_{}_state_{}", device_id, timestamp);
        match self.db.get_cf(self.cf_device_state(), key)? {
            Some(value) => {
                let state: DeviceState = serde_json::from_slice(&value)?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    pub fn get_latest_device_state(&self, device_id: &str) -> Result<Option<DeviceState>> {
        let prefix = format!("device_{}_state_", device_id);
        let mut iter = self
            .db
            .prefix_iterator_cf(self.cf_device_state(), prefix.as_bytes());

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
        let prefix = format!("device_{}_state_", device_id);
        let mut states = Vec::new();

        let iter = self
            .db
            .prefix_iterator_cf(self.cf_device_state(), prefix.as_bytes());
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
        let key = format!("sensor_{}_{}", data.device_id, data.timestamp);
        let value = serde_json::to_vec(data)?;
        self.db.put_cf(self.cf_sensor_data(), key, value)?;
        Ok(())
    }

    pub fn get_sensor_data(&self, device_id: &str, timestamp: u64) -> Result<Option<SensorData>> {
        let key = format!("sensor_{}_{}", device_id, timestamp);
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
        let prefix = format!("sensor_{}_", device_id);
        let mut data_list = Vec::new();

        let iter = self
            .db
            .prefix_iterator_cf(self.cf_sensor_data(), prefix.as_bytes());
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
        let prefix = format!("sensor_{}_", device_id);
        let mut iter = self
            .db
            .prefix_iterator_cf(self.cf_sensor_data(), prefix.as_bytes());

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
}

impl Default for DataLake {
    fn default() -> Self {
        Self::new().expect("Failed to create DataLake")
    }
}
