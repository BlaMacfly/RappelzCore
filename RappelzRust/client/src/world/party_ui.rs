use super::*;

impl WorldState {
    pub(super) fn draw_party(&mut self, ui: &RzUi, out: &mut Vec<NetCmd>) -> bool {
        let Some(state) = self.party.clone() else {
            return false;
        };
        let rows = state.members.len();
        let height = 90.0 + rows as f32 * 48.0 + if state.invite.is_some() { 62.0 } else { 0.0 };
        let r = Rect::new(
            (screen_width() - 318.0).max(8.0),
            (screen_height() - height - 86.0).clamp(8.0, 230.0),
            310.0,
            height,
        );
        draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(12, 18, 30, 235));
        draw_rectangle_lines(
            r.x,
            r.y,
            r.w,
            r.h,
            1.0,
            Color::from_rgba(100, 140, 180, 255),
        );
        ui.label(
            "Groupe (P) — discussion : /p message",
            r.x + 10.0,
            r.y + 20.0,
            SKYBLUE,
        );
        let mouse: Vec2 = mouse_position().into();
        let button = |label: &str, rect: Rect, action: &str, out: &mut Vec<NetCmd>| {
            let hover = rect.contains(mouse);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if hover {
                    Color::from_rgba(70, 100, 140, 255)
                } else {
                    Color::from_rgba(35, 50, 75, 255)
                },
            );
            ui.label_in(label, rect, WHITE);
            if hover && is_mouse_button_pressed(MouseButton::Left) {
                out.push(Self::party_action(action));
            }
        };
        if let Some(invite) = &state.invite {
            ui.label(
                &format!("{} vous invite ({} s)", invite.name, invite.seconds_left),
                r.x + 10.0,
                r.y + 44.0,
                WHITE,
            );
            button(
                "Accepter",
                Rect::new(r.x + 10.0, r.y + 54.0, 135.0, 25.0),
                "accept",
                out,
            );
            button(
                "Refuser",
                Rect::new(r.x + 160.0, r.y + 54.0, 135.0, 25.0),
                "decline",
                out,
            );
        } else if rows == 0 {
            ui.label(
                "Aucun groupe. Sélectionnez un joueur.",
                r.x + 10.0,
                r.y + 44.0,
                LIGHTGRAY,
            );
        }
        for (i, m) in state.members.iter().enumerate() {
            let y = r.y + 30.0 + i as f32 * 48.0;
            ui.label(
                &format!(
                    "{}{} · niv. {} · couche {}",
                    if m.handle == state.leader {
                        "Chef "
                    } else {
                        ""
                    },
                    m.name,
                    m.level,
                    m.layer
                ),
                r.x + 10.0,
                y + 14.0,
                WHITE,
            );
            ui.label(
                &format!("PV {}/{}   PM {}/{}", m.hp, m.max_hp, m.mp, m.max_mp),
                r.x + 10.0,
                y + 33.0,
                if m.hp == 0 { PINK } else { GREEN },
            );
            if Rect::new(r.x + 10.0, y, r.w - 20.0, 18.0).contains(mouse)
                && is_mouse_button_pressed(MouseButton::Left)
            {
                if let Some(command) = self.select_target(m.handle) {
                    out.push(command);
                }
            }
            if state.leader == self.handle && m.handle != self.handle {
                button(
                    "Chef",
                    Rect::new(r.x + 212.0, y + 18.0, 40.0, 20.0),
                    &format!("leader {}", m.name),
                    out,
                );
                button(
                    "X",
                    Rect::new(r.x + 258.0, y + 18.0, 40.0, 20.0),
                    &format!("kick {}", m.name),
                    out,
                );
            }
        }
        let bottom = r.y + r.h - 42.0;
        if state.invite.is_none() {
            if let Some(target) = self
                .target
                .and_then(|h| self.entities.get(&h))
                .filter(|e| e.kind == Kind::Player)
            {
                let name = target.name.strip_prefix("<GM>").unwrap_or(&target.name);
                button(
                    "Inviter",
                    Rect::new(r.x + 10.0, bottom, 94.0, 25.0),
                    &format!("invite {name}"),
                    out,
                );
            }
            if rows != 0 {
                if state.leader == self.handle {
                    button(
                        "Dissoudre",
                        Rect::new(r.x + 110.0, bottom, 94.0, 25.0),
                        "disband",
                        out,
                    );
                }
                button(
                    "Quitter",
                    Rect::new(r.x + 210.0, bottom, 94.0, 25.0),
                    "leave",
                    out,
                );
            }
        }
        r.contains(mouse)
    }
}
