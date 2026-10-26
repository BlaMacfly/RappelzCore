//! Ensemble de cartes en coordonnées monde, avec raccord du relief entre cartes voisines.

use std::collections::HashMap;

use crate::nfm::{interpolate, NfmMap};

/// Cartes indexées par (x, y) de `mXXX_YYY`.
#[derive(Debug, Default)]
pub struct TerrainWorld {
    pub maps: HashMap<(u32, u32), NfmMap>,
}

impl TerrainWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, map_x: u32, map_y: u32, map: NfmMap) {
        self.maps.insert((map_x, map_y), map);
    }

    /// Carte contenant le point monde (x, y) et coordonnées locales.
    pub fn locate(&self, world_x: f32, world_y: f32) -> Option<((u32, u32), f32, f32)> {
        let any = self.maps.values().next()?;
        let ext = any.extent();
        if world_x < 0.0 || world_y < 0.0 {
            return None;
        }
        let (mx, my) = ((world_x / ext) as u32, (world_y / ext) as u32);
        self.maps.contains_key(&(mx, my)).then_some(((mx, my), world_x - mx as f32 * ext, world_y - my as f32 * ext))
    }

    /// Hauteur du sol au point monde (x, y). Les sommets au-delà du bord sont pris dans la carte
    /// voisine si elle est chargée, sinon la dernière rangée de la carte est prolongée.
    pub fn height_at(&self, world_x: f32, world_y: f32) -> Option<f32> {
        let ((mx, my), _, _) = self.locate(world_x, world_y)?;
        let map = &self.maps[&(mx, my)];
        let n = map.size as i64;
        let tl = map.tile_length;
        Some(interpolate(world_x / tl, world_y / tl, |gi, gj| {
            let (nx, ny) = (gi.div_euclid(n), gj.div_euclid(n));
            match self.maps.get(&(nx as u32, ny as u32)).filter(|_| nx >= 0 && ny >= 0) {
                Some(m) => m.vertex_height(gi.rem_euclid(n), gj.rem_euclid(n)),
                None => map.vertex_height(gi - mx as i64 * n, gj - my as i64 * n),
            }
        }))
    }
}
