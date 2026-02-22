use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::env;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct ChatMessage {
    from: String,
    content: String,
    timestamp: i64,
}

impl ChatMessage {
    fn new(from: &str, content: &str) -> Self {
        Self {
            from: from.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    fn to_json(&self) -> String {
        serde_json::json!({
            "from": self.from,
            "content": self.content,
            "timestamp": self.timestamp,
        })
        .to_string()
    }

    fn from_json(json: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(json).ok()?;
        Some(Self {
            from: value["from"].as_str()?.to_string(),
            content: value["content"].as_str()?.to_string(),
            timestamp: value["timestamp"].as_i64()?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let username = env::args()
        .nth(1)
        .unwrap_or_else(|| "Anonymous".to_string());

    let broker_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1".to_string());

    println!("=== MQTT Chat Client ===");
    println!("Username: {}", username);
    println!("Broker: {}:1883", broker_addr);
    println!();

    let mut mqttoptions = MqttOptions::new(
        format!("chat_client_{}", username),
        broker_addr,
        1883,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_credentials(&username, "password");

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 100);

    let personal_topic = format!("chat/u/{}", username);
    let broadcast_topic = "chat/broadcast";

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                println!("✓ Connected to MQTT broker");
                break;
            }
            Ok(Event::Incoming(Packet::Disconnect)) => {
                println!("✗ Connection rejected");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
                return Err(e.into());
            }
            _ => {}
        }
    }

    client.subscribe(&personal_topic, QoS::AtMostOnce).await?;
    client.subscribe(broadcast_topic, QoS::AtMostOnce).await?;

    println!("✓ Subscribed to:");
    println!("  - {} (personal inbox)", personal_topic);
    println!("  - {} (broadcast)", broadcast_topic);
    println!();
    println!("=== Commands ===");
    println!("  send <user> <message>  - Send private message");
    println!("  broadcast <message>     - Send broadcast message");
    println!("  whoami                  - Show your username");
    println!("  quit                    - Exit chat");
    println!();
    println!("=== Start chatting! ===");
    println!();

    let (tx, mut rx) = mpsc::channel::<String>(100);

    let tx_clone = tx.clone();
    let username_clone = username.clone();
    tokio::spawn(async move {
        use std::io::{self, Write};
        
        loop {
            print!("{}> ", username_clone);
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim().to_string();
            if input.is_empty() {
                continue;
            }
            
            if tx_clone.send(input).await.is_err() {
                break;
            }
        }
    });

    loop {
        tokio::select! {
            event = eventloop.poll() => {
                match event {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        let topic = publish.topic.clone();
                        let payload = String::from_utf8_lossy(&publish.payload);
                        
                        if let Some(msg) = ChatMessage::from_json(&payload) {
                            if msg.from != username {
                                println!("\r[{}] {}: {}", 
                                    if topic.contains("broadcast") { "BROADCAST" } else { "PRIVATE" },
                                    msg.from, 
                                    msg.content
                                );
                                print!("{}> ", username);
                            }
                        }
                    }
                    Ok(Event::Incoming(Packet::SubAck(_))) => {}
                    Ok(Event::Incoming(Packet::Disconnect)) => {
                        println!("\n✗ Disconnected from broker");
                        break;
                    }
                    Err(e) => {
                        eprintln!("\nConnection error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            
            Some(input) = rx.recv() => {
                let parts: Vec<&str> = input.splitn(3, ' ').collect();
                
                match parts[0].to_lowercase().as_str() {
                    "send" => {
                        if parts.len() < 3 {
                            println!("Usage: send <user> <message>");
                            continue;
                        }
                        let target = parts[1];
                        let message = parts[2];
                        
                        let chat_msg = ChatMessage::new(&username, message);
                        let topic = format!("chat/u/{}", target);
                        
                        client.publish(&topic, QoS::AtMostOnce, false, chat_msg.to_json()).await?;
                        println!("✓ Sent to {}: {}", target, message);
                    }
                    "broadcast" => {
                        if parts.len() < 2 {
                            println!("Usage: broadcast <message>");
                            continue;
                        }
                        let message = parts[1];
                        
                        let chat_msg = ChatMessage::new(&username, message);
                        client.publish("chat/broadcast", QoS::AtMostOnce, false, chat_msg.to_json()).await?;
                        println!("✓ Broadcast sent: {}", message);
                    }
                    "whoami" => {
                        println!("Your username: {}", username);
                    }
                    "quit" | "exit" => {
                        println!("Goodbye!");
                        client.disconnect().await?;
                        break;
                    }
                    _ => {
                        println!("Unknown command. Use 'help' for available commands.");
                    }
                }
            }
        }
    }

    Ok(())
}
