/**
 * Arduino MQTT Example for ZeroInsect Bridge
 * 
 * This example shows how to send MQTT messages from Arduino (ESP32/ESP8266)
 * to the ZeroInsect MQTT broker.
 * 
 * Hardware: ESP32 or ESP8266
 * 
 * Required Libraries (via Arduino Library Manager):
 * - PubSubClient
 * - WiFi (built-in for ESP32/ESP8266)
 * 
 * Connections:
 * - ESP32/ESP8266 with WiFi
 * 
 * Topic: bridge/command/{device_id}
 * 
 * Example usage:
 * - Send: "move arm to position [0.5, 0.3, 0.2]"
 * - The Bridge Hub will parse intent via LLM and execute action
 */

#include <WiFi.h>
#include <PubSubClient.h>

// ============== CONFIGURATION ==============
// WiFi credentials
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// MQTT Broker (ZeroInsect Bridge Hub)
const char* mqtt_server = "192.168.1.100";  // Change to your broker IP
const int mqtt_port = 1883;
const char* mqtt_client_id = "arduino_esp32_001";
const char* mqtt_username = "user";           // Change if auth enabled
const char* mqtt_password = "password";       // Change if auth enabled

// Device configuration
const char* device_id = "esp32_robot";        // Unique device ID
const char* command_topic = "bridge/command/esp32_robot";
const char* status_topic = "bridge/status/esp32_robot";

// ============== VARIABLES ==============
WiFiClient espClient;
PubSubClient client(espClient);

unsigned long lastMsgTime = 0;
#define MSG_INTERVAL 5000  // Send message every 5 seconds

// ============== WIFI SETUP ==============
void setup_wifi() {
  delay(10);
  Serial.println();
  Serial.print("Connecting to WiFi: ");
  Serial.println(ssid);

  WiFi.mode(WIFI_STA);
  WiFi.begin(ssid, password);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  Serial.println("");
  Serial.println("WiFi connected");
  Serial.println("IP address: ");
  Serial.println(WiFi.localIP());
}

// ============== MQTT CALLBACK ==============
void callback(char* topic, byte* payload, unsigned int length) {
  Serial.print("Message arrived [");
  Serial.print(topic);
  Serial.print("] ");
  for (int i = 0; i < length; i++) {
    Serial.print((char)payload[i]);
  }
  Serial.println();
}

// ============== MQTT RECONNECT ==============
void reconnect() {
  while (!client.connected()) {
    Serial.print("Attempting MQTT connection...");
    
    if (client.connect(mqtt_client_id, mqtt_username, mqtt_password)) {
      Serial.println("connected");
      
      // Subscribe to command topic
      client.subscribe(command_topic);
      Serial.print("Subscribed to: ");
      Serial.println(command_topic);
      
      // Publish online status
      client.publish(status_topic, "{\"status\": \"online\"}");
    } else {
      Serial.print("failed, rc=");
      Serial.print(client.state());
      Serial.println(" try again in 5 seconds");
      delay(5000);
    }
  }
}

// ============== SEND SENSOR DATA ==============
void send_sensor_data() {
  // Simulate sensor data
  float temperature = 25.0 + random(0, 100) / 100.0;
  float humidity = 60.0 + random(0, 200) / 100.0;
  
  char payload[256];
  snprintf(payload, sizeof(payload),
    "{\"device_id\": \"%s\", \"sensors\": {\"temperature\": %.2f, \"humidity\": %.2f}, \"timestamp\": %lu}",
    device_id, temperature, humidity, millis());
  
  client.publish("bridge/topics/sensor_data", payload);
  Serial.println("Published sensor data");
}

// ============== SEND COMMAND RESPONSE ==============
void send_command_response(const char* command, const char* result) {
  char payload[512];
  snprintf(payload, sizeof(payload),
    "{\"device_id\": \"%s\", \"command\": \"%s\", \"result\": \"%s\", \"timestamp\": %lu}",
    device_id, command, result, millis());
  
  client.publish("bridge/response/arduino", payload);
  Serial.println("Published command response");
}

// ============== SETUP ==============
void setup() {
  Serial.begin(115200);
  
  setup_wifi();
  
  client.setServer(mqtt_server, mqtt_port);
  client.setCallback(callback);
  
  Serial.println("Arduino MQTT Example for ZeroInsect Bridge");
  Serial.print("Device ID: ");
  Serial.println(device_id);
  Serial.print("Command topic: ");
  Serial.println(command_topic);
}

// ============== MAIN LOOP ==============
void loop() {
  if (!client.connected()) {
    reconnect();
  }
  client.loop();

  unsigned long now = millis();
  if (now - lastMsgTime > MSG_INTERVAL) {
    lastMsgTime = now;
    
    // Send periodic sensor data
    send_sensor_data();
    
    // Example: Send a command to the bridge
    // Uncomment to test:
    // send_command_response("get_status", "ok");
  }
}

// ============== ALTERNATIVE: SEND CUSTOM MESSAGE ==============
/**
 * Use this function to send custom messages to the bridge
 * 
 * Example:
 *   send_to_bridge("move arm to [0.5, 0.3, 0.2]");
 */
void send_to_bridge(const char* message) {
  char payload[512];
  snprintf(payload, sizeof(payload),
    "{\"device_id\": \"%s\", \"message\": \"%s\", \"timestamp\": %lu}",
    device_id, message, millis());
  
  client.publish(command_topic, payload);
  Serial.print("Sent to bridge: ");
  Serial.println(message);
}
