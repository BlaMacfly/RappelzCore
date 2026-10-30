//! Native renderer and manager for the shared Lua runtime.
use std::path::Path;

use macroquad::prelude::*;
use rz_addon::{AddonRuntime, WidgetKind};

use crate::gui::{Gui, GOLD};

pub struct Addons {
    pub runtime: AddonRuntime,
    pub manager: bool,
    pub mouse_captured: bool,
    pub error: Option<String>,
    scroll: usize,
}

impl Addons {
    pub fn new(root: &Path) -> Self {
        let mut runtime = AddonRuntime::new(root);
        let error = runtime.load().err().map(|e| format!("{e:#}"));
        Self {
            runtime,
            manager: false,
            mouse_captured: false,
            error,
            scroll: 0,
        }
    }

    pub fn reload(&mut self) {
        self.error = self.runtime.reload().err().map(|e| format!("{e:#}"));
    }

    pub fn begin_frame(&mut self, in_world: bool) {
        if is_key_pressed(KeyCode::F9) {
            self.manager = !self.manager;
        }
        let button = Rect::new(screen_width() / 2.0 - 65.0, 6.0, 130.0, 28.0);
        let mouse: Vec2 = mouse_position().into();
        if button.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            self.manager = !self.manager;
        }
        if self.manager || !in_world {
            self.runtime
                .pointer(screen_width(), screen_height(), -1.0, -1.0, false);
            self.mouse_captured = self.manager || button.contains(mouse);
        } else {
            self.mouse_captured = self.runtime.pointer(
                screen_width(),
                screen_height(),
                mouse.x,
                mouse.y,
                is_mouse_button_pressed(MouseButton::Left),
            ) || button.contains(mouse);
        }
    }

    pub fn draw(&mut self, gui: &Gui, in_world: bool) {
        if in_world {
            for w in self.runtime.draw_widgets(screen_width(), screen_height()) {
                let r = Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height);
                let c = w.widget.color;
                let color = Color::new(c[0], c[1], c[2], c[3]);
                match w.widget.kind {
                    WidgetKind::Frame | WidgetKind::GameTooltip => gui.panel(r, ""),
                    WidgetKind::Button => {
                        let _ = gui.button(r, &fit(&w.widget.text, r.w));
                    }
                    WidgetKind::Texture => draw_rectangle(r.x, r.y, r.w, r.h, color),
                    WidgetKind::StatusBar => {
                        draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.05, 0.05, 0.05, 0.9));
                        let fraction = ((w.widget.value - w.widget.min)
                            / (w.widget.max - w.widget.min).max(f32::EPSILON))
                        .clamp(0.0, 1.0);
                        draw_rectangle(r.x, r.y, r.w * fraction, r.h, color);
                    }
                    WidgetKind::FontString => {}
                }
                if w.widget.kind != WidgetKind::Button {
                    let size = (15.0 * w.scale).clamp(8.0, 40.0) as u16;
                    let line_height = size as f32 + 2.0;
                    for (line, text) in w
                        .widget
                        .text
                        .lines()
                        .take((r.h / line_height).max(0.0) as usize)
                        .enumerate()
                    {
                        gui.text(
                            &fit(text, r.w / w.scale),
                            r.x + 2.0,
                            r.y + line_height * (line + 1) as f32,
                            size,
                            color,
                        );
                    }
                }
            }
        }
        let toggle = Rect::new(screen_width() / 2.0 - 65.0, 6.0, 130.0, 28.0);
        let _ = gui.button(toggle, "AddOns (F9)"); // Input handled before world input.
        if !self.manager {
            return;
        }
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.62),
        );
        let r = Rect::new(
            (screen_width() - 780.0) / 2.0,
            (screen_height() - 540.0) / 2.0,
            780.0,
            540.0,
        );
        gui.panel(r, "AddOns - Lua 5.1");
        gui.text(
            "Activation appliquee au rechargement. Molette : parcourir la liste.",
            r.x + 18.0,
            r.y + 53.0,
            16,
            LIGHTGRAY,
        );
        let max_scroll = self.runtime.addons().len().saturating_sub(6);
        let (_, wheel) = mouse_wheel();
        if r.contains(mouse_position().into()) {
            if wheel > 0.0 {
                self.scroll = self.scroll.saturating_sub(1);
            }
            if wheel < 0.0 {
                self.scroll = (self.scroll + 1).min(max_scroll);
            }
        }
        self.scroll = self.scroll.min(max_scroll);
        let infos: Vec<_> = self
            .runtime
            .addons()
            .iter()
            .skip(self.scroll)
            .take(6)
            .cloned()
            .collect();
        for (row, info) in infos.iter().enumerate() {
            let y = r.y + 68.0 + row as f32 * 48.0;
            let title = format!(
                "{}  {}",
                info.manifest.title(),
                info.manifest
                    .metadata
                    .get("version")
                    .map(String::as_str)
                    .unwrap_or("")
            );
            gui.text(&fit(&title, 420.0), r.x + 18.0, y + 17.0, 17, GOLD);
            let details = info.error.as_deref().unwrap_or(&info.status);
            gui.text(
                &fit(details, 535.0),
                r.x + 18.0,
                y + 36.0,
                13,
                if info.error.is_some() {
                    PINK
                } else {
                    LIGHTGRAY
                },
            );
            if gui.button(
                Rect::new(r.x + 560.0, y, 100.0, 30.0),
                if info.enabled { "Active" } else { "Desactive" },
            ) {
                self.error = self
                    .runtime
                    .set_enabled(&info.manifest.name, !info.enabled)
                    .err()
                    .map(|e| format!("{e:#}"));
            }
            if info.status == "On demand"
                && gui.button(Rect::new(r.x + 668.0, y, 94.0, 30.0), "Charger")
            {
                self.error = self
                    .runtime
                    .request_load(&info.manifest.name)
                    .err()
                    .map(|e| format!("{e:#}"));
            }
        }
        if infos.is_empty() {
            gui.text(
                "Aucun addon dans Interface/AddOns.",
                r.x + 18.0,
                r.y + 110.0,
                18,
                WHITE,
            );
        }
        let errors: Vec<_> = self
            .error
            .iter()
            .map(String::as_str)
            .chain(self.runtime.errors().iter().rev().map(String::as_str))
            .take(4)
            .collect();
        for (i, error) in errors.iter().enumerate() {
            gui.text(
                &fit(error, r.w - 36.0),
                r.x + 18.0,
                r.y + 390.0 + i as f32 * 18.0,
                13,
                PINK,
            );
        }
        if gui.button(
            Rect::new(r.x + 18.0, r.y + r.h - 54.0, 260.0, 36.0),
            "Recharger l'interface (/reload)",
        ) {
            self.reload();
        }
        if gui.button(
            Rect::new(r.x + r.w - 158.0, r.y + r.h - 54.0, 140.0, 36.0),
            "Fermer",
        ) {
            self.manager = false;
        }
    }
}

fn fit(text: &str, width: f32) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take((width / 7.5).max(0.0) as usize)
        .collect()
}
