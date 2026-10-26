//! Fichiers WDBC, le format des bases client de World of Warcraft (`DBFilesClient\*.dbc`).
//!
//! Disposition : en-tête de 20 octets (`WDBC`, nb d'enregistrements, nb de champs,
//! taille d'enregistrement, taille du bloc de chaînes), enregistrements de taille fixe,
//! puis bloc de chaînes (qui commence par un `\0`). Les chaînes sont des offsets u32.
//!
//! Comme dans AzerothCore (`DBCfmt.h`), chaque table est décrite par une chaîne de format :
//! `n` index u32, `i` u32/i32, `f` f32, `s` chaîne, `b` u8, `x` u32 ignoré, `X` u8 ignoré.

use std::collections::HashMap;
use std::path::Path;

pub const DBC_MAGIC: &[u8; 4] = b"WDBC";

#[derive(Debug, thiserror::Error)]
pub enum DbcError {
    #[error("E/S : {0}")]
    Io(#[from] std::io::Error),
    #[error("signature WDBC absente")]
    BadMagic,
    #[error("fichier DBC tronqué")]
    Truncated,
    #[error("le format « {format} » décrit {expected} octets, le fichier en a {actual} par enregistrement")]
    FormatMismatch { format: String, expected: usize, actual: usize },
    #[error("valeur {index} incompatible avec le format « {format} »")]
    ValueMismatch { format: String, index: usize },
}

pub type Result<T> = std::result::Result<T, DbcError>;

/// Taille en octets d'un enregistrement décrit par `format`.
pub fn format_record_size(format: &str) -> usize {
    format.chars().map(|c| if matches!(c, 'b' | 'X') { 1 } else { 4 }).sum()
}

#[derive(Clone, Debug)]
pub struct DbcFile {
    pub field_count: u32,
    pub record_size: u32,
    records: Vec<u8>,
    strings: Vec<u8>,
    record_count: u32,
}

impl DbcFile {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(DbcError::Truncated);
        }
        if &data[..4] != DBC_MAGIC {
            return Err(DbcError::BadMagic);
        }
        let rd = |o: usize| u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        let (record_count, field_count, record_size, string_size) = (rd(4), rd(8), rd(12), rd(16));
        let rec_len = record_count as usize * record_size as usize;
        if data.len() < 20 + rec_len + string_size as usize {
            return Err(DbcError::Truncated);
        }
        Ok(Self {
            field_count,
            record_size,
            record_count,
            records: data[20..20 + rec_len].to_vec(),
            strings: data[20 + rec_len..20 + rec_len + string_size as usize].to_vec(),
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::parse(&std::fs::read(path)?)
    }

    /// Vérifie que la chaîne de format correspond à la taille d'enregistrement.
    pub fn check_format(&self, format: &str) -> Result<()> {
        let expected = format_record_size(format);
        if expected != self.record_size as usize {
            return Err(DbcError::FormatMismatch {
                format: format.to_string(),
                expected,
                actual: self.record_size as usize,
            });
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.record_count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.record_count == 0
    }

    pub fn record(&self, row: usize) -> Record<'_> {
        let size = self.record_size as usize;
        Record {
            data: &self.records[row * size..(row + 1) * size],
            strings: &self.strings,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Record<'_>> {
        (0..self.len()).map(|i| self.record(i))
    }
}

#[derive(Clone, Copy)]
pub struct Record<'a> {
    data: &'a [u8],
    strings: &'a [u8],
}

impl<'a> Record<'a> {
    pub fn u32_at(&self, offset: usize) -> u32 {
        let d = &self.data[offset..offset + 4];
        u32::from_le_bytes([d[0], d[1], d[2], d[3]])
    }

    /// Champ u32 numéro `field` (pour les tables sans champs `b`).
    pub fn u32(&self, field: usize) -> u32 {
        self.u32_at(field * 4)
    }

    pub fn i32(&self, field: usize) -> i32 {
        self.u32(field) as i32
    }

    pub fn f32(&self, field: usize) -> f32 {
        f32::from_bits(self.u32(field))
    }

    pub fn u8_at(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    pub fn str_at(&self, offset: usize) -> &'a str {
        let start = self.u32_at(offset) as usize;
        let tail = self.strings.get(start..).unwrap_or(&[]);
        let end = tail.iter().position(|&b| b == 0).unwrap_or(tail.len());
        std::str::from_utf8(&tail[..end]).unwrap_or("")
    }

    pub fn str(&self, field: usize) -> &'a str {
        self.str_at(field * 4)
    }

    pub fn raw(&self) -> &'a [u8] {
        self.data
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    U32(u32),
    I32(i32),
    F32(f32),
    U8(u8),
    Str(String),
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::U32(v)
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::I32(v)
    }
}
impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::F32(v)
    }
}
impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::U8(v)
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Str(v.to_string())
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Str(v)
    }
}

/// Construit un fichier WDBC à partir d'une chaîne de format et de lignes de valeurs.
pub struct DbcWriter {
    format: String,
    records: Vec<u8>,
    strings: Vec<u8>,
    string_index: HashMap<String, u32>,
    count: u32,
}

impl DbcWriter {
    pub fn new(format: &str) -> Self {
        Self {
            format: format.to_string(),
            records: Vec::new(),
            strings: vec![0],
            string_index: HashMap::from([(String::new(), 0)]),
            count: 0,
        }
    }

    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&off) = self.string_index.get(s) {
            return off;
        }
        let off = self.strings.len() as u32;
        self.strings.extend_from_slice(s.as_bytes());
        self.strings.push(0);
        self.string_index.insert(s.to_string(), off);
        off
    }

    pub fn push(&mut self, values: &[Value]) -> Result<()> {
        let format = self.format.clone();
        if values.len() != format.len() {
            return Err(DbcError::ValueMismatch { format, index: values.len() });
        }
        for (i, (c, v)) in format.chars().zip(values).enumerate() {
            match (c, v) {
                ('n' | 'i' | 'x' | 'd' | 'l', Value::U32(x)) => self.records.extend(x.to_le_bytes()),
                ('n' | 'i' | 'x' | 'd' | 'l', Value::I32(x)) => self.records.extend(x.to_le_bytes()),
                ('f', Value::F32(x)) => self.records.extend(x.to_le_bytes()),
                ('b' | 'X', Value::U8(x)) => self.records.push(*x),
                ('s', Value::Str(s)) => {
                    let off = self.intern(s);
                    self.records.extend(off.to_le_bytes());
                }
                _ => return Err(DbcError::ValueMismatch { format, index: i }),
            }
        }
        self.count += 1;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(20 + self.records.len() + self.strings.len());
        out.extend_from_slice(DBC_MAGIC);
        out.extend(self.count.to_le_bytes());
        out.extend((self.format.len() as u32).to_le_bytes());
        out.extend((format_record_size(&self.format) as u32).to_le_bytes());
        out.extend((self.strings.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.records);
        out.extend_from_slice(&self.strings);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut w = DbcWriter::new("nsfi");
        w.push(&[1u32.into(), "Épée".into(), 1.5f32.into(), (-3i32).into()]).unwrap();
        w.push(&[2u32.into(), "Bouclier".into(), 0.0f32.into(), 7u32.into()]).unwrap();
        let dbc = DbcFile::parse(&w.to_bytes()).unwrap();
        dbc.check_format("nsfi").unwrap();
        assert_eq!(dbc.len(), 2);
        assert_eq!(dbc.record(0).str(1), "Épée");
        assert_eq!(dbc.record(0).f32(2), 1.5);
        assert_eq!(dbc.record(0).i32(3), -3);
        assert_eq!(dbc.record(1).u32(0), 2);
        assert_eq!(dbc.record(1).str(1), "Bouclier");
    }
}
