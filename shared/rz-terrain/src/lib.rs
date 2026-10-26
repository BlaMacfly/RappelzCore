//! Cartes Rappelz (moteur « nFlavor ») : relief `.nfm`, objets placés, eau `.nfw`,
//! polygones d'attribut `.nfa`, zones `.nfc`/`.nfe`/`.nfp`, lumières `.nfl`,
//! objets de quête `.qpf`, scripts `.nfs` et fichiers de configuration du monde.
//!
//! Format complet, adresses des fonctions du client et niveau de certitude :
//! `docs/re/terrain_format.md`.
//!
//! # Système de coordonnées
//!
//! * Une carte `mXXX_YYY` couvre `segments_per_map × tiles_per_segment × tile_length`
//!   = 64 × 6 × 42 = **16128** unités de côté. `XXX` est l'indice en **X**, `YYY` en **Y**
//!   (`sprintf("m%03d_%03d", x, y)` dans le client) ; l'origine monde de la carte est
//!   `(XXX·16128, YYY·16128)`.
//! * Les coordonnées « locales » de cette crate sont relatives à cette origine :
//!   `x` croît vers l'est (colonnes), `y` vers le nord (lignes), `z` est la hauteur (vers le haut),
//!   dans la même unité que les positions des PNJ.
//! * Le relief est une grille de 384 × 384 sommets espacés de 42 unités ; le sommet `(i, j)`
//!   est en `(i·42, j·42)`. La dernière rangée de triangles d'une carte utilise les sommets 0 de la
//!   carte voisine (voir [`TerrainWorld`]).

mod cfg;
mod nfm;
mod reader;
mod sidecar;
mod world;

pub use cfg::{
    parse_prop_info, parse_seamless_world, prop_info_by_u16, parse_texture_info, MapFile, PropInfo, PropInfoTable, SeamlessWorld,
    TextureInfo, TextureInfoTable,
};
pub use nfm::{
    parse_nfm, GrassColony, Light, MapProperties, NfmHeader, NfmMap, PropPlacement, PropRecord, Rgb,
    SegmentInfo, SpeedGrass, SpeedGrassPoint, TextureLayer, NFM_SIGNATURE,
};
pub use sidecar::{
    parse_nfa, parse_nfc, parse_nfe, parse_nfl, parse_nfp, parse_nfs, parse_nfw, parse_qpf, AttrPolygon, EventArea,
    LocationArea, NflFile, NfpArea, NfsFile, PointLight, PropScript, QuestProp, ScriptArea, ScriptEvent,
    ScriptFunction, WaterArea, NFS_SIGNATURE, QPF_SIGNATURE,
};
pub use world::TerrainWorld;

/// Erreur de lecture d'un fichier de carte.
#[derive(Debug, thiserror::Error)]
pub enum TerrainError {
    #[error("{format} : signature absente")]
    BadSignature { format: &'static str },
    #[error("{format} : version {version} non prise en charge")]
    UnsupportedVersion { format: &'static str, version: u32 },
    #[error("{what} : fin de données à l'octet {offset} ({need} octet(s) demandé(s), {len} disponibles)")]
    Truncated { what: &'static str, offset: usize, need: usize, len: usize },
    #[error("{what} : {consumed} octet(s) lus sur {len}")]
    TrailingBytes { what: &'static str, consumed: usize, len: usize },
    #[error("{0}")]
    Inconsistent(String),
}

pub type Result<T> = std::result::Result<T, TerrainError>;

/// Décode une chaîne du client (coréen EUC-KR/CP949, ASCII sinon), sans le `\0` final.
pub(crate) fn decode_text(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let (s, _, _) = encoding_rs::EUC_KR.decode(&bytes[..end]);
    s.into_owned()
}

#[cfg(test)]
mod tests;
