use crate::storage::kv_store::KvStore;
use crate::session::manager::SessionManager;
use crate::router::radix_tree::TopicTree;
use crate::types::PublishPacket;
use crate::storage::schema::{ChatMessage, MessageType};
use uuid::Uuid;
use anyhow::Result;

pub struct MessageDispatcher {
    store: KvStore,
    session_manager: SessionManager,
    topic_tree: TopicTree,
}

impl MessageDispatcher {
    pub fn new(store: KvStore, session_manager: SessionManager, topic_tree: TopicTree) -> Self {
        Self {
            store,
            session_manager,
            topic_tree,
        }
    }

    pub async fn dispatch(&self, publish: PublishPacket) -> Result<usize> {
        let topic = publish.topic.clone();
        let subscribers = self.topic_tree.get_subscribers(&topic);
        
        if subscribers.is_empty() {
            if let Some(user_id) = self.extract_user_from_topic(&topic) {
                if publish.qos > 0 {
                    self.store_offline_message(&user_id, &publish).await?;
                }
            }
            return Ok(0);
        }

        let mut delivered = 0;
        for client_id in subscribers {
            if let Some(handle) = self.session_manager.get_handle(&client_id).await {
                let mqtt_packet = crate::broker::packet::PacketParser::encode_publish(
                    &topic,
                    &publish.payload,
                    match publish.qos {
                        0 => mqttrs::QoS::AtMostOnce,
                        1 => mqttrs::QoS::AtLeastOnce,
                        _ => mqttrs::QoS::ExactlyOnce,
                    },
                    publish.packet_id,
                );
                
                let encoded = crate::broker::packet::PacketParser::new().encode(&mqtt_packet)?;
                let _ = handle.sender.send(encoded).await;
                delivered += 1;
            }
        }

        Ok(delivered)
    }

    fn extract_user_from_topic(&self, topic: &str) -> Option<String> {
        if topic.starts_with("chat/u/") {
            let parts: Vec<&str> = topic.split('/').collect();
            if parts.len() >= 3 {
                return Some(parts[2].to_string());
            }
        }
        None
    }

    async fn store_offline_message(&self, user_id: &str, publish: &PublishPacket) -> Result<()> {
        let msg = ChatMessage {
            msg_id: Uuid::new_v4().to_string(),
            from_user_id: "system".to_string(),
            to_user_id: Some(user_id.to_string()),
            group_id: None,
            content: String::from_utf8_lossy(&publish.payload).to_string(),
            msg_type: MessageType::Text,
            timestamp: chrono::Utc::now().timestamp(),
            qos: publish.qos,
        };
        
        self.store.save_offline_msg(user_id, &msg)?;
        Ok(())
    }
}
