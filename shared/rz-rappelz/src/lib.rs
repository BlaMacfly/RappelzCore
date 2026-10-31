//! Formats du client Rappelz d'origine, reconstruits par rétro-ingénierie de `SFrame.exe`
//! (voir `docs/re/`). Sert à convertir le client vers le format RappelzRust (MPQ/DBC).

pub mod archive;
pub mod dbcstore;
pub mod layout;
pub mod progression;
pub mod rdb;
pub mod skills;
pub mod restoration;
mod rdb_schemas;
pub mod to_dbc;

pub use archive::{DataArchive, Entry};
