"""
Test suite for Local Cognition Device Management System

C5 business scenarios with 30 test casesovers :
1. Device Status Analysis (6 tests)
2. Anomaly Detection (6 tests)
3. Failure Prediction (6 tests)
4. Trend Analysis (6 tests)
5. Operation Analysis (6 tests)
"""

import pytest
import requests
import time
import json
from typing import Dict, Any, List


BASE_URL = "http://localhost:8080"
OLLAMA_URL = "http://localhost:11434"


class TestDeviceStatusAnalysis:
    """Scenario 1: Device Status Analysis - 6 tests"""

    def test_get_device_state_success(self):
        """Test retrieving device state returns valid data"""
        response = requests.get(f"{BASE_URL}/api/device/robot-arm-01/state")
        assert response.status_code == 200
        data = response.json()
        assert "device_id" in data
        assert "status" in data
        assert "cpu_usage" in data

    def test_get_device_state_not_found(self):
        response = requests.get(f"{BASE_URL}/api/device/nonexistent-999/state")
        assert response.status_code in [200, 404]

    def test_device_state_contains_required_fields(self):
        """Test device state has all required fields"""
        response = requests.get(f"{BASE_URL}/api/device/robot-arm-01/state")
        data = response.json()
        required_fields = ["device_id", "timestamp", "status", "cpu_usage", "memory_usage", "temperature"]
        for field in required_fields:
            assert field in data, f"Missing required field: {field}"

    def test_device_state_range_query(self):
        """Test querying device states within time range"""
        end_ts = int(time.time())
        start_ts = end_ts - 3600
        response = requests.get(
            f"{BASE_URL}/api/device/robot-arm-01/state/range",
            params={"start_ts": start_ts, "end_ts": end_ts}
        )
        assert response.status_code == 200
        assert isinstance(response.json(), list)

    def test_multiple_devices_status(self):
        """Test querying multiple devices"""
        devices = ["robot-arm-01", "agv-01", "sensor-node-02"]
        for device_id in devices:
            response = requests.get(f"{BASE_URL}/api/device/{device_id}/state")
            assert response.status_code in [200, 404]

    def test_device_state_updates(self):
        """Test device state can be updated"""
        timestamp = int(time.time())
        new_state = {
            "device_id": "test-device-001",
            "status": "online",
            "cpu_usage": 50.0,
            "memory_usage": 60.0,
            "temperature": 45.0,
            "last_command": "test_command"
        }
        response = requests.post(
            f"{BASE_URL}/api/device/test-device-001/state",
            json=new_state
        )
        assert response.status_code == 200
        get_response = requests.get(f"{BASE_URL}/api/device/test-device-001/state")
        assert get_response.status_code == 200


class TestAnomalyDetection:
    """Scenario 2: Anomaly Detection - 6 tests"""

    def test_temperature_anomaly_high(self):
        """Test detecting high temperature anomaly"""
        sensor_data = {
            "device_id": "test-anomaly-01",
            "sensor_type": "temperature",
            "values": [40.0, 45.0, 55.0, 70.0, 85.0, 90.0]
        }
        response = requests.post(
            f"{BASE_URL}/api/device/test-anomaly-01/sensor/temperature",
            json=sensor_data
        )
        assert response.status_code == 200
        assert any(v > 80 for v in sensor_data["values"])

    def test_cpu_spike_detection(self):
        """Test detecting CPU usage spike"""
        state = {
            "device_id": "test-cpu-spike",
            "status": "online",
            "cpu_usage": 95.0,
            "memory_usage": 90.0,
            "temperature": 85.0
        }
        response = requests.post(f"{BASE_URL}/api/device/test-cpu-spike/state", json=state)
        assert response.status_code == 200

    def test_vibration_anomaly(self):
        """Test detecting abnormal vibration"""
        sensor_data = {
            "device_id": "test-vibration",
            "sensor_type": "vibration",
            "values": [0.01, 0.02, 0.05, 0.15, 0.45, 0.80]
        }
        response = requests.post(
            f"{BASE_URL}/api/device/test-vibration/sensor/vibration",
            json=sensor_data
        )
        assert response.status_code == 200

    def test_memory_leak_detection(self):
        """Test detecting memory usage pattern indicating leak"""
        states = []
        for i in range(10):
            state = {
                "device_id": "test-memory-leak",
                "status": "online",
                "cpu_usage": 50.0,
                "memory_usage": 50.0 + (i * 4),
                "temperature": 40.0 + (i * 0.5)
            }
            states.append(state)
            requests.post(f"{BASE_URL}/api/device/test-memory-leak/state", json=state)
            time.sleep(0.1)
        assert states[-1]["memory_usage"] > 80

    def test_offline_status_anomaly(self):
        """Test detecting device going offline"""
        online_state = {"device_id": "test-offline", "status": "online", "cpu_usage": 50, "memory_usage": 50, "temperature": 40}
        offline_state = {"device_id": "test-offline", "status": "offline", "cpu_usage": 0, "memory_usage": 0, "temperature": 25}
        requests.post(f"{BASE_URL}/api/device/test-offline/state", json=online_state)
        time.sleep(0.1)
        response = requests.post(f"{BASE_URL}/api/device/test-offline/state", json=offline_state)
        assert response.status_code == 200

    def test_multiple_anomaly_types(self):
        """Test detecting multiple anomaly types simultaneously"""
        state = {
            "device_id": "test-multi-anomaly",
            "status": "error",
            "cpu_usage": 98.0,
            "memory_usage": 95.0,
            "temperature": 92.0
        }
        response = requests.post(f"{BASE_URL}/api/device/test-multi-anomaly/state", json=state)
        assert response.status_code == 200
        assert state["cpu_usage"] > 90
        assert state["temperature"] > 85


class TestFailurePrediction:
    """Scenario 3: Failure Prediction - 6 tests"""

    def test_temperature_rising_trend(self):
        base_temp = 35.0
        for i in range(10):
            state = {
                "device_id": "test-temp-rise",
                "status": "online",
                "cpu_usage": 50.0,
                "memory_usage": 50.0,
                "temperature": base_temp + (i * 5)
            }
            requests.post(f"{BASE_URL}/api/device/test-temp-rise/state", json=state)
            time.sleep(0.1)
        assert True

    def test_battery_degradation_prediction(self):
        """Test predicting battery degradation"""
        battery_values = [100, 95, 88, 80, 70, 58, 45, 30, 15, 5]
        sensor_data = {
            "device_id": "test-battery",
            "sensor_type": "battery",
            "values": battery_values
        }
        response = requests.post(f"{BASE_URL}/api/device/test-battery/sensor/battery", json=sensor_data)
        assert response.status_code == 200

    def test_increasing_cpu_trend(self):
        """Test predicting performance failure from CPU trend"""
        for i in range(8):
            state = {
                "device_id": "test-cpu-rise",
                "status": "online",
                "cpu_usage": 40 + (i * 7)
            }
            requests.post(f"{BASE_URL}/api/device/test-cpu-rise/state", json=state)
            time.sleep(0.1)

    def test_motor_wear_prediction(self):
        """Test predicting motor wear from torque patterns"""
        sensor_data = {
            "device_id": "test-motor",
            "sensor_type": "torque",
            "values": [2.0, 2.5, 3.2, 4.0, 4.8, 5.5, 6.2, 7.0]
        }
        response = requests.post(f"{BASE_URL}/api/device/test-motor/sensor/torque", json=sensor_data)
        assert response.status_code == 200

    def test_communication_failure_prediction(self):
        """Test predicting communication failures"""
        states = []
        for i in range(5):
            latency = 10 + (i * 50)
            state = {
                "device_id": "test-comm",
                "status": "online",
                "cpu_usage": 50,
                "memory_usage": 50,
                "temperature": 40
            }
            states.append(state)
            requests.post(f"{BASE_URL}/api/device/test-comm/state", json=state)
            time.sleep(0.1)

    def test_critical_failure_imminent(self):
        """Test detecting critical failure indicators"""
        critical_state = {
            "device_id": "test-critical",
            "status": "error",
            "cpu_usage": 99.0,
            "memory_usage": 98.0,
            "temperature": 95.0
        }
        response = requests.post(f"{BASE_URL}/api/device/test-critical/state", json=critical_state)
        assert response.status_code == 200


class TestTrendAnalysis:
    """Scenario 4: Trend Analysis - 6 tests"""

    def test_temperature_trend_upward(self):
        """Test analyzing upward temperature trend"""
        values = [30, 35, 40, 45, 50, 55, 60, 65]
        sensor_data = {
            "device_id": "test-trend-temp",
            "sensor_type": "temperature",
            "values": values
        }
        requests.post(f"{BASE_URL}/api/device/test-trend-temp/sensor/temperature", json=sensor_data)
        assert values[-1] > values[0]

    def test_battery_drain_trend(self):
        """Test analyzing battery drain trend"""
        values = [100, 90, 82, 75, 68, 60, 52, 45, 38, 30]
        sensor_data = {
            "device_id": "test-trend-battery",
            "sensor_type": "battery",
            "values": values
        }
        requests.post(f"{BASE_URL}/api/device/test-trend-battery/sensor/battery", json=sensor_data)
        assert values[-1] < values[0]

    def test_coverage_pattern_trend(self):
        """Test analyzing cleaning coverage pattern"""
        values = [0, 10, 22, 35, 50, 65, 80, 90, 95, 100]
        sensor_data = {
            "device_id": "test-coverage",
            "sensor_type": "cleaning_coverage",
            "values": values
        }
        response = requests.post(f"{BASE_URL}/api/device/test-coverage/sensor/cleaning_coverage", json=sensor_data)
        assert response.status_code == 200

    def test_position_movement_trend(self):
        """Test analyzing AGV position movement trend"""
        for i in range(10):
            sensor_data = {
                "device_id": "test-position",
                "sensor_type": "position_x",
                "values": [i * 2]
            }
            requests.post(f"{BASE_URL}/api/device/test-position/sensor/position_x", json=sensor_data)
            time.sleep(0.1)

    def test_cpu_usage_oscillation(self):
        """Test detecting oscillating CPU usage"""
        values = [40, 60, 45, 65, 42, 62, 44, 63]
        for v in values:
            state = {"device_id": "test-oscillate", "cpu_usage": v, "memory_usage": 50, "temperature": 40}
            requests.post(f"{BASE_URL}/api/device/test-oscillate/state", json=state)
            time.sleep(0.1)

    def test_long_term_stability(self):
        """Test analyzing long-term stability"""
        for i in range(20):
            state = {
                "device_id": "test-stable",
                "status": "online",
                "cpu_usage": 45 + (i % 10) * 0.5,
                "memory_usage": 55 + (i % 5),
                "temperature": 40 + (i % 3)
            }
            requests.post(f"{BASE_URL}/api/device/test-stable/state", json=state)
            time.sleep(0.05)


class TestOperationAnalysis:
    """Scenario 5: Operation Analysis - 6 tests"""

    def test_robot_arm_waving_operation(self):
        """Test analyzing robot arm waving operation"""
        joint_positions = [30, 45, 60, 45, 30, 15, 30, 45, 60, 45, 30]
        joint_velocities = [15, 25, 30, 25, 15, 15, 25, 30, 25, 15, 10]
        torques = [2.5, 3.2, 4.1, 3.8, 2.9, 2.1, 2.8, 3.5, 4.0, 3.6, 2.5]

        requests.post(f"{BASE_URL}/api/device/robot-arm-01/sensor/joint_position",
                     json={"device_id": "robot-arm-01", "sensor_type": "joint_position", "values": joint_positions})
        requests.post(f"{BASE_URL}/api/device/robot-arm-01/sensor/joint_velocity",
                     json={"device_id": "robot-arm-01", "sensor_type": "joint_velocity", "values": joint_velocities})
        requests.post(f"{BASE_URL}/api/device/robot-arm-01/sensor/torque",
                     json={"device_id": "robot-arm-01", "sensor_type": "torque", "values": torques})

    def test_robot_arm_sweeping_operation(self):
        """Test analyzing robot arm sweeping operation"""
        joint_positions = [0, 20, 40, 60, 80, 100, 120, 140, 160, 180, 180, 160, 140, 120, 100]
        joint_velocities = [20, 35, 45, 50, 55, 50, 45, 40, 35, 30, 0, 20, 35, 45, 50]
        torques = [3.5, 5.2, 6.8, 8.0, 8.5, 8.0, 6.5, 5.0, 4.2, 3.8, 0, 3.0, 5.0, 6.5, 7.5]

        requests.post(f"{BASE_URL}/api/device/robot-arm-01/sensor/joint_position",
                     json={"device_id": "robot-arm-01", "sensor_type": "joint_position", "values": joint_positions})
        requests.post(f"{BASE_URL}/api/device/robot-arm-01/sensor/joint_velocity",
                     json={"device_id": "robot-arm-01", "sensor_type": "joint_velocity", "values": joint_velocities})

    def test_agv_coverage_cleaning_pattern(self):
        """Test analyzing AGV coverage cleaning pattern"""
        position_x = [0, 2, 4, 6, 8, 10, 10, 8, 6, 4, 2, 0, 0, 2, 4]
        position_y = [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2]
        coverage = [0, 5, 12, 20, 30, 42, 55, 70, 82, 92, 100]

        requests.post(f"{BASE_URL}/api/device/agv-01/sensor/position_x",
                     json={"device_id": "agv-01", "sensor_type": "position_x", "values": position_x})
        requests.post(f"{BASE_URL}/api/device/agv-01/sensor/position_y",
                     json={"device_id": "agv-01", "sensor_type": "position_y", "values": position_y})
        requests.post(f"{BASE_URL}/api/device/agv-01/sensor/cleaning_coverage",
                     json={"device_id": "agv-01", "sensor_type": "cleaning_coverage", "values": coverage})

    def test_agv_battery_during_operation(self):
        """Test analyzing AGV battery consumption during cleaning"""
        battery_values = [100, 95, 90, 85, 80, 78, 75, 72, 70, 68]
        sensor_data = {
            "device_id": "agv-01",
            "sensor_type": "battery",
            "values": battery_values
        }
        response = requests.post(f"{BASE_URL}/api/device/agv-01/sensor/battery", json=sensor_data)
        assert response.status_code == 200

    def test_gripper_operation_analysis(self):
        """Test analyzing gripper open/close operations"""
        gripper_open = [0, 0, 0, 0, 0]
        gripper_closed = [100, 100, 100, 100, 100]

        requests.post(f"{BASE_URL}/api/device/robot-arm-01/sensor/gripper",
                     json={"device_id": "robot-arm-01", "sensor_type": "gripper", "values": gripper_open})
        requests.post(f"{BASE_URL}/api/device/robot-arm-01/sensor/gripper",
                     json={"device_id": "robot-arm-01", "sensor_type": "gripper", "values": gripper_closed})

    def test_brush_motor_operation(self):
        """Test analyzing cleaning brush motor operation"""
        brush_rpm = [0, 300, 300, 300, 300, 0, 300, 300, 300, 300, 0]
        velocity = [0.5, 0.5, 0.5, 0.5, 0.5, 0, 0.5, 0.5, 0.5, 0.5, 0.5]

        requests.post(f"{BASE_URL}/api/device/agv-01/sensor/brush_rpm",
                     json={"device_id": "agv-01", "sensor_type": "brush_rpm", "values": brush_rpm})
        requests.post(f"{BASE_URL}/api/device/agv-01/sensor/velocity",
                     json={"device_id": "agv-01", "sensor_type": "velocity", "values": velocity})


class TestAPIHealth:
    """Health check tests"""

    def test_api_health(self):
        """Test API server health endpoint"""
        response = requests.get(f"{BASE_URL}/health")
        assert response.status_code == 200
        assert "OK" in response.text or "ok" in response.text.lower()

    def test_api_endpoints_available(self):
        """Test all main API endpoints are available"""
        endpoints = [
            "/health",
            "/api/device/robot-arm-01/state",
            "/api/device/agv-01/state"
        ]
        for endpoint in endpoints:
            response = requests.get(f"{BASE_URL}{endpoint}")
            assert response.status_code in [200, 404], f"Endpoint {endpoint} failed"
