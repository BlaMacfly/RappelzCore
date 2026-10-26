//! Bases client `.rdb` de Rappelz (détails : `docs/re/rdb_format.md`).
//!
//! En-tête commun de 132 octets : date ASCII `YYYYMMDD` (8 o), texte libre jusqu'à 0x80,
//! puis `i32 count`. Les enregistrements suivent, packés sans alignement, petit-boutiste.
//! Formats particuliers : `db_string` (enregistrements de longueur variable),
//! `db_skilltree` (groupes `i32 n` + n lignes), `db_item.ref` (table de noms de fichiers),
//! et `db_monster.id` stocké bit-permuté.

use crate::rdb_schemas::SCHEMAS;

pub const HEADER_LEN: usize = 0x80;

#[derive(Debug, thiserror::Error)]
pub enum RdbError {
    #[error("fichier RDB tronqué ({table}, offset {offset:#x})")]
    Truncated { table: String, offset: usize },
    #[error("schéma inconnu pour la table « {0} »")]
    UnknownTable(String),
    #[error("{table} : {consumed} octets lus sur {len}")]
    TrailingData { table: String, consumed: usize, len: usize },
}

pub type Result<T> = std::result::Result<T, RdbError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldType {
    Bit,
    I8,
    I16,
    I32,
    F32,
    F64,
    /// i32 représentant une valeur décimale ×100.
    Dec2,
    /// Chaîne de taille fixe, terminée par NUL, dans la code page du client.
    Char(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RdbValue {
    Int(i64),
    Float(f64),
    Str(String),
}

impl RdbValue {
    pub fn as_i64(&self) -> i64 {
        match self {
            RdbValue::Int(v) => *v,
            RdbValue::Float(v) => *v as i64,
            RdbValue::Str(s) => s.parse().unwrap_or(0),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            RdbValue::Int(v) => *v as f64,
            RdbValue::Float(v) => *v,
            RdbValue::Str(s) => s.parse().unwrap_or(0.0),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            RdbValue::Str(s) => s,
            _ => "",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: &'static str,
    pub ty: FieldType,
    pub offset: usize,
    pub bit: u8,
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub table: &'static str,
    pub fields: Vec<Field>,
    pub record_size: usize,
}

impl Layout {
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }
}

/// Disposition calculée d'une table (les champs `Bit` consécutifs partagent un octet).
pub fn layout(table: &str) -> Result<Layout> {
    let (name, schema) = SCHEMAS
        .iter()
        .find(|(n, _)| *n == table)
        .ok_or_else(|| RdbError::UnknownTable(table.to_string()))?;
    let mut fields = Vec::with_capacity(schema.len());
    let (mut off, mut bits) = (0usize, 0u8);
    for &(fname, ty) in schema.iter() {
        if ty == FieldType::Bit {
            fields.push(Field { name: fname, ty, offset: off, bit: bits });
            bits += 1;
            if bits == 8 {
                off += 1;
                bits = 0;
            }
            continue;
        }
        if bits != 0 {
            off += 1;
            bits = 0;
        }
        let size = match ty {
            FieldType::I8 => 1,
            FieldType::I16 => 2,
            FieldType::I32 | FieldType::F32 | FieldType::Dec2 => 4,
            FieldType::F64 => 8,
            FieldType::Char(n) => n,
            FieldType::Bit => unreachable!(),
        };
        fields.push(Field { name: fname, ty, offset: off, bit: 0 });
        off += size;
    }
    if bits != 0 {
        off += 1;
    }
    Ok(Layout { table: name, fields, record_size: off })
}

pub fn known_tables() -> impl Iterator<Item = &'static str> {
    SCHEMAS.iter().map(|(n, _)| *n)
}

/// Nom de table à partir d'un nom de fichier (`db_String(windows-1252).rdb` → `string`).
pub fn table_name(file_name: &str) -> Option<String> {
    let lower = file_name.to_ascii_lowercase();
    let stem = lower.strip_prefix("db_")?;
    let stem = stem.strip_suffix(".rdb").or_else(|| stem.strip_suffix(".ref").map(|_| "item.ref"))?;
    let stem = stem.split('(').next()?;
    Some(stem.to_string())
}

pub struct Decoder {
    encoding: &'static encoding_rs::Encoding,
}

impl Decoder {
    /// `codepage` comme dans les noms de fichiers du client : `windows-1252`, `windows-1250`…
    pub fn new(codepage: &str) -> Self {
        Self {
            encoding: encoding_rs::Encoding::for_label(codepage.as_bytes())
                .unwrap_or(encoding_rs::WINDOWS_1252),
        }
    }

    pub fn decode(&self, bytes: &[u8]) -> String {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        self.encoding.decode_without_bom_handling(&bytes[..end]).0.into_owned()
    }
}

#[derive(Clone, Debug)]
pub struct RdbHeader {
    pub date: String,
    pub comment: String,
    pub count: i32,
}

fn read_header(table: &str, data: &[u8]) -> Result<RdbHeader> {
    if data.len() < HEADER_LEN + 4 {
        return Err(RdbError::Truncated { table: table.to_string(), offset: 0 });
    }
    let text = |a: usize, b: usize| {
        let s = &data[a..b];
        let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        s[..end].iter().map(|&c| c as char).collect::<String>()
    };
    Ok(RdbHeader {
        date: text(0, 8),
        comment: text(16, HEADER_LEN),
        count: i32_at(data, HEADER_LEN),
    })
}

fn i32_at(d: &[u8], o: usize) -> i32 {
    i32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn read_value(dec: &Decoder, rec: &[u8], f: &Field) -> RdbValue {
    let p = f.offset;
    match f.ty {
        FieldType::Bit => RdbValue::Int(((rec[p] >> f.bit) & 1) as i64),
        FieldType::I8 => RdbValue::Int(rec[p] as i8 as i64),
        FieldType::I16 => RdbValue::Int(i16::from_le_bytes([rec[p], rec[p + 1]]) as i64),
        FieldType::I32 => RdbValue::Int(i32_at(rec, p) as i64),
        FieldType::F32 => RdbValue::Float(f32::from_le_bytes(rec[p..p + 4].try_into().unwrap()) as f64),
        FieldType::F64 => RdbValue::Float(f64::from_le_bytes(rec[p..p + 8].try_into().unwrap())),
        FieldType::Dec2 => RdbValue::Float(i32_at(rec, p) as f64 / 100.0),
        FieldType::Char(n) => RdbValue::Str(dec.decode(&rec[p..p + n])),
    }
}

/// Table RDB décodée : disposition + lignes (une valeur par champ du schéma).
#[derive(Clone, Debug)]
pub struct RdbTable {
    pub header: RdbHeader,
    pub layout: Layout,
    pub rows: Vec<Vec<RdbValue>>,
}

impl RdbTable {
    pub fn get(&self, row: usize, field: &str) -> Option<&RdbValue> {
        self.layout.index_of(field).map(|i| &self.rows[row][i])
    }

    pub fn int(&self, row: usize, field: &str) -> i64 {
        self.get(row, field).map(RdbValue::as_i64).unwrap_or(0)
    }

    pub fn float(&self, row: usize, field: &str) -> f64 {
        self.get(row, field).map(RdbValue::as_f64).unwrap_or(0.0)
    }

    pub fn str(&self, row: usize, field: &str) -> &str {
        self.get(row, field).map(RdbValue::as_str).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

pub fn parse_table(table: &str, data: &[u8], dec: &Decoder) -> Result<RdbTable> {
    let header = read_header(table, data)?;
    let layout = layout(table)?;
    let grouped = table == "skilltree";
    let mut rows = Vec::new();
    let mut o = HEADER_LEN + 4;
    let truncated = |offset| RdbError::Truncated { table: table.to_string(), offset };
    let read_rec = |o: &mut usize, rows: &mut Vec<Vec<RdbValue>>| -> Result<()> {
        let rec = data.get(*o..*o + layout.record_size).ok_or_else(|| truncated(*o))?;
        rows.push(layout.fields.iter().map(|f| read_value(dec, rec, f)).collect());
        *o += layout.record_size;
        Ok(())
    };
    for _ in 0..header.count.max(0) {
        if grouped {
            let n = i32_at(data.get(o..o + 4).ok_or_else(|| truncated(o))?, 0);
            o += 4;
            for _ in 0..n.max(0) {
                read_rec(&mut o, &mut rows)?;
            }
        } else {
            read_rec(&mut o, &mut rows)?;
        }
    }
    if o != data.len() {
        return Err(RdbError::TrailingData { table: table.to_string(), consumed: o, len: data.len() });
    }
    if table == "monster" {
        let id = layout.index_of("id").expect("monster.id");
        for row in &mut rows {
            row[id] = RdbValue::Int(monster_id_decode(row[id].as_i64() as u32) as i64);
        }
    }
    Ok(RdbTable { header, layout, rows })
}

/// Permutation de bits de `db_monster.id` (compute_bit_swapping, RappelzRDBTool).
fn monster_bit_table() -> [u8; 32] {
    let mut enc: [u8; 32] = core::array::from_fn(|i| i as u8);
    let mut j = 3usize;
    for i in 0..32 {
        enc.swap(i, j);
        j = (j + i + 3) % 32;
    }
    enc
}

pub fn monster_id_decode(raw: u32) -> u32 {
    let enc = monster_bit_table();
    let mut dec = [0u8; 32];
    for (i, &e) in enc.iter().enumerate() {
        dec[e as usize] = i as u8;
    }
    (0..32).filter(|i| raw >> i & 1 != 0).fold(0u32, |r, i| r | 1 << dec[i])
}

pub fn monster_id_encode(id: u32) -> u32 {
    let enc = monster_bit_table();
    (0..32).filter(|i| id >> i & 1 != 0).fold(0u32, |r, i| r | 1 << enc[i])
}

#[derive(Clone, Debug)]
pub struct StringEntry {
    pub code: i32,
    pub group_id: i32,
    pub name: String,
    pub value: String,
}

/// `db_string(<codepage>).rdb` : enregistrements de longueur variable.
pub fn parse_strings(data: &[u8], dec: &Decoder) -> Result<Vec<StringEntry>> {
    let header = read_header("string", data)?;
    let truncated = |offset| RdbError::Truncated { table: "string".into(), offset };
    let mut o = HEADER_LEN + 4;
    let mut out = Vec::with_capacity(header.count.max(0) as usize);
    for _ in 0..header.count.max(0) {
        let lens = data.get(o..o + 8).ok_or_else(|| truncated(o))?;
        let (nl, vl) = (i32_at(lens, 0) as usize, i32_at(lens, 4) as usize);
        o += 8;
        let name = data.get(o..o + nl).ok_or_else(|| truncated(o))?;
        o += nl;
        let value = data.get(o..o + vl).ok_or_else(|| truncated(o))?;
        o += vl;
        let tail = data.get(o..o + 24).ok_or_else(|| truncated(o))?;
        o += 24;
        out.push(StringEntry {
            code: i32_at(tail, 0),
            group_id: i32_at(tail, 4),
            name: dec.decode(name),
            value: dec.decode(value),
        });
    }
    if o != data.len() {
        return Err(RdbError::TrailingData { table: "string".into(), consumed: o, len: data.len() });
    }
    Ok(out)
}

/// `db_item.ref` : `u32 count`, `u32 strlen`, puis `count × char[strlen]`.
pub fn parse_item_ref(data: &[u8], dec: &Decoder) -> Result<Vec<String>> {
    let truncated = |offset| RdbError::Truncated { table: "item.ref".into(), offset };
    let head = data.get(..8).ok_or_else(|| truncated(0))?;
    let (n, len) = (i32_at(head, 0) as usize, i32_at(head, 4) as usize);
    let body = data.get(8..8 + n * len).ok_or_else(|| truncated(8))?;
    Ok(body.chunks_exact(len.max(1)).map(|c| dec.decode(c)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn layouts_match_observed_sizes() {
        for (t, size) in [("item", 376), ("monster", 552), ("skill", 827), ("job", 292), ("npcresource", 997), ("quest", 268), ("worldlocation", 1143), ("combineres", 436), ("questlink", 11)] {
            assert_eq!(layout(t).unwrap().record_size, size, "{t}");
        }
    }

    #[test]
    fn monster_bits_roundtrip() {
        for id in [1u32, 30001, 123456, 0xDEAD_BEEF] {
            assert_eq!(monster_id_decode(monster_id_encode(id)), id);
        }
    }

    #[test]
    fn table_names() {
        assert_eq!(table_name("db_String(windows-1252).rdb").as_deref(), Some("string"));
        assert_eq!(table_name("db_item.ref").as_deref(), Some("item.ref"));
        assert_eq!(table_name("db_monster.rdb").as_deref(), Some("monster"));
    }

    #[test]
    fn real_resource_if_present() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Rappelz NoLimits/Resource");
        if !dir.is_dir() {
            return;
        }
        let dec = Decoder::new("windows-1252");
        for entry in std::fs::read_dir(&dir).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            let Some(table) = table_name(&name) else { continue };
            let data = std::fs::read(dir.join(&name)).unwrap();
            match table.as_str() {
                "string" => assert!(!parse_strings(&data, &dec).unwrap().is_empty(), "{name}"),
                "item.ref" => assert!(!parse_item_ref(&data, &dec).unwrap().is_empty()),
                t => {
                    let tbl = parse_table(t, &data, &dec).unwrap_or_else(|e| panic!("{name}: {e}"));
                    if t == "monster" {
                        let ok = (0..tbl.len()).filter(|&r| tbl.int(r, "name_id") == tbl.int(r, "id") + 30000).count();
                        assert!(ok * 100 / tbl.len() > 95);
                    }
                }
            }
        }
    }
}
