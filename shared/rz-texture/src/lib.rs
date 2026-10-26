//! Textures du client Rappelz décodées en RGBA8 : DDS (DXT1/DXT3/DXT5 et formats non
//! compressés 32/24/16 bits, luminance), et TGA/JPEG/PNG/BMP via la crate `image`.
//! Seul le premier niveau de mipmap est décodé.

#[derive(Debug, thiserror::Error)]
pub enum TextureError {
    #[error("DDS : {0}")]
    Dds(String),
    #[error("image : {0}")]
    Image(#[from] image::ImageError),
}

pub type Result<T> = std::result::Result<T, TextureError>;

#[derive(Clone, Debug)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Décode d'après la signature (DDS) ou l'extension du nom (TGA n'a pas de signature).
pub fn decode(name: &str, bytes: &[u8]) -> Result<Rgba> {
    if bytes.starts_with(b"DDS ") {
        return decode_dds(bytes);
    }
    let lower = name.to_ascii_lowercase();
    let img = if lower.ends_with(".tga") {
        image::load_from_memory_with_format(bytes, image::ImageFormat::Tga)?
    } else {
        image::load_from_memory(bytes)?
    };
    let rgba = img.to_rgba8();
    Ok(Rgba { width: rgba.width(), height: rgba.height(), pixels: rgba.into_raw() })
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

const DDPF_ALPHAPIXELS: u32 = 0x1;
const DDPF_FOURCC: u32 = 0x4;
const DDPF_RGB: u32 = 0x40;
const DDPF_LUMINANCE: u32 = 0x20000;

pub fn decode_dds(b: &[u8]) -> Result<Rgba> {
    if b.len() < 128 || &b[..4] != b"DDS " {
        return Err(TextureError::Dds("en-tête invalide".into()));
    }
    let height = u32_at(b, 12);
    let width = u32_at(b, 16);
    let pf_flags = u32_at(b, 80);
    let fourcc = &b[84..88];
    let bits = u32_at(b, 88);
    let masks = [u32_at(b, 92), u32_at(b, 96), u32_at(b, 100), u32_at(b, 104)];
    let data = &b[128..];
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err(TextureError::Dds(format!("dimensions {width}×{height}")));
    }
    let pixels = if pf_flags & DDPF_FOURCC != 0 {
        match fourcc {
            b"DXT1" => decode_bc(data, width, height, Bc::Dxt1)?,
            b"DXT2" | b"DXT3" => decode_bc(data, width, height, Bc::Dxt3)?,
            b"DXT4" | b"DXT5" => decode_bc(data, width, height, Bc::Dxt5)?,
            other => return Err(TextureError::Dds(format!("FourCC {} non géré", String::from_utf8_lossy(other)))),
        }
    } else if pf_flags & (DDPF_RGB | DDPF_LUMINANCE) != 0 {
        let has_alpha = pf_flags & DDPF_ALPHAPIXELS != 0;
        decode_masked(data, width, height, bits, masks, has_alpha, pf_flags & DDPF_LUMINANCE != 0)?
    } else {
        return Err(TextureError::Dds(format!("format de pixel non géré (flags {pf_flags:#x})")));
    };
    Ok(Rgba { width, height, pixels })
}

#[derive(Clone, Copy, PartialEq)]
enum Bc {
    Dxt1,
    Dxt3,
    Dxt5,
}

fn rgb565(c: u16) -> [u8; 3] {
    let r = ((c >> 11) & 0x1F) as u32;
    let g = ((c >> 5) & 0x3F) as u32;
    let b = (c & 0x1F) as u32;
    [((r * 527 + 23) >> 6) as u8, ((g * 259 + 33) >> 6) as u8, ((b * 527 + 23) >> 6) as u8]
}

fn color_block(block: &[u8], out: &mut [[u8; 4]; 16], dxt1: bool) {
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);
    let (a, b) = (rgb565(c0), rgb565(c1));
    let mut pal = [[a[0], a[1], a[2], 255], [b[0], b[1], b[2], 255], [0; 4], [0; 4]];
    let mix = |x: u8, y: u8, wx: u32, wy: u32, d: u32| ((x as u32 * wx + y as u32 * wy) / d) as u8;
    if c0 > c1 || !dxt1 {
        pal[2] = [mix(a[0], b[0], 2, 1, 3), mix(a[1], b[1], 2, 1, 3), mix(a[2], b[2], 2, 1, 3), 255];
        pal[3] = [mix(a[0], b[0], 1, 2, 3), mix(a[1], b[1], 1, 2, 3), mix(a[2], b[2], 1, 2, 3), 255];
    } else {
        pal[2] = [mix(a[0], b[0], 1, 1, 2), mix(a[1], b[1], 1, 1, 2), mix(a[2], b[2], 1, 1, 2), 255];
        pal[3] = [0, 0, 0, 0];
    }
    let idx = u32_at(block, 4);
    for (i, px) in out.iter_mut().enumerate() {
        *px = pal[((idx >> (2 * i)) & 3) as usize];
    }
}

fn decode_bc(data: &[u8], w: u32, h: u32, kind: Bc) -> Result<Vec<u8>> {
    let block_size = if kind == Bc::Dxt1 { 8 } else { 16 };
    let (bw, bh) = (w.div_ceil(4) as usize, h.div_ceil(4) as usize);
    if data.len() < bw * bh * block_size {
        return Err(TextureError::Dds("données compressées tronquées".into()));
    }
    let mut out = vec![0u8; (w * h * 4) as usize];
    let mut px = [[0u8; 4]; 16];
    for by in 0..bh {
        for bx in 0..bw {
            let blk = &data[(by * bw + bx) * block_size..][..block_size];
            match kind {
                Bc::Dxt1 => color_block(blk, &mut px, true),
                Bc::Dxt3 => {
                    color_block(&blk[8..], &mut px, false);
                    for (i, p) in px.iter_mut().enumerate() {
                        let nib = (blk[i / 2] >> ((i % 2) * 4)) & 0xF;
                        p[3] = nib * 17;
                    }
                }
                Bc::Dxt5 => {
                    color_block(&blk[8..], &mut px, false);
                    let (a0, a1) = (blk[0] as u32, blk[1] as u32);
                    let mut alphas = [a0, a1, 0, 0, 0, 0, 0, 0];
                    if a0 > a1 {
                        for i in 1..7 {
                            alphas[i + 1] = ((7 - i as u32) * a0 + i as u32 * a1) / 7;
                        }
                    } else {
                        for i in 1..5 {
                            alphas[i + 1] = ((5 - i as u32) * a0 + i as u32 * a1) / 5;
                        }
                        alphas[6] = 0;
                        alphas[7] = 255;
                    }
                    let bits = blk[2..8].iter().rev().fold(0u64, |acc, &b| acc << 8 | b as u64);
                    for (i, p) in px.iter_mut().enumerate() {
                        p[3] = alphas[((bits >> (3 * i)) & 7) as usize] as u8;
                    }
                }
            }
            for (i, p) in px.iter().enumerate() {
                let (x, y) = (bx * 4 + i % 4, by * 4 + i / 4);
                if x < w as usize && y < h as usize {
                    let o = (y * w as usize + x) * 4;
                    out[o..o + 4].copy_from_slice(p);
                }
            }
        }
    }
    Ok(out)
}

fn channel(v: u32, mask: u32) -> u8 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let max = mask >> shift;
    (((v & mask) >> shift) * 255 / max) as u8
}

fn decode_masked(data: &[u8], w: u32, h: u32, bits: u32, m: [u32; 4], alpha: bool, luminance: bool) -> Result<Vec<u8>> {
    let bpp = (bits / 8) as usize;
    if !(1..=4).contains(&bpp) || data.len() < (w * h) as usize * bpp {
        return Err(TextureError::Dds(format!("format {bits} bits non géré ou tronqué")));
    }
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for px in data[..(w * h) as usize * bpp].chunks_exact(bpp) {
        let v = px.iter().rev().fold(0u32, |acc, &b| acc << 8 | b as u32);
        if luminance {
            let l = channel(v, m[0]);
            let a = if alpha { channel(v, m[3]) } else { 255 };
            out.extend([l, l, l, a]);
        } else {
            let a = if alpha { channel(v, m[3]) } else { 255 };
            out.extend([channel(v, m[0]), channel(v, m[1]), channel(v, m[2]), a]);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;

    #[test]
    fn real_client_textures_if_present() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Rappelz NoLimits");
        if !dir.join("data.000").is_file() {
            return;
        }
        let mut a = rz_rappelz::DataArchive::open(&dir).unwrap();
        let entries: Vec<_> = a
            .entries()
            .iter()
            .filter(|e| e.name.ends_with(".dds") || e.name.ends_with(".tga"))
            .step_by(25)
            .cloned()
            .collect();
        let mut errors: BTreeMap<String, usize> = BTreeMap::new();
        let mut ok = 0;
        for e in &entries {
            let bytes = a.read(e).unwrap();
            match decode(&e.name, &bytes) {
                Ok(img) => {
                    assert_eq!(img.pixels.len(), (img.width * img.height * 4) as usize);
                    ok += 1;
                }
                Err(err) => *errors.entry(err.to_string()).or_default() += 1,
            }
        }
        let failed: usize = errors.values().sum();
        eprintln!("{ok} textures décodées, {failed} échecs : {errors:?}");
        assert!(failed * 100 <= entries.len(), "trop d'échecs : {errors:?}");
    }
}
