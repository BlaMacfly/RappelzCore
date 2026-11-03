//! Prepare one optional atlas replacement without changing original files or its alpha mask.
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use anyhow::{ensure, Context, Result};
use image::{ImageBuffer, ImageFormat, ImageReader, Rgb, RgbaImage};
use rz_mpq::MpqWriter;
use serde_json::json;

const MAX_SIDE: u32 = 4096;
const MAX_PNG_BYTES: u64 = 128 * 1024 * 1024;
const MAX_DECODE_BYTES: u64 = 128 * 1024 * 1024;

fn read_png(path: &Path) -> Result<RgbaImage> {
    let file = File::open(path).with_context(|| format!("open PNG {}", path.display()))?;
    let metadata = file.metadata()?;
    ensure!(metadata.is_file(), "PNG input must be a regular file");
    ensure!(metadata.len() <= MAX_PNG_BYTES, "PNG file exceeds 128 MiB");
    // Parse precisely these bounded bytes, rather than reopening the input after preflight.
    let mut bytes = Vec::new();
    file.take(MAX_PNG_BYTES + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 == metadata.len() && bytes.len() as u64 <= MAX_PNG_BYTES,
        "PNG input changed size or exceeded the file limit"
    );
    ensure!(
        bytes.len() >= 33
            && &bytes[..8] == b"\x89PNG\r\n\x1a\n"
            && &bytes[12..16] == b"IHDR"
            && bytes[24] == 8,
        "input must be an 8-bit PNG"
    );
    let (width, height) = ImageReader::with_format(Cursor::new(&bytes), ImageFormat::Png)
        .into_dimensions()
        .context("read PNG dimensions")?;
    ensure!(
        (1..=MAX_SIDE).contains(&width) && (1..=MAX_SIDE).contains(&height),
        "PNG dimensions must be from 1 to 4096 per side"
    );
    let mut reader = ImageReader::with_format(Cursor::new(&bytes), ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    reader.limits(limits);
    Ok(reader.decode().context("decode PNG")?.to_rgba8())
}

fn entry_name(name: &str) -> Result<String> {
    let name = name.replace('/', "\\");
    let parts: Vec<_> = name.split('\\').collect();
    ensure!(
        parts.len() == 2,
        "entry requires one allowed folder and one DDS basename"
    );
    let folder = [
        "Character",
        "Creature",
        "Textures",
        "Spells",
        "Interface",
        "World",
    ]
    .into_iter()
    .find(|folder| folder.eq_ignore_ascii_case(parts[0]))
    .context("unsupported MPQ folder")?;
    let base = parts[1];
    ensure!(
        base.is_ascii()
            && base.len() > 4
            && base.to_ascii_lowercase().ends_with(".dds")
            && !base.starts_with('.')
            && base.trim() == base
            && base
                .split('.')
                .all(|part| !part.is_empty() && part.trim() == part)
            && !base.bytes().any(|c| c < 32 || c == 127)
            && !base.contains(['<', '>', ':', '"', '|', '?', '*']),
        "invalid or aliased DDS basename"
    );
    let stem = base.split('.').next().unwrap().to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && (b'1'..=b'9').contains(&stem.as_bytes()[3]));
    ensure!(!reserved, "reserved Windows DDS basename");
    Ok(format!("{folder}\\{base}"))
}

fn ratio_matches(a: (u32, u32), b: (u32, u32)) -> bool {
    let cross_a = a.0 as u64 * b.1 as u64;
    let cross_b = b.0 as u64 * a.1 as u64;
    cross_a.abs_diff(cross_b) * 100 <= cross_a.min(cross_b)
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

struct LinearImage {
    width: u32,
    height: u32,
    pixels: Vec<[f32; 4]>,
}

impl LinearImage {
    fn prepare(edited: &RgbaImage, original: &RgbaImage, width: u32, height: u32) -> Self {
        let rgb: ImageBuffer<Rgb<f32>, Vec<f32>> =
            ImageBuffer::from_fn(edited.width(), edited.height(), |x, y| {
                let p = edited.get_pixel(x, y);
                Rgb([linear(p[0]), linear(p[1]), linear(p[2])])
            });
        // imageops clamps sampling to image boundaries; atlases must not wrap into other islands.
        let resized =
            image::imageops::resize(&rgb, width, height, image::imageops::FilterType::Lanczos3);
        let pixels = resized
            .enumerate_pixels()
            .map(|(x, y, p)| {
                let ox = x as u64 * original.width() as u64 / width as u64;
                let oy = y as u64 * original.height() as u64 / height as u64;
                let alpha = original.get_pixel(ox as u32, oy as u32)[3];
                [
                    p[0].clamp(0., 1.),
                    p[1].clamp(0., 1.),
                    p[2].clamp(0., 1.),
                    alpha as f32 / 255.,
                ]
            })
            .collect();
        Self {
            width,
            height,
            pixels,
        }
    }

    fn next_mip(&self) -> Self {
        let width = (self.width / 2).max(1);
        let height = (self.height / 2).max(1);
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let mut premultiplied = [0.; 3];
                let mut alpha = 0.;
                let mut count = 0.;
                for sy in y * 2..(y * 2 + 2).min(self.height) {
                    for sx in x * 2..(x * 2 + 2).min(self.width) {
                        let p = self.pixels[(sy * self.width + sx) as usize];
                        for c in 0..3 {
                            premultiplied[c] += p[c] * p[3];
                        }
                        alpha += p[3];
                        count += 1.;
                    }
                }
                let mut p = [0.; 4];
                if alpha > 0. {
                    for c in 0..3 {
                        p[c] = premultiplied[c] / alpha;
                    }
                }
                p[3] = alpha / count;
                pixels.push(p);
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    fn append_rgba(&self, bytes: &mut Vec<u8>) {
        for p in &self.pixels {
            bytes.extend_from_slice(&[
                srgb(p[0]),
                srgb(p[1]),
                srgb(p[2]),
                (p[3] * 255.).round().clamp(0., 255.) as u8,
            ]);
        }
    }
}

fn encode_dds(mut level: LinearImage) -> (Vec<u8>, usize) {
    let mut count = 1;
    let (mut w, mut h) = (level.width, level.height);
    while w > 1 || h > 1 {
        w = (w / 2).max(1);
        h = (h / 2).max(1);
        count += 1;
    }
    let mut words = [0u32; 31];
    words[0] = 124;
    words[1] = 0x2100f;
    words[2] = level.height;
    words[3] = level.width;
    words[4] = level.width * 4;
    words[6] = count as u32;
    words[18] = 32;
    words[19] = 0x41;
    words[21] = 32;
    words[22] = 0xff;
    words[23] = 0xff00;
    words[24] = 0xff0000;
    words[25] = 0xff000000;
    words[26] = 0x401008;
    let mut bytes = b"DDS ".to_vec();
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    loop {
        level.append_rgba(&mut bytes);
        if level.width == 1 && level.height == 1 {
            break;
        }
        level = level.next_mip();
    }
    (bytes, count)
}

pub fn run(
    image: &Path,
    original: &Path,
    entry: &str,
    out: &Path,
    width: u32,
    height: u32,
) -> Result<()> {
    ensure!(
        [width, height]
            .into_iter()
            .all(|n| n.is_power_of_two() && (32..=2048).contains(&n)),
        "output width and height must be powers of two from 32 to 2048"
    );
    let entry = entry_name(entry)?;
    let edited = read_png(image)?;
    let reference = read_png(original)?;
    ensure!(
        edited.pixels().all(|p| p[3] == 255),
        "edited PNG must be opaque; alpha comes only from the original reference"
    );
    ensure!(
        ratio_matches(edited.dimensions(), reference.dimensions())
            && ratio_matches(edited.dimensions(), (width, height))
            && ratio_matches(reference.dimensions(), (width, height)),
        "edited, original and output aspect ratios must agree within 1 percent"
    );
    let top = LinearImage::prepare(&edited, &reference, width, height);
    let (bytes, mip_levels) = encode_dds(top);
    // Atomic creation refuses pre-existing directories, files and symlinks, including dangling links.
    std::fs::create_dir(out)
        .with_context(|| format!("output must be a new directory: {}", out.display()))?;
    let basename = entry.rsplit('\\').next().unwrap();
    std::fs::write(out.join(basename), &bytes)?;
    let mut archive = MpqWriter::create(out.join("patch.MPQ"), 3)?;
    archive.add_file(&entry, &bytes, true)?;
    archive.finish()?;
    let manifest = json!({
        "format":"masked-atlas-pack-v1", "entry":entry,
        "edited_input":image.file_name().map(|n|n.to_string_lossy()),
        "original_reference":original.file_name().map(|n|n.to_string_lossy()),
        "edited_size":[edited.width(),edited.height()], "original_size":[reference.width(),reference.height()],
        "size":[width,height], "mip_levels":mip_levels, "dds_bytes":bytes.len(),
        "decoded_top_rgba_bytes":width as u64*height as u64*4,
        "pixel_encoding":"straight RGBA8 DDS; sRGB RGB, linear alpha",
        "resampling":"RGB Lanczos3 in linear light with clamped image edges",
        "top_alpha_policy":"exact reference u8 alpha at floor(x*original_width/output_width), floor(y*original_height/output_height)",
        "mip_policy":"linear-light premultiplied-alpha box filter, stored as straight RGBA8",
        "mpq":"patch.MPQ", "original_files_modified":false,
        "runtime_note":"File mip levels are prepared; this does not establish that a renderer consumes them or validates atlas layout on a model."
    });
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "{}: {}x{}, {} mip levels, original alpha preserved; separate patch.MPQ",
        entry, width, height, mip_levels
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use rz_mpq::{MpqArchive, MpqChain};

    fn inputs(dir: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
        let image = dir.join("edited.png");
        let original = dir.join("original.png");
        RgbaImage::from_fn(8, 4, |x, _| {
            Rgba([if x < 4 { 255 } else { 0 }, 80, 120, 255])
        })
        .save(&image)
        .unwrap();
        RgbaImage::from_fn(4, 2, |x, y| {
            Rgba([1, 2, 3, if (x + y) % 2 == 0 { 153 } else { 255 }])
        })
        .save(&original)
        .unwrap();
        (image, original)
    }

    #[test]
    fn exact_reference_mask_and_rectangular_mpq_preserve_sources_and_unrelated_entries() {
        let dir = tempfile::tempdir().unwrap();
        let (image, original) = inputs(dir.path());
        let before = (
            std::fs::read(&image).unwrap(),
            std::fs::read(&original).unwrap(),
        );
        let reference = image::open(&original).unwrap().to_rgba8();
        let out = dir.path().join("pack");
        run(&image, &original, "character/face.dds", &out, 64, 32).unwrap();
        let bytes = std::fs::read(out.join("face.dds")).unwrap();
        assert_eq!(&bytes[..4], b"DDS ");
        let word = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        assert_eq!((word(16), word(12), word(28)), (64, 32, 7));
        assert_eq!(
            bytes.len(),
            128 + 4 * (64 * 32 + 32 * 16 + 16 * 8 + 8 * 4 + 4 * 2 + 2 + 1)
        );
        assert_eq!(
            (word(92), word(96), word(100), word(104)),
            (0xff, 0xff00, 0xff0000, 0xff000000)
        );
        for y in 0..32 {
            for x in 0..64 {
                assert_eq!(
                    bytes[128 + ((y * 64 + x) * 4) as usize + 3],
                    reference.get_pixel(x * 4 / 64, y * 2 / 32)[3]
                );
            }
        }
        let base = dir.path().join("common.MPQ");
        let mut writer = MpqWriter::create(&base, 3).unwrap();
        writer
            .add_file("Character\\face.dds", b"old", true)
            .unwrap();
        writer
            .add_file("World\\untouched.nx3", b"unchanged", true)
            .unwrap();
        writer.finish().unwrap();
        let mut chain = MpqChain::new();
        chain.push(MpqArchive::open(base).unwrap());
        let mut patch = MpqArchive::open(out.join("patch.MPQ")).unwrap();
        let list = patch.list().unwrap();
        assert_eq!(
            list.iter()
                .filter(|n| n.as_str() != "(listfile)")
                .cloned()
                .collect::<Vec<_>>(),
            vec!["Character\\face.dds"]
        );
        chain.push(patch);
        assert_eq!(chain.read_file("Character\\face.dds").unwrap(), bytes);
        assert_eq!(
            chain.read_file("World\\untouched.nx3").unwrap(),
            b"unchanged"
        );
        assert_eq!(
            (
                std::fs::read(image).unwrap(),
                std::fs::read(original).unwrap()
            ),
            before
        );
    }

    #[test]
    fn mip_filter_uses_linear_light_and_excludes_invisible_color() {
        let checker = LinearImage {
            width: 2,
            height: 1,
            pixels: vec![[1., 1., 1., 1.], [0., 0., 0., 1.]],
        };
        let mut bytes = Vec::new();
        checker.next_mip().append_rgba(&mut bytes);
        assert_eq!(bytes, [188, 188, 188, 255]);
        let edge = LinearImage {
            width: 2,
            height: 1,
            pixels: vec![[1., 0., 0., 1.], [0., 0., 1., 0.]],
        };
        bytes.clear();
        edge.next_mip().append_rgba(&mut bytes);
        assert_eq!(bytes, [255, 0, 0, 128]);
        let transparent = LinearImage {
            width: 1,
            height: 2,
            pixels: vec![[1., 0., 0., 0.], [0., 1., 0., 0.]],
        };
        bytes.clear();
        transparent.next_mip().append_rgba(&mut bytes);
        assert_eq!(bytes, [0, 0, 0, 0]);
    }

    #[test]
    fn rgb_resize_uses_linear_light_and_reference_alpha_instead_of_generated_content() {
        let edited = RgbaImage::from_fn(2, 1, |x, _| {
            Rgba(if x == 0 {
                [0, 0, 0, 255]
            } else {
                [255, 255, 255, 255]
            })
        });
        let reference = RgbaImage::from_pixel(1, 1, Rgba([230, 50, 5, 153]));
        let top = LinearImage::prepare(&edited, &reference, 1, 1);
        let mut bytes = Vec::new();
        top.append_rgba(&mut bytes);
        assert!(bytes[..3].iter().all(|v| (187..=189).contains(v)));
        assert_eq!(bytes[3], 153);
    }

    #[test]
    fn rejects_path_aliases_traversal_and_reserved_names() {
        for name in [
            "../x.dds",
            "Textures/../x.dds",
            "Textures//x.dds",
            "C:/x.dds",
            "Fonts/x.dds",
            "Textures/.dds",
            "Textures/..dds",
            "Textures/x.dds ",
            "Textures/ x.dds",
            "Textures/CON .dds",
            "Textures/a..dds",
            "Textures/x:stream.dds",
            "Textures/CON.dds",
            "Textures/lpt1.dds",
            "Textures/a\0.dds",
            "Textures/a?.dds",
        ] {
            assert!(entry_name(name).is_err(), "{name:?}");
        }
        assert_eq!(
            entry_name("character/asm_face.dds").unwrap(),
            "Character\\asm_face.dds"
        );
        assert_eq!(
            entry_name("Textures/1handswd_master.dds").unwrap(),
            "Textures\\1handswd_master.dds"
        );
    }

    #[test]
    fn refuses_existing_output_without_overwriting_it_or_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let (image, original) = inputs(dir.path());
        let before = (
            std::fs::read(&image).unwrap(),
            std::fs::read(&original).unwrap(),
        );
        let out = dir.path().join("existing");
        std::fs::create_dir(&out).unwrap();
        std::fs::write(out.join("sentinel"), b"keep").unwrap();
        assert!(run(&image, &original, "Textures/a.dds", &out, 64, 32).is_err());
        assert_eq!(std::fs::read(out.join("sentinel")).unwrap(), b"keep");
        assert_eq!(std::fs::read_dir(&out).unwrap().count(), 1);
        let file = dir.path().join("existing-file");
        std::fs::write(&file, b"keep-file").unwrap();
        assert!(run(&image, &original, "Textures/a.dds", &file, 64, 32).is_err());
        assert_eq!(std::fs::read(file).unwrap(), b"keep-file");
        assert_eq!(
            (
                std::fs::read(image).unwrap(),
                std::fs::read(original).unwrap()
            ),
            before
        );
    }

    #[test]
    fn rejects_bad_dimensions_ratio_and_generated_alpha_before_creating_output() {
        let dir = tempfile::tempdir().unwrap();
        let (image, original) = inputs(dir.path());
        let out = dir.path().join("not-created");
        for (width, height) in [(31, 32), (4096, 2048), (64, 64), (1254, 627)] {
            assert!(run(&image, &original, "Textures/a.dds", &out, width, height).is_err());
            assert!(!out.exists());
        }
        assert!(ratio_matches((1774, 886), (2, 1)));
        assert!(!ratio_matches((16, 9), (2, 1)));
        RgbaImage::from_pixel(8, 4, Rgba([1, 2, 3, 254]))
            .save(&image)
            .unwrap();
        assert!(run(&image, &original, "Textures/a.dds", &out, 64, 32).is_err());
        assert!(!out.exists());
    }

    #[test]
    fn bounded_png_preflight_rejects_oversize_files_and_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let oversized = dir.path().join("too-large.png");
        File::create(&oversized)
            .unwrap()
            .set_len(MAX_PNG_BYTES + 1)
            .unwrap();
        assert!(read_png(&oversized)
            .unwrap_err()
            .to_string()
            .contains("128 MiB"));
        let wide = dir.path().join("too-wide.png");
        RgbaImage::from_pixel(4097, 1, Rgba([1, 2, 3, 255]))
            .save(&wide)
            .unwrap();
        assert!(read_png(&wide).unwrap_err().to_string().contains("4096"));
        let not_png = dir.path().join("fake.png");
        std::fs::write(&not_png, b"not a PNG").unwrap();
        assert!(read_png(&not_png).is_err());
    }
}
