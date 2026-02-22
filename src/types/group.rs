use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub name: String,
    pub owner_id: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
pub members: Vec<GroupMember>,
    pub created_at: i64,
    pub settings: GroupSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: String,
    pub role: GroupRole,
    pub joined_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupRole {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    pub max_members: u32,
    pub allow_invite: bool,
    pub message_history_visible: bool,
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            max_members: 500,
            allow_invite: true,
            message_history_visible: true,
        }
    }
}
