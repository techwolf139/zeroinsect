use crate::storage::schema::{ChatMessage, GroupInfo, SessionState, UserProfile};
use anyhow::Result;
use rocksdb::{Options, DB};
use std::sync::Arc;

const DB_PATH: &str = "./rmc_storage";

#[derive(Clone)]
pub struct KvStore {
    db: Arc<DB>,
}

impl KvStore {
    pub fn new() -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_keep_log_file_num(10);
        opts.increase_parallelism(4);
        opts.set_max_open_files(10000);

        let db = DB::open(&opts, DB_PATH)?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn new_temp() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, temp_dir.path())?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn save_user(&self, user: &UserProfile) -> Result<()> {
        let key = format!("user:{}", user.user_id);
        let value = serde_json::to_vec(user)?;
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn get_user(&self, user_id: &str) -> Result<Option<UserProfile>> {
        let key = format!("user:{}", user_id);
        match self.db.get(key)? {
            Some(value) => {
                let user: UserProfile = serde_json::from_slice(&value)?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub fn update_user_status(
        &self,
        user_id: &str,
        status: crate::storage::schema::UserStatus,
    ) -> Result<()> {
        if let Some(mut user) = self.get_user(user_id)? {
            user.status = status;
            self.save_user(&user)?;
        }
        Ok(())
    }

    pub fn save_group(&self, group: &GroupInfo) -> Result<()> {
        let key = format!("group:{}", group.group_id);
        let value = serde_json::to_vec(group)?;
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn get_group(&self, group_id: &str) -> Result<Option<GroupInfo>> {
        let key = format!("group:{}", group_id);
        match self.db.get(key)? {
            Some(value) => {
                let group: GroupInfo = serde_json::from_slice(&value)?;
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    pub fn add_group_member(&self, group_id: &str, user_id: &str) -> Result<()> {
        if let Some(mut group) = self.get_group(group_id)? {
            if !group.members.contains(&user_id.to_string()) {
                group.members.push(user_id.to_string());
                self.save_group(&group)?;
            }
        }
        Ok(())
    }

    pub fn save_offline_msg(&self, target_user_id: &str, msg: &ChatMessage) -> Result<()> {
        let key = format!(
            "inbox:{}:{:020}:{}",
            target_user_id, msg.timestamp, msg.msg_id
        );
        let value = serde_json::to_vec(msg)?;
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn pop_offline_msgs(&self, target_user_id: &str) -> Result<Vec<ChatMessage>> {
        let prefix = format!("inbox:{}:", target_user_id);
        let mut messages = Vec::new();

        let iter = self.db.prefix_iterator(prefix.as_bytes());

        for item in iter {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);

            if !key_str.starts_with(&prefix) {
                break;
            }

            let msg: ChatMessage = serde_json::from_slice(&value)?;
            messages.push(msg);

            self.db.delete(key)?;
        }

        Ok(messages)
    }

    pub fn count_offline_msgs(&self, target_user_id: &str) -> Result<usize> {
        let prefix = format!("inbox:{}:", target_user_id);
        let mut count = 0;

        let iter = self.db.prefix_iterator(prefix.as_bytes());
        for item in iter {
            let (key, _) = item?;
            let key_str = String::from_utf8_lossy(&key);

            if !key_str.starts_with(&prefix) {
                break;
            }
            count += 1;
        }

        Ok(count)
    }

    pub fn save_message_history(&self, chat_id: &str, msg: &ChatMessage) -> Result<()> {
        let key = format!("history:{}:{:020}:{}", chat_id, msg.timestamp, msg.msg_id);
        let value = serde_json::to_vec(msg)?;
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn get_message_history(&self, chat_id: &str, limit: usize) -> Result<Vec<ChatMessage>> {
        let prefix = format!("history:{}:", chat_id);
        let mut messages = Vec::new();

        let iter = self.db.prefix_iterator(prefix.as_bytes());
        for item in iter.take(limit) {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);

            if !key_str.starts_with(&prefix) {
                break;
            }

            let msg: ChatMessage = serde_json::from_slice(&value)?;
            messages.push(msg);
        }

        messages.sort_by_key(|m| m.timestamp);
        Ok(messages)
    }

    pub fn save_session(&self, session: &SessionState) -> Result<()> {
        let key = format!("session:{}", session.client_id);
        let value = serde_json::to_vec(session)?;
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn get_session(&self, client_id: &str) -> Result<Option<SessionState>> {
        let key = format!("session:{}", client_id);
        match self.db.get(key)? {
            Some(value) => {
                let session: SessionState = serde_json::from_slice(&value)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub fn delete_session(&self, client_id: &str) -> Result<()> {
        let key = format!("session:{}", client_id);
        self.db.delete(key)?;
        Ok(())
    }
}
