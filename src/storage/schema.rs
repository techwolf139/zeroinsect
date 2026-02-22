use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: i64,
    pub status: UserStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Online,
    Offline,
    Away,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub owner_id: String,
    pub name: String,
    pub members: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub msg_id: String,
    pub from_user_id: String,
    pub to_user_id: Option<String>,
    pub group_id: Option<String>,
    pub content: String,
    pub msg_type: MessageType,
    pub timestamp: i64,
    pub qos: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub client_id: String,
    pub user_id: Option<String>,
    pub subscriptions: Vec<String>,
    pub clean_session: bool,
    pub last_activity: i64,
}
