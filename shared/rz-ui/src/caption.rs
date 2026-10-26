//! Légendes à balises des `.nui` (parseur client `FUN_00b2d640`, table de balises
//! `0x00d891a0..0x00d8926c`, comparaison sans casse).
//!
//! Exemple : `<font:font_default><#c2b0af><size:11><glow><vcenter><$6074:아이디>`.
//! - `<#rrggbb>` couleur du texte, `<@rrggbb>` couleur du contour/halo [D] ;
//! - `<size:N>`, `<font:font_default|font_01|font_02|font_03>` (polices de `localeinfo.ini`) ;
//! - `<left> <right> <hcenter> <center>` / `<top> <bottom> <vcenter>` : alignement dans le rect ;
//! - `<b>…</b>`, `<u>`, `<strike>`, `<out>` (contour), `<glow>`, `<2xglow>`, `<shadow>`, `<shd>` ;
//! - `<offsety:N>`, `<br>` / `<p>` (saut de ligne) ;
//! - `<$id:texte>` : chaîne n° `id` de la table de chaînes (`db_string`), `texte` = texte de
//!   secours (coréen) si l'id n'existe pas.

/// Alignement horizontal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Alignement vertical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum VAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

/// Style courant d'un morceau de texte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextStyle {
    /// Couleur 0xRRGGBB.
    pub color: u32,
    /// Couleur de contour/halo (`<@rrggbb>`).
    pub outline_color: Option<u32>,
    /// Taille (`<size:N>`), en pixels [D].
    pub size: u32,
    /// Police logique (`font_default`, `font_01`…) ; vide = défaut.
    pub font: String,
    pub bold: bool,
    pub underline: bool,
    pub strike: bool,
    pub outline: bool,
    pub glow: bool,
    pub shadow: bool,
    pub offset_y: i32,
}

/// Taille et couleur par défaut quand la légende n'en donne pas [D].
pub const DEFAULT_SIZE: u32 = 10;
pub const DEFAULT_COLOR: u32 = 0xFFFFFF;

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: DEFAULT_COLOR,
            outline_color: None,
            size: DEFAULT_SIZE,
            font: String::new(),
            bold: false,
            underline: false,
            strike: false,
            outline: false,
            glow: false,
            shadow: false,
            offset_y: 0,
        }
    }
}

/// Contenu d'un morceau.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptionText {
    Literal(String),
    /// `<$id:secours>`.
    StringRef { id: u32, fallback: String },
    LineBreak,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptionRun {
    pub text: CaptionText,
    pub style: TextStyle,
}

/// Légende analysée.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Caption {
    pub runs: Vec<CaptionRun>,
    pub halign: HAlign,
    pub valign: VAlign,
    /// Balises non reconnues (pour le diagnostic).
    pub unknown_tags: Vec<String>,
}

/// Morceau de texte résolu (chaînes `$id` remplacées).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextRun {
    /// Texte ; `"\n"` pour un saut de ligne.
    pub text: String,
    pub style: TextStyle,
}

fn hex_color(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.len() == 6 || s.len() == 8 {
        u32::from_str_radix(&s[s.len() - 6..], 16).ok()
    } else {
        None
    }
}

/// Analyse une légende. Les balises inconnues sont ignorées (et listées).
pub fn parse_caption(s: &str) -> Caption {
    let mut cap = Caption::default();
    let mut style = TextStyle::default();
    parse_into(s, &mut style, &mut cap);
    cap
}

fn push_literal(cap: &mut Caption, style: &TextStyle, t: &str) {
    if t.is_empty() {
        return;
    }
    if let Some(CaptionRun { text: CaptionText::Literal(prev), style: ps }) = cap.runs.last_mut() {
        if ps == style {
            prev.push_str(t);
            return;
        }
    }
    cap.runs.push(CaptionRun { text: CaptionText::Literal(t.to_string()), style: style.clone() });
}

fn parse_into(s: &str, style: &mut TextStyle, cap: &mut Caption) {
    let mut rest = s;
    while !rest.is_empty() {
        let Some(open) = rest.find('<') else {
            push_literal(cap, style, rest);
            break;
        };
        push_literal(cap, style, &rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('>') else {
            push_literal(cap, style, &rest[open..]);
            break;
        };
        let tag = &after[..close];
        rest = &after[close + 1..];
        apply_tag(tag, style, cap);
    }
}

fn apply_tag(tag: &str, style: &mut TextStyle, cap: &mut Caption) {
    let t = tag.trim();
    if let Some(c) = t.strip_prefix('#') {
        match hex_color(c) {
            Some(v) => style.color = v,
            None => cap.unknown_tags.push(tag.to_string()),
        }
        return;
    }
    if let Some(c) = t.strip_prefix('@') {
        match hex_color(c) {
            Some(v) => style.outline_color = Some(v),
            None => cap.unknown_tags.push(tag.to_string()),
        }
        return;
    }
    if let Some(r) = t.strip_prefix('$') {
        let (id, fallback) = match r.split_once(':') {
            Some((a, b)) => (a, b),
            None => (r, ""),
        };
        match id.trim().parse::<u32>() {
            Ok(id) => cap.runs.push(CaptionRun {
                text: CaptionText::StringRef { id, fallback: fallback.to_string() },
                style: style.clone(),
            }),
            Err(_) => cap.unknown_tags.push(tag.to_string()),
        }
        return;
    }
    let (name, arg) = match t.split_once(':') {
        Some((a, b)) => (a.trim().to_ascii_uppercase(), Some(b.trim())),
        None => (t.to_ascii_uppercase(), None),
    };
    match (name.as_str(), arg) {
        ("SIZE", Some(a)) => match a.parse() {
            Ok(v) => style.size = v,
            Err(_) => cap.unknown_tags.push(tag.to_string()),
        },
        ("FONT", Some(a)) => style.font = a.to_string(),
        ("OFFSETY", Some(a)) | ("OFFSET", Some(a)) => style.offset_y = a.parse().unwrap_or(0),
        ("LEFT", None) | ("LERT", None) => cap.halign = HAlign::Left,
        ("RIGHT", None) => cap.halign = HAlign::Right,
        ("HCENTER", None) => cap.halign = HAlign::Center,
        ("CENTER", None) => {
            cap.halign = HAlign::Center;
        }
        ("TOP", None) => cap.valign = VAlign::Top,
        ("BOTTOM", None) => cap.valign = VAlign::Bottom,
        ("VCENTER", None) => cap.valign = VAlign::Center,
        ("B", None) | ("BOLD", None) => style.bold = true,
        ("/B", None) | ("/BOLD", None) => style.bold = false,
        ("U", None) => style.underline = true,
        ("/U", None) => style.underline = false,
        ("STRIKE", None) => style.strike = true,
        ("/STRIKE", None) => style.strike = false,
        ("OUT", None) | ("OUTLINE", None) => style.outline = true,
        ("/OUT", None) | ("/OUTLINE", None) => style.outline = false,
        ("GLOW", None) | ("2XGLOW", None) => style.glow = true,
        ("/GLOW", None) | ("/2XGLOW", None) => style.glow = false,
        ("SHADOW", None) | ("SHD", None) => style.shadow = true,
        ("/SHADOW", None) | ("/SHD", None) => style.shadow = false,
        ("BR", None) | ("P", None) => cap.runs.push(CaptionRun { text: CaptionText::LineBreak, style: style.clone() }),
        ("%%", None) => push_literal(cap, style, "%"),
        ("INV", None) | ("/INV", None) | ("CARET", None) | ("/CARET", None) | ("WB", None) | ("^", None) => {}
        _ => cap.unknown_tags.push(tag.to_string()),
    }
}

impl Caption {
    pub fn is_empty(&self) -> bool {
        self.runs.iter().all(|r| matches!(&r.text, CaptionText::Literal(s) if s.trim().is_empty()))
    }

    /// Remplace les `<$id>` par `lookup(id)` (ou le texte de secours) ; une chaîne trouvée
    /// peut elle-même contenir des balises de style, interprétées avec le style courant.
    pub fn resolve(&self, lookup: impl Fn(u32) -> Option<String>) -> Vec<TextRun> {
        let mut out: Vec<TextRun> = Vec::new();
        for r in &self.runs {
            match &r.text {
                CaptionText::Literal(s) => out.push(TextRun { text: s.clone(), style: r.style.clone() }),
                CaptionText::LineBreak => out.push(TextRun { text: "\n".into(), style: r.style.clone() }),
                CaptionText::StringRef { id, fallback } => {
                    let s = lookup(*id).unwrap_or_else(|| fallback.clone());
                    if s.contains('<') {
                        let mut sub = Caption::default();
                        let mut st = r.style.clone();
                        parse_into(&s, &mut st, &mut sub);
                        for sr in sub.runs {
                            match sr.text {
                                CaptionText::Literal(t) => out.push(TextRun { text: t, style: sr.style }),
                                CaptionText::LineBreak => out.push(TextRun { text: "\n".into(), style: sr.style }),
                                CaptionText::StringRef { fallback, .. } => out.push(TextRun { text: fallback, style: sr.style }),
                            }
                        }
                    } else {
                        out.push(TextRun { text: s, style: r.style.clone() });
                    }
                }
            }
        }
        out
    }

    /// Texte brut (chaînes `$id` remplacées par leur texte de secours).
    pub fn plain_text(&self) -> String {
        self.resolve(|_| None).into_iter().map(|r| r.text).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_caption() {
        let c = parse_caption("<#c2b0af><size:11><glow><vcenter><$6074:ID>");
        assert_eq!(c.valign, VAlign::Center);
        assert_eq!(c.runs.len(), 1);
        assert_eq!(c.runs[0].style.color, 0xc2b0af);
        assert_eq!(c.runs[0].style.size, 11);
        assert!(c.runs[0].style.glow);
        assert_eq!(c.runs[0].text, CaptionText::StringRef { id: 6074, fallback: "ID".into() });
        let r = c.resolve(|id| (id == 6074).then(|| "Account".to_string()));
        assert_eq!(r[0].text, "Account");
        assert!(c.unknown_tags.is_empty());
    }

    #[test]
    fn mixed() {
        let c = parse_caption("<font:font_01><left><#ffffff><size:9><vcenter>00000<br><B>x</B>y");
        assert_eq!(c.halign, HAlign::Left);
        assert_eq!(c.plain_text(), "00000\nxy");
        assert!(c.runs.iter().any(|r| r.style.bold));
    }
}
