//! Cadrage des messages (TS_MESSAGE) : `u32 size` (total), `u16 id`, `u8 checksum`, corps.

use std::sync::OnceLock;

pub const HEADER_LEN: usize = 7;
/// Paquets leurres envoyés par le client (TM_NONE) : à ignorer.
pub const ID_DUMMY: u16 = 9999;
/// Garde-fou contre un champ `size` aberrant.
pub const MAX_PACKET: usize = 1 << 20;

#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("paquet {id} tronqué : {needed} octets nécessaires, {available} disponibles")]
    Truncated { id: u16, needed: usize, available: usize },
    #[error("taille de paquet invalide : {0}")]
    BadSize(usize),
    #[error("paquet {id} : champ invalide ({field})")]
    Invalid { id: u16, field: &'static str },
}

pub type Result<T> = std::result::Result<T, PacketError>;

pub fn checksum(size: u32, id: u16) -> u8 {
    size.to_le_bytes()
        .iter()
        .chain(id.to_le_bytes().iter())
        .fold(0u8, |a, b| a.wrapping_add(*b))
}

static ENCODING: OnceLock<&'static encoding_rs::Encoding> = OnceLock::new();

/// Code page des chaînes du protocole (noms, chat) ; windows-1252 par défaut.
pub fn set_codepage(label: &str) {
    if let Some(enc) = encoding_rs::Encoding::for_label(label.as_bytes()) {
        let _ = ENCODING.set(enc);
    }
}

fn encoding() -> &'static encoding_rs::Encoding {
    ENCODING.get().copied().unwrap_or(encoding_rs::WINDOWS_1252)
}

pub fn encode_str(s: &str) -> Vec<u8> {
    encoding().encode(s).0.into_owned()
}

pub fn decode_str(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    encoding().decode_without_bom_handling(&b[..end]).0.into_owned()
}

/// Message reçu : identifiant + corps (sans l'en-tête).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawPacket {
    pub id: u16,
    pub body: Vec<u8>,
}

impl RawPacket {
    pub fn reader(&self) -> Reader<'_> {
        Reader { id: self.id, data: &self.body, pos: 0 }
    }

    /// Paquet complet (en-tête compris), en clair.
    pub fn to_bytes(&self) -> Vec<u8> {
        let size = (HEADER_LEN + self.body.len()) as u32;
        let mut out = Vec::with_capacity(size as usize);
        out.extend(size.to_le_bytes());
        out.extend(self.id.to_le_bytes());
        out.push(checksum(size, self.id));
        out.extend_from_slice(&self.body);
        out
    }
}

/// Construction du corps d'un message.
#[derive(Default)]
pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.buf.push(v);
        self
    }
    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.u8(v as u8)
    }
    pub fn u16(&mut self, v: u16) -> &mut Self {
        self.buf.extend(v.to_le_bytes());
        self
    }
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.buf.extend(v.to_le_bytes());
        self
    }
    pub fn i32(&mut self, v: i32) -> &mut Self {
        self.buf.extend(v.to_le_bytes());
        self
    }
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.buf.extend(v.to_le_bytes());
        self
    }
    pub fn i64(&mut self, v: i64) -> &mut Self {
        self.buf.extend(v.to_le_bytes());
        self
    }
    pub fn f32(&mut self, v: f32) -> &mut Self {
        self.buf.extend(v.to_le_bytes());
        self
    }
    pub fn bytes(&mut self, v: &[u8]) -> &mut Self {
        self.buf.extend_from_slice(v);
        self
    }
    pub fn zeros(&mut self, n: usize) -> &mut Self {
        self.buf.resize(self.buf.len() + n, 0);
        self
    }
    /// `char[n]` terminé par NUL (tronqué à n-1 caractères, comme `strncpy` + 0).
    pub fn fixed_str(&mut self, s: &str, n: usize) -> &mut Self {
        let mut b = encode_str(s);
        b.truncate(n.saturating_sub(1));
        b.resize(n, 0);
        self.bytes(&b)
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
    pub fn finish(self, id: u16) -> RawPacket {
        RawPacket { id, body: self.buf }
    }
}

/// Lecture d'un corps de message ; les lectures au-delà de la fin renvoient une erreur.
pub struct Reader<'a> {
    id: u16,
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos + n;
        let s = self.data.get(self.pos..end).ok_or(PacketError::Truncated {
            id: self.id,
            needed: HEADER_LEN + end,
            available: HEADER_LEN + self.data.len(),
        })?;
        self.pos = end;
        Ok(s)
    }
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn bool(&mut self) -> Result<bool> {
        Ok(self.u8()? != 0)
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn i64(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.take(n)
    }
    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.take(n).map(|_| ())
    }
    pub fn fixed_str(&mut self, n: usize) -> Result<String> {
        Ok(decode_str(self.take(n)?))
    }
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }
    pub fn rest(&mut self) -> &'a [u8] {
        let r = &self.data[self.pos..];
        self.pos = self.data.len();
        r
    }
}

/// Découpe un flux déchiffré en messages ; ignore les leurres (id 9999).
#[derive(Default)]
pub struct Framer {
    buf: Vec<u8>,
}

impl Framer {
    pub fn push(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn next_packet(&mut self) -> Result<Option<RawPacket>> {
        loop {
            if self.buf.len() < HEADER_LEN {
                return Ok(None);
            }
            let size = u32::from_le_bytes(self.buf[..4].try_into().unwrap()) as usize;
            if !(HEADER_LEN..=MAX_PACKET).contains(&size) {
                return Err(PacketError::BadSize(size));
            }
            if self.buf.len() < size {
                return Ok(None);
            }
            let id = u16::from_le_bytes([self.buf[4], self.buf[5]]);
            let body = self.buf[HEADER_LEN..size].to_vec();
            self.buf.drain(..size);
            if id == ID_DUMMY {
                continue;
            }
            return Ok(Some(RawPacket { id, body }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_vector() {
        assert_eq!(checksum(0x1b, 0x2711), 0x53);
    }

    #[test]
    fn framing_skips_dummies() {
        let mut w = Writer::new();
        w.u32(42);
        let real = w.finish(2).to_bytes();
        let mut dummy = RawPacket { id: ID_DUMMY, body: vec![1, 2, 3] }.to_bytes();
        let mut f = Framer::default();
        dummy.extend(&real[..5]);
        f.push(&dummy);
        assert!(f.next_packet().unwrap().is_none());
        f.push(&real[5..]);
        let p = f.next_packet().unwrap().unwrap();
        assert_eq!(p.id, 2);
        assert_eq!(p.reader().u32().unwrap(), 42);
    }

    #[test]
    fn fixed_strings() {
        let mut w = Writer::new();
        w.fixed_str("Épée", 6);
        let p = w.finish(1);
        assert_eq!(p.body.len(), 6);
        assert_eq!(p.reader().fixed_str(6).unwrap(), "Épée");
    }
}
