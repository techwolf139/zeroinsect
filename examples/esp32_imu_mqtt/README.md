# ESP32 IMU MQTT Example

This example reads IMU (Inertial Measurement Unit) sensor data from ESP32 and sends it via MQTT to the ZeroInsect Bridge Hub.

## Hardware

- **ESP32** development board
- **IMU Sensor**: MPU6050 or ICM20948 (I2C)

## Required Libraries

Install via Arduino Library Manager:

- **PubSubClient** - MQTT client
- **Adafruit MPU6050** - For MPU6050 sensor
- **Adafruit ICM20948** - For ICM20948 sensor  
- **Adafruit Sensor** - Required dependency

## Wiring (I2C)

| ESP32 | IMU Sensor |
|-------|------------|
| GPIO21 | SDA |
| GPIO22 | SCL |
| 3.3V | VCC |
| GND | GND |

## Configuration

Edit `esp32_imu_mqtt.ino`:

```cpp
// WiFi
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// MQTT Broker
const char* mqtt_server = "192.168.1.100";
```

## Select IMU Sensor

Uncomment ONE sensor in the code:

```cpp
// Uncomment one of these:
#define USE_MPU6050
// #define USE_ICM20948
```

## MQTT Topics

| Topic | Purpose |
|-------|---------|
| `bridge/topics/imu` | IMU sensor data |
| `bridge/status/{device_id}` | Device status |

## Data Format

```json
{
  "device_id": "esp32_imu_001",
  "timestamp": 1234567890,
  "imu": {
    "accelerometer": {
      "x": 0.1234,
      "y": -0.5678,
      "z": 9.8100
    },
    "gyroscope": {
      "x": 0.0100,
      "y": -0.0200,
      "z": 0.0050
    },
    "temperature": 25.50
  }
}
```

## Data Flow

```
┌──────────┐     MQTT      ┌─────────────┐     DDS      ┌─────────┐
│  ESP32   │ ──────────► │ Bridge Hub  │ ──────────► │ ROS Robot│
│  + IMU   │   imu data  │  + LLM      │  /scan     │         │
└──────────┘              └─────────────┘             └─────────┘
```

The Bridge Hub receives IMU data and can:
1. Forward to ROS topics (e.g., `/imu_raw`)
2. Use LLM to analyze sensor patterns
3. Route to other connected systems
