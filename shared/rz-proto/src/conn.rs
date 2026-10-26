//! Connexion TCP chiffrée (deux états XRC4 indépendants, un par sens), au-dessus de tokio.

use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::cipher::XRc4;
use crate::msg::Message;
use crate::packet::{Framer, RawPacket};

pub struct PacketReader {
    half: OwnedReadHalf,
    cipher: XRc4,
    framer: Framer,
    buf: Vec<u8>,
}

impl PacketReader {
    /// Prochain message (les leurres 9999 sont ignorés) ; `None` si la connexion est fermée.
    pub async fn recv(&mut self) -> io::Result<Option<RawPacket>> {
        loop {
            match self.framer.next_packet() {
                Ok(Some(p)) => return Ok(Some(p)),
                Ok(None) => {}
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            }
            let n = self.half.read(&mut self.buf).await?;
            if n == 0 {
                return Ok(None);
            }
            let chunk = &mut self.buf[..n];
            self.cipher.apply(chunk);
            self.framer.push(chunk);
        }
    }
}

pub struct PacketWriter {
    half: OwnedWriteHalf,
    cipher: XRc4,
}

impl PacketWriter {
    pub async fn send_raw(&mut self, packet: &RawPacket) -> io::Result<()> {
        let mut bytes = packet.to_bytes();
        self.cipher.apply(&mut bytes);
        self.half.write_all(&bytes).await
    }

    pub async fn send<M: Message>(&mut self, msg: &M) -> io::Result<()> {
        self.send_raw(&msg.to_packet()).await
    }

    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.half.shutdown().await
    }
}

/// Sépare un flux en lecteur/écrivain chiffrés avec la clé donnée.
pub fn split(stream: TcpStream, key: &[u8]) -> (PacketReader, PacketWriter) {
    let _ = stream.set_nodelay(true);
    let (r, w) = stream.into_split();
    (
        PacketReader { half: r, cipher: XRc4::new(key), framer: Framer::default(), buf: vec![0u8; 16 * 1024] },
        PacketWriter { half: w, cipher: XRc4::new(key) },
    )
}
