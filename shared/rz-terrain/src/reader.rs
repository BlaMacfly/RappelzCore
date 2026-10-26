//! Lecture little-endian bornée.

use crate::{Result, TerrainError};

pub(crate) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
    what: &'static str,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8], what: &'static str) -> Self {
        Self { buf, pos: 0, what }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn seek(&mut self, pos: usize) -> Result<()> {
        if pos > self.buf.len() {
            return Err(self.truncated(pos - self.pos.min(pos)));
        }
        self.pos = pos;
        Ok(())
    }

    fn truncated(&self, need: usize) -> TerrainError {
        TerrainError::Truncated { what: self.what, offset: self.pos, need, len: self.buf.len() }
    }

    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.remaining() < n {
            return Err(self.truncated(n));
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16> {
        let b = self.bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn i16(&mut self) -> Result<i16> {
        Ok(self.u16()? as i16)
    }

    pub fn u32(&mut self) -> Result<u32> {
        let b = self.bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn i32(&mut self) -> Result<i32> {
        Ok(self.u32()? as i32)
    }

    pub fn u64(&mut self) -> Result<u64> {
        let lo = self.u32()? as u64;
        let hi = self.u32()? as u64;
        Ok(lo | (hi << 32))
    }

    pub fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.u32()?))
    }

    pub fn vec3(&mut self) -> Result<[f32; 3]> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    /// Compteur u32 dont chaque élément occupe au moins `min_item` octets : refuse les valeurs
    /// impossibles avant toute allocation.
    pub fn count(&mut self, min_item: usize) -> Result<usize> {
        let n = self.u32()? as usize;
        if n.saturating_mul(min_item.max(1)) > self.remaining() {
            return Err(TerrainError::Inconsistent(format!(
                "{} : compteur {} à l'octet {} incompatible avec les {} octet(s) restants",
                self.what,
                n,
                self.pos - 4,
                self.remaining()
            )));
        }
        Ok(n)
    }

    /// Chaîne préfixée par sa longueur (u32), décodée EUC-KR.
    pub fn string(&mut self) -> Result<String> {
        let n = self.count(1)?;
        Ok(crate::decode_text(self.bytes(n)?))
    }

    /// Vérifie que tout le tampon a été lu.
    pub fn finish(&self) -> Result<()> {
        if self.pos != self.buf.len() {
            return Err(TerrainError::TrailingBytes { what: self.what, consumed: self.pos, len: self.buf.len() });
        }
        Ok(())
    }
}
