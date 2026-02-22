use crate::network::tcp::TcpTransport;
use crate::broker::packet::PacketParser;
use crate::broker::auth::Authenticator;
use crate::session::manager::SessionManager;
use crate::router::radix_tree::TopicTree;
use crate::storage::kv_store::KvStore;
use crate::storage::schema::UserStatus;

use mqttrs::*;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::net::SocketAddr;

pub struct BrokerServer {
    transport: TcpTransport,
    auth: Arc<Authenticator>,
    session_manager: Arc<SessionManager>,
    topic_tree: Arc<TopicTree>,
    store: Arc<KvStore>,
}

impl BrokerServer {
    pub fn new(store: KvStore) -> Self {
        let auth = Arc::new(Authenticator::new(store.clone()));
        let session_manager = Arc::new(SessionManager::new());
        let topic_tree = Arc::new(TopicTree::new());
        
        Self {
            transport: TcpTransport::new(),
            auth,
            session_manager,
            topic_tree,
            store: Arc::new(store),
        }
    }

    pub async fn start(&self, addr: &str) -> Result<()> {
        let listener = self.transport.listen(addr).await?;
        println!("MQTT Broker listening on {}", addr);
        
        loop {
            let (stream, addr) = listener.accept().await?;
            
            let auth = self.auth.clone();
            let session_manager = self.session_manager.clone();
            let topic_tree = self.topic_tree.clone();
            let store = self.store.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(
                    stream,
                    addr,
                    auth,
                    session_manager,
                    topic_tree,
                    store,
                ).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        auth: Arc<Authenticator>,
        session_manager: Arc<SessionManager>,
        topic_tree: Arc<TopicTree>,
        store: Arc<KvStore>,
    ) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(8192);
        let mut client_id: Option<String> = None;
        let mut authenticated_user: Option<String> = None;
        
        loop {
            match Self::read_mqtt_packet(&mut stream, &mut buffer).await {
                Ok(Some(packet)) => {
                    match packet {
                        Packet::Connect(connect) => {
                            client_id = Some(connect.client_id.to_string());
                            
                            let username = connect.username.as_deref().unwrap_or("anonymous");
                            let password = connect.password.map(|p| std::str::from_utf8(p).unwrap_or("password")).unwrap_or("password");
                            
                            // Auto-register user if not exists (for demo purposes)
                            let auth_result = match auth.authenticate(username, password).await {
                                crate::broker::auth::AuthResult::Success(user_id) => {
                                    crate::broker::auth::AuthResult::Success(user_id)
                                }
                                crate::broker::auth::AuthResult::UserNotFound => {
                                    // Auto-register new user
                                    match auth.register_user(username, password).await {
                                        Ok(user_id) => crate::broker::auth::AuthResult::Success(user_id),
                                        Err(_) => crate::broker::auth::AuthResult::InvalidCredentials,
                                    }
                                }
                                other => other,
                            };
                            
                            match auth_result {
                                crate::broker::auth::AuthResult::Success(user_id) => {
                                    authenticated_user = Some(user_id.clone());
                                    let _ = store.update_user_status(&user_id, UserStatus::Online);
                                    
                                    let connack = PacketParser::encode_connack(false, 0);
                                    let encoded = PacketParser::new().encode(&connack)?;
                                    eprintln!("[DEBUG] Sending CONNACK: {:?}", &encoded[..encoded.len().min(20)]);
                                    stream.write_all(&encoded).await?;
                                    stream.flush().await?;
                                    
                                    println!("Client {} connected as user {}", addr, user_id);
                                }
                                _ => {
                                    let connack = PacketParser::encode_connack(false, 4);
                                    stream.write_all(&PacketParser::new().encode(&connack)?).await?;
                                    break;
                                }
                            }
                        }
                        Packet::Publish(publish) => {
                            if authenticated_user.is_none() {
                                break;
                            }
                            
                            let topic = publish.topic_name.to_string();
                            let payload = publish.payload.to_vec();
                            let qos = match publish.qospid {
                                QosPid::AtMostOnce => 0,
                                QosPid::AtLeastOnce(_) => 1,
                                QosPid::ExactlyOnce(_) => 2,
                            };
                            
                            let subscribers = topic_tree.get_subscribers(&topic);
                            for sub_client_id in subscribers {
                                if let Some(handle) = session_manager.get_handle(&sub_client_id).await {
                                    let mqtt_packet = PacketParser::encode_publish(
                                        &topic,
                                        &payload,
                                        match qos {
                                            0 => QoS::AtMostOnce,
                                            1 => QoS::AtLeastOnce,
                                            _ => QoS::ExactlyOnce,
                                        },
                                        None,
                                    );
                                    if let Ok(encoded) = PacketParser::new().encode(&mqtt_packet) {
                                        let _ = handle.send(encoded).await;
                                    }
                                }
                            }
                            
                            if qos > 0 {
                                if let QosPid::AtLeastOnce(pid) | QosPid::ExactlyOnce(pid) = publish.qospid {
                                    let puback = PacketParser::encode_puback(pid.get());
                                    stream.write_all(&PacketParser::new().encode(&puback)?).await?;
                                }
                            }
                        }
                        Packet::Subscribe(subscribe) => {
                            if authenticated_user.is_none() {
                                break;
                            }
                            
                            let cid = client_id.as_ref().ok_or_else(|| anyhow!("No client ID"))?;
                            
                            for sub_topic in subscribe.topics.iter() {
                                topic_tree.insert(&sub_topic.topic_path, cid);
                                println!("Client {} subscribed to {}", cid, sub_topic.topic_path);
                            }
                            
                            let return_codes: Vec<SubscribeReturnCodes> = subscribe.topics.iter()
                                .map(|sub| SubscribeReturnCodes::Success(sub.qos))
                                .collect();
                            let suback = PacketParser::encode_suback(subscribe.pid.get(), return_codes);
                            stream.write_all(&PacketParser::new().encode(&suback)?).await?;
                        }
                        Packet::Unsubscribe(unsubscribe) => {
                            if authenticated_user.is_none() {
                                break;
                            }
                            
                            let cid = client_id.as_ref().ok_or_else(|| anyhow!("No client ID"))?;
                            
                            for topic in unsubscribe.topics.iter() {
                                topic_tree.remove(topic, cid);
                            }
                            
                            let unsuback = PacketParser::encode_unsuback(unsubscribe.pid.get());
                            stream.write_all(&PacketParser::new().encode(&unsuback)?).await?;
                        }
                        Packet::Pingreq => {
                            let pingresp = PacketParser::encode_pingresp();
                            stream.write_all(&PacketParser::new().encode(&pingresp)?).await?;
                        }
                        Packet::Disconnect => {
                            if let Some(ref cid) = client_id {
                                session_manager.remove_session(cid).await;
                                if let Some(ref user_id) = authenticated_user {
                                    let _ = store.update_user_status(user_id, UserStatus::Offline);
                                }
                            }
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
        
        if let Some(ref cid) = client_id {
            session_manager.remove_session(cid).await;
            if let Some(ref user_id) = authenticated_user {
                let _ = store.update_user_status(user_id, UserStatus::Offline);
            }
        }
        
        Ok(())
    }

    async fn read_mqtt_packet<'a>(stream: &mut TcpStream, buffer: &'a mut BytesMut) -> Result<Option<Packet<'a>>> {
        let mut temp = vec![0u8; 4096];
        
        let n = stream.read(&mut temp).await?;
        if n == 0 {
            return Ok(None);
        }
        
        buffer.extend_from_slice(&temp[..n]);
        
        match mqttrs::decode_slice(buffer) {
            Ok(Some(pkt)) => Ok(Some(pkt)),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow!("Decode error: {}", e)),
        }
    }
}
