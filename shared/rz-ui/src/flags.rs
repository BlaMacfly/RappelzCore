//! Constantes `KSTYLE_*`, `KFLAG_*`, `KANCHOR_*` des `.nui`.
//!
//! Valeurs lues dans `FUN_00b3ab30` (initialisation des tables `DAT_00edd564` style,
//! `DAT_00edd56c` flag, `DAT_00edd574` anchor). Les styles « par contrôle » (BUTTON_*, CHECK_*,
//! SB_*, TAB_*, NUMBER_*, GAUGE_*, MSG_*) réutilisent les bits bas : leur sens dépend du type
//! du contrôle.

/// Styles (`style = A | B | …;`).
pub mod style {
    pub const NOCLOSE: u32 = 0x2;
    pub const NOMINIMIZE: u32 = 0x4;
    pub const NORESIZE: u32 = 0x8;
    pub const NOTITLE: u32 = 0x10;
    pub const NOSTATUSBAR: u32 = 0x20;
    pub const VSCROLL: u32 = 0x40;
    pub const HSCROLL: u32 = 0x80;
    pub const NOFRAME: u32 = 0x100;
    pub const NOTOPFRAME: u32 = 0x200;
    pub const NOBACKGROUND: u32 = 0x400;
    pub const RESIZE_LEFT: u32 = 0x800;
    pub const RESIZE_TOP: u32 = 0x1000;
    pub const MOVE_BY_TITLEBAR: u32 = 0x2000;
    pub const MOVE_BY_TOPFRAME: u32 = 0x4000;
    pub const MOVE_BY_ALL: u32 = 0x8000;
    pub const MOVE_BY_CUSTOM: u32 = 0x10000;
    pub const NO_CLIPPING: u32 = 0x20000;
    pub const VERTICAL_REPEAT: u32 = 0x40000;
    pub const HORIZONTAL_REPEAT: u32 = 0x80000;
    pub const SPR_CAPTION: u32 = 0x100000;
    pub const ALPHA_SHOW: u32 = 0x200000;
    pub const BUTTON_LEFTSIDE: u32 = 0x2;
    pub const BUTTON_RIGHTSIDE: u32 = 0x4;
    pub const BUTTON_CANDRAG: u32 = 0x8;
    pub const BUTTON_VERTICAL: u32 = 0x10;
    pub const CHECK_CAPTION_RIGHT: u32 = 0x2;
    pub const SB_NOLEFTSIDE: u32 = 0x2;
    pub const SB_NORIGHTSIDE: u32 = 0x4;
    pub const TAB_VERTICAL: u32 = 0x2;
    pub const NUMBER_LEFT: u32 = 0x2;
    pub const NUMBER_CENTER: u32 = 0x4;
    pub const NUMBER_RIGHT: u32 = 0x8;
    pub const GAUGE_WITH_GRADUATION: u32 = 0x2;
    pub const MSG_VERTICAL_REPEAT: u32 = 0x1;
    pub const MSG_HORIZONTAL_REPEAT: u32 = 0x2;
}

/// Drapeaux (`flag = …;`).
pub mod flag {
    pub const NO_GET_MESSAGE: u32 = 0x2;
    pub const GET_PASS_MESSAGE: u32 = 0x4;
    pub const CAN_DRAG: u32 = 0x8;
    pub const SINGLE_LINE: u32 = 0x10;
    pub const NO_GET_FOCUS: u32 = 0x20;
}

/// Ancrages (`anchor = …;`).
pub mod anchor {
    pub const LEFT: u32 = 0x2;
    pub const RIGHT: u32 = 0x4;
    pub const TOP: u32 = 0x8;
    pub const BOTTOM: u32 = 0x10;
}

const STYLES: &[(&str, u32)] = &[
    ("KSTYLE_NOCLOSE", style::NOCLOSE),
    ("KSTYLE_NOMINIMIZE", style::NOMINIMIZE),
    ("KSTYLE_NORESIZE", style::NORESIZE),
    ("KSTYLE_NOTITLE", style::NOTITLE),
    ("KSTYLE_NOSTATUSBAR", style::NOSTATUSBAR),
    ("KSTYLE_VSCROLL", style::VSCROLL),
    ("KSTYLE_HSCROLL", style::HSCROLL),
    ("KSTYLE_NOFRAME", style::NOFRAME),
    ("KSTYLE_NOTOPFRAME", style::NOTOPFRAME),
    ("KSTYLE_NOBACKGROUND", style::NOBACKGROUND),
    ("KSTYLE_RESIZE_LEFT", style::RESIZE_LEFT),
    ("KSTYLE_RESIZE_TOP", style::RESIZE_TOP),
    ("KSTYLE_MOVE_BY_TITLEBAR", style::MOVE_BY_TITLEBAR),
    ("KSTYLE_MOVE_BY_TOPFRAME", style::MOVE_BY_TOPFRAME),
    ("KSTYLE_MOVE_BY_ALL", style::MOVE_BY_ALL),
    ("KSTYLE_MOVE_BY_CUSTOM", style::MOVE_BY_CUSTOM),
    ("KSTYLE_NO_CLIPPING", style::NO_CLIPPING),
    ("KSTYLE_VERTICAL_REPEAT", style::VERTICAL_REPEAT),
    ("KSTYLE_HORIZONTAL_REPEAT", style::HORIZONTAL_REPEAT),
    ("KSTYLE_SPR_CAPTION", style::SPR_CAPTION),
    ("KSTYLE_ALPHA_SHOW", style::ALPHA_SHOW),
    ("KSTYLE_BUTTON_LEFTSIDE", style::BUTTON_LEFTSIDE),
    ("KSTYLE_BUTTON_RIGHTSIDE", style::BUTTON_RIGHTSIDE),
    ("KSTYLE_BUTTON_CANDRAG", style::BUTTON_CANDRAG),
    ("KSTYLE_BUTTON_VERTICAL", style::BUTTON_VERTICAL),
    ("KSTYLE_CHECK_CAPTION_RIGHT", style::CHECK_CAPTION_RIGHT),
    ("KSTYLE_SB_NOLEFTSIDE", style::SB_NOLEFTSIDE),
    ("KSTYLE_SB_NORIGHTSIDE", style::SB_NORIGHTSIDE),
    ("KSTYLE_TAB_VERTICAL", style::TAB_VERTICAL),
    ("KSTYLE_NUMBER_LEFT", style::NUMBER_LEFT),
    ("KSTYLE_NUMBER_CENTER", style::NUMBER_CENTER),
    ("KSTYLE_NUMBER_RIGHT", style::NUMBER_RIGHT),
    ("KSTYLE_GAUGE_WITH_GRADUATION", style::GAUGE_WITH_GRADUATION),
    ("KSTYLE_MSG_VERTICAL_REPEAT", style::MSG_VERTICAL_REPEAT),
    ("KSTYLE_MSG_HORIZONTAL_REPEAT", style::MSG_HORIZONTAL_REPEAT),
];

const FLAGS: &[(&str, u32)] = &[
    ("KFLAG_NO_GET_MESSAGE", flag::NO_GET_MESSAGE),
    ("KFLAG_GET_PASS_MESSAGE", flag::GET_PASS_MESSAGE),
    ("KFLAG_CAN_DRAG", flag::CAN_DRAG),
    ("KFLAG_SINGLE_LINE", flag::SINGLE_LINE),
    ("KFLAG_NO_GET_FOCUS", flag::NO_GET_FOCUS),
];

const ANCHORS: &[(&str, u32)] = &[
    ("KANCHOR_LEFT", anchor::LEFT),
    ("KANCHOR_RIGHT", anchor::RIGHT),
    ("KANCHOR_TOP", anchor::TOP),
    ("KANCHOR_BOTTOM", anchor::BOTTOM),
];

/// Quelle table de noms utiliser pour une clé `.nui`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitTable {
    Style,
    Flag,
    Anchor,
}

fn table(t: BitTable) -> &'static [(&'static str, u32)] {
    match t {
        BitTable::Style => STYLES,
        BitTable::Flag => FLAGS,
        BitTable::Anchor => ANCHORS,
    }
}

/// Valeur d'un nom (`KSTYLE_NOCLOSE` → 2), insensible à la casse.
pub fn bit_value(t: BitTable, name: &str) -> Option<u32> {
    table(t).iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|&(_, v)| v)
}

/// Évalue `A | B | 0x10 |` : renvoie la valeur et les noms inconnus. Un nombre décimal ou
/// hexadécimal est accepté.
pub fn parse_bits(t: BitTable, expr: &str) -> (u32, Vec<String>) {
    let mut v = 0;
    let mut unknown = Vec::new();
    for part in expr.split('|').map(str::trim).filter(|p| !p.is_empty()) {
        if let Some(b) = bit_value(t, part) {
            v |= b;
        } else if let Some(n) = parse_int(part) {
            v |= n as u32;
        } else {
            unknown.push(part.to_string());
        }
    }
    (v, unknown)
}

/// Noms des bits posés (pour l'affichage), d'après la table donnée.
pub fn bit_names(t: BitTable, v: u32) -> Vec<&'static str> {
    table(t).iter().filter(|(_, b)| v & b != 0).map(|&(n, _)| n).collect()
}

pub(crate) fn parse_int(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits() {
        let (v, u) = parse_bits(BitTable::Style, "KSTYLE_NOCLOSE | KSTYLE_NOTITLE | KSTYLE_ALPHA_SHOW |");
        assert_eq!(v, 0x2 | 0x10 | 0x200000);
        assert!(u.is_empty());
        let (v, u) = parse_bits(BitTable::Flag, "KFLAG_NO_GET_MESSAGE ; FOO");
        assert_eq!(v, 0);
        assert_eq!(u.len(), 1);
        assert_eq!(bit_value(BitTable::Anchor, "kanchor_bottom"), Some(0x10));
    }
}
