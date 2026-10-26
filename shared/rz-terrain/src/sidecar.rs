//! Fichiers associés à une carte : `.nfa`, `.nfw`, `.nfe`, `.nfc`, `.nfp`, `.nfl`, `.qpf`, `.nfs`.
//! Toutes les positions sont relatives à l'origine de la carte (voir la doc de la crate).

use crate::nfm::{PropRecord, PROP_RECORD_SIZE};
use crate::reader::Reader;
use crate::{Result, TerrainError};

/// Polygone en unités d'attribut (1/8 de case = 5,25 unités pour une case de 42).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrPolygon {
    pub points: Vec<[i32; 2]>,
}

impl AttrPolygon {
    /// Points en unités monde relatives à la carte.
    pub fn local_points(&self, tile_length: f32) -> Vec<[f32; 2]> {
        let k = tile_length / 8.0;
        self.points.iter().map(|p| [p[0] as f32 * k, p[1] as f32 * k]).collect()
    }

    /// Test pair-impair, (x, y) relatifs à la carte.
    pub fn contains(&self, local_x: f32, local_y: f32, tile_length: f32) -> bool {
        let (x, y) = (local_x * 8.0 / tile_length, local_y * 8.0 / tile_length);
        let pts = &self.points;
        let mut inside = false;
        let mut j = pts.len().wrapping_sub(1);
        for i in 0..pts.len() {
            let (xi, yi) = (pts[i][0] as f32, pts[i][1] as f32);
            let (xj, yj) = (pts[j][0] as f32, pts[j][1] as f32);
            if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
}

fn read_attr_polygon(r: &mut Reader<'_>) -> Result<AttrPolygon> {
    let n = r.count(8)?;
    let mut points = Vec::with_capacity(n);
    for _ in 0..n {
        points.push([r.i32()?, r.i32()?]);
    }
    Ok(AttrPolygon { points })
}

fn read_attr_polygons(r: &mut Reader<'_>) -> Result<Vec<AttrPolygon>> {
    let n = r.count(4)?;
    (0..n).map(|_| read_attr_polygon(r)).collect()
}

/// `.nfa` (et section « vector attribute » du `.nfm`) : polygones de blocage.
/// `u32 n`, puis n × (`u32 points`, points × `i32 x, i32 y`). Fichier vide = aucun polygone.
pub fn parse_nfa(bytes: &[u8]) -> Result<Vec<AttrPolygon>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let mut r = Reader::new(bytes, "nfa");
    let v = read_attr_polygons(&mut r)?;
    r.finish()?;
    Ok(v)
}

/// Zone d'eau rectangulaire (44 octets).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterArea {
    /// Premier coin (x, y, niveau).
    pub corner_a: [f32; 3],
    /// Coin opposé (x, y, niveau).
    pub corner_b: [f32; 3],
    /// Centre du rectangle (x, y, niveau).
    pub center: [f32; 3],
    /// 0 ou 1 (rôle inconnu).
    pub flags: u32,
    /// Type d'eau (1000..1037 observés).
    pub water_id: u32,
}

impl WaterArea {
    /// Niveau de la surface.
    pub fn level(&self) -> f32 {
        self.center[2]
    }

    /// (x, y) relatifs à la carte dans le rectangle.
    pub fn contains(&self, local_x: f32, local_y: f32) -> bool {
        let (x0, x1) = (self.corner_a[0].min(self.corner_b[0]), self.corner_a[0].max(self.corner_b[0]));
        let (y0, y1) = (self.corner_a[1].min(self.corner_b[1]), self.corner_a[1].max(self.corner_b[1]));
        (x0..=x1).contains(&local_x) && (y0..=y1).contains(&local_y)
    }
}

/// `.nfw` (et section eau du `.nfm`) : `u32 n`, n × 44 octets. Fichier vide = pas d'eau.
pub fn parse_nfw(bytes: &[u8]) -> Result<Vec<WaterArea>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let mut r = Reader::new(bytes, "nfw");
    let n = r.count(44)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(WaterArea {
            corner_a: r.vec3()?,
            corner_b: r.vec3()?,
            center: r.vec3()?,
            flags: r.u32()?,
            water_id: r.u32()?,
        });
    }
    r.finish()?;
    Ok(v)
}

/// Zone d'événement (`.nfe`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventArea {
    pub id: u32,
    pub polygons: Vec<AttrPolygon>,
}

/// `.nfe` (et dernière section du `.nfm`) : `u32 n`, n × (`u32 id`, polygones comme `.nfa`).
pub fn parse_nfe(bytes: &[u8]) -> Result<Vec<EventArea>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let mut r = Reader::new(bytes, "nfe");
    let n = r.count(8)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let id = r.u32()?;
        v.push(EventArea { id, polygons: read_attr_polygons(&mut r)? });
    }
    r.finish()?;
    Ok(v)
}

/// Zone nommée (`.nfc`) : sert à déclencher `call_lc_In(<id WorldLocation>)`.
#[derive(Debug, Clone, PartialEq)]
pub struct LocationArea {
    /// 1..4 observés (priorité supposée).
    pub priority: u32,
    pub center: [f32; 3],
    /// Toujours 0 observé (rayon supposé).
    pub radius: f32,
    /// Nom (coréen).
    pub description: String,
    /// Script, ex. `call_lc_In( 10100 )`.
    pub script: String,
    pub polygons: Vec<AttrPolygon>,
}

/// `.nfc` : `u32 n`, n × (`u32`, 4 × `f32`, chaîne, chaîne, polygones comme `.nfa`).
pub fn parse_nfc(bytes: &[u8]) -> Result<Vec<LocationArea>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let mut r = Reader::new(bytes, "nfc");
    let n = r.count(32)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(LocationArea {
            priority: r.u32()?,
            center: r.vec3()?,
            radius: r.f32()?,
            description: r.string()?,
            script: r.string()?,
            polygons: read_attr_polygons(&mut r)?,
        });
    }
    r.finish()?;
    Ok(v)
}

/// Zone de la section 6 / `.nfp` (polygones 3D nommés ; rôle inconnu).
#[derive(Debug, Clone, PartialEq)]
pub struct NfpArea {
    pub id: u32,
    /// Points (x, y, z) relatifs à la carte.
    pub polygons: Vec<Vec<[f32; 3]>>,
    /// Ex. `M010_003|`.
    pub name: String,
}

/// `.nfp` : `u32 n`, n × (`u32 id`, `u32 polygones`, polygones × (`u32 points`, points × 3 `f32`), chaîne).
pub fn parse_nfp(bytes: &[u8]) -> Result<Vec<NfpArea>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let mut r = Reader::new(bytes, "nfp");
    let n = r.count(12)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let id = r.u32()?;
        let np = r.count(4)?;
        let mut polygons = Vec::with_capacity(np);
        for _ in 0..np {
            let c = r.count(12)?;
            let mut pts = Vec::with_capacity(c);
            for _ in 0..c {
                pts.push(r.vec3()?);
            }
            polygons.push(pts);
        }
        v.push(NfpArea { id, polygons, name: r.string()? });
    }
    r.finish()?;
    Ok(v)
}

/// Lumière ponctuelle (`.nfl`, 44 octets).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub position: [f32; 3],
    /// Trois réels (portée / atténuation supposées).
    pub params: [f32; 3],
    pub value: f32,
    /// Trois couleurs ARGB.
    pub colors: [u32; 3],
    pub kind: u32,
}

/// `.nfl` : 24 octets d'en-tête, `u32 n`, n × 44 octets.
#[derive(Debug, Clone, PartialEq)]
pub struct NflFile {
    pub header: [u32; 6],
    pub lights: Vec<PointLight>,
}

pub fn parse_nfl(bytes: &[u8]) -> Result<NflFile> {
    let mut r = Reader::new(bytes, "nfl");
    let mut header = [0u32; 6];
    for h in &mut header {
        *h = r.u32()?;
    }
    let n = r.count(44)?;
    let mut lights = Vec::with_capacity(n);
    for _ in 0..n {
        lights.push(PointLight {
            position: r.vec3()?,
            params: r.vec3()?,
            value: r.f32()?,
            colors: [r.u32()?, r.u32()?, r.u32()?],
            kind: r.u32()?,
        });
    }
    r.finish()?;
    Ok(NflFile { header, lights })
}

/// Objet de quête (`.qpf`) : enregistrement de 49 octets comme dans le `.nfm`, mais positions
/// relatives à la carte ; `record.index` = identifiant d'objet de quête (FieldPropResource).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuestProp {
    pub quest_prop_id: u32,
    pub record: PropRecord,
}

pub const QPF_SIGNATURE: &[u8; 17] = b"nFlavor QuestProp";

/// `.qpf` : signature 18 octets, `u32 version` (3), `u32 n`, n × 49 octets (`FUN_0099cc60`).
pub fn parse_qpf(bytes: &[u8]) -> Result<Vec<QuestProp>> {
    if bytes.len() < 0x1a || &bytes[..QPF_SIGNATURE.len()] != QPF_SIGNATURE {
        return Err(TerrainError::BadSignature { format: "qpf" });
    }
    let mut r = Reader::new(bytes, "qpf");
    r.seek(0x12)?;
    let version = r.u32()?;
    if version != 3 {
        return Err(TerrainError::UnsupportedVersion { format: "qpf", version });
    }
    let n = r.count(PROP_RECORD_SIZE)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let record = PropRecord::read(&mut r)?;
        v.push(QuestProp { quest_prop_id: record.index, record });
    }
    r.finish()?;
    Ok(v)
}

/// Fonction de script : `trigger` 0 = initialisation, 1 = contact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptFunction {
    pub trigger: u32,
    pub script: String,
}

/// Rectangle de script, en cases (×42 pour des unités monde).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptArea {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub name: String,
}

/// Événement attaché à un rectangle de [`NfsFile::areas`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptEvent {
    pub area_index: u32,
    pub functions: Vec<ScriptFunction>,
}

/// Script d'un objet du `.nfm` (PNJ posé comme objet, coffre…).
#[derive(Debug, Clone, PartialEq)]
pub struct PropScript {
    /// Identifiant global de l'objet (voir [`crate::PropPlacement::global_id`]).
    pub prop_id: u32,
    /// Relatif à la carte.
    pub position: [f32; 2],
    pub prop_num: u16,
    pub functions: Vec<ScriptFunction>,
}

/// `.nfs` « nFlavor Script ».
#[derive(Debug, Clone, PartialEq)]
pub struct NfsFile {
    pub version: u32,
    pub areas: Vec<ScriptArea>,
    pub events: Vec<ScriptEvent>,
    pub prop_scripts: Vec<PropScript>,
}

pub const NFS_SIGNATURE: &[u8; 14] = b"nFlavor Script";

fn read_functions(r: &mut Reader<'_>) -> Result<Vec<ScriptFunction>> {
    let n = r.count(8)?;
    (0..n).map(|_| Ok(ScriptFunction { trigger: r.u32()?, script: r.string()? })).collect()
}

/// `.nfs` : signature 16 octets, `u32 version` (2), 3 décalages de sections, puis
/// rectangles, événements, scripts d'objets (`FUN_00996570`).
pub fn parse_nfs(bytes: &[u8]) -> Result<NfsFile> {
    if bytes.len() < 0x20 || &bytes[..NFS_SIGNATURE.len()] != NFS_SIGNATURE {
        return Err(TerrainError::BadSignature { format: "nfs" });
    }
    let mut r = Reader::new(bytes, "nfs");
    r.seek(16)?;
    let version = r.u32()?;
    if version != 1 && version != 2 {
        return Err(TerrainError::UnsupportedVersion { format: "nfs", version });
    }
    let offs = [r.u32()? as usize, r.u32()? as usize, r.u32()? as usize];
    let check = |r: &Reader<'_>, i: usize| -> Result<()> {
        if r.pos() != offs[i] {
            return Err(TerrainError::Inconsistent(format!(
                "nfs : section {i} en {:#x}, l'en-tête indique {:#x}",
                r.pos(),
                offs[i]
            )));
        }
        Ok(())
    };
    check(&r, 0)?;
    let n = r.count(20)?;
    let mut areas = Vec::with_capacity(n);
    for _ in 0..n {
        areas.push(ScriptArea { left: r.i32()?, top: r.i32()?, right: r.i32()?, bottom: r.i32()?, name: r.string()? });
    }
    check(&r, 1)?;
    let n = r.count(8)?;
    let mut events = Vec::with_capacity(n);
    for _ in 0..n {
        events.push(ScriptEvent { area_index: r.u32()?, functions: read_functions(&mut r)? });
    }
    check(&r, 2)?;
    let n = r.count(18)?;
    let mut prop_scripts = Vec::with_capacity(n);
    for _ in 0..n {
        prop_scripts.push(PropScript {
            prop_id: r.u32()?,
            position: [r.f32()?, r.f32()?],
            prop_num: r.u16()?,
            functions: read_functions(&mut r)?,
        });
    }
    r.finish()?;
    Ok(NfsFile { version, areas, events, prop_scripts })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_roundtrip_layout() {
        let mut b = 1u32.to_le_bytes().to_vec();
        for f in [0.0f32, -36.0, 102.0, 16182.0, 16128.0, 102.0, 8091.0, 8046.0, 102.0] {
            b.extend(f.to_le_bytes());
        }
        b.extend(0u32.to_le_bytes());
        b.extend(1012u32.to_le_bytes());
        let w = parse_nfw(&b).unwrap();
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].water_id, 1012);
        assert_eq!(w[0].level(), 102.0);
        assert!(w[0].contains(100.0, 100.0));
        assert!(parse_nfw(&b[..b.len() - 1]).is_err());
    }

    #[test]
    fn polygon_contains() {
        let p = AttrPolygon { points: vec![[0, 0], [8, 0], [8, 8], [0, 8]] };
        assert!(p.contains(21.0, 21.0, 42.0));
        assert!(!p.contains(50.0, 21.0, 42.0));
    }
}
