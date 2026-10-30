//! rz-extractor : passerelle entre le client Rappelz d'origine et l'écosystème RappelzRust.
//!
//! - `client` : convertit `data.000-008` + `Resource\` en une installation façon WoW 3.3.5 :
//!   ```text
//!   <sortie>/
//!     Data/common.MPQ, common-2.MPQ, …          ressources (World, Character, Creature…)
//!     Data/<locale>/locale-<locale>.MPQ         DBFilesClient\*.dbc + écran de connexion
//!     Data/<locale>/realmlist.wtf               « set realmlist 127.0.0.1 »
//!     WTF/Config.wtf                            options du client
//!   ```
//! - `dbc` : comme le `map_extractor` d'AzerothCore, extrait `DBFilesClient\*.dbc` des MPQ
//!   d'un client RappelzRust vers le `DataDir` du worldserver.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use rz_mpq::{MpqChain, MpqWriter, PreparedFile};
use rz_rappelz::rdb::{self, Decoder};
use rz_rappelz::{layout, to_dbc, DataArchive};

#[derive(Parser)]
#[command(name = "rz-extractor", version, about = "Conversion client Rappelz → RappelzRust (MPQ/DBC)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Installe l'addon Lua de demonstration, sans modifier les fichiers deja presents.
    Addons {
        #[arg(long)]
        out: PathBuf,
    },
    /// Convertit un client Rappelz en données RappelzRust.
    Client {
        /// Dossier du client Rappelz (contient data.000 et Resource\).
        #[arg(long)]
        client: PathBuf,
        /// Dossier d'installation RappelzRust à produire.
        #[arg(long)]
        out: PathBuf,
        /// Locale façon WoW (enUS, frFR, deDE…).
        #[arg(long, default_value = "frFR")]
        locale: String,
        /// Code page des textes Rappelz (db_string(<codepage>).rdb).
        #[arg(long, default_value = "windows-1252")]
        codepage: String,
        /// Taille maximale d'un MPQ avant de passer au suivant (common-2.MPQ…), en Mio.
        #[arg(long, default_value_t = 1900)]
        max_archive_mb: u64,
        /// Niveau zlib (0 = stocké, 9 = maximum).
        #[arg(long, default_value_t = 6)]
        compression: u32,
        /// Adresse du serveur d'authentification écrite dans realmlist.wtf.
        #[arg(long, default_value = "127.0.0.1")]
        realmlist: String,
        /// Ne produire que le MPQ de locale (DBC), sans les ressources.
        #[arg(long)]
        dbc_only: bool,
    },
    /// Vérifie, octet par octet, que les MPQ contiennent toutes les ressources du client Rappelz.
    Verify {
        #[arg(long)]
        client: PathBuf,
        /// Dossier `Data/` du client RappelzRust.
        #[arg(long)]
        data: PathBuf,
        #[arg(long, default_value = "frFR")]
        locale: String,
    },
    /// Extrait les DBC d'un client RappelzRust vers le DataDir du worldserver.
    Dbc {
        /// Dossier `Data/` du client RappelzRust.
        #[arg(long)]
        data: PathBuf,
        #[arg(long, default_value = "frFR")]
        locale: String,
        /// DataDir du worldserver (les fichiers vont dans `<out>/dbc/`).
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    rz_common::init_logging("info");
    match Cli::parse().command {
        Command::Addons { out } => write_example_addon(&out),
        Command::Client {
            client,
            out,
            locale,
            codepage,
            max_archive_mb,
            compression,
            realmlist,
            dbc_only,
        } => {
            let started = Instant::now();
            let data_dir = out.join("Data");
            std::fs::create_dir_all(data_dir.join(&locale))?;
            if !dbc_only {
                write_common(&client, &data_dir, max_archive_mb << 20, compression)?;
            }
            write_locale(&client, &data_dir, &locale, &codepage, compression)?;
            write_config(&out, &locale, &realmlist)?;
            write_example_addon(&out)?;
            tracing::info!("Terminé en {:.0?} → {}", started.elapsed(), out.display());
            Ok(())
        }
        Command::Dbc { data, locale, out } => extract_dbc(&data, &locale, &out),
        Command::Verify { client, data, locale } => verify(&client, &data, &locale),
    }
}

fn write_example_addon(out: &Path) -> Result<()> {
    use std::io::Write;
    let directory = out.join("Interface").join("AddOns").join("RappelzDebug");
    std::fs::create_dir_all(&directory)?;
    let files: [(&str, &[u8]); 3] = [
        ("RappelzDebug.toc", include_bytes!("../../../Interface/AddOns/RappelzDebug/RappelzDebug.toc")),
        ("RappelzDebug.lua", include_bytes!("../../../Interface/AddOns/RappelzDebug/RappelzDebug.lua")),
        ("RappelzDebug.xml", include_bytes!("../../../Interface/AddOns/RappelzDebug/RappelzDebug.xml")),
    ];
    for (name, content) in files {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(directory.join(name)) {
            Ok(mut file) => file.write_all(content)?,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(e) => return Err(e.into()),
        }
    }
    tracing::info!("Addon RappelzDebug disponible dans {}", directory.display());
    Ok(())
}

fn verify(client: &Path, data: &Path, locale: &str) -> Result<()> {
    let mut archive = DataArchive::open(client)?;
    let mut chain = MpqChain::open_client_data(data, locale)?;
    let entries: Vec<_> = archive
        .entries()
        .iter()
        .filter(|e| layout::wow_folder(&e.name) != "DBFilesClient\\")
        .cloned()
        .collect();
    let (mut ok, mut bad) = (0usize, 0usize);
    for (i, e) in entries.iter().enumerate() {
        let original = archive.read(e)?;
        match chain.read_file(&layout::wow_path(&e.name)) {
            Ok(converted) if converted == original => ok += 1,
            Ok(_) => {
                bad += 1;
                tracing::error!("contenu différent : {}", e.name);
            }
            Err(err) => {
                bad += 1;
                tracing::error!("{} : {err}", e.name);
            }
        }
        if (i + 1) % 20_000 == 0 {
            tracing::info!("{}/{} vérifiés", i + 1, entries.len());
        }
    }
    tracing::info!("{ok} fichiers identiques, {bad} en erreur");
    anyhow::ensure!(bad == 0, "{bad} fichiers incorrects");
    Ok(())
}

/// Archives `common*.MPQ`, avec bascule vers l'archive suivante au-delà de `max_size`.
struct RollingWriter {
    dir: PathBuf,
    prefix: &'static str,
    index: usize,
    max_size: u64,
    compression: u32,
    current: Option<MpqWriter>,
    total_files: usize,
}

impl RollingWriter {
    fn new(dir: &Path, prefix: &'static str, max_size: u64, compression: u32) -> Self {
        Self {
            dir: dir.to_path_buf(),
            prefix,
            index: 0,
            max_size,
            compression,
            current: None,
            total_files: 0,
        }
    }

    fn path(&self, index: usize) -> PathBuf {
        if index <= 1 {
            self.dir.join(format!("{}.MPQ", self.prefix))
        } else {
            self.dir.join(format!("{}-{index}.MPQ", self.prefix))
        }
    }

    fn add(&mut self, file: PreparedFile) -> Result<()> {
        let needs_new = match &self.current {
            None => true,
            Some(w) => w.size() + file.stored_size() as u64 > self.max_size,
        };
        if needs_new {
            self.close()?;
            self.index += 1;
            let path = self.path(self.index);
            tracing::info!("Écriture de {}", path.display());
            let mut w = MpqWriter::create(&path, 3)?;
            w.set_compression_level(self.compression);
            self.current = Some(w);
        }
        self.current.as_mut().expect("archive ouverte").add_prepared(file)?;
        self.total_files += 1;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if let Some(w) = self.current.take() {
            let n = w.file_count();
            w.finish()?;
            tracing::info!("  {} fichiers", n);
        }
        Ok(())
    }
}

fn write_common(client: &Path, data_dir: &Path, max_size: u64, compression: u32) -> Result<()> {
    let mut archive = DataArchive::open(client)
        .with_context(|| format!("ouverture de {}", client.join("data.000").display()))?;
    // Ordre WoW (dossier puis nom) ; les .rdb vont dans le MPQ de locale sous forme de DBC.
    let mut entries: Vec<_> = archive
        .entries()
        .iter()
        .filter(|e| layout::wow_folder(&e.name) != "DBFilesClient\\")
        .cloned()
        .collect();
    entries.sort_by_cached_key(|e| layout::wow_path(&e.name).to_ascii_lowercase());
    // Retirer d'éventuelles anciennes archives (common-2.MPQ…) d'une conversion précédente.
    for entry in std::fs::read_dir(data_dir)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if name.starts_with("common") && name.ends_with(".MPQ") {
            std::fs::remove_file(data_dir.join(&name))?;
        }
    }

    let mut writer = RollingWriter::new(data_dir, "common", max_size, compression);
    let total = entries.len();
    let (mut done, mut bytes) = (0usize, 0u64);
    let started = Instant::now();
    const BATCH_BYTES: u64 = 256 << 20;

    let mut batch: Vec<(String, Vec<u8>)> = Vec::new();
    let mut batch_bytes = 0u64;
    let flush = |batch: &mut Vec<(String, Vec<u8>)>, writer: &mut RollingWriter| -> Result<()> {
        let prepared: Vec<PreparedFile> = batch
            .par_iter()
            .map(|(name, data)| {
                PreparedFile::new(&layout::wow_path(name), data, layout::should_compress(name), 3, compression)
            })
            .collect::<std::result::Result<_, _>>()?;
        for p in prepared {
            writer.add(p)?;
        }
        batch.clear();
        Ok(())
    };

    for entry in &entries {
        let data = archive.read(entry)?;
        bytes += data.len() as u64;
        batch_bytes += data.len() as u64;
        batch.push((entry.name.clone(), data));
        if batch_bytes >= BATCH_BYTES {
            done += batch.len();
            flush(&mut batch, &mut writer)?;
            batch_bytes = 0;
            let secs = started.elapsed().as_secs_f64().max(0.001);
            tracing::info!(
                "{done}/{total} fichiers, {} Mio lus ({:.0} Mio/s)",
                bytes >> 20,
                (bytes >> 20) as f64 / secs
            );
        }
    }
    flush(&mut batch, &mut writer)?;

    // Musique et images hors archive (Resource\music, Resource\jpg).
    for (sub, folder) in [("music", "Sound\\Music\\"), ("jpg", "Interface\\Glues\\LoadingScreens\\")] {
        let dir = client.join("Resource").join(sub);
        let Ok(read) = std::fs::read_dir(&dir) else { continue };
        let mut files: Vec<_> = read.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect();
        files.sort();
        for path in files {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let data = std::fs::read(&path)?;
            writer.add(PreparedFile::new(&format!("{folder}{name}"), &data, false, 3, compression)?)?;
        }
    }
    writer.close()?;
    tracing::info!("{} fichiers répartis dans {} archive(s) common", writer.total_files, writer.index);
    Ok(())
}

/// Collecte les .rdb : ceux de l'archive, puis ceux de `Resource\` qui les remplacent
/// (avec l'option `notenc`, le client lit ./Resource/ en clair).
fn collect_rdb(client: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    if let Ok(mut archive) = DataArchive::open(client) {
        let rdbs: Vec<_> = archive
            .entries()
            .iter()
            .filter(|e| layout::wow_folder(&e.name) == "DBFilesClient\\")
            .cloned()
            .collect();
        for e in rdbs {
            files.insert(e.name.to_ascii_lowercase(), archive.read(&e)?);
        }
    }
    if let Ok(read) = std::fs::read_dir(client.join("Resource")) {
        for entry in read.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if name.ends_with(".rdb") || name.ends_with(".ref") {
                    files.insert(name, std::fs::read(&path)?);
                }
            }
        }
    }
    Ok(files)
}

fn write_locale(client: &Path, data_dir: &Path, locale: &str, codepage: &str, compression: u32) -> Result<()> {
    let path = data_dir.join(locale).join(format!("locale-{locale}.MPQ"));
    tracing::info!("Écriture de {}", path.display());
    let mut w = MpqWriter::create(&path, 3)?;
    w.set_compression_level(compression);
    let dec = Decoder::new(codepage);
    let wanted_strings = format!("db_string({codepage}).rdb").to_ascii_lowercase();
    let files = collect_rdb(client)?;
    let has_cp_strings = files.contains_key(&wanted_strings);
    let mut converted = 0;
    for (name, data) in &files {
        let Some(table) = rdb::table_name(name) else { continue };
        let result = match table.as_str() {
            "string" => {
                // Ne garder que la code page demandée (db_string.rdb sinon).
                if (has_cp_strings && *name != wanted_strings) || (!has_cp_strings && name.contains('(')) {
                    continue;
                }
                rdb::parse_strings(data, &dec).map(|s| to_dbc::strings_to_dbc(&s))
            }
            "item.ref" => rdb::parse_item_ref(data, &dec).map(|r| to_dbc::item_ref_to_dbc(&r)),
            t if rdb::known_tables().any(|k| k == t) => rdb::parse_table(t, data, &dec).map(|tbl| to_dbc::table_to_dbc(&tbl)),
            _ => {
                // Table sans schéma connu : conservée brute pour ne rien perdre.
                w.add_file(&format!("DBFilesClient\\Rappelz\\{name}"), data, true)?;
                continue;
            }
        };
        match result {
            Ok(dbc) => {
                let dbc_name = to_dbc::dbc_name(&table);
                w.add_file(&dbc_name, &dbc, true)?;
                tracing::info!("  {name} → {dbc_name} ({} octets)", dbc.len());
                converted += 1;
            }
            Err(e) => {
                tracing::warn!("  {name} : conversion impossible ({e}), copie brute");
                w.add_file(&format!("DBFilesClient\\Rappelz\\{name}"), data, true)?;
            }
        }
    }
    w.finish()?;
    tracing::info!("{converted} tables converties en DBC");
    Ok(())
}

fn write_config(out: &Path, locale: &str, realmlist: &str) -> Result<()> {
    std::fs::write(
        out.join("Data").join(locale).join("realmlist.wtf"),
        format!("set realmlist {realmlist}\r\n"),
    )?;
    let wtf = out.join("WTF");
    std::fs::create_dir_all(&wtf)?;
    let config = wtf.join("Config.wtf");
    if !config.exists() {
        std::fs::write(
            &config,
            format!(
                "SET locale \"{locale}\"\r\nSET realmName \"RappelzCore\"\r\nSET gxResolution \"1280x720\"\r\nSET gxWindow \"1\"\r\nSET accountName \"\"\r\nSET Sound_MusicVolume \"0.5\"\r\n"
            ),
        )?;
    }
    Ok(())
}

fn extract_dbc(data: &Path, locale: &str, out: &Path) -> Result<()> {
    let mut chain = MpqChain::open_client_data(data, locale)
        .with_context(|| format!("ouverture des MPQ de {}", data.display()))?;
    anyhow::ensure!(!chain.archives().is_empty(), "aucun MPQ trouvé dans {}", data.display());
    let dbc_dir = out.join("dbc");
    std::fs::create_dir_all(&dbc_dir)?;
    let mut n = 0;
    for name in chain.list()? {
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("dbfilesclient\\") && lower.ends_with(".dbc") {
            let bytes = chain.read_file(&name)?;
            let file = name.rsplit('\\').next().unwrap_or(&name);
            std::fs::write(dbc_dir.join(file), bytes)?;
            n += 1;
        }
    }
    tracing::info!("{n} fichiers DBC extraits vers {}", dbc_dir.display());
    Ok(())
}
