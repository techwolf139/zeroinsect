/**
 * ESP32 IMU MQTT Example for ZeroInsect Bridge
 * 
 * This example reads IMU sensor data (accelerometer, gyroscope)
 * and sends it via MQTT to the ZeroInsect Bridge Hub.
 * 
 * Hardware: ESP32 + IMU Sensor (MPU6050 or ICM20948)
 * 
 * Required Libraries (via Arduino Library Manager):
 * - PubSubClient
 * - WiFi (built-in)
 * - Adafruit MPU6050 (for MPU6050)
 * - Adafruit ICM20948 (for ICM20948)
 * - Adafruit Sensor (required for both)
 * 
 * Wiring (I2C):
 *   ESP32       MPU6050/ICM20948
 *   -----       ---------------
 *   GPIO21      SDA
 *   GPIO22      SCL
 *   3.3V        VCC
 *   GND         GND
 * 
 * Topics:
 *   bridge/topics/imu - IMU data
 *   bridge/status/{device_id} - Device status
 */

#include <WiFi.h>
#include <PubSubClient.h>
#include <Wire.h>

// ============== CONFIGURATION ==============
// WiFi credentials
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// MQTT Broker
const char* mqtt_server = "192.168.1.100";  // Change to your broker IP
const int mqtt_port = 1883;
const char* mqtt_client_id = "esp32_imu_001";
const char* mqtt_username = "user";
const char* mqtt_password = "password";

// Device configuration
const char* device_id = "esp32_imu_001";

// I2C pins (ESP32 default: SDA=21, SCL=22)
#define I2C_SDA 21
#define I2C_SCL 22

// ============== SELECT IMU SENSOR ==============
// Uncomment one of these:
// #define USE_MPU6050
#define USE_ICM20948

// ============== INCLUDE IMU LIBRARY ==============
#ifdef USE_MPU6050
#include <Adafruit_MPU6050.h>
Adafruit_MPU6050 mpu;
#endif

#ifdef USE_ICM20948
#include <Adafruit_ICM20948.h>
Adafruit_ICM20948 mpu;
#endif

// ============== MQTT ==============
WiFiClient espClient;
PubSubClient client(espClient);

unsigned long lastMsgTime = 0;
#define MSG_INTERVAL 50  // Send IMU data every 50ms (20Hz)

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

// ============== MQTT RECONNECT ==============
void reconnect() {
  while (!client.connected()) {
    Serial.print("Attempting MQTT connection...");
    
    if (client.connect(mqtt_client_id, mqtt_username, mqtt_password)) {
      Serial.println("connected");
      
      // Publish online status
      char status_topic[64];
      snprintf(status_topic, sizeof(status_topic), "bridge/status/%s", device_id);
      client.publish(status_topic, "{\"status\": \"online\", \"type\": \"imu_sensor\"}");
    } else {
      Serial.print("failed, rc=");
      Serial.print(client.state());
      Serial.println(" try again in 5 seconds");
      delay(5000);
    }
  }
}

// ============== READ IMU DATA ==============
void send_imu_data() {
#ifdef USE_MPU6050
  sensors_event_t a, g, temp;
  mpu.getEvent(&a, &g, &temp);

  // Create JSON payload
  char payload[512];
  snprintf(payload, sizeof(payload),
    "{"
    "\"device_id\": \"%s\","
    "\"timestamp\": %lu,"
    "\"imu\": {"
    "\"accelerometer\": {"
    "\"x\": %.4f,"
    "\"y\": %.4f,"
    "\"z\": %.4f"
    "},"
    "\"gyroscope\": {"
    "\"x\": %.4f,"
    "\"y\": %.4f,"
    "\"z\": %.4f"
    "},"
    "\"temperature\": %.2f"
    "}"
    "}",
    device_id, millis(),
    a.acceleration.x, a.acceleration.y, a.acceleration.z,
    g.gyro.x, g.gyro.y, g.gyro.z,
    temp.temperature
  );
#endif

#ifdef USE_ICM20948
  sensors_event_t accel, gyro, temp;
  mpu.getEvent(&accel, &gyro, &temp);

  char payload[512];
  snprintf(payload, sizeof(payload),
    "{"
    "\"device_id\": \"%s\","
    "\"timestamp\": %lu,"
    "\"imu\": {"
    "\"accelerometer\": {"
    "\"x\": %.4f,"
    "\"y\": %.4f,"
    "\"z\": %.4f"
    "},"
    "\"gyroscope\": {"
    "\"x\": %.4f,"
    "\"y\": %.4f,"
    "\"z\": %.4f"
    "},"
    "\"temperature\": %.2f"
    "}"
    "}",
    device_id, millis(),
    accel.acceleration.x, accel.acceleration.y, accel.acceleration.z,
    gyro.gyro.x, gyro.gyro.y, gyro.gyro.z,
    temp.temperature
  );
#endif

  client.publish("bridge/topics/imu", payload);
}

// ============== INIT IMU SENSOR ==============
bool init_imu() {
  // Initialize I2C
  Wire.begin(I2C_SDA, I2C_SCL);

#ifdef USE_MPU6050
  Serial.println("Initializing MPU6050...");
  if (!mpu.begin(0x68, &Wire)) {
    Serial.println("MPU6050 not found!");
    return false;
  }
  mpu.setAccelerometerRange(MPU6050_RANGE_2_G);
  mpu.setGyroRange(MPU6050_RANGE_250_DEG);
  mpu.setFilterBandwidth(MPU6050_BAND_5_HZ);
  Serial.println("MPU6050 initialized!");
#endif

#ifdef USE_ICM20948
  Serial.println("Initializing ICM20948...");
  if (!mpu.begin_ICM20948(&Wire)) {
    Serial.println("ICM20948 not found!");
    return false;
  }
  mpu.setAccelRateDivisor(0xFF);  // 1.125 kHz
  mpu.setGyroRateDivisor(0xFF);    // 1.125 kHz
  Serial.println("ICM20948 initialized!");
#endif

  return true;
}

// ============== SETUP ==============
void setup() {
  Serial.begin(115200);
  delay(1000);
  
  // Initialize IMU
  if (!init_imu()) {
    Serial.println("IMU initialization failed!");
    while (1) delay(1000);
  }
  
  // Setup WiFi
  setup_wifi();
  
  // Setup MQTT
  client.setServer(mqtt_server, mqtt_port);
  
  Serial.println("ESP32 IMU MQTT Example for ZeroInsect Bridge");
  Serial.print("Device ID: ");
  Serial.println(device_id);
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
    send_imu_data();
  }
}
