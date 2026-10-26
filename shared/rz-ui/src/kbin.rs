//! `.spr` binaires (`minimap.spr`, `ui_icon.spr`, `worldmap.spr`) : sérialisation à balises
//! des gabarits de `sprite.tml` (sprite_type → animation_type → frame_type, library_type).
//!
//! ```text
//! 0x00  "DIFKNIBK" u32 1 …          en-tête de 0x80 octets
//! 0x80  "KLBK" u32 taille_données … en-tête de bloc de 0x20 octets
//! 0xA0  données :
//!   0x1F GUID(16) u32 n_membres  membres…      structure (GUID du gabarit .tml)
//!   0x29 u32                                   dword
//!   0x2A f32                                   float
//!   0x33 u32 len octets[len] (avec \0 final)   chaîne
//!   0x34 0x29|0x2A u32 n u32 0 n×4 octets      tableau de dword / float
//!   0x34 0x1F GUID u32 n u32 len nom[len] n×structure   tableau de structures
//! ```
//! Les images d'animation y sont des sous-rectangles (`library_rect = x1,y1,x2,y2`) d'une
//! planche (`library_path`), référencés par `library_name` depuis la frame.

use crate::spr::{Animation, Frame};
use crate::UiError;

pub const MAGIC: &[u8; 8] = b"DIFKNIBK";

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(u32),
    Float(f32),
    Str(String),
    Ints(Vec<u32>),
    Floats(Vec<f32>),
    Struct { guid: [u8; 16], members: Vec<Value> },
    Structs { guid: [u8; 16], size_name: String, items: Vec<Value> },
}

struct Reader<'a> {
    b: &'a [u8],
    p: usize,
}

fn err(p: usize, msg: &str) -> UiError {
    UiError::Spr { line: 0, msg: format!("binaire, octet 0x{p:x} : {msg}") }
}

impl<'a> Reader<'a> {
    fn u8(&mut self) -> Result<u8, UiError> {
        let v = *self.b.get(self.p).ok_or_else(|| err(self.p, "fin de fichier"))?;
        self.p += 1;
        Ok(v)
    }
    fn u32(&mut self) -> Result<u32, UiError> {
        let s = self.b.get(self.p..self.p + 4).ok_or_else(|| err(self.p, "fin de fichier"))?;
        self.p += 4;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn bytes(&mut self, n: usize) -> Result<&'a [u8], UiError> {
        let s = self.b.get(self.p..self.p + n).ok_or_else(|| err(self.p, "fin de fichier"))?;
        self.p += n;
        Ok(s)
    }
    fn guid(&mut self) -> Result<[u8; 16], UiError> {
        let mut g = [0; 16];
        g.copy_from_slice(self.bytes(16)?);
        Ok(g)
    }
    fn string(&mut self) -> Result<String, UiError> {
        let n = self.u32()? as usize;
        let s = self.bytes(n)?;
        let s = s.split(|&c| c == 0).next().unwrap_or(s);
        Ok(crate::decode_text(s))
    }
    fn structure(&mut self, depth: u32) -> Result<Value, UiError> {
        let guid = self.guid()?;
        let n = self.u32()?;
        if n > 64 {
            return Err(err(self.p, "trop de membres"));
        }
        let mut members = Vec::with_capacity(n as usize);
        for _ in 0..n {
            members.push(self.value(depth + 1)?);
        }
        Ok(Value::Struct { guid, members })
    }
    fn value(&mut self, depth: u32) -> Result<Value, UiError> {
        if depth > 16 {
            return Err(err(self.p, "imbrication trop profonde"));
        }
        let at = self.p;
        match self.u8()? {
            0x29 => Ok(Value::Int(self.u32()?)),
            0x2a => Ok(Value::Float(f32::from_bits(self.u32()?))),
            0x33 => Ok(Value::Str(self.string()?)),
            0x1f => self.structure(depth),
            0x34 => match self.u8()? {
                t @ (0x29 | 0x2a) => {
                    let n = self.u32()? as usize;
                    let _zero = self.u32()?;
                    let raw = self.bytes(n * 4)?;
                    let it = raw.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]));
                    Ok(if t == 0x29 { Value::Ints(it.collect()) } else { Value::Floats(it.map(f32::from_bits).collect()) })
                }
                0x1f => {
                    let guid = self.guid()?;
                    let n = self.u32()? as usize;
                    let size_name = self.string()?;
                    let mut items = Vec::with_capacity(n.min(1 << 16));
                    for _ in 0..n {
                        if self.u8()? != 0x1f {
                            return Err(err(self.p - 1, "structure attendue dans le tableau"));
                        }
                        items.push(self.structure(depth + 1)?);
                    }
                    Ok(Value::Structs { guid, size_name, items })
                }
                t => Err(err(self.p - 1, &format!("type de tableau inconnu 0x{t:02x}"))),
            },
            t => Err(err(at, &format!("balise inconnue 0x{t:02x}"))),
        }
    }
}

/// Lit l'arbre de valeurs d'un fichier binaire.
pub fn parse_tree(b: &[u8]) -> Result<Value, UiError> {
    if b.len() < 0xa0 || &b[..8] != MAGIC || &b[0x80..0x84] != b"KLBK" {
        return Err(err(0, "en-tête DIFKNIBK/KLBK absent"));
    }
    let mut r = Reader { b, p: 0xa0 };
    if r.u8()? != 0x1f {
        return Err(err(0xa0, "structure racine attendue"));
    }
    r.structure(0)
}

fn members(v: &Value) -> &[Value] {
    match v {
        Value::Struct { members, .. } => members,
        _ => &[],
    }
}

fn arrays(v: &Value) -> impl Iterator<Item = &Vec<Value>> {
    members(v).iter().filter_map(|m| if let Value::Structs { items, .. } = m { Some(items) } else { None })
}

/// Convertit un `.spr` binaire en animations (images = sous-rectangles des planches).
pub fn parse_binary_spr(b: &[u8], source: &str) -> Result<Vec<Animation>, UiError> {
    let root = parse_tree(b)?;
    let mut lists = arrays(&root);
    let anims = lists.next().cloned().unwrap_or_default();
    let libs = lists.next().cloned().unwrap_or_default();
    // library_type { name, path, rect[4] }
    let mut lib = std::collections::HashMap::new();
    for l in &libs {
        let m = members(l);
        let strs: Vec<&String> = m.iter().filter_map(|v| if let Value::Str(s) = v { Some(s) } else { None }).collect();
        let rect = m.iter().find_map(|v| if let Value::Ints(r) = v { Some(r.clone()) } else { None });
        if let (Some(name), Some(path), Some(r)) = (strs.first(), strs.get(1), rect) {
            if r.len() == 4 {
                let (x1, y1, x2, y2) = (r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32);
                lib.insert(name.to_ascii_lowercase(), ((*path).clone(), (x1, y1, x2 - x1, y2 - y1)));
            }
        }
    }
    let mut out = Vec::new();
    for a in &anims {
        let m = members(a);
        let Some(Value::Str(name)) = m.first() else { continue };
        let mut frames = Vec::new();
        if let Some(fs) = arrays(a).next() {
            for f in fs {
                let id = members(f).iter().find_map(|v| if let Value::Str(s) = v { Some(s) } else { None });
                let Some(id) = id else { continue };
                let (image, sub) = match lib.get(&id.to_ascii_lowercase()) {
                    Some((p, r)) => (p.clone(), Some(*r)),
                    None => (id.clone(), None),
                };
                frames.push(Frame { image, spr_size: (0, 0), spr_extra: 0, sub, file: None, w: 0, h: 0 });
            }
        }
        out.push(Animation { name: name.clone(), frames, source: source.to_string() });
    }
    Ok(out)
}
