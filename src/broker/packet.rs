use bytes::BytesMut;
use mqttrs::*;

pub struct PacketParser;

impl PacketParser {
    pub fn new() -> Self {
        Self
    }

    pub fn decode<'a>(&self, data: &[u8], buf: &'a mut BytesMut) -> Result<Option<Packet<'a>>, mqttrs::Error> {
        buf.clear();
        buf.extend_from_slice(data);
        decode_slice(buf)
    }

    pub fn encode(&self, packet: &Packet<'_>) -> Result<Vec<u8>, mqttrs::Error> {
        let mut buf = vec![0u8; 4096];
        let len = encode_slice(packet, &mut buf)?;
        buf.truncate(len);
        Ok(buf)
    }

    pub fn encode_connect(client_id: &str, clean_session: bool, keep_alive: u16) -> Packet<'_> {
        Packet::Connect(Connect {
            protocol: Protocol::MQTT311,
            keep_alive,
            client_id,
            clean_session,
            last_will: None,
            username: None,
            password: None,
        })
    }

    pub fn encode_connack(session_present: bool, return_code: u8) -> Packet<'static> {
        let code = match return_code {
            0 => ConnectReturnCode::Accepted,
            1 => ConnectReturnCode::RefusedProtocolVersion,
            2 => ConnectReturnCode::RefusedIdentifierRejected,
            3 => ConnectReturnCode::ServerUnavailable,
            4 => ConnectReturnCode::BadUsernamePassword,
            5 => ConnectReturnCode::NotAuthorized,
            _ => ConnectReturnCode::BadUsernamePassword,
        };
        Packet::Connack(Connack {
            session_present,
            code,
        })
    }

    pub fn encode_publish<'a>(
        topic: &'a str,
        payload: &'a [u8],
        qos: QoS,
        packet_id: Option<u16>,
    ) -> Packet<'a> {
        let qospid = match (qos, packet_id) {
            (QoS::AtMostOnce, _) => QosPid::AtMostOnce,
            (QoS::AtLeastOnce, Some(pid)) => QosPid::AtLeastOnce(Pid::try_from(pid).unwrap()),
            (QoS::AtLeastOnce, None) => QosPid::AtLeastOnce(Pid::try_from(0).unwrap()),
            (QoS::ExactlyOnce, Some(pid)) => QosPid::ExactlyOnce(Pid::try_from(pid).unwrap()),
            (QoS::ExactlyOnce, None) => QosPid::ExactlyOnce(Pid::try_from(0).unwrap()),
        };
        Packet::Publish(Publish {
            dup: false,
            qospid,
            retain: false,
            topic_name: topic,
            payload,
        })
    }

    pub fn encode_puback(packet_id: u16) -> Packet<'static> {
        Packet::Puback(Pid::try_from(packet_id).unwrap())
    }

    pub fn encode_suback(
        packet_id: u16,
        return_codes: Vec<SubscribeReturnCodes>,
    ) -> Packet<'static> {
        Packet::Suback(Suback {
            pid: Pid::try_from(packet_id).unwrap(),
            return_codes,
        })
    }

    pub fn encode_unsuback(packet_id: u16) -> Packet<'static> {
        Packet::Unsuback(Pid::try_from(packet_id).unwrap())
    }

    pub fn encode_pingresp() -> Packet<'static> {
        Packet::Pingresp
    }

    pub fn encode_disconnect() -> Packet<'static> {
        Packet::Disconnect
    }
}

impl Default for PacketParser {
    fn default() -> Self {
        Self::new()
    }
}
