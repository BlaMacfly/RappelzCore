//! Jeux de sprites `.spr` (et `.sdb`, format source non lu par le client).
//!
//! `.spr` (parseur client `FUN_00addb80`) : texte CRLF,
//! ```text
//! <nom d'animation>
//! <N>
//! N × { <nom de fichier image> / <ligne "w,h"> / <ligne "0"> }
//! ```
//! Le client ne lit que la 1re ligne de chaque triplet : « w,h » et « 0 » sont ignorées, la
//! taille d'une image est sa taille réelle une fois chargée (`FUN_00ade650`).

use crate::{nui::NuiDocument, UiError};
use std::collections::HashMap;

/// Une image d'animation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Nom de fichier tel qu'écrit dans le `.spr` (ex. `button_common_new_middle01.png`).
    pub image: String,
    /// Ligne « w,h » du `.spr` (ignorée par le client, gardée pour information).
    pub spr_size: (i32, i32),
    /// Ligne « 0 » du `.spr` (ignorée par le client ; toujours 0 dans NoLimits).
    pub spr_extra: i32,
    /// Sous-rectangle (x, y, w, h) de l'image dans sa planche (`.spr` binaires seulement).
    pub sub: Option<(i32, i32, i32, i32)>,
    /// Nom réel trouvé dans l'archive après résolution (`None` tant que non résolu ou absent).
    pub file: Option<String>,
    /// Taille réelle de l'image (0 tant que non résolue).
    pub w: u32,
    pub h: u32,
}

impl Frame {
    pub fn is_resolved(&self) -> bool {
        self.file.is_some()
    }
}

/// Une animation (= un « ani » des `.nui`) : liste d'images.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<Frame>,
    /// Fichier `.spr` d'où vient la définition retenue.
    pub source: String,
}

impl Animation {
    /// Vrai si toutes les images sont résolues. Le client abandonne l'animation entière si une
    /// texture manque (« Texture Resource Not Found », `FUN_00ae5100`).
    pub fn is_drawable(&self) -> bool {
        !self.frames.is_empty() && self.frames.iter().all(Frame::is_resolved)
    }
}

/// Lit un `.spr`. Les noms d'animation vides sont ignorés.
pub fn parse_spr(text: &str, source: &str) -> Result<Vec<Animation>, UiError> {
    let lines: Vec<&str> = text.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let name = lines[i].trim();
        i += 1;
        if name.is_empty() {
            continue;
        }
        let count_line = lines.get(i).copied().unwrap_or("").trim();
        let n: usize = count_line.parse().map_err(|_| UiError::Spr {
            line: i + 1,
            msg: format!("nombre d'images attendu après « {name} », trouvé « {count_line} »"),
        })?;
        i += 1;
        let mut frames = Vec::with_capacity(n);
        for k in 0..n {
            let image = lines.get(i).copied().ok_or_else(|| UiError::Spr {
                line: i + 1,
                msg: format!("« {name} » : image {k}/{n} manquante (fin de fichier)"),
            })?;
            let size = lines.get(i + 1).copied().unwrap_or("");
            let extra = lines.get(i + 2).copied().unwrap_or("");
            i += 3;
            let spr_size = size
                .split_once(',')
                .and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))
                .unwrap_or((0, 0));
            frames.push(Frame {
                image: image.trim().to_string(),
                spr_size,
                spr_extra: extra.trim().parse().unwrap_or(0),
                sub: None,
                file: None,
                w: 0,
                h: 0,
            });
        }
        out.push(Animation { name: name.to_string(), frames, source: source.to_string() });
    }
    Ok(out)
}

/// Lit un `.spr` texte ou binaire (`DIFKNIBK`, voir [`crate::kbin`]).
pub fn parse_spr_bytes(bytes: &[u8], source: &str) -> Result<Vec<Animation>, UiError> {
    if bytes.starts_with(crate::kbin::MAGIC) {
        crate::kbin::parse_binary_spr(bytes, source)
    } else {
        parse_spr(&crate::decode_text(bytes), source)
    }
}

/// Lit un `.sdb` (base source de l'outil de sprites : `nom / N / N × { chemin complet / 0 }`).
/// Le client ne lit pas ces fichiers (aucune chaîne « .sdb » dans `SFrame.exe`) ; l'image
/// retenue est le nom de fichier du chemin.
pub fn parse_sdb(text: &str, source: &str) -> Result<Vec<Animation>, UiError> {
    let lines: Vec<&str> = text.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let name = lines[i].trim();
        i += 1;
        if name.is_empty() {
            continue;
        }
        let n: usize = lines.get(i).map(|l| l.trim()).unwrap_or("").parse().map_err(|_| UiError::Spr {
            line: i + 1,
            msg: format!("nombre d'images attendu après « {name} »"),
        })?;
        i += 1;
        let mut frames = Vec::with_capacity(n);
        for _ in 0..n {
            let path = lines.get(i).copied().unwrap_or("").trim();
            i += 2;
            let image = path.rsplit(['\\', '/']).next().unwrap_or(path).to_string();
            frames.push(Frame { image, spr_size: (0, 0), spr_extra: 0, sub: None, file: None, w: 0, h: 0 });
        }
        out.push(Animation { name: name.to_string(), frames, source: source.to_string() });
    }
    Ok(out)
}

/// Noms candidats d'un fichier image : le nom exact d'abord (seul essayé par le chemin de
/// chargement observé, `FUN_00ae5100`), puis la même base avec les autres extensions connues
/// de l'archive (repli de RappelzRust, non observé dans le client).
pub fn image_candidates(name: &str) -> Vec<String> {
    let mut v = vec![name.to_string()];
    let stem = match name.rfind('.') {
        Some(p) => &name[..p],
        None => name,
    };
    for ext in ["dds", "tga", "png", "jpg", "bmp"] {
        let c = format!("{stem}.{ext}");
        if !c.eq_ignore_ascii_case(name) {
            v.push(c);
        }
    }
    v
}

/// Statistiques de résolution des images.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolveStats {
    /// Animations demandées (références `ani` / `caption` d'un document, ou toutes).
    pub animations: usize,
    /// Animations inconnues du jeu de sprites.
    pub missing_animations: Vec<String>,
    /// Images trouvées sous leur nom exact.
    pub exact: usize,
    /// Images trouvées seulement avec une autre extension.
    pub by_extension: usize,
    /// Images introuvables.
    pub missing_images: Vec<String>,
}

/// Ensemble d'animations indexées par nom (insensible à la casse).
#[derive(Clone, Debug, Default)]
pub struct SpriteSet {
    anims: HashMap<String, Animation>,
}

/// Fichiers préchargés par `LoadSpriteSet` (`FUN_00ac2820`, table `0x00e98444`) avant le
/// `.spr` demandé.
pub const PRELOADED_SPR: [&str; 6] = ["01_equip.spr", "02_item.spr", "03_skill.spr", "04_event.spr", "05_ui.spr", "06_live.spr"];

impl SpriteSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.anims.len()
    }

    pub fn is_empty(&self) -> bool {
        self.anims.is_empty()
    }

    pub fn animation(&self, name: &str) -> Option<&Animation> {
        self.anims.get(&name.to_ascii_lowercase())
    }

    pub fn animation_mut(&mut self, name: &str) -> Option<&mut Animation> {
        self.anims.get_mut(&name.to_ascii_lowercase())
    }

    pub fn animations(&self) -> impl Iterator<Item = &Animation> {
        self.anims.values()
    }

    /// Ajoute des animations ; une animation déjà présente est REMPLACÉE (comportement d'un
    /// même fichier, `FUN_00abfcd0`).
    pub fn insert_replace(&mut self, anims: Vec<Animation>) {
        for a in anims {
            self.anims.insert(a.name.to_ascii_lowercase(), a);
        }
    }

    /// Fusionne des animations ; une animation déjà présente est GARDÉE (fusion de jeux,
    /// `FUN_00ac2a60`), sauf les noms commençant par `static_common_rank` qui sont remplacés.
    pub fn merge(&mut self, anims: Vec<Animation>) {
        for a in anims {
            let key = a.name.to_ascii_lowercase();
            if key.starts_with("static_common_rank") || !self.anims.contains_key(&key) {
                self.anims.insert(key, a);
            }
        }
    }

    /// Lit un `.spr` (texte) et le fusionne (première définition gardée entre fichiers,
    /// dernière gardée à l'intérieur d'un même fichier).
    pub fn merge_spr(&mut self, text: &str, source: &str) -> Result<usize, UiError> {
        self.merge_anims(parse_spr(text, source)?)
    }

    /// Comme [`merge_spr`](Self::merge_spr), pour un `.spr` texte ou binaire.
    pub fn merge_spr_bytes(&mut self, bytes: &[u8], source: &str) -> Result<usize, UiError> {
        self.merge_anims(parse_spr_bytes(bytes, source)?)
    }

    fn merge_anims(&mut self, anims: Vec<Animation>) -> Result<usize, UiError> {
        let mut one = SpriteSet::new();
        one.insert_replace(anims);
        let n = one.len();
        self.merge(one.anims.into_values().collect());
        Ok(n)
    }

    /// Construit le jeu utilisé par les fenêtres `.nui` dont `spr = <name>` : les six `.spr`
    /// préchargés (`PRELOADED_SPR`), puis `name`, puis `ui_frame_patch.spr` s'il existe.
    /// `read` renvoie le contenu d'un fichier de l'archive (ou `None` s'il est absent).
    pub fn load_ui_set(mut read: impl FnMut(&str) -> Option<Vec<u8>>, name: &str) -> SpriteSet {
        let mut set = SpriteSet::new();
        let patch = name.eq_ignore_ascii_case("ui_frame.spr").then_some("ui_frame_patch.spr");
        for f in PRELOADED_SPR.iter().copied().chain(std::iter::once(name)).chain(patch) {
            if let Some(bytes) = read(f) {
                // Un .spr illisible est ignoré comme le ferait le client (journal d'erreur).
                let _ = set.merge_spr_bytes(&bytes, f);
            }
        }
        set
    }

    /// Résout les images d'une animation. `lookup(nom)` renvoie `Some((nom réel, w, h))` si le
    /// fichier existe dans l'archive (essayé pour chaque candidat de [`image_candidates`]).
    pub fn resolve_animation(
        &mut self,
        name: &str,
        lookup: &mut impl FnMut(&str) -> Option<(String, u32, u32)>,
        stats: &mut ResolveStats,
    ) -> bool {
        stats.animations += 1;
        let Some(anim) = self.anims.get_mut(&name.to_ascii_lowercase()) else {
            stats.missing_animations.push(name.to_string());
            return false;
        };
        for f in &mut anim.frames {
            if f.file.is_some() {
                stats.exact += 1;
                continue;
            }
            let mut found = None;
            for (k, c) in image_candidates(&f.image).iter().enumerate() {
                if let Some(r) = lookup(c) {
                    found = Some((k, r));
                    break;
                }
            }
            match found {
                Some((k, (file, w, h))) => {
                    if k == 0 {
                        stats.exact += 1;
                    } else {
                        stats.by_extension += 1;
                    }
                    f.file = Some(file);
                    match f.sub {
                        Some((_, _, sw, sh)) => (f.w, f.h) = (sw.max(0) as u32, sh.max(0) as u32),
                        None => (f.w, f.h) = (w, h),
                    }
                }
                None => stats.missing_images.push(f.image.clone()),
            }
        }
        anim.is_drawable()
    }

    /// Résout toutes les animations référencées par un document (`ani` et, pour les
    /// `captionbutton`, `caption`).
    pub fn resolve_document(
        &mut self,
        doc: &NuiDocument,
        mut lookup: impl FnMut(&str) -> Option<(String, u32, u32)>,
    ) -> ResolveStats {
        let mut stats = ResolveStats::default();
        for name in doc.animation_refs() {
            self.resolve_animation(&name, &mut lookup, &mut stats);
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPR: &str = "button_a\r\n3\r\na_01.tga\r\n13,24\r\n0\r\na_02.tga\r\n0,0\r\n0\r\na_03.tga\r\n0,0\r\n0\r\nNew Sprite 39\r\n0\r\nlogo\r\n1\r\nlogo.tga\r\n1024,768\r\n0\r\n";

    #[test]
    fn parse() {
        let a = parse_spr(SPR, "t.spr").unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(a[0].frames.len(), 3);
        assert_eq!(a[0].frames[0].spr_size, (13, 24));
        assert_eq!(a[1].frames.len(), 0);
        assert_eq!(a[2].frames[0].image, "logo.tga");
    }

    #[test]
    fn merge_rules() {
        let mut s = SpriteSet::new();
        s.merge_spr("x\r\n1\r\nfirst.tga\r\n0,0\r\n0\r\nx\r\n1\r\nsecond.tga\r\n0,0\r\n0\r\n", "a.spr").unwrap();
        assert_eq!(s.animation("X").unwrap().frames[0].image, "second.tga");
        s.merge_spr("x\r\n1\r\nthird.tga\r\n0,0\r\n0\r\n", "b.spr").unwrap();
        assert_eq!(s.animation("x").unwrap().frames[0].image, "second.tga");
    }

    #[test]
    fn candidates() {
        let c = image_candidates("a_01.tga");
        assert_eq!(c[0], "a_01.tga");
        assert!(c.contains(&"a_01.png".to_string()));
        assert_eq!(c.len(), 5);
    }

    #[test]
    fn sdb() {
        let a = parse_sdb("icon\r\n1\r\n\\x\\y\\icon.jpg\r\n0\r\n", "t.sdb").unwrap();
        assert_eq!(a[0].frames[0].image, "icon.jpg");
    }
}
