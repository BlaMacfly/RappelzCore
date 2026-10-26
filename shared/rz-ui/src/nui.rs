//! Fenêtres `.nui` (parseur client `FUN_00b39f10`, valeurs `FUN_00b39c00`).
//!
//! ```text
//! begin <type>          // 1er bloc : genwnd (la fenêtre), puis ses contrôles
//! id = window_login;
//! spr = ui_frame.spr;
//! ani = static_common_outbox_blue;
//! rect = 32,30,340,146; // x1,y1,x2,y2 ; x2/y2 EXCLUS (largeur = x2-x1)
//! pos = 11,11;          // à la place de rect : taille = taille de l'image `frame`
//! caption = "<#c2b0af><size:11><$6074:...>";
//! style = KSTYLE_NOCLOSE | KSTYLE_NOTITLE |;
//! flag = KFLAG_NO_GET_MESSAGE |;
//! end
//! ```
//! Les lignes commençant par `//` sont des commentaires. Coordonnées relatives au coin
//! haut-gauche de la fenêtre (rect du genwnd = `0,0,largeur,hauteur`).

use crate::flags::{parse_bits, parse_int, BitTable};
use crate::UiError;

/// Rectangle `x1,y1,x2,y2` ; `x2`, `y2` exclus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.x2 - self.x1
    }
    pub fn height(&self) -> i32 {
        self.y2 - self.y1
    }
}

/// Types de contrôles (nom après `begin`, table de fabrique `FUN_00b3ec90`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlKind {
    GenWnd,
    Static,
    IconStatic,
    MultiIcon,
    Button,
    SimpleButton,
    CaptionButton,
    CaptionButtonCheck,
    ButtonCheck,
    Check,
    Edit,
    Gauge,
    ChaosGauge,
    Number,
    VScroll,
    VScrollEx,
    VScrollSmallEx,
    HSlider,
    ComboBox,
    SimpleTab,
    TabSimpleHead,
    TabSheet,
    TabHead,
    DetailTab,
    List,
    StatusBar,
    ColorBox,
    QuickEffect,
    QuestInfoMgr,
    Static3d,
    QjtvStatic,
    DynamicTextureStatic,
    MinimapStatic,
    /// Type non répertorié ici (nom brut dans `NuiControl::type_name`).
    Other,
}

impl ControlKind {
    /// Barres de défilement (`v_scroll`, `v_scrollEx`, `v_scrollSmallEx`).
    pub fn is_scroll(&self) -> bool {
        matches!(self, Self::VScroll | Self::VScrollEx | Self::VScrollSmallEx)
    }

    pub fn from_name(n: &str) -> Self {
        match n.to_ascii_lowercase().as_str() {
            "genwnd" => Self::GenWnd,
            "static" => Self::Static,
            "iconstatic" => Self::IconStatic,
            "multi_icon" => Self::MultiIcon,
            "button" => Self::Button,
            "simplebutton" => Self::SimpleButton,
            "captionbutton" => Self::CaptionButton,
            "captionbuttoncheck" => Self::CaptionButtonCheck,
            "buttoncheck" => Self::ButtonCheck,
            "check" => Self::Check,
            "edit" => Self::Edit,
            "gauge" => Self::Gauge,
            "chaos_gauge" => Self::ChaosGauge,
            "number" => Self::Number,
            "v_scroll" => Self::VScroll,
            "v_scrollex" => Self::VScrollEx,
            "v_scrollsmallex" => Self::VScrollSmallEx,
            "h_slider" => Self::HSlider,
            "combo_box" => Self::ComboBox,
            "simple_tab" => Self::SimpleTab,
            "tab_simple_head" => Self::TabSimpleHead,
            "tab_sheet" => Self::TabSheet,
            "tab_head" => Self::TabHead,
            "detail_tab" => Self::DetailTab,
            "statusbar" => Self::StatusBar,
            "colorbox" => Self::ColorBox,
            "quick_effect" => Self::QuickEffect,
            "quest_info_mgr" => Self::QuestInfoMgr,
            "3dstatic" => Self::Static3d,
            "qjtv_static" => Self::QjtvStatic,
            "dynamictexture_static" => Self::DynamicTextureStatic,
            "minimap_static" => Self::MinimapStatic,
            s if s.starts_with("list^") => Self::List,
            _ => Self::Other,
        }
    }
}

/// Un bloc `begin … end`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NuiControl {
    pub kind: ControlKind,
    /// Nom du type tel qu'écrit (`static`, `list^item_string^h`…).
    pub type_name: String,
    pub id: String,
    pub rect: Option<Rect>,
    /// `pos = x,y` (le client met alors x2 = y2 = -1 et prend la taille de l'image).
    pub pos: Option<(i32, i32)>,
    /// Animation (`None` si absente ou vide : `ani = ;`).
    pub ani: Option<String>,
    pub spr: Option<String>,
    /// Légende, sans les guillemets (balises non interprétées, voir [`crate::caption`]).
    pub caption: Option<String>,
    /// Indice d'image pour le mode `pos` (clé `frame`, 0 par défaut).
    pub frame: i32,
    pub style: u32,
    pub flags: u32,
    pub anchor: u32,
    pub info: Option<String>,
    /// Noms de bits non reconnus dans style/flag/anchor.
    pub unknown_bits: Vec<String>,
    /// Attributs non interprétés (clé, valeur brute).
    pub attrs: Vec<(String, String)>,
    /// Ligne du `begin` (1-based).
    pub line: usize,
}

impl NuiControl {
    fn new(type_name: &str, line: usize) -> Self {
        Self {
            kind: ControlKind::from_name(type_name),
            type_name: type_name.to_string(),
            id: String::new(),
            rect: None,
            pos: None,
            ani: None,
            spr: None,
            caption: None,
            frame: 0,
            style: 0,
            flags: 0,
            anchor: 0,
            info: None,
            unknown_bits: Vec::new(),
            attrs: Vec::new(),
            line,
        }
    }

    /// Parties de `ani`. Les contrôles composites listent plusieurs animations séparées par
    /// `/` : barres de défilement `fond/curseur/haut/bas` (ou `fond/curseur/début/haut/fin/bas`),
    /// onglets `bouton/xxx<#couleur>`. Les balises `<…>` sont retirées.
    pub fn ani_parts(&self) -> Vec<&str> {
        let Some(a) = self.ani.as_deref() else { return Vec::new() };
        a.split('/')
            .map(|p| p.split('<').next().unwrap_or("").trim())
            .filter(|p| !p.is_empty())
            .collect()
    }

    /// Animation principale (1re partie de `ani`).
    pub fn main_ani(&self) -> Option<&str> {
        self.ani_parts().into_iter().next()
    }

    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    }
}

/// Un fichier `.nui` : la fenêtre (1er bloc `genwnd`) et ses contrôles, dans l'ordre du
/// fichier (= ordre de création et de dessin, `FUN_00b3a4f0`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NuiDocument {
    pub controls: Vec<NuiControl>,
}

impl NuiDocument {
    /// Le bloc `genwnd` (normalement le premier).
    pub fn window(&self) -> Option<&NuiControl> {
        self.controls.iter().find(|c| c.kind == ControlKind::GenWnd)
    }

    pub fn find(&self, id: &str) -> Option<&NuiControl> {
        self.controls.iter().find(|c| c.id.eq_ignore_ascii_case(id))
    }

    /// Taille de la fenêtre (rect du genwnd), sinon boîte englobante des contrôles.
    pub fn size(&self) -> (i32, i32) {
        if let Some(r) = self.window().and_then(|w| w.rect) {
            return (r.width(), r.height());
        }
        let (mut w, mut h) = (0, 0);
        for r in self.controls.iter().filter_map(|c| c.rect) {
            w = w.max(r.x2);
            h = h.max(r.y2);
        }
        (w, h)
    }

    /// Noms d'animations référencés (`ani`, et `caption` des `captionbutton`), sans doublon.
    pub fn animation_refs(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for c in &self.controls {
            let mut push = |s: &str| {
                if !s.is_empty() && !v.iter().any(|x| x.eq_ignore_ascii_case(s)) {
                    v.push(s.to_string());
                }
            };
            // Seules les barres de défilement listent plusieurs animations ; pour les onglets,
            // la 2e partie (`tab_inventory<#a0a0a0>`) n'est pas une animation [I].
            let parts = c.ani_parts();
            let n = if c.kind.is_scroll() { parts.len() } else { parts.len().min(1) };
            for a in &parts[..n] {
                push(a);
            }
            if matches!(c.kind, ControlKind::CaptionButton | ControlKind::CaptionButtonCheck) {
                if let Some(cap) = &c.caption {
                    if !cap.contains('<') {
                        push(cap.trim());
                    }
                }
            }
        }
        v
    }
}

fn ints(v: &str) -> Option<Vec<i32>> {
    v.split(',').map(|p| parse_int(p).map(|n| n as i32)).collect()
}

/// Lit un `.nui`. Tolérant comme le client : un attribut hors bloc ou un bloc non fermé en
/// fin de fichier n'est pas une erreur ; un `begin` imbriqué ou une valeur numérique
/// illisible en est une.
pub fn parse_nui(text: &str) -> Result<NuiDocument, UiError> {
    let mut doc = NuiDocument::default();
    let mut cur: Option<NuiControl> = None;
    for (n, raw) in text.split('\n').enumerate() {
        let line = n + 1;
        let l = raw.trim_end_matches('\r').trim();
        if l.is_empty() || l.starts_with("//") {
            continue;
        }
        let err = |msg: String| UiError::Nui { line, msg };
        let (word, rest) = match l.find(|c: char| c.is_whitespace() || c == '=') {
            Some(p) => (&l[..p], l[p..].trim_start()),
            None => (l, ""),
        };
        if word.eq_ignore_ascii_case("begin") {
            if let Some(c) = cur.take() {
                return Err(err(format!("« begin » imbriqué (bloc « {} » ligne {} non fermé)", c.id, c.line)));
            }
            let ty = rest.split_whitespace().next().unwrap_or("");
            if ty.is_empty() {
                return Err(err("type manquant après « begin »".into()));
            }
            cur = Some(NuiControl::new(ty, line));
            continue;
        }
        if word.eq_ignore_ascii_case("end") {
            if let Some(c) = cur.take() {
                doc.controls.push(c);
            }
            continue;
        }
        let Some(value) = rest.strip_prefix('=') else {
            return Err(err(format!("ligne non reconnue : « {l} »")));
        };
        let value = value.trim();
        let key = word.to_ascii_lowercase();
        // Valeur : chaîne entre guillemets (peut contenir « ; »), sinon jusqu'au « ; ».
        let value = if key == "caption" || value.starts_with('"') {
            match (value.find('"'), value.rfind('"')) {
                (Some(a), Some(b)) if b > a => value[a + 1..b].to_string(),
                _ => value.trim_end_matches(';').trim().to_string(),
            }
        } else {
            value.split(';').next().unwrap_or("").trim().to_string()
        };
        let Some(c) = cur.as_mut() else {
            // Attribut hors bloc : ignoré (le client n'a rien à quoi l'appliquer).
            continue;
        };
        match key.as_str() {
            "id" => c.id = value,
            "ani" => c.ani = (!value.is_empty()).then_some(value),
            "spr" => c.spr = (!value.is_empty()).then_some(value),
            "caption" => c.caption = Some(value),
            "info" => c.info = Some(value),
            "frame" => c.frame = parse_int(&value).ok_or_else(|| err(format!("frame invalide « {value} »")))? as i32,
            "rect" => match ints(&value).as_deref() {
                Some(&[x1, y1, x2, y2]) => c.rect = Some(Rect { x1, y1, x2, y2 }),
                _ => return Err(err(format!("rect invalide « {value} »"))),
            },
            "pos" => match ints(&value).as_deref() {
                Some(&[x, y]) => c.pos = Some((x, y)),
                _ => return Err(err(format!("pos invalide « {value} »"))),
            },
            "style" | "flag" | "anchor" => {
                let t = match key.as_str() {
                    "style" => BitTable::Style,
                    "flag" => BitTable::Flag,
                    _ => BitTable::Anchor,
                };
                let (v, unknown) = parse_bits(t, &value);
                match t {
                    BitTable::Style => c.style |= v,
                    BitTable::Flag => c.flags |= v,
                    BitTable::Anchor => c.anchor |= v,
                }
                c.unknown_bits.extend(unknown);
            }
            _ => c.attrs.push((key, value)),
        }
    }
    if let Some(c) = cur.take() {
        doc.controls.push(c);
    }
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_login() {
        let t = "begin genwnd\r\nid = window_login;\r\nspr = ui_frame.spr;\r\nrect = 0,0,376,184;\r\nstyle = KSTYLE_NOCLOSE | KSTYLE_NOTITLE |;\r\nend\r\n\r\n// commentaire\r\nbegin edit \r\nid = accountedit;\r\nani = ;\r\ncaption = \"<#ffffff><$6074:a;b>\";\r\nrect = 147,45,308,65;\r\nflag = KFLAG_NO_GET_MESSAGE |;\r\nend\r\nbegin static\r\nid = s;\r\npos = 11,11;\r\nfoo = bar;\r\nend\r\n";
        let d = parse_nui(t).unwrap();
        assert_eq!(d.controls.len(), 3);
        assert_eq!(d.size(), (376, 184));
        assert_eq!(d.window().unwrap().style, 0x12);
        let e = d.find("accountedit").unwrap();
        assert_eq!(e.kind, ControlKind::Edit);
        assert_eq!(e.ani, None);
        assert_eq!(e.caption.as_deref(), Some("<#ffffff><$6074:a;b>"));
        assert_eq!(e.rect.unwrap().width(), 161);
        assert_eq!(e.flags, 2);
        let s = d.find("s").unwrap();
        assert_eq!(s.pos, Some((11, 11)));
        assert_eq!(s.attr("foo"), Some("bar"));
    }

    #[test]
    fn nested_begin_is_error() {
        assert!(parse_nui("begin static\nid = a;\nbegin static\n").is_err());
    }
}
