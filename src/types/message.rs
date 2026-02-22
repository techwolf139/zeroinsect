use serde::{Deserialize, Serialize};

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
