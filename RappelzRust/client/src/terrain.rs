//! Terrain Rappelz (`rz-terrain`) : relief des `.nfm`, textures de sol mélangées (3 couches par
//! segment, `TerrainTextureInfo.cfg`) précalculées par morceau de 1008 unités, éclairage par
//! sommet comme le client (`couleur × min(1, diffuse·max(0, −n·dir) + ambiante)`), décors placés
//! (`TerrainPropInfo.cfg` → `.cob`).
//!
//! Repère monde : X est, Y nord, Z haut ; tuile `mXXX_YYY` à l'origine (XXX·16128, YYY·16128).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use macroquad::models::{Mesh as QMesh, Vertex};
use macroquad::prelude::*;
use rz_terrain::{NfmMap, PropInfoTable, TextureInfoTable};
use rz_texture::Rgba;

use crate::assets::Assets;
use crate::models::{self, ModelCache};

pub const MAP_SIZE: f32 = 16128.0;
/// Côté d'un morceau de rendu (= une tuile de minicarte, 24 cases de 42).
pub const CHUNK: f32 = 1008.0;
const CHUNK_CELLS: usize = 24;
const BAKE_PX: usize = 128;
/// Morceaux construits par image au plus (évite les saccades).
const BUILDS_PER_FRAME: usize = 2;

struct Chunk {
    mesh: QMesh,
}

pub struct Terrain {
    tex_info: TextureInfoTable,
    prop_info: PropInfoTable,
    maps: HashMap<(i32, i32), Option<Arc<NfmMap>>>,
    chunks: HashMap<(i32, i32), Chunk>,
    layers: HashMap<String, Option<Arc<Rgba>>>,
    pub view_chunks: i32,
    pub prop_distance: f32,
}

impl Terrain {
    pub fn new(assets: &mut Assets) -> Self {
        let tex_info = assets.read("TerrainTextureInfo.cfg").map(|b| rz_terrain::parse_texture_info(&b)).unwrap_or_default();
        let prop_info = assets.read("TerrainPropInfo.cfg").map(|b| rz_terrain::parse_prop_info(&b)).unwrap_or_default();
        tracing::info!("terrain : {} textures de sol, {} décors référencés", tex_info.len(), prop_info.len());
        Self {
            tex_info,
            prop_info,
            maps: HashMap::new(),
            chunks: HashMap::new(),
            layers: HashMap::new(),
            view_chunks: 3,
            prop_distance: 1800.0,
        }
    }

    pub fn map(&mut self, assets: &mut Assets, mx: i32, my: i32) -> Option<Arc<NfmMap>> {
        if mx < 0 || my < 0 {
            return None;
        }
        if let Some(m) = self.maps.get(&(mx, my)) {
            return m.clone();
        }
        let name = format!("m{mx:03}_{my:03}.nfm");
        let map = assets.read(&name).and_then(|b| match rz_terrain::parse_nfm(&b) {
            Ok(mut m) => {
                m.resolve_names(Some(&self.tex_info), Some(&self.prop_info));
                Some(Arc::new(m))
            }
            Err(e) => {
                tracing::warn!("{name} : {e}");
                None
            }
        });
        self.maps.insert((mx, my), map.clone());
        map
    }

    /// Hauteur du sol au point monde (x, y) ; 0 hors carte.
    pub fn height_at(&mut self, assets: &mut Assets, x: f32, y: f32) -> f32 {
        let (mx, my) = ((x / MAP_SIZE).floor() as i32, (y / MAP_SIZE).floor() as i32);
        match self.map(assets, mx, my) {
            Some(m) => m.height_at(x - mx as f32 * MAP_SIZE, y - my as f32 * MAP_SIZE),
            None => 0.0,
        }
    }

    fn layer_image(&mut self, assets: &mut Assets, name: &str) -> Option<Arc<Rgba>> {
        if name.is_empty() {
            return None;
        }
        if let Some(l) = self.layers.get(name) {
            return l.clone();
        }
        let img = assets.resolve_image(name).and_then(|(n, b)| rz_texture::decode(&n, &b).ok()).map(Arc::new);
        self.layers.insert(name.to_string(), img.clone());
        img
    }

    /// Texture du sol d'un morceau : mélange des 3 couches de chaque segment.
    fn bake(&mut self, assets: &mut Assets, map: &NfmMap, ox: f32, oy: f32) -> Texture2D {
        let step = CHUNK / BAKE_PX as f32;
        let mut px = vec![0u8; BAKE_PX * BAKE_PX * 4];
        let layer_info: HashMap<u16, (String, f32)> = map
            .texture_layers
            .iter()
            .map(|l| (l.texture_id, (l.texture.clone(), l.detail.unwrap_or(4.0).max(0.5) * map.tile_length)))
            .collect();
        let mut images: HashMap<u16, Option<(Arc<Rgba>, f32)>> = HashMap::new();
        for py in 0..BAKE_PX {
            for pxx in 0..BAKE_PX {
                let (lx, ly) = (ox + (pxx as f32 + 0.5) * step, oy + (py as f32 + 0.5) * step);
                let ids = map.textures_at(lx, ly);
                let weights = map.layer_weights_at(lx, ly);
                let mut c = [90.0f32, 110.0, 80.0];
                for (layer, (&id, &w)) in ids.iter().zip(weights.iter()).enumerate() {
                    if w == 0 {
                        continue;
                    }
                    let entry = images.entry(id).or_insert_with(|| {
                        let (name, repeat) = layer_info.get(&id)?.clone();
                        Some((self.layer_image(assets, &name)?, repeat))
                    });
                    let Some((img, repeat)) = entry else { continue };
                    let u = (lx / *repeat).rem_euclid(1.0);
                    let v = (ly / *repeat).rem_euclid(1.0);
                    let (ix, iy) = ((u * img.width as f32) as usize, (v * img.height as f32) as usize);
                    let o = (iy.min(img.height as usize - 1) * img.width as usize + ix.min(img.width as usize - 1)) * 4;
                    let a = if layer == 0 { 1.0 } else { w as f32 / 255.0 };
                    for k in 0..3 {
                        c[k] = c[k] * (1.0 - a) + img.pixels[o + k] as f32 * a;
                    }
                }
                let o = (py * BAKE_PX + pxx) * 4;
                px[o..o + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            }
        }
        let tex = Texture2D::from_rgba8(BAKE_PX as u16, BAKE_PX as u16, &px);
        tex.set_filter(FilterMode::Linear);
        tex
    }

    /// Couleur éclairée d'un sommet (formule du client, FUN_00af9960).
    fn lit(map: &NfmMap, i: i64, j: i64) -> [u8; 4] {
        let tl = map.tile_length;
        let h = |a: i64, b: i64| {
            let n = map.size as i64 - 1;
            map.vertex_height(a.clamp(0, n), b.clamp(0, n))
        };
        let nx = h(i - 1, j) - h(i + 1, j);
        let ny = h(i, j - 1) - h(i, j + 1);
        let nz = 2.0 * tl;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let n = [nx / len, ny / len, nz / len];
        let light = &map.properties.primary_light;
        let d = light.direction;
        let dot = (-(n[0] * d[0] + n[1] * d[1] + n[2] * d[2])).max(0.0);
        let c = map.vertex_color(i.clamp(0, map.size as i64 - 1), j.clamp(0, map.size as i64 - 1));
        let f = |base: u8, diff: u8, amb: u8| {
            let k = (diff as f32 / 255.0 * dot + amb as f32 / 255.0).min(1.0);
            (base as f32 * k).clamp(0.0, 255.0) as u8
        };
        [
            f(c.r, light.diffuse.r, light.ambient.r),
            f(c.g, light.diffuse.g, light.ambient.g),
            f(c.b, light.diffuse.b, light.ambient.b),
            255,
        ]
    }

    fn build_chunk(&mut self, assets: &mut Assets, gx: i32, gy: i32) -> Option<Chunk> {
        let per_map = (MAP_SIZE / CHUNK) as i32;
        let (mx, my) = (gx.div_euclid(per_map), gy.div_euclid(per_map));
        let map = self.map(assets, mx, my)?;
        let (cx, cy) = (gx.rem_euclid(per_map), gy.rem_euclid(per_map));
        let (ox, oy) = (cx as f32 * CHUNK, cy as f32 * CHUNK);
        let texture = self.bake(assets, &map, ox, oy);
        let n = CHUNK_CELLS + 1;
        let (wx0, wy0) = (mx as f32 * MAP_SIZE + ox, my as f32 * MAP_SIZE + oy);
        let (i0, j0) = ((cx as usize * CHUNK_CELLS) as i64, (cy as usize * CHUNK_CELLS) as i64);
        let last = map.size as i64 - 1;
        let mut vertices = Vec::with_capacity(n * n);
        for j in 0..n {
            for i in 0..n {
                let (vi, vj) = ((i0 + i as i64).min(last), (j0 + j as i64).min(last));
                let h = map.vertex_height(vi, vj);
                vertices.push(Vertex {
                    position: vec3(wx0 + i as f32 * map.tile_length, h, wy0 + j as f32 * map.tile_length),
                    uv: vec2(i as f32 / CHUNK_CELLS as f32, j as f32 / CHUNK_CELLS as f32),
                    color: Self::lit(&map, vi, vj),
                    normal: Vec4::ZERO,
                });
            }
        }
        let mut indices = Vec::with_capacity(CHUNK_CELLS * CHUNK_CELLS * 6);
        for j in 0..CHUNK_CELLS {
            for i in 0..CHUNK_CELLS {
                let a = (j * n + i) as u16;
                let (b, c, d) = (a + 1, a + n as u16, a + n as u16 + 1);
                indices.extend([a, b, d, a, d, c]);
            }
        }
        Some(Chunk { mesh: QMesh { vertices, indices, texture: Some(texture) } })
    }

    /// Construit les morceaux manquants autour de (x, y) et dessine le sol.
    pub fn draw(&mut self, assets: &mut Assets, x: f32, y: f32) {
        let (gx, gy) = ((x / CHUNK).floor() as i32, (y / CHUNK).floor() as i32);
        let r = self.view_chunks;
        let mut wanted: Vec<(i32, i32)> = (gy - r..=gy + r)
            .flat_map(|cy| (gx - r..=gx + r).map(move |cx| (cx, cy)))
            .filter(|&(cx, cy)| cx >= 0 && cy >= 0)
            .collect();
        wanted.sort_by_key(|&(cx, cy)| (cx - gx).abs() + (cy - gy).abs());
        let mut built = 0;
        for &key in &wanted {
            if built >= BUILDS_PER_FRAME {
                break;
            }
            if !self.chunks.contains_key(&key) {
                if let Some(c) = self.build_chunk(assets, key.0, key.1) {
                    self.chunks.insert(key, c);
                }
                built += 1;
            }
        }
        // Libère les morceaux éloignés.
        let keep: HashSet<(i32, i32)> = wanted.iter().copied().collect();
        if self.chunks.len() > keep.len() * 2 {
            self.chunks.retain(|k, _| keep.contains(k));
        }
        for key in &wanted {
            if let Some(c) = self.chunks.get(key) {
                draw_mesh(&c.mesh);
            }
        }
    }

    /// Dessine les décors (arbres, bâtiments…) des cartes proches de (x, y).
    pub fn draw_props(&mut self, assets: &mut Assets, cache: &mut ModelCache, x: f32, y: f32, budget: &mut usize) {
        let (mx, my) = ((x / MAP_SIZE).floor() as i32, (y / MAP_SIZE).floor() as i32);
        let d2 = self.prop_distance * self.prop_distance;
        for (tx, ty) in [(mx, my), (mx - 1, my), (mx + 1, my), (mx, my - 1), (mx, my + 1)] {
            let Some(map) = self.map(assets, tx, ty) else { continue };
            let (ox, oy) = (tx as f32 * MAP_SIZE, ty as f32 * MAP_SIZE);
            for p in &map.props {
                let (wx, wy) = (ox + p.position[0], oy + p.position[1]);
                if (wx - x).powi(2) + (wy - y).powi(2) > d2 || p.model.is_empty() {
                    continue;
                }
                // Le chargement d'un nouveau modèle est limité par image.
                let model = if *budget > 0 {
                    let before = cache_len_hint(cache);
                    let m = cache.static_model(assets, &p.model);
                    if cache_len_hint(cache) != before {
                        *budget -= 1;
                    }
                    m
                } else {
                    cache.static_model_cached(&p.model)
                };
                let Some(model) = model else { continue };
                let z = if p.height_locked { p.locked_height } else { map.height_at(p.position[0], p.position[1]) + p.z_offset };
                let m = models::placement([wx, wy, z], p.rotation, p.scale);
                models::draw_static(&model, m);
            }
        }
    }
}

fn cache_len_hint(cache: &ModelCache) -> usize {
    cache.static_count()
}
