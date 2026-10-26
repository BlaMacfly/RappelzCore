//! Configuration façon WoW : `WTF/Config.wtf` (`SET clé "valeur"`) et
//! `Data/<locale>/realmlist.wtf` (`set realmlist hôte[:port]`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct ClientConfig {
    path: PathBuf,
    values: BTreeMap<String, String>,
}

impl ClientConfig {
    pub fn load(root: &Path) -> Self {
        let path = root.join("WTF").join("Config.wtf");
        let mut values = BTreeMap::new();
        if let Ok(text) = std::fs::read_to_string(&path) {
            for line in text.lines() {
                let mut it = line.trim().splitn(3, ' ');
                if !it.next().unwrap_or("").eq_ignore_ascii_case("SET") {
                    continue;
                }
                if let (Some(k), Some(v)) = (it.next(), it.next()) {
                    values.insert(k.to_string(), v.trim().trim_matches('"').to_string());
                }
            }
        }
        Self { path, values }
    }

    pub fn get(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn save(&self) {
        let text: String = self.values.iter().map(|(k, v)| format!("SET {k} \"{v}\"\r\n")).collect();
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&self.path, text);
    }
}

/// Adresse de l'authserver depuis realmlist.wtf (port 4615 par défaut, comme NoLimits).
pub fn realmlist(root: &Path, locale: &str) -> (String, u16) {
    let text = std::fs::read_to_string(root.join("Data").join(locale).join("realmlist.wtf")).unwrap_or_default();
    let value = text
        .lines()
        .find_map(|l| {
            let l = l.trim();
            l.to_ascii_lowercase().starts_with("set realmlist").then(|| l[13..].trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "127.0.0.1".into());
    match value.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(4615)),
        None => (value, 4615),
    }
}
