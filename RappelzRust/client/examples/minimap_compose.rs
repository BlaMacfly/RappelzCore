//! Assemble les 16×16 morceaux de minicarte d'une tuile dans les deux ordres possibles,
//! pour vérifier l'orientation : `cargo run --example minimap_compose -- <Data> <tx> <ty> <sortie>`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut chain = rz_mpq::MpqChain::open_client_data(&a[1], "frFR").unwrap();
    let (tx, ty): (u32, u32) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    for (label, swap) in [("xy", false), ("yx", true)] {
        let mut out = image::RgbaImage::new(16 * 128, 16 * 128);
        for i in 0..16u32 {
            for j in 0..16u32 {
                let name = format!(r"World\Maps\Tiles\m{tx:03}_{ty:03}_{i}_{j}.jpg");
                let Ok(bytes) = chain.read_file(&name) else { continue };
                let img = image::load_from_memory(&bytes).unwrap().to_rgba8();
                let (px, py) = if swap { (j, i) } else { (i, j) };
                image::imageops::overlay(&mut out, &img, (px * 128) as i64, (py * 128) as i64);
            }
        }
        let small = image::imageops::resize(&out, 768, 768, image::imageops::FilterType::Triangle);
        small.save(format!("{}_{label}.png", a[4])).unwrap();
    }
}
