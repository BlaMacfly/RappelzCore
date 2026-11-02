//! Shared repeating textures for strictly uniform chunks. Mixed layers still use the baker.
use super::bake;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Source {
    pub name: String,
    pub repeat: f32,
    pub width: u32,
    pub height: u32,
}

pub(super) fn mip_bytes(mut width: u32, mut height: u32) -> usize {
    let mut bytes = 0;
    loop {
        bytes += width as usize * height as usize * 4;
        if width == 1 && height == 1 {
            return bytes;
        }
        width = (width / 2).max(1);
        height = (height / 2).max(1);
    }
}

impl Source {
    pub fn from_input(input: &bake::Input) -> Option<Self> {
        if input.map.tile_length != 42. {
            return None;
        }
        Self::from_layers(&input.images, |id| {
            Some(
                input
                    .map
                    .texture_layers
                    .iter()
                    .find(|l| l.texture_id == id)?
                    .texture
                    .clone(),
            )
        })
    }

    fn from_layers(
        images: &std::collections::HashMap<u16, Option<(std::sync::Arc<rz_texture::Rgba>, f32)>>,
        name: impl Fn(u16) -> Option<String>,
    ) -> Option<Self> {
        // Conservative: even different IDs resolving to the same filename stay baked.
        if images.len() != 1 {
            return None;
        }
        let (&id, value) = images.iter().next()?;
        let (image, repeat) = value.as_ref()?;
        if !repeat.is_finite()
            || *repeat <= 0.
            || !image.width.is_power_of_two()
            || !image.height.is_power_of_two()
            || image.width > 1024
            || image.height > 1024
            || image.pixels.len() != image.width as usize * image.height as usize * 4
            || image.pixels.chunks_exact(4).any(|p| p[3] != 255)
        {
            return None;
        }
        Some(Self {
            name: name(id)?,
            repeat: *repeat,
            width: image.width,
            height: image.height,
        })
    }

    pub fn fits(&self, chunks: usize) -> bool {
        mip_bytes(self.width, self.height) <= chunks * super::BAKE_PX * super::BAKE_PX * 4
    }

    pub fn uv(
        &self,
        ox: f32,
        oy: f32,
        i: usize,
        j: usize,
        tile_length: f32,
    ) -> macroquad::math::Vec2 {
        macroquad::math::vec2(
            (ox + i as f32 * tile_length) / self.repeat,
            (oy + j as f32 * tile_length) / self.repeat,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(size: u32) -> Source {
        Source {
            name: "grass".into(),
            repeat: 84.,
            width: size,
            height: size,
        }
    }
    #[test]
    fn full_mip_cost_must_fit_the_baked_chunks_it_replaces() {
        assert_eq!(mip_bytes(512, 512), 1_398_100);
        assert_eq!(mip_bytes(1024, 1024), 5_592_404);
        assert!(!source(1024).fits(1));
        assert!(source(1024).fits(2));
        assert!(source(512).fits(1));
    }
    #[test]
    fn repeat_phase_is_continuous_at_chunk_edges() {
        let source = source(1024);
        for origin in [0., 1008., 15120.] {
            assert_eq!(
                source.uv(origin, 0., 24, 3, 42.),
                source.uv(origin + 1008., 0., 0, 3, 42.)
            );
        }
        assert_eq!(source.uv(0., 0., 2, 0, 42.).x, 1.);
    }
    #[test]
    fn only_one_resolved_opaque_power_of_two_layer_can_bypass_blending() {
        use rz_texture::Rgba;
        use std::sync::Arc;
        let image = Arc::new(Rgba {
            width: 2,
            height: 2,
            pixels: vec![255; 16],
        });
        let mut layers = std::collections::HashMap::from([(0, Some((image.clone(), 84.)))]);
        let select = |layers: &_| Source::from_layers(layers, |_| Some("grass".into()));
        assert!(select(&layers).is_some());
        layers.insert(1, Some((image.clone(), 84.)));
        assert!(select(&layers).is_none()); // same pixels still represent two active IDs
        layers.remove(&1);
        for (width, height, pixels, repeat) in [
            (2, 2, vec![255; 15], 84.),
            (3, 2, vec![255; 24], 84.),
            (2, 2, vec![0; 16], 84.),
            (2, 2, vec![255; 16], f32::NAN),
            (2, 2, vec![255; 16], 0.),
        ] {
            layers.insert(
                0,
                Some((
                    Arc::new(Rgba {
                        width,
                        height,
                        pixels,
                    }),
                    repeat,
                )),
            );
            assert!(select(&layers).is_none());
        }
        layers.insert(0, None);
        assert!(select(&layers).is_none());
        layers.clear();
        assert!(select(&layers).is_none());
    }
}
