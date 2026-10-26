//! Lecture des fichiers `.conf` au format AzerothCore.
//!
//! ```text
//! [authserver]
//! # commentaire
//! LoginDatabaseInfo = "127.0.0.1;3306;acore;acore;acore_auth"
//! RealmServerPort   = 4500
//! ```
//!
//! Comme AzerothCore, une variable d'environnement `AC_<CLE_EN_SNAKE_CASE>` remplace
//! la valeur du fichier (ex. `AC_LOGIN_DATABASE_INFO`).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

#[derive(Clone, Debug, Default)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn parse(text: &str) -> Self {
        let mut values = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            values.insert(key.trim().to_string(), value.to_string());
        }
        Self { values }
    }

    /// Charge `path`, ou à défaut `path.dist` (comme `authserver.conf.dist`).
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => {
                let dist = path.with_extension(format!(
                    "{}.dist",
                    path.extension().and_then(|e| e.to_str()).unwrap_or("conf")
                ));
                tracing::warn!(
                    "{} introuvable, utilisation de {}",
                    path.display(),
                    dist.display()
                );
                std::fs::read_to_string(&dist)
                    .with_context(|| format!("lecture de {}", dist.display()))?
            }
        };
        Ok(Self::parse(&text))
    }

    fn env_key(key: &str) -> String {
        let mut out = String::from("AC_");
        let chars: Vec<char> = key.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if c == '.' {
                out.push('_');
                continue;
            }
            let prev_lower = i > 0 && chars[i - 1].is_ascii_lowercase();
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
            let prev_upper = i > 0 && chars[i - 1].is_ascii_uppercase();
            if c.is_ascii_uppercase() && i > 0 && (prev_lower || (prev_upper && next_lower)) {
                out.push('_');
            }
            out.push(c.to_ascii_uppercase());
        }
        out
    }

    pub fn get(&self, key: &str) -> Option<String> {
        std::env::var(Self::env_key(key))
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn string(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn int<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    pub fn float(&self, key: &str, default: f32) -> f32 {
        self.int(key, default)
    }

    pub fn bool(&self, key: &str, default: bool) -> bool {
        match self.get(key).as_deref() {
            Some("1" | "true" | "True") => true,
            Some("0" | "false" | "False") => false,
            _ => default,
        }
    }
}

/// Paramètres de connexion MySQL au format AzerothCore : `hôte;port;utilisateur;mdp;base`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DatabaseInfo {
    pub fn parse(info: &str) -> Result<Self> {
        let parts: Vec<&str> = info.split(';').collect();
        anyhow::ensure!(
            parts.len() >= 5,
            "format attendu « hôte;port;utilisateur;mot_de_passe;base », reçu « {info} »"
        );
        Ok(Self {
            host: parts[0].to_string(),
            port: parts[1].parse().context("port MySQL invalide")?,
            user: parts[2].to_string(),
            password: parts[3].to_string(),
            database: parts[4].to_string(),
        })
    }

    pub fn url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            pct(&self.user),
            pct(&self.password),
            self.host,
            self.port,
            self.database
        )
    }
}

fn pct(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_conf() {
        let c = Config::parse(
            "[authserver]\n# c\nLoginDatabaseInfo = \"127.0.0.1;3306;acore;acore;acore_auth\"\nRealmServerPort = 4500\nLogLevel=2\n",
        );
        assert_eq!(c.int("RealmServerPort", 0u16), 4500);
        assert_eq!(c.int("LogLevel", 0u8), 2);
        let db = DatabaseInfo::parse(&c.string("LoginDatabaseInfo", "")).unwrap();
        assert_eq!(db.database, "acore_auth");
        assert_eq!(db.url(), "mysql://acore:acore@127.0.0.1:3306/acore_auth");
    }

    #[test]
    fn env_keys() {
        assert_eq!(Config::env_key("LoginDatabaseInfo"), "AC_LOGIN_DATABASE_INFO");
        assert_eq!(Config::env_key("Rate.XP.Kill"), "AC_RATE_XP_KILL");
        assert_eq!(Config::env_key("WorldServerPort"), "AC_WORLD_SERVER_PORT");
    }
}
