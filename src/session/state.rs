use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub client_id: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub subscriptions: HashMap<String, u8>,
    pub clean_session: bool,
    pub connected: bool,
    pub keep_alive: u16,
    pub last_activity: i64,
}

impl SessionState {
    pub fn new(client_id: String, clean_session: bool, keep_alive: u16) -> Self {
        Self {
            client_id,
            user_id: None,
            username: None,
            subscriptions: HashMap::new(),
            clean_session,
            connected: false,
            keep_alive,
            last_activity: chrono::Utc::now().timestamp(),
        }
    }

    pub fn subscribe(&mut self, topic: String, qos: u8) {
        self.subscriptions.insert(topic, qos);
    }

    pub fn unsubscribe(&mut self, topic: &str) {
        self.subscriptions.remove(topic);
    }

    pub fn update_activity(&mut self) {
        self.last_activity = chrono::Utc::now().timestamp();
    }
}

pub struct SessionHandle {
    pub client_id: String,
    pub sender: mpsc::Sender<Vec<u8>>,
}

impl SessionHandle {
    pub fn new(client_id: String, sender: mpsc::Sender<Vec<u8>>) -> Self {
        Self { client_id, sender }
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<(), mpsc::error::SendError<Vec<u8>>> {
        self.sender.send(data).await
    }
}
