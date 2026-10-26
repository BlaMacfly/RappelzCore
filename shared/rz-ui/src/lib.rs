//! Interface du client Rappelz : sprites `.spr`, fenêtres `.nui`, légendes à balises et
//! règles de dessin des contrôles (moteur `KUI*` de `SFrame.exe`).
//!
//! Voir `docs/re/ui_format.md` pour les formats, les adresses des fonctions du client et le
//! niveau de preuve de chaque règle.
//!
//! Utilisation typique :
//! ```no_run
//! use rz_ui::{parse_nui, SpriteSet, layout, UiState};
//! # fn read(name: &str) -> Option<Vec<u8>> { None }
//! let mut sprites = SpriteSet::load_ui_set(|n| read(n), "ui_frame.spr");
//! let doc = parse_nui(&rz_ui::decode_text(&read("window_lobby_login.nui").unwrap())).unwrap();
//! sprites.resolve_document(&doc, |img| None /* (nom réel, largeur, hauteur) */);
//! let cmds = layout(&doc, &sprites, (100, 100), &UiState::default());
//! ```

pub mod caption;
pub mod flags;
pub mod kbin;
pub mod layout;
pub mod nui;
pub mod spr;

pub use caption::{parse_caption, Caption, CaptionText, HAlign, TextRun, TextStyle, VAlign};
pub use layout::{layout, ControlState, DrawCmd, ImageCmd, IRect, TextCmd, UiState};
pub use nui::{parse_nui, ControlKind, NuiControl, NuiDocument, Rect};
pub use spr::{image_candidates, parse_sdb, parse_spr, parse_spr_bytes, Animation, Frame, ResolveStats, SpriteSet};

/// Erreurs de lecture des fichiers d'interface.
#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error(".nui ligne {line} : {msg}")]
    Nui { line: usize, msg: String },
    #[error(".spr ligne {line} : {msg}")]
    Spr { line: usize, msg: String },
}

/// Décode un fichier texte du client. Les `.nui` contiennent des commentaires et des
/// légendes de secours en coréen codé CP949 (EUC-KR) : on décode donc en EUC-KR, qui laisse
/// l'ASCII intact.
pub fn decode_text(bytes: &[u8]) -> String {
    let (s, _, _) = encoding_rs::EUC_KR.decode(bytes);
    s.into_owned()
}
