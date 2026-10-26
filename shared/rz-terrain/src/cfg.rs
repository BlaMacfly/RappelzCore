//! Fichiers texte du monde (dans l'archive `data.00x`) :
//! `TerrainSeamlessWorld.cfg`, `TerrainTextureInfo.cfg`, `TerrainPropInfo.cfg`.
//! Lignes `CLE=valeur`, `;` en début de ligne = commentaire, texte EUC-KR.

use std::collections::HashMap;

use crate::{Result, TerrainError};

fn lines(bytes: &[u8]) -> impl Iterator<Item = (String, String)> {
    let (text, _, _) = encoding_rs::EUC_KR.decode(bytes);
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with(';') {
                return None;
            }
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_ascii_uppercase(), v.trim().to_string()))
        })
        .collect::<Vec<_>>()
        .into_iter()
}

/// Entrée `TEXTURE=id,fichier` de `TerrainTextureInfo.cfg`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextureInfo {
    pub file: String,
    pub category: String,
    pub detail: Option<f32>,
}

pub type TextureInfoTable = HashMap<u32, TextureInfo>;

/// `TerrainTextureInfo.cfg` : `CATEGORY` et `DETAIL` s'appliquent aux `TEXTURE` qui suivent ;
/// un identifiant répété garde la dernière définition.
pub fn parse_texture_info(bytes: &[u8]) -> TextureInfoTable {
    let mut out = HashMap::new();
    let (mut category, mut detail) = (String::new(), None);
    for (k, v) in lines(bytes) {
        match k.as_str() {
            "CATEGORY" => category = v,
            // « 4(X4) » existe : on garde le nombre de tête.
            "DETAIL" => {
                let num: String = v.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                detail = num.parse().ok();
            }
            "TEXTURE" => {
                if let Some((id, file)) = v.split_once(',') {
                    if let Ok(id) = id.trim().parse() {
                        out.insert(id, TextureInfo { file: file.trim().to_string(), category: category.clone(), detail });
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Entrée `PROPNAME=id,fichier` de `TerrainPropInfo.cfg`.
#[derive(Debug, Clone, PartialEq)]
pub struct PropInfo {
    pub file: String,
    pub category: String,
    /// `general` ou `building`.
    pub render_type: String,
    /// `normal`, `collision` ou `none`.
    pub lightmap_type: String,
    pub visible_ratio: f32,
    /// Rang de la définition dans le fichier (départage les numéros égaux modulo 65536).
    pub order: usize,
}

pub type PropInfoTable = HashMap<u32, PropInfo>;

/// Les objets du `.nfm` ne portent que 16 bits du numéro (`PROPNAME=215427` est référencé
/// par 18819) : table indexée par `numéro & 0xFFFF`, la dernière définition l'emportant.
pub fn prop_info_by_u16(table: &PropInfoTable) -> HashMap<u16, &PropInfo> {
    let mut out: HashMap<u16, &PropInfo> = HashMap::new();
    for (&id, info) in table {
        let e = out.entry(id as u16).or_insert(info);
        if info.order > e.order {
            *e = info;
        }
    }
    out
}

/// `TerrainPropInfo.cfg` : `CATEGORY`, `RENDERTYPE`, `LIGHTMAPTYPE`, `VISIBLE_RATIO`
/// s'appliquent aux `PROPNAME` suivants.
pub fn parse_prop_info(bytes: &[u8]) -> PropInfoTable {
    let mut out = HashMap::new();
    let mut cur = PropInfo {
        file: String::new(),
        category: String::new(),
        render_type: "general".into(),
        lightmap_type: "normal".into(),
        visible_ratio: 1.0,
        order: 0,
    };
    for (k, v) in lines(bytes) {
        match k.as_str() {
            "CATEGORY" => cur.category = v,
            "RENDERTYPE" => cur.render_type = v,
            "LIGHTMAPTYPE" => cur.lightmap_type = v,
            "VISIBLE_RATIO" => {
                let r: f32 = v.parse().unwrap_or(1.0);
                cur.visible_ratio = if r > 0.0 && r <= 1.0 { r } else { 1.0 };
            }
            "PROPNAME" => {
                if let Some((id, file)) = v.split_once(',') {
                    if let Ok(id) = id.trim().parse() {
                        let order = out.len();
                        out.insert(id, PropInfo { file: file.trim().to_string(), order, ..cur.clone() });
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Ligne `MAPFILE=x,y,couche,nom,id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapFile {
    pub x: u32,
    pub y: u32,
    pub layer: u32,
    /// Nom sans extension (`M000_000`).
    pub name: String,
    pub id: u32,
}

/// `TerrainSeamlessWorld.cfg` (lu par `FUN_00b1f920`).
#[derive(Debug, Clone, PartialEq)]
pub struct SeamlessWorld {
    pub tile_length: f32,
    pub tiles_per_segment: u32,
    pub segments_per_map: u32,
    pub fov: f32,
    pub map_layer: u32,
    /// `MAPSIZE=largeur,hauteur` en cartes (14, 16).
    pub map_size: (u32, u32),
    pub maps: Vec<MapFile>,
}

impl SeamlessWorld {
    /// Côté d'une carte en unités monde.
    pub fn map_extent(&self) -> f32 {
        self.tile_length * (self.tiles_per_segment * self.segments_per_map) as f32
    }
}

pub fn parse_seamless_world(bytes: &[u8]) -> Result<SeamlessWorld> {
    let mut w = SeamlessWorld {
        tile_length: 0.0,
        tiles_per_segment: 0,
        segments_per_map: 0,
        fov: 0.0,
        map_layer: 0,
        map_size: (0, 0),
        maps: Vec::new(),
    };
    let bad = |k: &str, v: &str| TerrainError::Inconsistent(format!("TerrainSeamlessWorld.cfg : {k}={v}"));
    for (k, v) in lines(bytes) {
        match k.as_str() {
            "TILE_LENGTH" => w.tile_length = v.parse().map_err(|_| bad(&k, &v))?,
            "TILECOUNT_PER_SEGMENT" => w.tiles_per_segment = v.parse().map_err(|_| bad(&k, &v))?,
            "SEGMENTCOUNT_PER_MAP" => w.segments_per_map = v.parse().map_err(|_| bad(&k, &v))?,
            "SETFOV" => w.fov = v.parse().unwrap_or(0.0),
            "MAPLAYER" => w.map_layer = v.parse().unwrap_or(0),
            "MAPSIZE" => {
                let (a, b) = v.split_once(',').ok_or_else(|| bad(&k, &v))?;
                w.map_size = (a.trim().parse().map_err(|_| bad(&k, &v))?, b.trim().parse().map_err(|_| bad(&k, &v))?);
            }
            "MAPFILE" => {
                let p: Vec<&str> = v.split(',').map(str::trim).collect();
                if p.len() < 4 {
                    return Err(bad(&k, &v));
                }
                let num = |s: &str| s.parse::<u32>().map_err(|_| bad(&k, &v));
                w.maps.push(MapFile {
                    x: num(p[0])?,
                    y: num(p[1])?,
                    layer: num(p[2])?,
                    name: p[3].to_string(),
                    id: p.get(4).map(|s| num(s)).transpose()?.unwrap_or(0),
                });
            }
            _ => {}
        }
    }
    if w.tile_length <= 0.0 || w.tiles_per_segment == 0 || w.segments_per_map == 0 {
        return Err(TerrainError::Inconsistent("TerrainSeamlessWorld.cfg : grille incomplète".into()));
    }
    Ok(w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_cfg() {
        let w = parse_seamless_world(
            b"; c\r\nTILE_LENGTH=42\r\nTILECOUNT_PER_SEGMENT=6\r\nSEGMENTCOUNT_PER_MAP=64\r\nMAPSIZE=14,16\r\nMAPFILE=7,3,0,M007_003,70300\r\n",
        )
        .unwrap();
        assert_eq!(w.map_extent(), 16128.0);
        assert_eq!(w.map_size, (14, 16));
        assert_eq!(w.maps[0], MapFile { x: 7, y: 3, layer: 0, name: "M007_003".into(), id: 70300 });
    }

    #[test]
    fn texture_and_prop_cfg() {
        let t = parse_texture_info(b"DETAIL=2\nCATEGORY=a\nTEXTURE=21101,grass@normal.DDS\nDETAIL=4(X4)\nTEXTURE=5,x.dds\n");
        assert_eq!(t[&21101].file, "grass@normal.DDS");
        assert_eq!(t[&21101].detail, Some(2.0));
        assert_eq!(t[&5].detail, Some(4.0));
        let p = parse_prop_info(b"RENDERTYPE=building\nVISIBLE_RATIO=0.5\nPROPNAME=9,zone001_stone07.cob\n");
        assert_eq!(p[&9].file, "zone001_stone07.cob");
        assert_eq!(p[&9].render_type, "building");
        assert_eq!(p[&9].visible_ratio, 0.5);
    }
}
