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
#[derive(Debug, Clone)]
pub struct PublishPacket {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
    pub packet_id: Option<u16>,
}
