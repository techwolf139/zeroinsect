use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::broker::packet::PacketParser;
use mqttrs::Packet;
use anyhow::{Result, anyhow};
use bytes::BytesMut;

pub struct TcpTransport {
    parser: PacketParser,
}

impl TcpTransport {
    pub fn new() -> Self {
        Self {
            parser: PacketParser::new(),
        }
    }

    pub async fn listen(&self, addr: &str) -> Result<TcpListener> {
        let listener = TcpListener::bind(addr).await?;
        Ok(listener)
    }

    pub async fn accept(&self, listener: &mut TcpListener) -> Result<(TcpStream, std::net::SocketAddr)> {
        let (stream, addr) = listener.accept().await?;
        Ok((stream, addr))
    }

    pub async fn read_packet<'a>(&self, stream: &mut TcpStream, buffer: &'a mut BytesMut) -> Result<Option<Packet<'a>>> {
        let mut temp = vec![0u8; 1024];
        
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

    pub async fn write_packet(&self, stream: &mut TcpStream, packet: &Packet<'_>) -> Result<()> {
        let data = self.parser.encode(packet)?;
        stream.write_all(&data).await?;
        stream.flush().await?;
        Ok(())
    }

    pub async fn close(&self, stream: &mut TcpStream) -> Result<()> {
        stream.shutdown().await?;
        Ok(())
    }
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new()
    }
}
