#!/usr/bin/env python3
import rclpy
from rclpy.node import Node
import json
import paho.mqtt.client as mqtt
import sys


class ROS1Adapter(Node):
    def __init__(self, device_id: str, mqtt_broker: str):
        super().__init__(f'ros1_adapter_{device_id}')
        self.device_id = device_id
        self.mqtt_broker = mqtt_broker
        self.mqtt_client = mqtt.Client()
        self.mqtt_client.on_connect = self._on_connect
        self.mqtt_client.on_disconnect = self._on_disconnect

    def _on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            self.get_logger().info(f'Connected to MQTT broker {self.mqtt_broker}')
            self.publish_capabilities()
        else:
            self.get_logger().error(f'Failed to connect to MQTT, return code: {rc}')

    def _on_disconnect(self, client, userdata, rc):
        self.get_logger().warning(f'Disconnected from MQTT broker, return code: {rc}')

    def connect(self):
        self.get_logger().info(f'Connecting to MQTT broker {self.mqtt_broker}...')
        self.mqtt_client.connect(self.mqtt_broker, 1883, 60)
        self.mqtt_client.loop_start()

    def disconnect(self):
        self.mqtt_client.loop_stop()
        self.mqtt_client.disconnect()

    def discover_capabilities(self):
        topic_names_and_types = self.get_topic_names_and_types()
        topics = [{'name': t[0], 'msg_type': t[1]} for t in topic_names_and_types]

        service_names = self.get_service_names_and_types()
        services = [{'name': s[0], 'srv_type': s[1]} for s in service_names]

        return {
            'ros_version': 'noetic',
            'device_id': self.device_id,
            'topics': topics,
            'services': services,
            'actions': []
        }

    def publish_capabilities(self):
        manifest = self.discover_capabilities()
        topic = f'bridge/capabilities/{self.device_id}'
        payload = json.dumps(manifest)
        self.mqtt_client.publish(topic, payload)
        self.get_logger().info(f'Published capabilities to {topic}')


def main(args=None):
    if len(sys.argv) < 3:
        print('Usage: ros1_adapter.py <device_id> <mqtt_broker>')
        sys.exit(1)

    device_id = sys.argv[1]
    mqtt_broker = sys.argv[2]

    rclpy.init(args=args)
    adapter = ROS1Adapter(device_id, mqtt_broker)
    adapter.connect()

    try:
        rclpy.spin(adapter)
    except KeyboardInterrupt:
        pass
    finally:
        adapter.disconnect()
        adapter.destroy_node()
        rclpy.shutdown()


if __name__ == '__main__':
    main()
