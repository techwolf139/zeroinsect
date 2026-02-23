use crate::cognition::{DataLake, DeviceState, SensorData};
use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DataLakeState {
    pub data_lake: Arc<RwLock<DataLake>>,
}

#[derive(Debug, Deserialize)]
pub struct StoreDeviceStateRequest {
    pub device_id: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub temperature: f32,
    pub last_command: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StoreSensorDataRequest {
    pub device_id: String,
    pub sensor_type: String,
    pub values: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct QueryRangeParams {
    pub start_ts: Option<u64>,
    pub end_ts: Option<u64>,
    pub sensor_type: Option<String>,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

async fn store_device_state(
    State(app_state): State<DataLakeState>,
    Json(payload): Json<StoreDeviceStateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let device_state = DeviceState {
        device_id: payload.device_id,
        timestamp: current_timestamp(),
        status: match payload.status.as_str() {
            "online" => crate::cognition::DeviceStatus::Online,
            "offline" => crate::cognition::DeviceStatus::Offline,
            "error" => crate::cognition::DeviceStatus::Error,
            _ => crate::cognition::DeviceStatus::Idle,
        },
        cpu_usage: payload.cpu_usage,
        memory_usage: payload.memory_usage,
        temperature: payload.temperature,
        last_command: payload.last_command,
    };

    let data_lake = app_state.data_lake.read().await;
    data_lake
        .store_device_state(&device_state)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Device state stored"
    })))
}

async fn get_device_state(
    State(app_state): State<DataLakeState>,
    Path(device_id): Path<String>,
) -> Result<Json<DeviceState>, StatusCode> {
    let data_lake = app_state.data_lake.read().await;
    data_lake
        .get_latest_device_state(&device_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_device_states_range(
    State(app_state): State<DataLakeState>,
    Path(device_id): Path<String>,
    Query(params): Query<QueryRangeParams>,
) -> Result<Json<Vec<DeviceState>>, StatusCode> {
    let data_lake = app_state.data_lake.read().await;
    let start_ts = params.start_ts.unwrap_or(0);
    let end_ts = params.end_ts.unwrap_or(current_timestamp());

    let states = data_lake
        .get_device_states_range(&device_id, start_ts, end_ts)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(states))
}

async fn store_sensor_data(
    State(app_state): State<DataLakeState>,
    Json(payload): Json<StoreSensorDataRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let data = SensorData {
        device_id: payload.device_id,
        sensor_type: payload.sensor_type,
        values: payload.values,
        timestamp: current_timestamp(),
    };

    let data_lake = app_state.data_lake.read().await;
    data_lake
        .store_sensor_data(&data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Sensor data stored"
    })))
}

async fn get_sensor_data(
    State(app_state): State<DataLakeState>,
    Path((device_id, _sensor_type)): Path<(String, String)>,
    Query(params): Query<QueryRangeParams>,
) -> Result<Json<Vec<SensorData>>, StatusCode> {
    let data_lake = app_state.data_lake.read().await;
    let start_ts = params.start_ts.unwrap_or(0);
    let end_ts = params.end_ts.unwrap_or(current_timestamp());

    let data = data_lake
        .get_sensor_data_range(&device_id, params.sensor_type.as_deref(), start_ts, end_ts)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(data))
}

pub async fn create_server(data_lake: DataLake, port: u16) -> ! {
    let state = DataLakeState {
        data_lake: Arc::new(RwLock::new(data_lake)),
    };

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/api/device/:device_id/state",
            post(store_device_state).get(get_device_state),
        )
        .route(
            "/api/device/:device_id/state/range",
            get(get_device_states_range),
        )
        .route(
            "/api/device/:device_id/sensor/:sensor_type",
            post(store_sensor_data).get(get_sensor_data),
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("Data Lake API server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.clone()).await.unwrap();

    unreachable!()
}
