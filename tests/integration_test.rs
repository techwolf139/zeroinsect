#[cfg(test)]
mod tests {
    use zeroinsect::broker::packet::PacketParser;
    use zeroinsect::storage::kv_store::KvStore;
    use zeroinsect::storage::schema::UserProfile;
    use zeroinsect::storage::schema::UserStatus;
    use mqttrs::*;

    #[test]
    fn test_packet_parser_encode_decode() {
        let parser = PacketParser::new();
        
        let publish = PacketParser::encode_publish(
            "test/topic",
            b"Hello World",
            QoS::AtLeastOnce,
            Some(1),
        );
        
        let encoded = parser.encode(&publish).unwrap();
        assert!(!encoded.is_empty());
        
        let mut buf = bytes::BytesMut::new();
        buf.extend_from_slice(&encoded);
        
        let decoded = mqttrs::decode_slice(&buf).unwrap();
        assert!(decoded.is_some());
        
        if let Some(Packet::Publish(p)) = decoded {
            assert_eq!(p.topic_name, "test/topic");
            assert_eq!(p.payload, b"Hello World");
        } else {
            panic!("Expected Publish packet");
        }
    }

    #[test]
    fn test_connack_packet() {
        let parser = PacketParser::new();
        
        let connack = PacketParser::encode_connack(false, 0);
        let encoded = parser.encode(&connack).unwrap();
        
        assert!(!encoded.is_empty());
    }

    #[tokio::test]
    async fn test_kv_store_user_operations() {
        let store = KvStore::new_temp().unwrap();
        
        let user = UserProfile {
            user_id: "test_user_123".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password".to_string(),
            created_at: 1234567890,
            status: UserStatus::Offline,
        };
        
        store.save_user(&user).unwrap();
        
        let retrieved = store.get_user("test_user_123").unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_user = retrieved.unwrap();
        assert_eq!(retrieved_user.username, "testuser");
    }

    #[tokio::test]
    async fn test_kv_store_offline_messages() {
        let store = KvStore::new_temp().unwrap();
        
        let msg = zeroinsect::storage::schema::ChatMessage {
            msg_id: "msg_001".to_string(),
            from_user_id: "user_a".to_string(),
            to_user_id: Some("user_b".to_string()),
            group_id: None,
            content: "Hello".to_string(),
            msg_type: zeroinsect::storage::schema::MessageType::Text,
            timestamp: 1234567890,
            qos: 1,
        };
        
        store.save_offline_msg("user_b", &msg).unwrap();
        
        let messages = store.pop_offline_msgs("user_b").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
    }
}
