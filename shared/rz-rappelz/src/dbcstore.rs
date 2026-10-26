//! Lecture par nom de champ des DBC produits par `rz-extractor` (équivalent du
//! `DBCStorage` d'AzerothCore). Les noms viennent des schémas RDB : la colonne i du DBC
//! est le champ i du schéma de la table.

use std::collections::HashMap;
use std::path::Path;

use rz_dbc::{DbcFile, Record};

use crate::rdb::{self, Layout};
use crate::to_dbc;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{path} : {source}")]
    Dbc { path: String, source: rz_dbc::DbcError },
    #[error(transparent)]
    Rdb(#[from] rdb::RdbError),
    #[error("{0} : nombre de colonnes différent du schéma ({1} au lieu de {2})")]
    FieldCount(String, u32, usize),
}

pub struct NamedDbc {
    pub file: DbcFile,
    pub layout: Layout,
    index: HashMap<&'static str, usize>,
    by_id: HashMap<i32, usize>,
}

impl NamedDbc {
    pub fn from_file(table: &str, file: DbcFile) -> Result<Self, StoreError> {
        let layout = rdb::layout(table)?;
        if file.field_count as usize != layout.fields.len() {
            return Err(StoreError::FieldCount(table.to_string(), file.field_count, layout.fields.len()));
        }
        let index = layout.fields.iter().enumerate().map(|(i, f)| (f.name, i)).collect();
        let by_id = (0..file.len()).map(|r| (file.record(r).i32(0), r)).collect();
        Ok(Self { file, layout, index, by_id })
    }

    /// Ouvre `<dir>/<Nom>.dbc` pour une table Rappelz (`item` → `Item.dbc`).
    pub fn open(dir: &Path, table: &str) -> Result<Self, StoreError> {
        let name = to_dbc::dbc_name(table);
        let path = dir.join(name.trim_start_matches("DBFilesClient\\"));
        let file = DbcFile::open(&path).map_err(|source| StoreError::Dbc { path: path.display().to_string(), source })?;
        Self::from_file(table, file)
    }

    pub fn len(&self) -> usize {
        self.file.len()
    }

    pub fn is_empty(&self) -> bool {
        self.file.is_empty()
    }

    pub fn row(&self, r: usize) -> Row<'_> {
        Row { rec: self.file.record(r), store: self }
    }

    pub fn rows(&self) -> impl Iterator<Item = Row<'_>> {
        (0..self.len()).map(|r| self.row(r))
    }

    /// Ligne dont la première colonne (id) vaut `id`.
    pub fn by_id(&self, id: i32) -> Option<Row<'_>> {
        self.by_id.get(&id).map(|&r| self.row(r))
    }

    pub fn has_field(&self, field: &str) -> bool {
        self.index.contains_key(field)
    }
}

#[derive(Clone, Copy)]
pub struct Row<'a> {
    rec: Record<'a>,
    store: &'a NamedDbc,
}

impl<'a> Row<'a> {
    fn idx(&self, field: &str) -> usize {
        *self
            .store
            .index
            .get(field)
            .unwrap_or_else(|| panic!("champ inconnu « {field} » dans {}", self.store.layout.table))
    }

    pub fn int(&self, field: &str) -> i32 {
        self.rec.i32(self.idx(field))
    }

    pub fn float(&self, field: &str) -> f32 {
        self.rec.f32(self.idx(field))
    }

    pub fn str(&self, field: &str) -> &'a str {
        self.rec.str(self.idx(field))
    }

    pub fn id(&self) -> i32 {
        self.rec.i32(0)
    }
}

/// `String.dbc` : code → texte (format `niss`).
pub struct StringTable {
    map: HashMap<i32, String>,
}

impl StringTable {
    pub fn open(dir: &Path) -> Result<Self, StoreError> {
        let path = dir.join("String.dbc");
        let file = DbcFile::open(&path).map_err(|source| StoreError::Dbc { path: path.display().to_string(), source })?;
        let map = file.iter().map(|r| (r.i32(0), r.str(3).to_string())).collect();
        Ok(Self { map })
    }

    pub fn empty() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn get(&self, code: i32) -> &str {
        self.map.get(&code).map(String::as_str).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl StringTable {
    /// Depuis le contenu de `String.dbc` (lu dans un MPQ par exemple).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StoreError> {
        let file = DbcFile::parse(bytes).map_err(|source| StoreError::Dbc { path: "String.dbc".into(), source })?;
        let map = file.iter().map(|r| (r.i32(0), r.str(3).to_string())).collect();
        Ok(Self { map })
    }
}

impl NamedDbc {
    /// Depuis le contenu d'un DBC (lu dans un MPQ par exemple).
    pub fn from_bytes(table: &str, bytes: &[u8]) -> Result<Self, StoreError> {
        let file = DbcFile::parse(bytes).map_err(|source| StoreError::Dbc { path: to_dbc::dbc_name(table), source })?;
        Self::from_file(table, file)
    }
}
