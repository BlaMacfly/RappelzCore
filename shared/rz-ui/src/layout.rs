//! Mise en page et règles de dessin des contrôles, indépendantes du moteur graphique.
//!
//! [`layout`] transforme un [`NuiDocument`] en une liste ordonnée de [`DrawCmd`] (images
//! découpées/étirées et zones de texte), à dessiner dans l'ordre, par-dessus les précédentes.
//!
//! Règles (voir `docs/re/ui_format.md` pour les preuves) — W×H = taille de l'image 0 :
//! - `pos` au lieu de `rect` : une image (`frame`), à sa taille, en (x, y) ;
//! - générique (`static`, `edit`, `iconstatic`… `FUN_009a9480`/`FUN_009a9050`) :
//!   * 1 image : étirée sur tout le rect ;
//!   * 3 images : gauche W×H, milieu étiré, droite W×H (hauteur H, pas celle du rect) ;
//!     vertical (haut/milieu/bas) si `style & 2` pour un static, toujours pour `v_scroll*` ;
//!   * 9 images : 9-slice, indice = ligne*3 + colonne, coins W×H, bords et centre étirés ;
//!   * autre nombre : rien (le client journalise « Error!!! : Frame »).
//! - `button` (`KUIControlButton`, 12 images) : indice = état*3 + morceau (gauche, milieu
//!   étiré, droite), états 0 normal, 1 survol, 2 appuyé, 3 désactivé ; `style & 2` sans
//!   gauche, `& 4` sans droite, `& 0x10` vertical ; 24 images = 2 jeux de 12 (coché, normal).
//! - `simplebutton` (4 images) : une image par état, étirée sur le rect ;
//! - `check` (4 images) : indice = coché*2 + survol ;
//! - `captionbutton` : fond = `ani` (comme `button`), `caption` = animation (1 image par
//!   état) centrée dans le rect ;
//! - `gauge` (9 images = 3 couches de 3 morceaux) : couche 2 = fond (rect entier), couche 0 =
//!   remplissage (largeur × valeur) ;
//! - `genwnd` : fond `window_tile` (sauf NOBACKGROUND, étiré ou répété), contrôles,
//!   `window_topframe` (sauf NOTOPFRAME), cadre `outframe` 9-slice (sauf NOFRAME) en dernier.

use crate::caption::{parse_caption, Caption};
use crate::flags::{flag, style};
use crate::nui::{ControlKind, NuiControl, NuiDocument, Rect};
use crate::spr::{Animation, Frame, SpriteSet};
use std::collections::{HashMap, HashSet};

/// Rectangle (x, y, largeur, hauteur).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct IRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

/// Dessin d'une image (ou d'une partie) dans un rectangle de l'écran (étirement si les
/// tailles diffèrent).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageCmd {
    /// Nom réel du fichier dans l'archive.
    pub image: String,
    /// Partie de l'image source, en pixels.
    pub src: IRect,
    /// Destination à l'écran.
    pub dst: IRect,
    /// `id` du contrôle d'origine.
    pub control: String,
}

/// Zone de texte : la légende est à rendre dans `rect` selon ses alignements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextCmd {
    pub caption: Caption,
    pub rect: IRect,
    /// `KFLAG_SINGLE_LINE` : une ligne, tronquée avec « ... ».
    pub single_line: bool,
    pub control: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawCmd {
    Image(ImageCmd),
    Text(TextCmd),
}

/// État visuel d'un bouton (indice d'état des animations à 4 états) [D sauf Disabled = 3 O].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum ControlState {
    #[default]
    Normal = 0,
    Hover = 1,
    Pressed = 2,
    Disabled = 3,
}

/// État dynamique de la fenêtre (le code du jeu le pilote).
#[derive(Clone, Debug, Default)]
pub struct UiState {
    /// État par défaut des boutons.
    pub default_state: ControlState,
    /// État par contrôle (`id` en minuscules).
    pub states: HashMap<String, ControlState>,
    /// Contrôles cochés (`check`, `buttoncheck`, onglets…).
    pub checked: HashSet<String>,
    /// Contrôles masqués (le jeu masque p. ex. les surbrillances de sélection).
    pub hidden: HashSet<String>,
    /// Valeur des jauges, 0..=1 (1 par défaut).
    pub gauges: HashMap<String, f32>,
    /// Légende remplaçant celle du `.nui` (texte saisi, nom de personnage…).
    pub captions: HashMap<String, String>,
}

impl UiState {
    fn key(id: &str) -> String {
        id.to_ascii_lowercase()
    }
    pub fn set_state(&mut self, id: &str, s: ControlState) -> &mut Self {
        self.states.insert(Self::key(id), s);
        self
    }
    pub fn hide(&mut self, id: &str) -> &mut Self {
        self.hidden.insert(Self::key(id));
        self
    }
    pub fn check(&mut self, id: &str) -> &mut Self {
        self.checked.insert(Self::key(id));
        self
    }
    pub fn set_gauge(&mut self, id: &str, v: f32) -> &mut Self {
        self.gauges.insert(Self::key(id), v);
        self
    }
    pub fn set_caption(&mut self, id: &str, c: &str) -> &mut Self {
        self.captions.insert(Self::key(id), c.to_string());
        self
    }
    fn state_of(&self, id: &str) -> ControlState {
        self.states.get(&Self::key(id)).copied().unwrap_or(self.default_state)
    }
    fn is_checked(&self, id: &str) -> bool {
        self.checked.contains(&Self::key(id))
    }
    fn is_hidden(&self, id: &str) -> bool {
        self.hidden.contains(&Self::key(id))
    }
}

/// Noms des animations par défaut d'un genwnd (table `0x00e98b7c`, `FUN_00b35070`).
pub mod genwnd_ani {
    pub const BACKGROUND: &str = "window_tile";
    pub const FRAME: &str = "outframe";
    pub const TOPFRAME: &str = "window_topframe";
    pub const TITLEBAR: &str = "window_titlebar01";
    pub const STATUSBAR: &str = "window_statusbar";
    pub const RESIZE: &str = "button_common_down";
}

struct Out<'a> {
    cmds: Vec<DrawCmd>,
    control: &'a str,
}

impl Out<'_> {
    fn image(&mut self, f: &Frame, dst: IRect) {
        self.image_src(f, IRect::new(0, 0, f.w as i32, f.h as i32), dst);
    }
    fn image_src(&mut self, f: &Frame, src: IRect, dst: IRect) {
        let Some(file) = &f.file else { return };
        if dst.w <= 0 || dst.h <= 0 || src.w <= 0 || src.h <= 0 {
            return;
        }
        let src = match f.sub {
            Some((sx, sy, _, _)) => IRect::new(src.x + sx, src.y + sy, src.w, src.h),
            None => src,
        };
        self.cmds.push(DrawCmd::Image(ImageCmd { image: file.clone(), src, dst, control: self.control.to_string() }));
    }
}

fn drawable<'s>(sprites: &'s SpriteSet, name: Option<&str>) -> Option<&'s Animation> {
    sprites.animation(name?).filter(|a| a.is_drawable())
}

fn wh(f: &Frame) -> (i32, i32) {
    (f.w as i32, f.h as i32)
}

/// 1 image étirée, ou répétée si VERTICAL/HORIZONTAL_REPEAT (la dernière tuile est coupée).
fn one(out: &mut Out, f: &Frame, r: IRect, style_bits: u32) {
    let (w, h) = wh(f);
    let vrep = style_bits & style::VERTICAL_REPEAT != 0 && h > 0;
    let hrep = style_bits & style::HORIZONTAL_REPEAT != 0 && w > 0;
    if !vrep && !hrep {
        out.image(f, r);
        return;
    }
    let (tw, th) = (if hrep { w } else { r.w }, if vrep { h } else { r.h });
    let mut y = 0;
    while y < r.h {
        let ch = th.min(r.h - y);
        let mut x = 0;
        while x < r.w {
            let cw = tw.min(r.w - x);
            let sw = if hrep { cw } else { w };
            let sh = if vrep { ch } else { h };
            out.image_src(f, IRect::new(0, 0, sw, sh), IRect::new(r.x + x, r.y + y, cw, ch));
            x += tw;
        }
        y += th;
    }
}

/// 3 morceaux : (début, milieu étiré, fin). `skip_start`/`skip_end` : morceau absent.
fn three(out: &mut Out, f: [&Frame; 3], r: IRect, vertical: bool, skip_start: bool, skip_end: bool) {
    if vertical {
        let hs = if skip_start { 0 } else { f[0].h as i32 };
        let he = if skip_end { 0 } else { f[2].h as i32 };
        if !skip_start {
            out.image(f[0], IRect::new(r.x, r.y, f[0].w as i32, hs));
        }
        out.image(f[1], IRect::new(r.x, r.y + hs, f[1].w as i32, r.h - hs - he));
        if !skip_end {
            out.image(f[2], IRect::new(r.x, r.y + r.h - he, f[2].w as i32, he));
        }
    } else {
        let ws = if skip_start { 0 } else { f[0].w as i32 };
        let we = if skip_end { 0 } else { f[2].w as i32 };
        if !skip_start {
            out.image(f[0], IRect::new(r.x, r.y, ws, f[0].h as i32));
        }
        out.image(f[1], IRect::new(r.x + ws, r.y, r.w - ws - we, f[1].h as i32));
        if !skip_end {
            out.image(f[2], IRect::new(r.x + r.w - we, r.y, we, f[2].h as i32));
        }
    }
}

/// 3 morceaux génériques (`FUN_009a9050`) : tous dimensionnés d'après l'image 0.
fn three_generic(out: &mut Out, a: &Animation, r: IRect, vertical: bool) {
    let (w, h) = wh(&a.frames[0]);
    let f = &a.frames;
    if vertical {
        out.image(&f[0], IRect::new(r.x, r.y, r.w, h));
        out.image(&f[1], IRect::new(r.x, r.y + h, r.w, r.h - 2 * h));
        out.image(&f[2], IRect::new(r.x, r.y + r.h - h, r.w, h));
    } else {
        out.image(&f[0], IRect::new(r.x, r.y, w, h));
        out.image(&f[1], IRect::new(r.x + w, r.y, r.w - 2 * w, h));
        out.image(&f[2], IRect::new(r.x + r.w - w, r.y, w, h));
    }
}

/// 9-slice, coins de la taille de l'image 0.
fn nine(out: &mut Out, f: &[Frame], r: IRect) {
    let (w, h) = wh(&f[0]);
    let xs = [r.x, r.x + w, r.x + r.w - w];
    let ws = [w, r.w - 2 * w, w];
    let ys = [r.y, r.y + h, r.y + r.h - h];
    let hs = [h, r.h - 2 * h, h];
    for row in 0..3 {
        for col in 0..3 {
            out.image(&f[row * 3 + col], IRect::new(xs[col], ys[row], ws[col], hs[row]));
        }
    }
}

/// Découpage générique selon le nombre d'images.
fn generic(out: &mut Out, a: &Animation, r: IRect, vertical: bool, style_bits: u32) {
    match a.frames.len() {
        1 => one(out, &a.frames[0], r, style_bits),
        3 => three_generic(out, a, r, vertical),
        9 => nine(out, &a.frames, r),
        _ => {}
    }
}

/// Bouton à 12 (ou 24) images : état*3 + morceau.
fn button(out: &mut Out, a: &Animation, r: IRect, st: ControlState, checked: bool, style_bits: u32) {
    let n = a.frames.len();
    let base = if n >= 24 && !checked { 12 } else { 0 };
    let s = st as usize;
    let idx = |piece: usize| base + s * 3 + piece;
    if idx(2) >= n {
        return;
    }
    let f = [&a.frames[idx(0)], &a.frames[idx(1)], &a.frames[idx(2)]];
    three(
        out,
        f,
        r,
        style_bits & style::BUTTON_VERTICAL != 0,
        style_bits & style::BUTTON_LEFTSIDE != 0,
        style_bits & style::BUTTON_RIGHTSIDE != 0,
    );
}

/// Barre de défilement composite `fond/curseur/haut/bas` ou `fond/curseur/début/haut/fin/bas` :
/// flèches en haut et en bas à leur taille, curseur en position 0 sous la flèche haute [D].
fn scroll_parts(out: &mut Out, c: &NuiControl, sprites: &SpriteSet, r: IRect) {
    let parts = c.ani_parts();
    let get = |i: usize| parts.get(i).and_then(|n| drawable(sprites, Some(n))).map(|a| &a.frames[0]);
    let (bar, top, bottom): (_, Vec<_>, Vec<_>) = match parts.len() {
        4 => (get(1), vec![get(2)], vec![get(3)]),
        6 => (get(1), vec![get(2), get(3)], vec![get(5), get(4)]),
        _ => return,
    };
    let mut y = r.y;
    for f in top.into_iter().flatten() {
        out.image(f, IRect::new(r.x + (r.w - f.w as i32) / 2, y, f.w as i32, f.h as i32));
        y += f.h as i32;
    }
    let mut yb = r.y + r.h;
    for f in bottom.into_iter().flatten() {
        yb -= f.h as i32;
        out.image(f, IRect::new(r.x + (r.w - f.w as i32) / 2, yb, f.w as i32, f.h as i32));
    }
    if let Some(f) = bar {
        out.image(f, IRect::new(r.x + (r.w - f.w as i32) / 2, y, f.w as i32, f.h as i32));
    }
}

fn state_frame(a: &Animation, i: usize) -> &Frame {
    &a.frames[i.min(a.frames.len() - 1)]
}

fn control_rect(c: &NuiControl, sprites: &SpriteSet, origin: (i32, i32)) -> Option<IRect> {
    if let Some(r) = c.rect {
        return Some(IRect::new(origin.0 + r.x1, origin.1 + r.y1, r.width(), r.height()));
    }
    let (x, y) = c.pos?;
    let a = drawable(sprites, c.main_ani())?;
    let f = a.frames.get(c.frame.max(0) as usize).unwrap_or(&a.frames[0]);
    Some(IRect::new(origin.0 + x, origin.1 + y, f.w as i32, f.h as i32))
}

fn draw_control(out: &mut Out, c: &NuiControl, sprites: &SpriteSet, origin: (i32, i32), st: &UiState) {
    let Some(r) = control_rect(c, sprites, origin) else { return };
    let state = if c.attr("disabled").is_some() { ControlState::Disabled } else { st.state_of(&c.id) };
    let checked = st.is_checked(&c.id);
    let ani = drawable(sprites, c.main_ani());

    // Mode `pos` : une seule image, celle de `frame`.
    if c.rect.is_none() {
        if let Some(a) = ani {
            let f = a.frames.get(c.frame.max(0) as usize).unwrap_or(&a.frames[0]);
            out.image(f, r);
        }
    } else if let Some(a) = ani {
        let n = a.frames.len();
        match c.kind {
            ControlKind::Button | ControlKind::ButtonCheck | ControlKind::CaptionButton | ControlKind::CaptionButtonCheck
                if n == 12 || n == 24 =>
            {
                button(out, a, r, state, checked, c.style)
            }
            ControlKind::SimpleButton | ControlKind::Button if n == 4 => out.image(state_frame(a, state as usize), r),
            ControlKind::Check if n == 4 => {
                let i = (checked as usize) * 2 + (state == ControlState::Hover) as usize;
                out.image(state_frame(a, i), r)
            }
            ControlKind::SimpleButton | ControlKind::Check if n == 1 => out.image(&a.frames[0], r),
            ControlKind::Gauge if n == 9 => {
                let v = st.gauges.get(&c.id.to_ascii_lowercase()).copied().unwrap_or(1.0).clamp(0.0, 1.0);
                let bg = [&a.frames[6], &a.frames[7], &a.frames[8]];
                three(out, bg, r, false, false, false);
                let fw = (r.w as f32 * v).round() as i32;
                if fw > 0 {
                    let fill = [&a.frames[0], &a.frames[1], &a.frames[2]];
                    let min = (fill[0].w + fill[2].w) as i32;
                    three(out, fill, IRect::new(r.x, r.y, fw.max(min.min(r.w)), r.h), false, false, false);
                }
            }
            ControlKind::VScroll | ControlKind::VScrollEx | ControlKind::VScrollSmallEx => {
                generic(out, a, r, true, c.style);
                scroll_parts(out, c, sprites, r);
            }
            ControlKind::Static | ControlKind::IconStatic | ControlKind::MultiIcon => {
                generic(out, a, r, c.style & 0x2 != 0, c.style)
            }
            _ => generic(out, a, r, false, c.style),
        }
    }

    // captionbutton : la légende est une animation, 1 image par état, centrée.
    if matches!(c.kind, ControlKind::CaptionButton | ControlKind::CaptionButtonCheck) {
        if let Some(cap) = drawable(sprites, c.caption.as_deref().map(str::trim)) {
            let f = state_frame(cap, state as usize);
            let x = r.x + ((r.w - f.w as i32) / 2).max(0);
            let y = r.y + ((r.h - f.h as i32) / 2).max(0);
            out.image(f, IRect::new(x, y, f.w as i32, f.h as i32));
        }
        return;
    }

    // Légende texte.
    let text = st.captions.get(&c.id.to_ascii_lowercase()).map(String::as_str).or(c.caption.as_deref());
    if let Some(text) = text {
        let cap = parse_caption(text);
        if !cap.is_empty() {
            out.cmds.push(DrawCmd::Text(TextCmd {
                caption: cap,
                rect: r,
                single_line: c.flags & flag::SINGLE_LINE != 0,
                control: c.id.clone(),
            }));
        }
    }
}

/// Met en page un document `.nui` dont le coin haut-gauche est placé en `origin` (écran).
/// Les animations doivent avoir été résolues ([`SpriteSet::resolve_document`]) : une
/// animation dont une image manque n'est pas dessinée (comme le client).
pub fn layout(doc: &NuiDocument, sprites: &SpriteSet, origin: (i32, i32), state: &UiState) -> Vec<DrawCmd> {
    let mut out = Out { cmds: Vec::new(), control: "" };
    let win = doc.window();
    let wstyle = win.map(|w| w.style).unwrap_or(style::NOBACKGROUND | style::NOFRAME | style::NOTOPFRAME);
    let wrect = win
        .and_then(|w| w.rect)
        .map(|r: Rect| IRect::new(origin.0 + r.x1, origin.1 + r.y1, r.width(), r.height()))
        .unwrap_or_else(|| {
            let (w, h) = doc.size();
            IRect::new(origin.0, origin.1, w, h)
        });
    let wid = win.map(|w| w.id.clone()).unwrap_or_default();

    // Fond de fenêtre (FUN_00b38530).
    if wstyle & style::NOBACKGROUND == 0 {
        if let Some(a) = drawable(sprites, Some(genwnd_ani::BACKGROUND)) {
            out.control = &wid;
            one(&mut out, &a.frames[0], wrect, wstyle);
        }
    }
    // Contrôles, dans l'ordre du fichier.
    for c in &doc.controls {
        if c.kind == ControlKind::GenWnd || state.is_hidden(&c.id) {
            continue;
        }
        let mut o = Out { cmds: std::mem::take(&mut out.cmds), control: &c.id };
        draw_control(&mut o, c, sprites, origin, state);
        out.cmds = o.cmds;
    }
    out.control = &wid;
    // Bandeau haut (sauf NOTOPFRAME), en (x1, y1+2) sur toute la largeur.
    if wstyle & style::NOTOPFRAME == 0 {
        if let Some(a) = drawable(sprites, Some(genwnd_ani::TOPFRAME)) {
            let f = &a.frames[0];
            out.image(f, IRect::new(wrect.x, wrect.y + 2, wrect.w, f.h as i32));
        }
    }
    // Cadre 9-slice en dernier (FUN_00b386d0).
    if wstyle & style::NOFRAME == 0 {
        if let Some(a) = drawable(sprites, Some(genwnd_ani::FRAME)).filter(|a| a.frames.len() == 9) {
            nine(&mut out, &a.frames, wrect);
        }
    }
    out.cmds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nui::parse_nui;

    fn sprites() -> SpriteSet {
        let mut s = SpriteSet::new();
        let mut spr = String::new();
        spr += "nine\r\n9\r\n";
        for i in 1..=9 {
            spr += &format!("n{i}.png\r\n0,0\r\n0\r\n");
        }
        spr += "btn\r\n12\r\n";
        for i in 1..=12 {
            spr += &format!("b{i:02}.png\r\n0,0\r\n0\r\n");
        }
        s.merge_spr(&spr, "t.spr").unwrap();
        let doc = parse_nui("begin static\nani = nine;\nend\nbegin button\nani = btn;\nend\n").unwrap();
        s.resolve_document(&doc, |n| {
            let i: u32 = n[1..n.len() - 4].parse().ok()?;
            let (w, h) = if n.starts_with('n') {
                ([4, 1, 4][(i as usize - 1) % 3], [5, 1, 5][(i as usize - 1) / 3])
            } else {
                ([19, 1, 21][(i as usize - 1) % 3], 30)
            };
            n.ends_with(".png").then(|| (n.to_string(), w, h))
        });
        s
    }

    #[test]
    fn nine_slice_and_button() {
        let s = sprites();
        let doc = parse_nui(
            "begin genwnd\nid = w;\nrect = 0,0,200,100;\nstyle = KSTYLE_NOFRAME;\nend\n\
             begin static\nid = f;\nani = nine;\nrect = 10,10,110,60;\nend\n\
             begin button\nid = ok;\nani = btn;\nrect = 0,60,108,90;\ncaption = \"<$1:OK>\";\nend\n",
        )
        .unwrap();
        let mut st = UiState::default();
        st.set_state("ok", ControlState::Pressed);
        let cmds = layout(&doc, &s, (100, 200), &st);
        let imgs: Vec<&ImageCmd> = cmds.iter().filter_map(|c| if let DrawCmd::Image(i) = c { Some(i) } else { None }).collect();
        assert_eq!(imgs.len(), 12);
        assert_eq!(imgs[0].dst, IRect::new(110, 210, 4, 5));
        assert_eq!(imgs[4].dst, IRect::new(114, 215, 92, 40));
        assert_eq!(imgs[8].dst, IRect::new(206, 255, 4, 5));
        // bouton appuyé : images 7, 8, 9
        assert_eq!(imgs[9].image, "b07.png");
        assert_eq!(imgs[10].dst, IRect::new(119, 260, 108 - 19 - 21, 30));
        assert_eq!(imgs[11].dst, IRect::new(100 + 108 - 21, 260, 21, 30));
        assert!(matches!(cmds.last(), Some(DrawCmd::Text(_))));
    }
}
