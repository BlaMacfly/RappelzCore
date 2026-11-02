//! Offline packaging of an edited diffuse tile; original game files are never opened for writing.
use std::f32::consts::PI;
use std::path::Path;

use anyhow::{ensure, Context, Result};
use image::RgbaImage;
use rz_mpq::MpqWriter;
use serde_json::json;

#[derive(Clone)]
struct LinearTile {
    side: usize,
    rgb: Vec<[f32; 3]>,
}

fn linear(v: u8) -> f32 {
    let c = v as f32 / 255.;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(v: f32) -> u8 {
    let c = v.clamp(0., 1.);
    let c = if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1. / 2.4) - 0.055
    };
    (c * 255.).round().clamp(0., 255.) as u8
}
fn lanczos(x: f32) -> f32 {
    let x = x.abs();
    if x < 1e-6 {
        1.
    } else if x >= 3. {
        0.
    } else {
        (PI * x).sin() / (PI * x) * (PI * x / 3.).sin() / (PI * x / 3.)
    }
}
fn weights(from: usize, to: usize) -> Vec<Vec<(usize, f32)>> {
    let ratio = from as f32 / to as f32;
    let filter = ratio.max(1.);
    (0..to)
        .map(|p| {
            let center = (p as f32 + 0.5) * ratio - 0.5;
            let lo = (center - 3. * filter).ceil() as isize;
            let hi = (center + 3. * filter).floor() as isize;
            let mut w: Vec<_> = (lo..=hi)
                .map(|q| {
                    (
                        q.rem_euclid(from as isize) as usize,
                        lanczos((q as f32 - center) / filter),
                    )
                })
                .collect();
            let sum: f32 = w.iter().map(|(_, v)| *v).sum();
            for (_, v) in &mut w {
                *v /= sum;
            }
            w
        })
        .collect()
}
impl LinearTile {
    fn from_image(im: &RgbaImage) -> Self {
        Self {
            side: im.width() as usize,
            rgb: im
                .pixels()
                .map(|p| [linear(p[0]), linear(p[1]), linear(p[2])])
                .collect(),
        }
    }
    fn resize(&self, side: usize) -> Self {
        if side == self.side {
            return self.clone();
        }
        let w = weights(self.side, side);
        let mut horizontal = vec![[0.; 3]; self.side * side];
        for y in 0..self.side {
            for x in 0..side {
                for &(sx, weight) in &w[x] {
                    for k in 0..3 {
                        horizontal[y * side + x][k] += self.rgb[y * self.side + sx][k] * weight;
                    }
                }
            }
        }
        let mut rgb = vec![[0.; 3]; side * side];
        for y in 0..side {
            for x in 0..side {
                for &(sy, weight) in &w[y] {
                    for k in 0..3 {
                        rgb[y * side + x][k] += horizontal[sy * side + x][k] * weight;
                    }
                }
            }
        }
        Self { side, rgb }
    }
    fn rgba(&self) -> Vec<u8> {
        self.rgb
            .iter()
            .flat_map(|p| [srgb(p[0]), srgb(p[1]), srgb(p[2]), 255])
            .collect()
    }
}

fn dds(tile: LinearTile) -> (Vec<u8>, usize) {
    let side = tile.side as u32;
    let mut levels = vec![tile];
    while levels.last().unwrap().side > 1 {
        let old = levels.last().unwrap();
        levels.push(old.resize(old.side / 2));
    }
    let mut words = [0u32; 31];
    words[0] = 124;
    words[1] = 0x2100f; // caps, width/height, pitch, pixel format, mip count
    words[2] = side;
    words[3] = side;
    words[4] = side * 4;
    words[6] = levels.len() as u32;
    words[18] = 32;
    words[19] = 0x41;
    words[21] = 32; // RGB + alpha, uncompressed RGBA8
    words[22] = 0xff;
    words[23] = 0xff00;
    words[24] = 0xff0000;
    words[25] = 0xff000000;
    words[26] = 0x401008; // TEXTURE | COMPLEX | MIPMAP
    let mut out = b"DDS ".to_vec();
    for v in words {
        out.extend(v.to_le_bytes());
    }
    for level in &levels {
        out.extend(level.rgba());
    }
    (out, levels.len())
}

fn entry_name(name: &str) -> Result<String> {
    let name = name.replace('/', "\\");
    let parts: Vec<_> = name.split('\\').collect();
    ensure!(
        parts.len() == 2 && parts[0].eq_ignore_ascii_case("Textures"),
        "terrain entry must be Textures\\<name>.dds"
    );
    let base = parts[1];
    ensure!(
        !base.is_empty()
            && base.to_ascii_lowercase().ends_with(".dds")
            && !base.contains([':', '\0'])
            && base != ".dds",
        "invalid DDS entry name"
    );
    Ok(format!("Textures\\{base}"))
}

pub fn run(input: &Path, texture: &str, out: &Path, size: u32) -> Result<()> {
    ensure!(
        size.is_power_of_two() && (128..=2048).contains(&size),
        "size must be a power of two from 128 to 2048"
    );
    let entry = entry_name(texture)?;
    let reader = image::ImageReader::open(input)?.with_guessed_format()?;
    let (w, h) = reader.into_dimensions()?;
    ensure!(
        w == h && (1..=4096).contains(&w),
        "source must be a square tile no larger than 4096 pixels"
    );
    let im = image::open(input).context("decode edited PNG")?.to_rgba8();
    ensure!(
        im.pixels().all(|p| p[3] == 255),
        "terrain diffuse tiles must be opaque"
    );
    let tile = LinearTile::from_image(&im).resize(size as usize);
    let (bytes, mips) = dds(tile);
    // This atomic directory creation rejects existing directories and symlinks too.
    // All subsequent files belong to that new directory; no original pack is edited.
    std::fs::create_dir(out)
        .with_context(|| format!("output must be a new directory: {}", out.display()))?;
    let name = entry.rsplit('\\').next().unwrap();
    std::fs::write(out.join(name), &bytes)?;
    let mut mpq = MpqWriter::create(out.join("patch.MPQ"), 3)?;
    mpq.add_file(&entry, &bytes, true)?;
    mpq.finish()?;
    let manifest = json!({"format":"terrain-texture-pack-v1","texture":entry,"input":input.file_name().map(|n|n.to_string_lossy()),
        "source_size":[w,h],"size":size,"mip_levels":mips,"dds_bytes":bytes.len(),"decoded_top_rgba_bytes":size as u64*size as u64*4,
        "pixel_encoding":"sRGB RGB, opaque alpha in legacy RGBA8 DDS",
        "resampling":"periodic Lanczos3 in linear RGB","mpq":"patch.MPQ","original_files_modified":false,
        "runtime_note":"The current CPU terrain decoder reads the top mip; lower mip levels are prepared for future consumers."});
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "{}: {}x{}, {} mip levels, separate patch.MPQ",
        entry, size, size, mips
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rz_mpq::{MpqArchive, MpqChain};
    #[test]
    fn mip_filter_preserves_linear_light_and_constant_fields() {
        let checker = RgbaImage::from_fn(2, 2, |x, y| {
            image::Rgba(if (x + y) % 2 == 0 {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 255]
            })
        });
        let mip = LinearTile::from_image(&checker).resize(1).rgba();
        assert!((mip[0] as i32 - 188).abs() <= 1);
        assert_eq!(mip[3], 255);
        let solid = RgbaImage::from_pixel(7, 7, image::Rgba([45, 88, 132, 255]));
        for n in [1, 2, 4, 16] {
            assert!(LinearTile::from_image(&solid)
                .resize(n)
                .rgba()
                .chunks_exact(4)
                .all(|p| p == [45, 88, 132, 255]));
        }
    }
    #[test]
    fn periodic_filter_commutes_with_integer_tile_shifts() {
        let im = RgbaImage::from_fn(8, 8, |x, y| {
            image::Rgba([(x * 29) as u8, (y * 31) as u8, ((x + y) * 15) as u8, 255])
        });
        let shifted = RgbaImage::from_fn(8, 8, |x, y| *im.get_pixel((x + 2) % 8, (y + 2) % 8));
        let a = LinearTile::from_image(&im).resize(4);
        let b = LinearTile::from_image(&shifted).resize(4);
        for y in 0..4 {
            for x in 0..4 {
                for k in 0..3 {
                    assert!(
                        (b.rgb[y * 4 + x][k] - a.rgb[((y + 1) % 4) * 4 + (x + 1) % 4][k]).abs()
                            < 1e-5
                    );
                }
            }
        }
    }
    #[test]
    fn legacy_dds_has_full_chain_and_expected_channel_masks() {
        let t =
            LinearTile::from_image(&RgbaImage::from_pixel(2, 2, image::Rgba([9, 50, 201, 255])));
        let (b, n) = dds(t);
        assert_eq!(n, 2);
        assert_eq!(&b[..4], b"DDS ");
        assert_eq!(b.len(), 128 + 16 + 4);
        let word = |o| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        assert_eq!(word(28), 2);
        assert_eq!(word(92), 0xff);
        assert_eq!(word(96), 0xff00);
        assert_eq!(word(100), 0xff0000);
        assert_eq!(word(104), 0xff000000);
        assert!(b[128..].chunks_exact(4).all(|p| p == [9, 50, 201, 255]));
    }
    #[test]
    fn patch_replaces_only_selected_texture_and_keeps_source() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.png");
        RgbaImage::from_pixel(4, 4, image::Rgba([51, 83, 29, 255]))
            .save(&input)
            .unwrap();
        let original = std::fs::read(&input).unwrap();
        let base = dir.path().join("common.MPQ");
        let mut w = MpqWriter::create(&base, 3).unwrap();
        w.add_file("Textures\\grass.dds", b"old", true).unwrap();
        w.add_file("unrelated.txt", b"untouched", true).unwrap();
        w.finish().unwrap();
        let out = dir.path().join("pack");
        run(&input, "Textures/grass.dds", &out, 128).unwrap();
        let mut c = MpqChain::new();
        c.push(MpqArchive::open(base).unwrap());
        c.push(MpqArchive::open(out.join("patch.MPQ")).unwrap());
        assert_eq!(
            c.read_file("Textures\\grass.dds").unwrap(),
            std::fs::read(out.join("grass.dds")).unwrap()
        );
        assert_eq!(c.read_file("unrelated.txt").unwrap(), b"untouched");
        assert_eq!(std::fs::read(input).unwrap(), original);
        assert!(run(
            &dir.path().join("input.png"),
            "Textures/grass.dds",
            &out,
            128
        )
        .is_err());
        assert_eq!(c.read_file("unrelated.txt").unwrap(), b"untouched");
    }
    #[test]
    fn rejects_transparency_bad_paths_and_sizes_before_creating_output() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("alpha.png");
        RgbaImage::from_pixel(2, 2, image::Rgba([50, 60, 70, 0]))
            .save(&input)
            .unwrap();
        let out = dir.path().join("pack");
        assert!(run(&input, "Textures/grass.dds", &out, 128).is_err());
        assert!(!out.exists());
        assert!(run(&input, "Textures/grass.dds", &out, 1254).is_err());
        assert!(!out.exists());
        for name in [
            "../grass.dds",
            "Textures/../x.dds",
            "C:\\x.dds",
            "Textures/",
            "Textures/.dds",
        ] {
            assert!(entry_name(name).is_err());
        }
    }
}
