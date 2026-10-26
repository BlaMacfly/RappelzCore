//! Petit kit d'interface immédiate dans le style des écrans de connexion WoW
//! (cadres sombres, boutons rouges à texte doré), dessiné avec le reste de la scène.

use macroquad::prelude::*;

pub const GOLD: Color = Color::new(1.0, 0.82, 0.35, 1.0);
const PANEL: Color = Color::new(0.03, 0.03, 0.07, 0.82);
const BORDER: Color = Color::new(0.55, 0.45, 0.25, 1.0);
const BUTTON: Color = Color::new(0.35, 0.07, 0.07, 0.95);
const BUTTON_HOVER: Color = Color::new(0.55, 0.12, 0.10, 0.98);

pub struct Gui {
    pub font: Option<Font>,
    clicked: std::cell::Cell<bool>,
    focus: Option<u32>,
    caret_timer: f32,
}

impl Gui {
    pub fn new(font: Option<Font>) -> Self {
        Self { font, clicked: Default::default(), focus: None, caret_timer: 0.0 }
    }

    pub fn begin_frame(&mut self) {
        self.caret_timer = (self.caret_timer + get_frame_time()) % 1.0;
    }

    pub fn text(&self, s: &str, x: f32, y: f32, size: u16, color: Color) {
        draw_text_ex(s, x, y, TextParams { font: self.font.as_ref(), font_size: size, color, ..Default::default() });
    }

    pub fn text_centered(&self, s: &str, cx: f32, y: f32, size: u16, color: Color) {
        let w = measure_text(s, self.font.as_ref(), size, 1.0).width;
        self.text(s, cx - w / 2.0, y, size, color);
    }

    pub fn panel(&self, r: Rect, title: &str) {
        draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, BORDER);
        if !title.is_empty() {
            self.text_centered(title, r.x + r.w / 2.0, r.y + 26.0, 20, GOLD);
        }
    }

    pub fn button(&self, r: Rect, label: &str) -> bool {
        let hover = r.contains(mouse_position().into());
        draw_rectangle(r.x, r.y, r.w, r.h, if hover { BUTTON_HOVER } else { BUTTON });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, BORDER);
        self.text_centered(label, r.x + r.w / 2.0, r.y + r.h / 2.0 + 6.0, 17, GOLD);
        let pressed = hover && is_mouse_button_pressed(MouseButton::Left);
        if pressed {
            self.clicked.set(true);
        }
        pressed
    }

    /// Vrai si un bouton a été cliqué depuis le dernier appel (son de clic).
    pub fn take_clicked(&self) -> bool {
        self.clicked.replace(false)
    }

    /// Ligne sélectionnable d'une liste.
    pub fn list_item(&self, r: Rect, label: &str, selected: bool) -> bool {
        let hover = r.contains(mouse_position().into());
        if selected || hover {
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.6, 0.45, 0.15, if selected { 0.45 } else { 0.2 }));
        }
        self.text(label, r.x + 8.0, r.y + r.h / 2.0 + 6.0, 17, if selected { GOLD } else { WHITE });
        hover && is_mouse_button_pressed(MouseButton::Left)
    }

    /// Champ de saisie ; Tab passe au champ suivant (géré par l'appelant via `focus_next`).
    pub fn text_field(&mut self, id: u32, r: Rect, value: &mut String, password: bool, max_len: usize) {
        if is_mouse_button_pressed(MouseButton::Left) && r.contains(mouse_position().into()) {
            self.focus = Some(id);
        }
        let focused = self.focus == Some(id);
        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.75));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, if focused { GOLD } else { BORDER });
        if focused {
            while let Some(c) = get_char_pressed() {
                if !c.is_control() && value.chars().count() < max_len {
                    value.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                value.pop();
            }
        }
        let shown: String = if password { "*".repeat(value.chars().count()) } else { value.clone() };
        let caret = if focused && self.caret_timer < 0.5 { "|" } else { "" };
        self.text(&format!("{shown}{caret}"), r.x + 8.0, r.y + r.h / 2.0 + 6.0, 18, WHITE);
    }

    pub fn focus(&mut self, id: u32) {
        self.focus = Some(id);
        while get_char_pressed().is_some() {}
    }

    pub fn focused(&self) -> Option<u32> {
        self.focus
    }

    /// Sélecteur « < valeur > ».
    pub fn selector(&self, r: Rect, options: &[&str], index: &mut usize) {
        if options.is_empty() {
            return;
        }
        let bw = r.h;
        if self.button(Rect::new(r.x, r.y, bw, r.h), "<") {
            *index = (*index + options.len() - 1) % options.len();
        }
        if self.button(Rect::new(r.x + r.w - bw, r.y, bw, r.h), ">") {
            *index = (*index + 1) % options.len();
        }
        draw_rectangle(r.x + bw, r.y, r.w - 2.0 * bw, r.h, Color::new(0.0, 0.0, 0.0, 0.6));
        self.text_centered(options[(*index).min(options.len() - 1)], r.x + r.w / 2.0, r.y + r.h / 2.0 + 6.0, 17, WHITE);
    }
}
