//! `.nfm` « nFlavor Map » : en-tête, propriétés d'éclairage, relief par segments,
//! objets placés et herbe, zones d'herbe, puis copies des fichiers `.nfa`, `.nfw`, `.nfe`.

use std::collections::BTreeMap;

use crate::cfg::{PropInfoTable, TextureInfoTable};
use crate::reader::Reader;
use crate::sidecar::{self, AttrPolygon, EventArea, WaterArea};
use crate::{Result, TerrainError};

/// Signature en tête de fichier (champ de 16 octets complété par des `\0`).
pub const NFM_SIGNATURE: &[u8; 11] = b"nFlavor Map";

/// Taille d'un enregistrement d'objet (`.nfm` et `.qpf`).
pub(crate) const PROP_RECORD_SIZE: usize = 49;
/// Taille d'un sommet du relief.
const VERTEX_SIZE: usize = 23;
/// En-tête d'un segment : u32 version + 3 × u16 textures.
const SEGMENT_HEADER_SIZE: usize = 10;

/// En-tête du `.nfm`.
#[derive(Debug, Clone, PartialEq)]
pub struct NfmHeader {
    /// 22 (`0x16`) pour toutes les cartes du monde ; 16 pour quelques cartes inutilisées.
    pub version: u32,
    pub map_properties_offset: u32,
    pub terrain_segment_offset: u32,
    pub prop_offset: u32,
    /// Polygones d'attribut (copie du `.nfa`).
    pub vector_attr_offset: u32,
    /// Eau (copie du `.nfw`).
    pub water_offset: u32,
    /// Zones d'herbe procédurale (« grass colony »), version 22 seulement.
    pub grass_colony_offset: Option<u32>,
    /// Zones d'événement (copie du `.nfe`), version 22 seulement.
    pub event_area_offset: Option<u32>,
    pub tiles_per_segment: u32,
    pub segments_per_map: u32,
    pub tile_length: f32,
    /// 0x3C (v22) ou 0x34 (v16).
    pub header_size: usize,
}

/// Couleur 8 bits par composante.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Lumière directionnelle (copiée dans un `D3DLIGHT9` : diffuse en +4, ambiante en +0x24, direction en +0x40).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub diffuse: Rgb,
    pub ambient: Rgb,
    pub direction: [f32; 3],
}

/// Section 1 (55 octets).
#[derive(Debug, Clone, PartialEq)]
pub struct MapProperties {
    pub primary_light: Light,
    pub secondary_light: Light,
    pub sky_color: Rgb,
    pub fog_color: Rgb,
    pub fog_near: f32,
    pub fog_far: f32,
    pub sky_type: u32,
    /// Octet lu après les propriétés (client : `+0x128`). Nom supposé.
    pub show_terrain: bool,
}

/// En-tête d'un segment (6 × 6 cases).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentInfo {
    /// Toujours 0 dans les données.
    pub version: u32,
    /// Identifiants `TerrainTextureInfo.cfg` : couche de base, couche 1 (poids `fill_bits[0]`),
    /// couche 2 (poids `fill_bits[1]`).
    pub textures: [u16; 3],
}

/// Texture de sol utilisée par au moins un segment de la carte.
#[derive(Debug, Clone, PartialEq)]
pub struct TextureLayer {
    pub texture_id: u16,
    /// Nom du fichier `.dds` (vide tant que [`NfmMap::resolve_names`] n'a pas été appelé
    /// ou si l'identifiant est absent de `TerrainTextureInfo.cfg`).
    pub texture: String,
    /// `DETAIL` de `TerrainTextureInfo.cfg` (nombre de cases couvertes par une répétition).
    pub detail: Option<f32>,
    /// Nombre d'emplois (segment × couche) dans la carte.
    pub uses: usize,
}

/// Enregistrement brut d'objet, 49 octets (`.nfm` section objets et `.qpf`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropRecord {
    /// Indice de l'objet dans la carte (20 bits bas utiles ; le client y ajoute
    /// `(map_y·MAPSIZE_X + map_x) << 20`). Dans `.qpf` : identifiant de l'objet de quête.
    pub index: u32,
    /// `.nfm` : x, y relatifs au segment, z = décalage par rapport au sol.
    /// `.qpf` : x, y relatifs à la carte.
    pub position: [f32; 3],
    /// Radians.
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    /// Numéro `PROPNAME` de `TerrainPropInfo.cfg`.
    pub prop_num: u16,
    pub height_locked: bool,
    pub locked_height: f32,
    pub texture_group: i16,
}

impl PropRecord {
    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self> {
        Ok(Self {
            index: r.u32()?,
            position: r.vec3()?,
            rotation: r.vec3()?,
            scale: r.vec3()?,
            prop_num: r.u16()?,
            height_locked: r.u8()? != 0,
            locked_height: r.f32()?,
            texture_group: r.i16()?,
        })
    }
}

/// Objet 3D posé sur la carte.
#[derive(Debug, Clone, PartialEq)]
pub struct PropPlacement {
    /// Indice du segment (`sy·64 + sx`).
    pub segment: usize,
    /// Champ `index` brut (voir [`PropRecord::index`]).
    pub index: u32,
    pub prop_num: u16,
    /// Modèle (`.cob`) d'après `TerrainPropInfo.cfg` (recherche sur 16 bits, voir
    /// [`crate::prop_info_by_u16`]) ; vide avant [`NfmMap::resolve_names`].
    pub model: String,
    /// Position relative à la carte ; `z` = hauteur du sol en (x, y) + `z_offset`
    /// (ou `locked_height` si `height_locked`).
    pub position: [f32; 3],
    pub z_offset: f32,
    /// Radians (x, y, z) ; z = lacet autour de la verticale.
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub height_locked: bool,
    pub locked_height: f32,
    pub texture_group: i16,
}

impl PropPlacement {
    /// Identifiant global calculé par le client (`FUN_00b00de0`) :
    /// `(index & 0xFFFFF) | ((map_y·map_size_x + map_x) << 20)`, `map_size_x` = 14 (`MAPSIZE`).
    pub fn global_id(&self, map_x: u32, map_y: u32, map_size_x: u32) -> u32 {
        (self.index & 0xF_FFFF) | ((map_y * map_size_x + map_x) << 20)
    }
}

/// Touffe d'herbe « SpeedGrass ».
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedGrassPoint {
    /// Relatif à la carte.
    pub position: [f32; 2],
    /// Vecteur unitaire (normale du sol, d'après les valeurs).
    pub normal: [f32; 3],
}

/// Groupe d'herbe « SpeedGrass » d'un segment (bloc d'objets, version 22 seulement).
#[derive(Debug, Clone, PartialEq)]
pub struct SpeedGrass {
    pub segment: usize,
    pub grass_id: u32,
    pub points: Vec<SpeedGrassPoint>,
}

/// Zone d'herbe procédurale (section 6 du `.nfm`, 205 octets pour une zone simple).
#[derive(Debug, Clone, PartialEq)]
pub struct GrassColony {
    pub id: u32,
    /// Cinq réels (densité, taille… : rôle exact inconnu).
    pub params: [f32; 5],
    /// Couleur ARGB.
    pub color: u32,
    /// Six réels (rôle inconnu ; le 5e vaut toujours 50).
    pub params2: [f32; 6],
    /// Texture de l'herbe (`grass_mix2.dds`…), parfois vide.
    pub texture: String,
    /// Polygones en unités monde relatives à la carte.
    pub polygons: Vec<Vec<[i32; 2]>>,
}

fn read_grass_colonies(sec: &[u8]) -> Result<Vec<GrassColony>> {
    let mut r = Reader::new(sec, "nfm grass colony");
    let n = r.count(60)?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let id = r.u32()?;
        let mut params = [0f32; 5];
        for p in &mut params {
            *p = r.f32()?;
        }
        let color = r.u32()?;
        let mut params2 = [0f32; 6];
        for p in &mut params2 {
            *p = r.f32()?;
        }
        let texture = r.string()?;
        let np = r.count(4)?;
        let mut polygons = Vec::with_capacity(np);
        for _ in 0..np {
            let c = r.count(8)?;
            let mut pts = Vec::with_capacity(c);
            for _ in 0..c {
                pts.push([r.i32()?, r.i32()?]);
            }
            polygons.push(pts);
        }
        v.push(GrassColony { id, params, color, params2, texture, polygons });
    }
    r.finish()?;
    Ok(v)
}

/// Carte `.nfm` décodée.
#[derive(Debug, Clone)]
pub struct NfmMap {
    pub header: NfmHeader,
    pub properties: MapProperties,
    pub tiles_per_segment: usize,
    pub segments_per_map: usize,
    /// Pas de la grille (42).
    pub tile_length: f32,
    /// Sommets par côté (`segments_per_map × tiles_per_segment` = 384).
    pub size: usize,
    /// Hauteurs, `heights[j·size + i]` = sommet en (i·tile_length, j·tile_length).
    pub heights: Vec<f32>,
    /// Couleur (éclairage précalculé) par sommet, même indexation.
    pub colors: Vec<Rgb>,
    /// Poids des couches 1 et 2 de la case dont ce sommet est le coin bas-gauche :
    /// 4 × 4 sous-cases × 2 bits (valeur 0..3, alpha = valeur × 85), bit `2·(sy·4 + sx)`.
    pub fill_bits: Vec<[u32; 2]>,
    /// Attribut de la case : 8 × 8 sous-cases × 1 bit, bit `sy·8 + sx`, 1 = bloqué.
    pub attributes: Vec<u64>,
    /// `segments[sy·segments_per_map + sx]`.
    pub segments: Vec<SegmentInfo>,
    /// Textures distinctes employées (triées par identifiant).
    pub texture_layers: Vec<TextureLayer>,
    pub props: Vec<PropPlacement>,
    pub speed_grass: Vec<SpeedGrass>,
    /// Section « vector attribute » (≈ `.nfa`).
    pub attr_polygons: Vec<AttrPolygon>,
    /// Section eau (format `.nfw`) ; vide dans toutes les cartes du monde de ce client.
    pub water: Vec<WaterArea>,
    /// Section 6 : zones d'herbe procédurale.
    pub grass_colonies: Vec<GrassColony>,
    /// Section 7 (= `.nfe`).
    pub event_areas: Vec<EventArea>,
    /// Sections annexes illisibles (laissées vides) ; vide pour toutes les cartes du monde.
    pub warnings: Vec<String>,
}

fn read_rgb(r: &mut Reader<'_>) -> Result<Rgb> {
    Ok(Rgb { r: r.u8()?, g: r.u8()?, b: r.u8()? })
}

fn read_light(r: &mut Reader<'_>) -> Result<Light> {
    Ok(Light { diffuse: read_rgb(r)?, ambient: read_rgb(r)?, direction: r.vec3()? })
}

fn read_properties(sec: &[u8]) -> Result<MapProperties> {
    let mut r = Reader::new(sec, "nfm propriétés");
    let p = MapProperties {
        primary_light: read_light(&mut r)?,
        secondary_light: read_light(&mut r)?,
        sky_color: read_rgb(&mut r)?,
        fog_color: read_rgb(&mut r)?,
        fog_near: r.f32()?,
        fog_far: r.f32()?,
        sky_type: r.u32()?,
        show_terrain: r.u8()? != 0,
    };
    r.finish()?;
    Ok(p)
}

fn read_header(bytes: &[u8]) -> Result<NfmHeader> {
    if bytes.len() < 16 || &bytes[..NFM_SIGNATURE.len()] != NFM_SIGNATURE {
        return Err(TerrainError::BadSignature { format: "nfm" });
    }
    let mut r = Reader::new(bytes, "nfm en-tête");
    r.seek(16)?;
    let version = r.u32()?;
    let n_offsets = match version {
        22 => 7,
        16 => 5,
        v => return Err(TerrainError::UnsupportedVersion { format: "nfm", version: v }),
    };
    let mut off = [0u32; 7];
    for o in off.iter_mut().take(n_offsets) {
        *o = r.u32()?;
    }
    let tiles_per_segment = r.u32()?;
    let segments_per_map = r.u32()?;
    let tile_length = r.f32()?;
    if !(1..=64).contains(&tiles_per_segment)
        || !(1..=1024).contains(&segments_per_map)
        || !(tile_length.is_finite() && tile_length > 0.0)
    {
        return Err(TerrainError::Inconsistent(format!(
            "nfm : grille invalide ({tiles_per_segment} cases/segment, {segments_per_map} segments/carte, case {tile_length})"
        )));
    }
    Ok(NfmHeader {
        version,
        map_properties_offset: off[0],
        terrain_segment_offset: off[1],
        prop_offset: off[2],
        vector_attr_offset: off[3],
        water_offset: off[4],
        grass_colony_offset: (n_offsets == 7).then_some(off[5]),
        event_area_offset: (n_offsets == 7).then_some(off[6]),
        tiles_per_segment,
        segments_per_map,
        tile_length,
        header_size: r.pos(),
    })
}

/// Bloc d'objets d'un segment : u32 n, n × 49 octets, puis (v22) u32 g et g groupes d'herbe.
type PropBlock = (Vec<PropRecord>, Vec<(u32, Vec<[f32; 5]>)>, usize);

fn read_prop_block(sec: &[u8], start: usize, with_grass: bool) -> Result<PropBlock> {
    let mut r = Reader::new(sec, "nfm objets");
    r.seek(start)?;
    let n = r.count(PROP_RECORD_SIZE)?;
    let mut props = Vec::with_capacity(n);
    for _ in 0..n {
        props.push(PropRecord::read(&mut r)?);
    }
    let mut grass = Vec::new();
    if with_grass {
        let g = r.count(8)?;
        for _ in 0..g {
            let id = r.u32()?;
            let c = r.count(20)?;
            let mut pts = Vec::with_capacity(c);
            for _ in 0..c {
                pts.push([r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?]);
            }
            grass.push((id, pts));
        }
    }
    Ok((props, grass, r.pos()))
}

/// Décode un `.nfm`.
///
/// L'en-tête, les propriétés, le relief et la table des objets doivent être cohérents (sinon
/// erreur). Un bloc d'objets ou une section annexe (`.nfa`/`.nfw`/herbe/`.nfe`) illisible est
/// ignoré et signalé dans [`NfmMap::warnings`] (cas de quelques cartes absentes de
/// `TerrainSeamlessWorld.cfg`).
pub fn parse_nfm(bytes: &[u8]) -> Result<NfmMap> {
    let header = read_header(bytes)?;

    // Sections dans l'ordre du fichier : chacune s'arrête au début de la suivante.
    let mut starts = vec![
        header.map_properties_offset,
        header.terrain_segment_offset,
        header.prop_offset,
        header.vector_attr_offset,
        header.water_offset,
    ];
    starts.extend(header.grass_colony_offset);
    starts.extend(header.event_area_offset);
    let mut bounds = Vec::with_capacity(starts.len());
    let mut expected = header.header_size;
    for (i, &s) in starts.iter().enumerate() {
        let s = s as usize;
        let e = starts.get(i + 1).map(|&x| x as usize).unwrap_or(bytes.len());
        if s != expected || e < s || e > bytes.len() {
            return Err(TerrainError::Inconsistent(format!(
                "nfm : section {i} en {s:#x}..{e:#x} (attendu à partir de {expected:#x}, fichier de {:#x} octets)",
                bytes.len()
            )));
        }
        bounds.push((s, e));
        expected = e;
    }
    let sec = |i: usize| &bytes[bounds[i].0..bounds[i].1];

    let properties = read_properties(sec(0))?;

    // Relief.
    let t = header.tiles_per_segment as usize;
    let segs = header.segments_per_map as usize;
    let size = t * segs;
    let seg_size = SEGMENT_HEADER_SIZE + t * t * VERTEX_SIZE;
    let terrain = sec(1);
    if terrain.len() != segs * segs * seg_size {
        return Err(TerrainError::Inconsistent(format!(
            "nfm : relief de {} octets, attendu {} ({}² segments × {seg_size})",
            terrain.len(),
            segs * segs * seg_size,
            segs
        )));
    }
    let mut heights = vec![0f32; size * size];
    let mut colors = vec![Rgb::default(); size * size];
    let mut fill_bits = vec![[0u32; 2]; size * size];
    let mut attributes = vec![0u64; size * size];
    let mut segments = Vec::with_capacity(segs * segs);
    let mut r = Reader::new(terrain, "nfm relief");
    for s in 0..segs * segs {
        let (sx, sy) = (s % segs, s / segs);
        segments.push(SegmentInfo { version: r.u32()?, textures: [r.u16()?, r.u16()?, r.u16()?] });
        for v in 0..t * t {
            let (vx, vy) = (v % t, v / t);
            let idx = (sy * t + vy) * size + sx * t + vx;
            heights[idx] = r.f32()?;
            fill_bits[idx] = [r.u32()?, r.u32()?];
            attributes[idx] = r.u64()?;
            // Ordre B, G, R (FUN_00af9960).
            let (b, g, rr) = (r.u8()?, r.u8()?, r.u8()?);
            colors[idx] = Rgb { r: rr, g, b };
        }
    }
    r.finish()?;

    let mut warnings = Vec::new();

    // Objets : table de segs² décalages absolus, puis un bloc par segment.
    let prop_sec = sec(2);
    let table_len = segs * segs * 4;
    if prop_sec.len() < table_len {
        return Err(TerrainError::Inconsistent(format!(
            "nfm : section objets de {} octets, table de {table_len}",
            prop_sec.len()
        )));
    }
    let with_grass = header.version >= 22;
    let seg_len = t as f32 * header.tile_length;
    let mut raw_props = Vec::new();
    let mut speed_grass = Vec::new();
    let mut blocks = Vec::with_capacity(segs * segs);
    let mut tr = Reader::new(prop_sec, "nfm table des objets");
    for s in 0..segs * segs {
        let abs = tr.u32()? as usize;
        let parsed = abs
            .checked_sub(bounds[2].0)
            .filter(|&rel| rel >= table_len && rel < prop_sec.len())
            .ok_or_else(|| TerrainError::Inconsistent(format!("décalage {abs:#x} hors section")))
            .and_then(|rel| read_prop_block(prop_sec, rel, with_grass).map(|b| (rel, b)));
        match parsed {
            Ok((rel, (props, colonies, end))) => {
                blocks.push((rel, end));
                let origin = [(s % segs) as f32 * seg_len, (s / segs) as f32 * seg_len];
                raw_props.extend(props.into_iter().map(|p| (s, origin, p)));
                for (grass_id, pts) in colonies {
                    speed_grass.push(SpeedGrass {
                        segment: s,
                        grass_id,
                        points: pts
                            .into_iter()
                            .map(|p| SpeedGrassPoint {
                                position: [origin[0] + p[0], origin[1] + p[1]],
                                normal: [p[2], p[3], p[4]],
                            })
                            .collect(),
                    });
                }
            }
            Err(e) => warnings.push(format!("objets du segment {s} illisibles : {e}")),
        }
    }
    blocks.sort_unstable();
    let mut cursor = table_len;
    for &(start, end) in &blocks {
        if start != cursor {
            warnings.push(format!("section objets : bloc en {start:#x}, attendu {cursor:#x}"));
            break;
        }
        cursor = end;
    }
    if warnings.is_empty() && cursor != prop_sec.len() {
        warnings.push(format!("section objets : {cursor} octet(s) lus sur {}", prop_sec.len()));
    }

    let mut map = NfmMap {
        header,
        properties,
        tiles_per_segment: t,
        segments_per_map: segs,
        tile_length: 0.0,
        size,
        heights,
        colors,
        fill_bits,
        attributes,
        segments,
        texture_layers: Vec::new(),
        props: Vec::new(),
        speed_grass,
        attr_polygons: Vec::new(),
        water: Vec::new(),
        grass_colonies: Vec::new(),
        event_areas: Vec::new(),
        warnings,
    };
    map.tile_length = map.header.tile_length;

    let mut uses: BTreeMap<u16, usize> = BTreeMap::new();
    for s in &map.segments {
        for &id in &s.textures {
            *uses.entry(id).or_default() += 1;
        }
    }
    map.texture_layers = uses
        .into_iter()
        .map(|(texture_id, uses)| TextureLayer { texture_id, texture: String::new(), detail: None, uses })
        .collect();

    map.props = raw_props
        .into_iter()
        .map(|(segment, origin, p)| {
            let (x, y) = (origin[0] + p.position[0], origin[1] + p.position[1]);
            let z = if p.height_locked { p.locked_height } else { map.height_at(x, y) + p.position[2] };
            PropPlacement {
                segment,
                index: p.index,
                prop_num: p.prop_num,
                model: String::new(),
                position: [x, y, z],
                z_offset: p.position[2],
                rotation: p.rotation,
                scale: p.scale,
                height_locked: p.height_locked,
                locked_height: p.locked_height,
                texture_group: p.texture_group,
            }
        })
        .collect();

    match sidecar::parse_nfa(sec(3)) {
        Ok(v) => map.attr_polygons = v,
        Err(e) => map.warnings.push(format!("section attributs (.nfa) : {e}")),
    }
    match sidecar::parse_nfw(sec(4)) {
        Ok(v) => map.water = v,
        Err(e) => map.warnings.push(format!("section eau (.nfw) : {e}")),
    }
    if map.header.grass_colony_offset.is_some() {
        match read_grass_colonies(sec(5)) {
            Ok(v) => map.grass_colonies = v,
            Err(e) => map.warnings.push(format!("section grass colony : {e}")),
        }
        match sidecar::parse_nfe(sec(6)) {
            Ok(v) => map.event_areas = v,
            Err(e) => map.warnings.push(format!("section événements (.nfe) : {e}")),
        }
    }
    Ok(map)
}

/// Interpolation du client (`FUN_00afa200`) : chaque case est coupée selon la diagonale
/// (i+1, j)–(i, j+1). `fx`, `fy` sont en cases ; `h(i, j)` donne la hauteur d'un sommet.
pub(crate) fn interpolate(fx: f32, fy: f32, h: impl Fn(i64, i64) -> f32) -> f32 {
    // Positions aberrantes (NaN, infinies) : bornées pour ne pas déborder les indices.
    let sane = |v: f32| if v.is_finite() { v.clamp(-1.0e7, 1.0e7) } else { 0.0 };
    let (fx, fy) = (sane(fx), sane(fy));
    let (i, j) = (fx.floor(), fy.floor());
    let (u, v) = (fx - i, fy - j);
    let (i, j) = (i as i64, j as i64);
    if u + v < 1.0 {
        let h00 = h(i, j);
        h00 + (h(i + 1, j) - h00) * u + (h(i, j + 1) - h00) * v
    } else {
        let h11 = h(i + 1, j + 1);
        h11 + (h(i, j + 1) - h11) * (1.0 - u) + (h(i + 1, j) - h11) * (1.0 - v)
    }
}

impl NfmMap {
    /// Côté de la carte en unités monde (16128).
    pub fn extent(&self) -> f32 {
        self.size as f32 * self.tile_length
    }

    fn vidx(&self, i: i64, j: i64) -> usize {
        let m = self.size as i64 - 1;
        (j.clamp(0, m) * self.size as i64 + i.clamp(0, m)) as usize
    }

    /// Hauteur du sommet (i, j), indices bornés à la grille.
    pub fn vertex_height(&self, i: i64, j: i64) -> f32 {
        self.heights[self.vidx(i, j)]
    }

    /// Couleur du sommet (i, j), indices bornés.
    pub fn vertex_color(&self, i: i64, j: i64) -> Rgb {
        self.colors[self.vidx(i, j)]
    }

    /// Hauteur du sol en (x, y) relatifs à la carte, triangulée comme le client.
    /// Au-delà du dernier sommet (x ou y > 383·42) la grille est prolongée par sa dernière
    /// rangée : utiliser [`TerrainWorld`] pour raccorder la carte voisine.
    pub fn height_at(&self, local_x: f32, local_y: f32) -> f32 {
        interpolate(local_x / self.tile_length, local_y / self.tile_length, |i, j| self.vertex_height(i, j))
    }

    /// (case, sous-case) pour une subdivision `div` de la case.
    fn cell(&self, local_x: f32, local_y: f32, div: f32) -> (usize, u32, u32) {
        let fx = (local_x / self.tile_length).max(0.0);
        let fy = (local_y / self.tile_length).max(0.0);
        let m = self.size - 1;
        let (i, j) = ((fx as usize).min(m), (fy as usize).min(m));
        let sx = (((fx - i as f32) * div) as u32).min(div as u32 - 1);
        let sy = (((fy - j as f32) * div) as u32).min(div as u32 - 1);
        (j * self.size + i, sx, sy)
    }

    /// Segment contenant (x, y).
    pub fn segment_at(&self, local_x: f32, local_y: f32) -> usize {
        let (idx, _, _) = self.cell(local_x, local_y, 1.0);
        let (i, j) = (idx % self.size, idx / self.size);
        (j / self.tiles_per_segment) * self.segments_per_map + i / self.tiles_per_segment
    }

    /// Identifiants des 3 textures du segment contenant (x, y).
    pub fn textures_at(&self, local_x: f32, local_y: f32) -> [u16; 3] {
        self.segments[self.segment_at(local_x, local_y)].textures
    }

    /// Opacité (0..255) des 3 couches en (x, y) : la base vaut toujours 255, les couches 1 et 2
    /// valent `valeur 2 bits × 85` de la sous-case 1/4 de case (`FUN_00af9d70`, `FUN_00afa150`).
    pub fn layer_weights_at(&self, local_x: f32, local_y: f32) -> [u8; 3] {
        let (idx, sx, sy) = self.cell(local_x, local_y, 4.0);
        let sh = 2 * (sy * 4 + sx);
        let f = self.fill_bits[idx];
        [255, (((f[0] >> sh) & 3) * 85) as u8, (((f[1] >> sh) & 3) * 85) as u8]
    }

    /// Texture « dominante » de la case (`FUN_00af9e80`) : couche 2 si `fill_bits[1] != 0`,
    /// sinon couche 1 si `fill_bits[0] != 0`, sinon la base.
    pub fn dominant_texture_at(&self, local_x: f32, local_y: f32) -> u16 {
        let (idx, _, _) = self.cell(local_x, local_y, 1.0);
        let tex = self.textures_at(local_x, local_y);
        let f = self.fill_bits[idx];
        if f[1] != 0 {
            tex[2]
        } else if f[0] != 0 {
            tex[1]
        } else {
            tex[0]
        }
    }

    /// Carte d'opacité d'une couche (1 ou 2) en 1/4 de case : `(size·4)²` octets, ligne `y` d'abord.
    pub fn layer_alpha_map(&self, layer: usize) -> Vec<u8> {
        assert!(layer == 1 || layer == 2, "couche 1 ou 2");
        let n = self.size * 4;
        let mut out = vec![0u8; n * n];
        for (idx, f) in self.fill_bits.iter().enumerate() {
            let (i, j) = (idx % self.size, idx / self.size);
            let bits = f[layer - 1];
            for sy in 0..4 {
                for sx in 0..4 {
                    out[(j * 4 + sy) * n + i * 4 + sx] = (((bits >> (2 * (sy * 4 + sx))) & 3) * 85) as u8;
                }
            }
        }
        out
    }

    /// Bit d'attribut (1 = bloqué) de la sous-case 1/8 contenant (x, y) (`FUN_00af98f0`).
    pub fn blocked_at(&self, local_x: f32, local_y: f32) -> bool {
        let (idx, sx, sy) = self.cell(local_x, local_y, 8.0);
        (self.attributes[idx] >> (sy * 8 + sx)) & 1 != 0
    }

    /// Renseigne les noms de textures et de modèles à partir des fichiers de configuration.
    pub fn resolve_names(&mut self, textures: Option<&TextureInfoTable>, props: Option<&PropInfoTable>) {
        if let Some(t) = textures {
            for l in &mut self.texture_layers {
                if let Some(info) = t.get(&(l.texture_id as u32)) {
                    l.texture = info.file.clone();
                    l.detail = info.detail;
                }
            }
        }
        if let Some(p) = props {
            let by16 = crate::cfg::prop_info_by_u16(p);
            for prop in &mut self.props {
                if let Some(info) = by16.get(&prop.prop_num) {
                    prop.model = info.file.clone();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation_hits_vertices_and_is_planar() {
        let h = |i: i64, j: i64| (i * 10 + j * 100) as f32;
        for (x, y) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0), (0.25, 0.5), (0.9, 0.8)] {
            assert!((interpolate(x, y, h) - (x * 10.0 + y * 100.0)).abs() < 1e-3);
        }
        // Coupe selon la diagonale (1,0)–(0,1) : seul le sommet (1,1) compte au-delà.
        let peak = |i: i64, j: i64| if i == 1 && j == 1 { 30.0 } else { 0.0 };
        assert_eq!(interpolate(0.4, 0.4, peak), 0.0);
        assert!((interpolate(0.75, 0.75, peak) - 15.0).abs() < 1e-4);
    }

    #[test]
    fn rejects_bad_signature() {
        assert!(matches!(parse_nfm(b"nFlavor Script\0\0\x16\0\0\0"), Err(TerrainError::BadSignature { .. })));
    }
}
