//! Accès aux ressources, comme un client WoW : chaîne de MPQ de `Data/`, DBC de
//! `DBFilesClient\`, textures et polices. Les ressources Rappelz se cherchent aussi par leur
//! nom d'origine (sans dossier) grâce aux `(listfile)`.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use macroquad::prelude::*;
use rz_mpq::MpqChain;
use rz_rappelz::dbcstore::{NamedDbc, StringTable};

pub struct Assets {
    chain: MpqChain,
    by_basename: HashMap<String, String>,
    textures: HashMap<String, Option<Texture2D>>,
    pub strings: StringTable,
    pub npcs: Option<NamedDbc>,
    pub jobs: Option<NamedDbc>,
    pub items: Option<NamedDbc>,
    pub monsters: Option<NamedDbc>,
    /// `ItemResource.dbc` : noms de fichiers référencés par les champs i16 d'Item.dbc.
    pub item_res: Vec<String>,
}

impl Assets {
    pub fn open(data_dir: &Path, locale: &str) -> Result<Self> {
        let mut chain = MpqChain::open_client_data(data_dir, locale)
            .with_context(|| format!("ouverture des MPQ de {}", data_dir.display()))?;
        anyhow::ensure!(
            !chain.archives().is_empty(),
            "aucun MPQ dans {} : lancez « rz-extractor client »",
            data_dir.display()
        );
        let mut by_basename = HashMap::new();
        for full in chain.list()? {
            let base = full.rsplit('\\').next().unwrap_or(&full).to_ascii_lowercase();
            by_basename.insert(base, full);
        }
        let strings = chain
            .read_file("DBFilesClient\\String.dbc")
            .ok()
            .and_then(|b| StringTable::from_bytes(&b).ok())
            .unwrap_or_else(StringTable::empty);
        let mut dbc = |table: &str, file: &str| {
            chain.read_file(&format!("DBFilesClient\\{file}")).ok().and_then(|b| NamedDbc::from_bytes(table, &b).ok())
        };
        let npcs = dbc("npcresource", "NpcResource.dbc");
        let jobs = dbc("job", "Job.dbc");
        let items = dbc("item", "Item.dbc");
        let monsters = dbc("monster", "Monster.dbc");
        let item_res = chain
            .read_file(r"DBFilesClient\ItemResource.dbc")
            .ok()
            .and_then(|b| rz_dbc::DbcFile::parse(&b).ok())
            .map(|f| f.iter().map(|r| r.str(1).to_string()).collect())
            .unwrap_or_default();
        tracing::info!(
            "{} archives, {} fichiers, {} textes",
            chain.archives().len(),
            by_basename.len(),
            strings.len()
        );
        Ok(Self { chain, by_basename, textures: HashMap::new(), strings, npcs, jobs, items, monsters, item_res })
    }

    /// Lit un fichier par chemin MPQ complet ou par nom Rappelz d'origine.
    pub fn read(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Ok(b) = self.chain.read_file(name) {
            return Some(b);
        }
        let full = self.by_basename.get(&name.to_ascii_lowercase())?.clone();
        self.chain.read_file(&full).ok()
    }

    /// Texture (JPEG/PNG/BMP) mise en cache ; `None` si absente ou illisible.
    pub fn texture(&mut self, name: &str) -> Option<Texture2D> {
        let key = name.to_ascii_lowercase();
        if let Some(t) = self.textures.get(&key) {
            return t.clone();
        }
        let tex = self.resolve_image(name).and_then(|(real, bytes)| {
            let img = rz_texture::decode(&real, &bytes).ok()?;
            let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.pixels);
            t.set_filter(FilterMode::Linear);
            Some(t)
        });
        self.textures.insert(key, tex.clone());
        tex
    }

    /// Comme le client Rappelz, une image citée en `.tga` peut exister en `.png`, `.jpg` ou `.dds`.
    pub fn resolve_image(&mut self, name: &str) -> Option<(String, Vec<u8>)> {
        if let Some(b) = self.read(name) {
            return Some((name.to_string(), b));
        }
        let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
        for ext in ["tga", "png", "jpg", "dds", "bmp"] {
            let candidate = format!("{stem}.{ext}");
            if let Some(b) = self.read(&candidate) {
                return Some((candidate, b));
            }
        }
        None
    }

    pub fn font(&mut self, name: &str) -> Option<Font> {
        let bytes = self.read(name)?;
        load_ttf_font_from_bytes(&bytes).ok()
    }

    pub fn npc_name(&self, npc_id: u32) -> String {
        self.npcs
            .as_ref()
            .and_then(|d| d.by_id(npc_id as i32))
            .map(|r| self.strings.get(r.int("name_text_id")).to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("PNJ {npc_id}"))
    }

    /// Métiers de départ (profondeur 0) : (id, nom).
    pub fn starting_jobs(&self) -> Vec<(i32, String)> {
        let Some(jobs) = &self.jobs else { return vec![(0, "Novice".into())] };
        let mut out: Vec<(i32, String)> = jobs
            .rows()
            .filter(|r| r.int("job_depth") == 0)
            .map(|r| (r.id(), self.strings.get(r.int("text_id")).to_string()))
            .collect();
        out.sort();
        if out.is_empty() {
            out.push((0, "Novice".into()));
        }
        out
    }

    pub fn job_name(&self, job: i32) -> String {
        self.jobs
            .as_ref()
            .and_then(|d| d.by_id(job))
            .map(|r| self.strings.get(r.int("text_id")).to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("Métier {job}"))
    }
}

impl Assets {
    /// Nom et icône d'un objet (Item.dbc → ItemResource.dbc).
    pub fn item_name(&self, code: u32) -> String {
        self.items
            .as_ref()
            .and_then(|d| d.by_id(code as i32))
            .map(|r| self.strings.get(r.int("name_id")).to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("Objet {code}"))
    }

    pub fn item_icon(&mut self, code: u32) -> Option<Texture2D> {
        let idx = self.items.as_ref()?.by_id(code as i32)?.int("icon_file_name");
        let name = self.item_res.get(idx.max(0) as usize)?.clone();
        if name.is_empty() {
            return None;
        }
        self.texture(&name)
    }

    /// Fichiers .nx3 d'un objet porté (`model_00..17`, indices dans ItemResource).
    pub fn item_models(&self, code: u32) -> Vec<String> {
        let Some(r) = self.items.as_ref().and_then(|d| d.by_id(code as i32)) else { return Vec::new() };
        (0..18)
            .map(|i| {
                let idx = r.int(&format!("model_{i:02}"));
                self.item_res.get(idx.max(0) as usize).cloned().unwrap_or_default()
            })
            .collect()
    }

    /// Ligne de Monster.dbc d'après l'id brut (bit-permuté) reçu dans TS_SC_ENTER.
    pub fn monster_by_raw(&self, raw_id: u32) -> Option<(String, String, i32)> {
        let id = rz_rappelz::rdb::monster_id_decode(raw_id) as i32;
        let r = self.monsters.as_ref()?.by_id(id)?;
        Some((self.strings.get(r.int("name_id")).to_string(), r.str("model").to_string(), r.int("level")))
    }

    pub fn npc_model(&self, npc_id: u32) -> Option<String> {
        let r = self.npcs.as_ref()?.by_id(npc_id as i32)?;
        Some(r.str("model_file").to_string()).filter(|s| !s.is_empty())
    }
}
