//! Interface d'origine de Rappelz : fenêtres `.nui` et sprites `.spr` (crate `rz-ui`) dessinées
//! avec macroquad. Les positions à l'écran sont choisies par le code du client (comme dans le
//! jeu) ; les clics et survols sont rapportés par identifiant de contrôle.

use std::collections::{HashMap, HashSet};

use macroquad::prelude::*;
use rz_ui::{layout, ControlState, DrawCmd, NuiDocument, SpriteSet, TextRun, UiState};

use crate::assets::Assets;

/// Zones cliquables d'une fenêtre dessinée (repère écran).
#[derive(Default)]
pub struct Frame {
    pub controls: Vec<(String, Rect)>,
    pub bounds: Rect,
}

impl Frame {
    pub fn rect(&self, id: &str) -> Option<Rect> {
        self.controls.iter().find(|(c, _)| c.eq_ignore_ascii_case(id)).map(|(_, r)| *r)
    }

    /// Identifiant du contrôle sous la souris (le dernier dessiné l'emporte).
    pub fn hovered(&self) -> Option<&str> {
        let m: Vec2 = mouse_position().into();
        self.controls.iter().rev().find(|(id, r)| !id.is_empty() && r.contains(m)).map(|(id, _)| id.as_str())
    }

    pub fn clicked(&self, id: &str) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && self.rect(id).is_some_and(|r| r.contains(mouse_position().into()))
    }

    pub fn contains_mouse(&self) -> bool {
        self.bounds.contains(mouse_position().into())
    }
}

pub struct RzUi {
    sprites: SpriteSet,
    docs: HashMap<String, Option<NuiDocument>>,
    resolved: HashSet<String>,
    font: Option<Font>,
}

impl RzUi {
    pub fn new(assets: &mut Assets, font: Option<Font>) -> Self {
        let sprites = SpriteSet::load_ui_set(|name| assets.read(name), "ui_frame.spr");
        tracing::info!("interface : {} animations de sprites", sprites.len());
        Self { sprites, docs: HashMap::new(), resolved: HashSet::new(), font }
    }

    fn doc(&mut self, assets: &mut Assets, name: &str) -> Option<&NuiDocument> {
        let key = name.to_ascii_lowercase();
        if !self.docs.contains_key(&key) {
            let doc = assets.read(&key).and_then(|b| rz_ui::parse_nui(&rz_ui::decode_text(&b)).ok());
            if let Some(d) = &doc {
                if self.resolved.insert(key.clone()) {
                    self.sprites.resolve_document(d, |img| {
                        let bytes = assets.read(img)?;
                        let t = rz_texture::decode(img, &bytes).ok()?;
                        Some((img.to_string(), t.width, t.height))
                    });
                }
            }
            self.docs.insert(key.clone(), doc);
        }
        self.docs.get(&key).and_then(|d| d.as_ref())
    }

    /// Taille de la fenêtre (rectangle du `genwnd`).
    pub fn size(&mut self, assets: &mut Assets, name: &str) -> Vec2 {
        self.doc(assets, name)
            .and_then(|d| d.window())
            .and_then(|w| w.rect)
            .map(|r| vec2(r.width() as f32, r.height() as f32))
            .unwrap_or(vec2(300.0, 200.0))
    }

    /// Masque les contrôles dont le rectangle vérifie `pred` (ex. rangées de raccourcis en trop).
    pub fn hide_where(&mut self, assets: &mut Assets, name: &str, state: &mut UiState, pred: impl Fn(&rz_ui::Rect) -> bool) {
        let Some(doc) = self.doc(assets, name) else { return };
        let ids: Vec<String> = doc
            .controls
            .iter()
            .filter(|c| c.kind != rz_ui::ControlKind::GenWnd && c.rect.as_ref().is_some_and(&pred))
            .map(|c| c.id.clone())
            .collect();
        for id in ids {
            state.hide(&id);
        }
    }

    /// Dessine une fenêtre à `origin` ; les boutons survolés/appuyés changent d'état.
    pub fn draw(&mut self, assets: &mut Assets, name: &str, origin: Vec2, state: &UiState) -> Frame {
        let Some(doc) = self.doc(assets, name).cloned() else { return Frame::default() };
        // Premier passage pour connaître les zones, second avec l'état de survol.
        let o = (origin.x as i32, origin.y as i32);
        let probe = layout(&doc, &self.sprites, o, state);
        let mut frame = Frame::default();
        let mut bounds: Option<Rect> = None;
        for cmd in &probe {
            let (id, r) = match cmd {
                DrawCmd::Image(i) => (&i.control, i.dst),
                DrawCmd::Text(t) => (&t.control, t.rect),
            };
            let rect = Rect::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32);
            bounds = Some(bounds.map_or(rect, |b| b.combine_with(rect)));
            match frame.controls.iter_mut().find(|(c, _)| c == id) {
                Some((_, existing)) => *existing = existing.combine_with(rect),
                None => frame.controls.push((id.clone(), rect)),
            }
        }
        frame.bounds = bounds.unwrap_or_default();
        let mut st = state.clone();
        if let Some(h) = frame.hovered().map(str::to_string) {
            if !st.states.contains_key(&h.to_ascii_lowercase()) {
                let s = if is_mouse_button_down(MouseButton::Left) { ControlState::Pressed } else { ControlState::Hover };
                st.set_state(&h, s);
            }
        }
        for cmd in layout(&doc, &self.sprites, o, &st) {
            match cmd {
                DrawCmd::Image(i) => {
                    if let Some(tex) = assets.texture(&i.image) {
                        draw_texture_ex(
                            &tex,
                            i.dst.x as f32,
                            i.dst.y as f32,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(i.dst.w as f32, i.dst.h as f32)),
                                source: Some(Rect::new(i.src.x as f32, i.src.y as f32, i.src.w as f32, i.src.h as f32)),
                                ..Default::default()
                            },
                        );
                    }
                }
                DrawCmd::Text(t) => {
                    let runs = t.caption.resolve(|id| {
                        let s = assets.strings.get(id as i32);
                        (!s.is_empty()).then(|| s.to_string())
                    });
                    let rect = Rect::new(t.rect.x as f32, t.rect.y as f32, t.rect.w as f32, t.rect.h as f32);
                    self.draw_runs(&runs, rect, t.caption.halign, t.caption.valign);
                }
            }
        }
        frame
    }

    /// Texte d'une légende dans un rectangle, selon ses alignements.
    pub fn draw_runs(&self, runs: &[TextRun], rect: Rect, halign: rz_ui::HAlign, valign: rz_ui::VAlign) {
        let lines: Vec<Vec<&TextRun>> = runs.split(|r| r.text == "\n").map(|l| l.iter().collect()).collect();
        let line_h = 15.0;
        let total_h = lines.len() as f32 * line_h;
        let mut y = match valign {
            rz_ui::VAlign::Top => rect.y + 12.0,
            rz_ui::VAlign::Bottom => rect.y + rect.h - total_h + 11.0,
            _ => rect.y + (rect.h - total_h) / 2.0 + 11.0,
        };
        for line in lines {
            let width: f32 = line.iter().map(|r| self.measure(&r.text, Self::size_of(r))).sum();
            let mut x = match halign {
                rz_ui::HAlign::Left => rect.x + 2.0,
                rz_ui::HAlign::Right => rect.x + rect.w - width - 2.0,
                _ => rect.x + (rect.w - width) / 2.0,
            };
            for r in line {
                let size = Self::size_of(r);
                let c = r.style.color;
                let color = Color::from_rgba((c >> 16) as u8, (c >> 8) as u8, c as u8, 255);
                self.text_shadow(&r.text, x, y, size, color);
                x += self.measure(&r.text, size);
            }
            y += line_h;
        }
    }

    fn size_of(r: &TextRun) -> u16 {
        (r.style.size as u16 + 3).clamp(10, 28)
    }

    pub fn measure(&self, s: &str, size: u16) -> f32 {
        measure_text(s, self.font.as_ref(), size, 1.0).width
    }

    pub fn text_shadow(&self, s: &str, x: f32, y: f32, size: u16, color: Color) {
        let p = |c: Color| TextParams { font: self.font.as_ref(), font_size: size, color: c, ..Default::default() };
        draw_text_ex(s, x + 1.0, y + 1.0, p(Color::from_rgba(0, 0, 0, 200)));
        draw_text_ex(s, x, y, p(color));
    }

    /// Texte simple (taille 13) avec ombre, aligné à gauche.
    pub fn label(&self, s: &str, x: f32, y: f32, color: Color) {
        self.text_shadow(s, x, y, 13, color);
    }

    /// Texte centré dans un rectangle.
    pub fn label_in(&self, s: &str, r: Rect, color: Color) {
        let w = self.measure(s, 13);
        self.text_shadow(s, r.x + (r.w - w) / 2.0, r.y + r.h / 2.0 + 5.0, 13, color);
    }
}
