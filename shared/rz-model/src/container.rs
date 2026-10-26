//! Conteneurs génériques à gabarits du moteur nFlavor : **BTRF** (binaire compact, gabarits par
//! GUID) et **KNIB** (`DIFKNIBK`, binaire balisé de `KBinaryFiler`). Les deux produisent le même
//! arbre d'[`Obj`] typés par les gabarits de [`crate::tml`]. Voir docs/re/model_format.md §2-3.

use crate::tml::{schema, Dim, Prim, Ty};
use crate::ModelError;

/// Valeur d'un membre de gabarit.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Membre absent (fichier écrit avec une version plus ancienne du gabarit).
    Missing,
    U8(u8),
    U16(u16),
    U32(u32),
    F32(f32),
    Str(String),
    Obj(Box<Obj>),
    U8s(Vec<u8>),
    U16s(Vec<u16>),
    U32s(Vec<u32>),
    F32s(Vec<f32>),
    Strs(Vec<String>),
    Objs(Vec<Obj>),
    Bytes(Vec<u8>),
}

/// Instance d'un gabarit : une valeur par membre de [`crate::tml::Template::members`]
/// (compteurs de tableaux compris, reconstitués pour BTRF).
#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    pub template: usize,
    pub values: Vec<Value>,
}

impl Obj {
    pub fn name(&self) -> &'static str {
        schema().templates[self.template].name.as_str()
    }
    pub fn get(&self, member: &str) -> Option<&Value> {
        let i = schema().templates[self.template].member_index(member)?;
        self.values.get(i)
    }
    pub fn u32(&self, m: &str) -> Option<u32> {
        match self.get(m)? {
            Value::U32(v) => Some(*v),
            Value::U16(v) => Some(*v as u32),
            Value::U8(v) => Some(*v as u32),
            _ => None,
        }
    }
    pub fn f32(&self, m: &str) -> Option<f32> {
        match self.get(m)? {
            Value::F32(v) => Some(*v),
            _ => None,
        }
    }
    pub fn str(&self, m: &str) -> &str {
        match self.get(m) {
            Some(Value::Str(s)) => s,
            _ => "",
        }
    }
    pub fn f32s(&self, m: &str) -> &[f32] {
        match self.get(m) {
            Some(Value::F32s(v)) => v,
            _ => &[],
        }
    }
    /// Tableau d'entiers (u32, u16 ou u8 convertis).
    pub fn u32s(&self, m: &str) -> Vec<u32> {
        match self.get(m) {
            Some(Value::U32s(v)) => v.clone(),
            Some(Value::U16s(v)) => v.iter().map(|&x| x as u32).collect(),
            Some(Value::U8s(v)) => v.iter().map(|&x| x as u32).collect(),
            _ => Vec::new(),
        }
    }
    pub fn strs(&self, m: &str) -> &[String] {
        match self.get(m) {
            Some(Value::Strs(v)) => v,
            _ => &[],
        }
    }
    pub fn objs(&self, m: &str) -> &[Obj] {
        match self.get(m) {
            Some(Value::Objs(v)) => v,
            _ => &[],
        }
    }
}

/// Type de conteneur rencontré.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// `BTRF`, version d'en-tête (major, minor) : 1.1 (courant) ou 1.0.
    Btrf { major: u16, minor: u16 },
    /// `DIFKNIBK` (KBinaryFiler).
    Knib,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub container: Container,
    pub objects: Vec<Obj>,
}

impl Document {
    /// Premier objet de premier niveau dont le gabarit porte ce nom.
    pub fn find(&self, template: &str) -> Option<&Obj> {
        self.objects.iter().find(|o| o.name() == template)
    }
}

/// Lit un document BTRF ou KNIB.
pub fn parse_document(bytes: &[u8]) -> Result<Document, ModelError> {
    if bytes.len() < 8 {
        // Fichiers vides de l'archive (ex. novice_builddustbeil_cacol.nx3).
        return Err(ModelError::Truncated(bytes.len()));
    }
    if bytes.starts_with(b"BTRF") {
        parse_btrf(bytes)
    } else if bytes.starts_with(b"DIFKNIBK") {
        parse_knib(bytes)
    } else {
        Err(ModelError::BadMagic)
    }
}

/// Décode une chaîne d'octets (Latin-1 octet par octet ; les chemins d'outils en CP949 restent
/// lisibles sans perte de leurs parties ASCII).
fn latin1(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

struct Cur<'a> {
    b: &'a [u8],
    p: usize,
    end: usize,
}

impl<'a> Cur<'a> {
    fn new(b: &'a [u8], p: usize, end: usize) -> Self {
        Cur {
            b,
            p,
            end: end.min(b.len()),
        }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], ModelError> {
        if self.p.checked_add(n).is_none_or(|e| e > self.end) {
            return Err(ModelError::Truncated(self.p));
        }
        let s = &self.b[self.p..self.p + n];
        self.p += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, ModelError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, ModelError> {
        let s = self.take(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, ModelError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn f32(&mut self) -> Result<f32, ModelError> {
        Ok(f32::from_bits(self.u32()?))
    }
    fn guid(&mut self) -> Result<[u8; 16], ModelError> {
        let mut g = [0u8; 16];
        g.copy_from_slice(self.take(16)?);
        Ok(g)
    }
}

fn invalid(s: impl Into<String>) -> ModelError {
    ModelError::Invalid(s.into())
}

fn guid_text(g: &[u8; 16]) -> String {
    let d1 = u32::from_le_bytes([g[0], g[1], g[2], g[3]]);
    let d2 = u16::from_le_bytes([g[4], g[5]]);
    let d3 = u16::from_le_bytes([g[6], g[7]]);
    let d4: String = g[8..].iter().map(|b| format!("{b:02X}")).collect();
    format!("{{{d1:08X}-{d2:04X}-{d3:04X}-{}-{}}}", &d4[..4], &d4[4..])
}

// ───────────────────────────────── BTRF ─────────────────────────────────

/// Entrée de la table d'identifiants : gabarit, nombre de membres stockés, taille fixe totale.
struct IdEntry {
    template: usize,
    count: usize,
    fixed: Option<usize>,
}

struct Btrf<'a> {
    b: &'a [u8],
    ids: Vec<IdEntry>,
    strings: Vec<String>,
}

/// Taille d'un membre dans BTRF (`None` = préfixée par un u32).
fn btrf_member_size(ty: &Ty, dim: &Dim) -> Option<usize> {
    match (ty, dim) {
        (Ty::Prim(p), Dim::Scalar) => p.btrf_size(),
        (Ty::Prim(p), Dim::Fixed(n)) => p.btrf_size().map(|s| s * n),
        _ => None,
    }
}

fn parse_btrf(b: &[u8]) -> Result<Document, ModelError> {
    let mut c = Cur::new(b, 4, b.len());
    let ext = c.u32()? as usize;
    let hdr = c.take(ext)?;
    let (major, minor) = if hdr.len() >= 4 {
        (
            u16::from_le_bytes([hdr[0], hdr[1]]),
            u16::from_le_bytes([hdr[2], hdr[3]]),
        )
    } else {
        (0, 0)
    };
    // FUN_005d07d0 : en-tête 1.0 → objets de premier niveau avec 5 octets d'en-tête au lieu de 4.
    let v10 = major == 1 && minor == 0;

    // Table d'identifiants (FUN_005d09b0) : N GUID puis N u16 (membres stockés), N = taille / 18.
    let idsz = c.u32()? as usize;
    let idblk = c.take(idsz)?;
    let n = idsz / 18;
    let sch = schema();
    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        let mut g = [0u8; 16];
        g.copy_from_slice(&idblk[i * 16..i * 16 + 16]);
        let count = u16::from_le_bytes([idblk[n * 16 + i * 2], idblk[n * 16 + i * 2 + 1]]) as usize;
        let template = sch
            .by_guid(&g)
            .ok_or_else(|| ModelError::UnknownTemplate(guid_text(&g)))?;
        let t = &sch.templates[template];
        if count > t.stored.len() {
            return Err(invalid(format!(
                "{} : {count} membres > {}",
                t.name,
                t.stored.len()
            )));
        }
        let mut fixed = Some(0usize);
        for &mi in &t.stored[..count] {
            let m = &t.members[mi];
            fixed = match (fixed, btrf_member_size(&m.ty, &m.dim)) {
                (Some(a), Some(s)) => Some(a + s),
                _ => None,
            };
        }
        ids.push(IdEntry {
            template,
            count,
            fixed,
        });
    }

    // Table de chaînes (FUN_005d0be0) : u32 taille, u32 n, n chaînes C, u32 nw, nw chaînes UTF-16.
    let ssz = c.u32()? as usize;
    let send = c.p + ssz;
    let mut s = Cur::new(b, c.p, send);
    let sc = s.u32()? as usize;
    let mut strings = Vec::with_capacity(sc);
    for _ in 0..sc {
        let st = s.p;
        while s.u8()? != 0 {}
        strings.push(latin1(&b[st..s.p - 1]));
    }
    // Les chaînes larges ne sont pas utilisées par les gabarits nx3/naf/cob : on les ignore.
    c.p = send;

    // Données (FUN_005d0570) : u32 taille, u32 n, n objets.
    let dsz = c.u32()? as usize;
    let dend = c.p + dsz;
    if dend > b.len() {
        return Err(ModelError::Truncated(b.len()));
    }
    let mut d = Cur::new(b, c.p, dend);
    let cnt = d.u32()? as usize;
    let ctx = Btrf { b, ids, strings };
    let mut objects = Vec::new();
    for _ in 0..cnt {
        let h = d.take(if v10 { 5 } else { 4 })?;
        let ty = h[h.len() - 1];
        let size = match ty {
            1 | 2 => ty as usize,
            3 | 5 | 7 | 8 => 4,
            4 | 6 => 8,
            _ => d.u32()? as usize,
        };
        let start = d.p;
        d.take(size)?;
        if ty == 9 {
            let mut oc = Cur::new(b, start, start + size);
            let o = ctx.object(&mut oc, None)?;
            if oc.p != start + size {
                return Err(invalid(format!(
                    "objet {} : fin à {} au lieu de {}",
                    o.name(),
                    oc.p,
                    start + size
                )));
            }
            objects.push(o);
        }
    }
    Ok(Document {
        container: Container::Btrf { major, minor },
        objects,
    })
}

impl Btrf<'_> {
    fn string(&self, idx: u32) -> String {
        if idx == 0 {
            String::new()
        } else {
            self.strings
                .get(idx as usize - 1)
                .cloned()
                .unwrap_or_default()
        }
    }

    /// Objet : si `entry` est `None`, il commence par son index de gabarit (u16, 1-based).
    fn object(&self, c: &mut Cur, entry: Option<usize>) -> Result<Obj, ModelError> {
        let e = match entry {
            Some(e) => e,
            None => {
                let idx = c.u16()? as usize;
                if idx == 0 || idx > self.ids.len() {
                    return Err(invalid(format!("index de gabarit {idx}")));
                }
                idx - 1
            }
        };
        let ent = &self.ids[e];
        let t = &schema().templates[ent.template];
        let mut values = vec![Value::Missing; t.members.len()];
        for &mi in &t.stored[..ent.count] {
            let m = &t.members[mi];
            let size = match btrf_member_size(&m.ty, &m.dim) {
                Some(s) => s,
                None => c.u32()? as usize,
            };
            let start = c.p;
            c.take(size)?;
            let mut mc = Cur::new(self.b, start, start + size);
            values[mi] = self.member(&mut mc, &m.ty, &m.dim, size)?;
            if mc.p != start + size {
                return Err(invalid(format!(
                    "{}.{} : {} octets lus sur {size}",
                    t.name,
                    m.name,
                    mc.p - start
                )));
            }
        }
        // Compteurs implicites des tableaux.
        for (i, m) in t.members.iter().enumerate() {
            if let Dim::Var(di) = m.dim {
                let len = match &values[i] {
                    Value::U8s(v) => v.len(),
                    Value::U16s(v) => v.len(),
                    Value::U32s(v) => v.len(),
                    Value::F32s(v) => v.len(),
                    Value::Strs(v) => v.len(),
                    Value::Objs(v) => v.len(),
                    _ => 0,
                };
                if values[di] == Value::Missing {
                    values[di] = Value::U32(len as u32);
                }
            }
        }
        Ok(Obj {
            template: ent.template,
            values,
        })
    }

    fn prim(&self, c: &mut Cur, p: Prim) -> Result<Value, ModelError> {
        Ok(match p {
            Prim::Char => Value::U8(c.u8()?),
            Prim::Word => Value::U16(c.u16()?),
            Prim::Dword => Value::U32(c.u32()?),
            Prim::Float => Value::F32(c.f32()?),
            Prim::Str => Value::Str(self.string(c.u32()?)),
            Prim::Dict => Value::Bytes(c.take(c.end - c.p)?.to_vec()),
        })
    }

    fn prim_array(&self, c: &mut Cur, p: Prim, n: usize) -> Result<Value, ModelError> {
        Ok(match p {
            Prim::Char => Value::U8s(c.take(n)?.to_vec()),
            Prim::Word => Value::U16s((0..n).map(|_| c.u16()).collect::<Result<_, _>>()?),
            Prim::Dword => Value::U32s((0..n).map(|_| c.u32()).collect::<Result<_, _>>()?),
            Prim::Float => Value::F32s((0..n).map(|_| c.f32()).collect::<Result<_, _>>()?),
            Prim::Str => Value::Strs(
                (0..n)
                    .map(|_| c.u32().map(|i| self.string(i)))
                    .collect::<Result<_, _>>()?,
            ),
            Prim::Dict => Value::Bytes(c.take(c.end - c.p)?.to_vec()),
        })
    }

    fn member(&self, c: &mut Cur, ty: &Ty, dim: &Dim, size: usize) -> Result<Value, ModelError> {
        match (ty, dim) {
            (Ty::Prim(p), Dim::Scalar) => self.prim(c, *p),
            (Ty::Prim(p), Dim::Fixed(n)) => self.prim_array(c, *p, *n),
            (Ty::Template(_), Dim::Scalar) => Ok(Value::Obj(Box::new(self.object(c, None)?))),
            (Ty::Prim(p), Dim::Var(_)) => {
                if size == 0 {
                    return self.prim_array(c, *p, 0);
                }
                // Tableau variable : 1 octet de type (1 char, 2 word, 3 dword, 5 float, 7 string)
                // puis les éléments.
                let code = c.u8()?;
                let es = match code {
                    1 => 1,
                    2 => 2,
                    3 | 5 | 7 | 8 => 4,
                    4 | 6 => 8,
                    _ => return Err(invalid(format!("code de type {code}"))),
                };
                if p.btrf_size() != Some(es) {
                    return Err(invalid(format!("type {code} incompatible avec {p:?}")));
                }
                if (size - 1) % es != 0 {
                    return Err(invalid("taille de tableau non multiple de l'élément"));
                }
                self.prim_array(c, *p, (size - 1) / es)
            }
            (Ty::Template(_), _) => {
                if size == 0 {
                    return Ok(Value::Objs(Vec::new()));
                }
                // Tableau d'objets : u16 index de gabarit, u32 nombre, puis les éléments, chacun
                // précédé d'un u32 de taille sauf si tous ses membres sont de taille fixe.
                let idx = c.u16()? as usize;
                if idx == 0 || idx > self.ids.len() {
                    return Err(invalid(format!("index de gabarit {idx}")));
                }
                let e = idx - 1;
                let n = c.u32()? as usize;
                let mut v = Vec::with_capacity(n.min(1 << 16));
                for _ in 0..n {
                    let es = match self.ids[e].fixed {
                        Some(s) => s,
                        None => c.u32()? as usize,
                    };
                    let start = c.p;
                    c.take(es)?;
                    let mut ec = Cur::new(self.b, start, start + es);
                    v.push(self.object(&mut ec, Some(e))?);
                    if ec.p != start + es {
                        return Err(invalid("élément de tableau mal délimité"));
                    }
                }
                Ok(Value::Objs(v))
            }
        }
    }
}

// ───────────────────────────────── KNIB ─────────────────────────────────

const TAG_OBJ: u8 = 0x1f;
const TAG_WORD: u8 = 0x28;
const TAG_DWORD: u8 = 0x29;
const TAG_FLOAT: u8 = 0x2a;
const TAG_CHAR: u8 = 0x2c;
const TAG_STRING: u8 = 0x33;
const TAG_ARRAY: u8 = 0x34;

fn parse_knib(b: &[u8]) -> Result<Document, ModelError> {
    // En-tête de 0x80 octets (FUN_005db1e0) : "DIFK" "NIBK", i32 version (>0), i32 (>=0), i32 blocs.
    let mut c = Cur::new(b, 8, b.len());
    let _version = c.u32()?;
    let _x = c.u32()?;
    let nblk = c.u32()? as usize;
    c.p = 0x80;
    let mut objects = Vec::new();
    for _ in 0..nblk {
        // Bloc : "KLBK", u32 taille du contenu, 24 octets réservés, puis un objet balisé.
        if c.take(4)? != b"KLBK" {
            return Err(invalid("bloc KLBK attendu"));
        }
        let size = c.u32()? as usize;
        c.take(24)?;
        let end = c.p + size;
        let mut bc = Cur::new(b, c.p, end);
        if bc.u8()? != TAG_OBJ {
            return Err(invalid("objet attendu en tête de bloc"));
        }
        objects.push(knib_object(&mut bc)?);
        if bc.p != end {
            return Err(invalid("bloc KLBK mal délimité"));
        }
        c.p = end;
    }
    Ok(Document {
        container: Container::Knib,
        objects,
    })
}

fn knib_string(c: &mut Cur) -> Result<String, ModelError> {
    let n = c.u32()? as usize;
    let s = c.take(n)?;
    Ok(latin1(s.strip_suffix(&[0]).unwrap_or(s)))
}

/// Objet (après la balise 0x1f) : GUID, u32 nombre de membres, membres balisés dans l'ordre du
/// gabarit (compteurs de tableaux compris).
fn knib_object(c: &mut Cur) -> Result<Obj, ModelError> {
    let g = c.guid()?;
    let template = schema()
        .by_guid(&g)
        .ok_or_else(|| ModelError::UnknownTemplate(guid_text(&g)))?;
    let t = &schema().templates[template];
    let n = c.u32()? as usize;
    if n > t.members.len() {
        return Err(invalid(format!("{} : {n} membres", t.name)));
    }
    let mut values = vec![Value::Missing; t.members.len()];
    for v in values.iter_mut().take(n) {
        let tag = c.u8()?;
        *v = knib_value(c, tag)?;
    }
    Ok(Obj { template, values })
}

fn knib_value(c: &mut Cur, tag: u8) -> Result<Value, ModelError> {
    Ok(match tag {
        TAG_OBJ => Value::Obj(Box::new(knib_object(c)?)),
        TAG_WORD => Value::U16(c.u16()?),
        TAG_DWORD => Value::U32(c.u32()?),
        TAG_FLOAT => Value::F32(c.f32()?),
        TAG_CHAR => Value::U8(c.u8()?),
        TAG_STRING => Value::Str(knib_string(c)?),
        TAG_ARRAY => {
            // Tableau : balise d'élément (+ GUID si objet), u32 nombre, nom du compteur (chaîne),
            // puis les éléments : bruts pour les primitifs, objets complets (0x1f GUID ...) sinon.
            let et = c.u8()?;
            if et == TAG_OBJ {
                c.guid()?;
            }
            let n = c.u32()? as usize;
            let _dim = knib_string(c)?;
            match et {
                TAG_OBJ => {
                    let mut v = Vec::with_capacity(n.min(1 << 16));
                    for _ in 0..n {
                        if c.u8()? != TAG_OBJ {
                            return Err(invalid("élément objet attendu"));
                        }
                        v.push(knib_object(c)?);
                    }
                    Value::Objs(v)
                }
                TAG_WORD => Value::U16s((0..n).map(|_| c.u16()).collect::<Result<_, _>>()?),
                TAG_DWORD => Value::U32s((0..n).map(|_| c.u32()).collect::<Result<_, _>>()?),
                TAG_FLOAT => Value::F32s((0..n).map(|_| c.f32()).collect::<Result<_, _>>()?),
                TAG_CHAR => Value::U8s(c.take(n)?.to_vec()),
                TAG_STRING => {
                    Value::Strs((0..n).map(|_| knib_string(c)).collect::<Result<_, _>>()?)
                }
                _ => return Err(invalid(format!("balise d'élément 0x{et:02x}"))),
            }
        }
        _ => return Err(invalid(format!("balise 0x{tag:02x} à 0x{:x}", c.p - 1))),
    })
}
