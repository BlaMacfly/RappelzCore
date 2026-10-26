//! Icône de RappelzRust générée depuis `assets/RappelzRust.png` :
//! - `RappelzRust.ico` (16 à 256 px) intégré à l'exécutable Windows avec ses informations de
//!   version ;
//! - icônes RGBA 16/32/64 px de la fenêtre (macroquad), incluses par `main.rs`.

use std::fs::File;
use std::path::PathBuf;

use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::imageops::FilterType;
use image::ExtendedColorType;

const LOGO: &str = "assets/RappelzRust.png";

fn main() {
    println!("cargo:rerun-if-changed={LOGO}");
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let logo = image::open(LOGO).expect("lecture du logo").to_rgba8();

    for size in [16u32, 32, 64] {
        let icon = image::imageops::resize(&logo, size, size, FilterType::Lanczos3);
        std::fs::write(out.join(format!("icon_{size}.rgba")), icon.as_raw()).expect("icône de fenêtre");
    }

    let frames: Vec<IcoFrame> = [16u32, 24, 32, 48, 64, 128, 256]
        .iter()
        .map(|&s| {
            let img = image::imageops::resize(&logo, s, s, FilterType::Lanczos3);
            IcoFrame::as_png(img.as_raw(), s, s, ExtendedColorType::Rgba8).expect("image d'icône")
        })
        .collect();
    let ico = out.join("RappelzRust.ico");
    IcoEncoder::new(File::create(&ico).expect("création du .ico")).encode_images(&frames).expect("écriture du .ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico.to_str().expect("chemin UTF-8"))
            .set("ProductName", "RappelzRust")
            .set("FileDescription", "RappelzRust — client Rappelz")
            .set("CompanyName", "BlaMacfly")
            .set("LegalCopyright", "GPL-2.0-or-later");
        if let Err(e) = res.compile() {
            // Sans Windows SDK (rc.exe), l'exe est produit sans icône intégrée.
            println!("cargo:warning=icône Windows non intégrée : {e}");
        }
    }
}
