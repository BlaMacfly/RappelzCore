//! Rangement des ressources Rappelz (noms plats, sans dossier) dans une arborescence
//! façon World of Warcraft à l'intérieur des MPQ :
//!
//! | WoW            | Contenu Rappelz                                              |
//! |----------------|--------------------------------------------------------------|
//! | `World\Maps\`  | cartes : .nfm .nfa .nfc .nfs .nfw .nfl .nfp .nfe .nfk .qpf .pvs, tuiles `m###_###` |
//! | `Character\`   | modèles/animations de personnages joueurs (préfixes de race/sexe) |
//! | `Creature\`    | monstres, invocations, PNJ                                    |
//! | `World\`       | décors (.nx3/.naf/.cob restants), donjons                     |
//! | `Spells\`      | effets visuels (rcfx, rcwfx, fx, wfx, .spt, .fx)              |
//! | `Interface\`   | UI : .spr .nui, icônes (`Interface\Icons\`), panneaux, boutons |
//! | `Sound\`       | .wav                                                          |
//! | `Fonts\`       | .ttf .otf                                                     |
//! | `DBFilesClient\` | .rdb d'origine (convertis en .dbc par l'extracteur)         |
//! | `Textures\`    | textures restantes                                            |
//!
//! Le nom de fichier d'origine est conservé, donc le client retrouve une ressource par
//! son nom Rappelz via l'index des `(listfile)`.

/// Préfixes des modèles de personnages joueurs (as/de/ga = Asura/Deva/Gaia, m/f = sexe).
const CHARACTER_PREFIXES: [&str; 8] = ["asm", "asf", "dem", "def", "gam", "gaf", "ain", "novice"];
const CREATURE_PREFIXES: [&str; 8] = ["m", "beast", "spirit", "demon", "undead", "angel", "npc", "mechanic"];
const EFFECT_PREFIXES: [&str; 6] = ["rcfx", "rcwfx", "rcsf", "fx", "wfx", "effect"];
const INTERFACE_PREFIXES: [&str; 6] = ["game", "button", "window", "common", "worldmap", "card"];

fn ext(name: &str) -> String {
    name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default()
}

/// Premier mot du nom (avant `_`, chiffre ou `.`), en minuscules.
fn stem_prefix(name: &str) -> String {
    name.to_ascii_lowercase()
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect()
}

fn is_terrain_tile(name: &str) -> bool {
    // m002_001.bmp, m003_000.jpg… : tuiles de terrain des cartes.
    let b = name.as_bytes();
    b.len() >= 8
        && b[0].eq_ignore_ascii_case(&b'm')
        && b[1..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'_'
        && b[5..8].iter().all(u8::is_ascii_digit)
}

/// Dossier WoW (avec `\` final) pour une ressource Rappelz.
pub fn wow_folder(name: &str) -> &'static str {
    let e = ext(name);
    let p = stem_prefix(name);
    match e.as_str() {
        "nfm" | "nfa" | "nfc" | "nfs" | "nfw" | "nfl" | "nfp" | "nfe" | "nfk" | "qpf" | "pvs" => {
            return "World\\Maps\\"
        }
        "rdb" | "ref" => return "DBFilesClient\\",
        "wav" | "m4v" => return "Sound\\",
        "ttf" | "otf" => return "Fonts\\",
        "spr" | "nui" => return "Interface\\",
        "lua" | "xml" | "cfg" | "ini" | "txt" | "lst" | "tml" => return "Interface\\Scripts\\",
        "spt" | "fx" => return "Spells\\",
        _ => {}
    }
    if is_terrain_tile(name) {
        return "World\\Maps\\Tiles\\";
    }
    if p == "icon" {
        return "Interface\\Icons\\";
    }
    if EFFECT_PREFIXES.contains(&p.as_str()) {
        return "Spells\\";
    }
    if INTERFACE_PREFIXES.contains(&p.as_str()) {
        return "Interface\\";
    }
    if CHARACTER_PREFIXES.contains(&p.as_str()) {
        return "Character\\";
    }
    if CREATURE_PREFIXES.contains(&p.as_str()) {
        return "Creature\\";
    }
    match e.as_str() {
        "nx3" | "naf" | "cob" | "itc" | "itm" | "itb" => "World\\",
        _ => "Textures\\",
    }
}

pub fn wow_path(name: &str) -> String {
    format!("{}{}", wow_folder(name), name)
}

/// Les formats déjà compressés ne gagnent rien au zlib.
pub fn should_compress(name: &str) -> bool {
    !matches!(ext(name).as_str(), "jpg" | "png" | "m4v" | "ogg" | "mp3")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders() {
        assert_eq!(wow_path("m002_001.bmp"), "World\\Maps\\Tiles\\m002_001.bmp");
        assert_eq!(wow_folder("db_item.rdb"), "DBFilesClient\\");
        assert_eq!(wow_folder("icon_skill_01.dds"), "Interface\\Icons\\");
        assert_eq!(wow_folder("asm_body_01.nx3"), "Character\\");
        assert_eq!(wow_folder("rcfx_fire.cob"), "Spells\\");
        assert_eq!(wow_folder("hit.wav"), "Sound\\");
        assert_eq!(wow_folder("dungeon_wall.dds"), "Textures\\");
    }
}
