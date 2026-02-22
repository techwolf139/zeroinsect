# Arduino MQTT Example

This example shows how to connect an Arduino (ESP32/ESP8266) to the ZeroInsect MQTT Bridge and send/receive messages.

## Hardware

- **ESP32** or **ESP8266** development board
- WiFi connection

## Required Libraries

Install via Arduino Library Manager:

- **PubSubClient** - MQTT client library
- **WiFi** - Built-in for ESP32/ESP8266

## Configuration

Edit the following in `arduino_mqtt.ino`:

```cpp
// WiFi credentials
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// MQTT Broker (ZeroInsect Bridge Hub)
const char* mqtt_server = "192.168.1.100";  // Your broker IP
const int mqtt_port = 1883;
const char* mqtt_username = "user";
const char* mqtt_password = "password";

// Device ID
const char* device_id = "esp32_robot";
```

## MQTT Topics

| Topic | Direction | Purpose |
|-------|-----------|---------|
| `bridge/command/{device_id}` | Send | Send commands to bridge |
| `bridge/status/{device_id}` | Send | Device status updates |
| `bridge/response/{device_id}` | Receive | Command responses |
| `bridge/topics/sensor_data` | Send | Sensor data publishing |

## Usage

1. Configure WiFi and MQTT broker settings
2. Upload to ESP32/ESP8266
3. Open Serial Monitor (115200 baud)
4. Device will connect to WiFi and MQTT broker
5. Device publishes sensor data every 5 seconds

## Send Commands

Use `send_to_bridge()` function:

```cpp
send_to_bridge("move arm to [0.5, 0.3, 0.2]");
send_to_bridge("get status");
send_to_bridge("navigate to kitchen");
```

The Bridge Hub will:
1. Receive the message
2. Use LLM to parse intent
3. Execute appropriate action
4. Return result via response topic

## Example Flow

```
Arduino                           Bridge Hub                          ROS Robot
  |                                   |                                   |
  |--- publish ---------------------->|                                   |
  |    "move arm to [0.5,0.3,0.2]"  |                                   |
  |                                   |--- LLM intent parsing ---------->|
  |                                   |<-- Intent: MoveArm -------------|
  |                                   |--- execute action -------------->|
  |                                   |<-- result: success ------------|
  |<-- publish ----------------------|                                   |
  |    {"result": "success"}         |                                   |
```
