//! Pure CPU blending. Inputs are immutable snapshots; all asset/GPU access stays on the caller.
use super::{BAKE_PX, CHUNK};
use rz_terrain::NfmMap;
use rz_texture::Rgba;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

pub(super) struct Input {
    pub map: Arc<NfmMap>,
    pub progress: Arc<AtomicUsize>,
    pub ox: f32,
    pub oy: f32,
    pub images: HashMap<u16, Option<(Arc<Rgba>, f32)>>,
}

pub(super) fn run(input: Input, cancel: &AtomicBool) -> Option<Vec<u8>> {
    let map = &input.map;
    let (ox, oy) = (input.ox, input.oy);
    let step = CHUNK / BAKE_PX as f32;
    let mut px = vec![0u8; BAKE_PX * BAKE_PX * 4];
    for py in 0..BAKE_PX {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        for pxx in 0..BAKE_PX {
            let (lx, ly) = (
                ox + (pxx as f32 + 0.5) * step,
                oy + (py as f32 + 0.5) * step,
            );
            let ids = map.textures_at(lx, ly);
            let weights = map.layer_weights_at(lx, ly);
            let mut c = [90.0f32, 110.0, 80.0];
            for (layer, (&id, &w)) in ids.iter().zip(weights.iter()).enumerate() {
                if w == 0 {
                    continue;
                }
                let Some(Some((img, repeat))) = input.images.get(&id) else {
                    continue;
                };
                let u = (lx / *repeat).rem_euclid(1.0);
                let v = (ly / *repeat).rem_euclid(1.0);
                let (sx, sy) = (u * img.width as f32 - 0.5, v * img.height as f32 - 0.5);
                let (x0, y0) = (sx.floor() as i32, sy.floor() as i32);
                let (fx, fy) = (sx - sx.floor(), sy - sy.floor());
                let sample = |dx: i32, dy: i32, k: usize| {
                    let x = (x0 + dx).rem_euclid(img.width as i32) as usize;
                    let y = (y0 + dy).rem_euclid(img.height as i32) as usize;
                    img.pixels[(y * img.width as usize + x) * 4 + k] as f32
                };
                let a = if layer == 0 { 1.0 } else { w as f32 / 255.0 };
                for k in 0..3 {
                    let top = sample(0, 0, k) * (1. - fx) + sample(1, 0, k) * fx;
                    let bottom = sample(0, 1, k) * (1. - fx) + sample(1, 1, k) * fx;
                    c[k] = c[k] * (1.0 - a) + (top * (1. - fy) + bottom * fy) * a;
                }
            }
            let o = (py * BAKE_PX + pxx) * 4;
            px[o..o + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
        }
        input.progress.store(py + 1, Ordering::Relaxed);
    }
    Some(px)
}
