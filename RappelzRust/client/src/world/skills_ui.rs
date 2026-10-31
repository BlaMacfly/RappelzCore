use super::*;
use rz_proto::skills::{catalog, reason};
fn explanation(code: u8) -> &'static str {
    match code {
        reason::READY => "Disponible",
        reason::JP => "JP insuffisants",
        reason::PREREQUISITE => "Compétence préalable requise",
        reason::JOB_LEVEL => "Niveau de métier insuffisant",
        reason::PLAYER_LEVEL => "Niveau du personnage insuffisant",
        reason::MAXIMUM => "Niveau maximal atteint",
        reason::WRONG_JOB => "Hors de votre arbre actuel",
        reason::UNSUPPORTED => "Règle non prise en charge",
        _ => "Données indisponibles",
    }
}
impl WorldState {
    pub(super) fn draw_skills(
        &mut self,
        assets: &mut Assets,
        ui: &RzUi,
        out: &mut Vec<NetCmd>,
    ) -> bool {
        let r = Rect::new(
            (screen_width() - 490.0).max(8.0),
            100.0,
            480.0,
            (screen_height() - 190.0).clamp(300.0, 530.0),
        );
        draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(12, 18, 30, 245));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, SKYBLUE);
        ui.label(
            "Compétences (K)",
            r.x + 12.0,
            r.y + 22.0,
            SKYBLUE,
        );
        let mouse: Vec2 = mouse_position().into();
        let close = Rect::new(r.x + r.w - 35.0, r.y + 5.0, 28.0, 25.0);
        ui.label_in("X", close, WHITE);
        if close.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            self.show_skills = false;
        }
        let job=self.job_progress.clone();
        if let Some(job)=job {self.draw_job_progression(&job,r,ui,out);}
        let Some(state) = self.skill_catalog.as_ref() else {
            ui.label(
                "Informations d'apprentissage indisponibles.",
                r.x + 12.0,
                r.y + 55.0,
                LIGHTGRAY,
            );
            return r.contains(mouse);
        };
        ui.label(
            &format!(
                "{} · métier niv. {} · JP : {}",
                assets.job_name(state.job),
                state.job_level,
                state.jp
            ),
            r.x + 12.0,
            r.y + 47.0,
            WHITE,
        );
        if state.status != catalog::READY {
            ui.label(
                "Apprentissage indisponible pour ce personnage.",
                r.x + 12.0,
                r.y + 75.0,
                PINK,
            );
            return r.contains(mouse);
        }
        let per_page = ((r.h - 225.0) / 62.0).floor().max(1.0) as usize;
        let pages = state.entries.len().div_ceil(per_page).max(1);
        self.skill_page = self.skill_page.min(pages - 1);
        for (i, entry) in state
            .entries
            .iter()
            .skip(self.skill_page * per_page)
            .take(per_page)
            .enumerate()
        {
            let y = r.y + 152.0 + i as f32 * 62.0;
            if let Some(icon) = assets.skill_icon(entry.id) {
                draw_texture_ex(
                    &icon,
                    r.x + 10.0,
                    y + 2.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(38.0, 38.0)),
                        ..Default::default()
                    },
                );
            }
            let name = assets.skill_name(entry.id);
            let title: String = name.chars().take(32).collect();
            ui.label(
                &format!("{title} · niv. {}/{}", entry.learned, entry.maximum),
                r.x + 55.0,
                y + 15.0,
                WHITE,
            );
            ui.label(
                &format!(
                    "{} · coût : {} JP",
                    if entry.passive { "Passif" } else { "Actif" },
                    entry
                        .cost
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "?".into())
                ),
                r.x + 55.0,
                y + 33.0,
                LIGHTGRAY,
            );
            let detail = match entry.reason {
                reason::PREREQUISITE => entry
                    .prerequisites
                    .iter()
                    .find(|(id, lv)| {
                        *id != 0
                            && self
                                .learned_skills
                                .as_ref()
                                .and_then(|s| s.get(id))
                                .copied()
                                .unwrap_or(0)
                                < *lv
                    })
                    .map(|(id, lv)| {
                        format!(
                            "Requis : {} niv. {lv}",
                            assets.skill_name(*id).chars().take(28).collect::<String>()
                        )
                    })
                    .unwrap_or_else(|| explanation(entry.reason).into()),
                reason::JOB_LEVEL => format!("Métier niv. {} requis", entry.job_level),
                reason::PLAYER_LEVEL => format!("Personnage niv. {} requis", entry.player_level),
                _ => explanation(entry.reason).into(),
            };
            let detail=assets.restoration_quote(entry.id,entry.learned).map_or(detail,|q|
                format!("Lancer : {} PM · {:.2} s · recharge {:.0} s",q.mp,f64::from(q.cast)/100.,f64::from(q.cooldown)/100.));
            ui.label(
                &detail,
                r.x + 55.0,
                y + 50.0,
                if entry.reason == reason::READY {
                    GREEN
                } else {
                    YELLOW
                },
            );
            let button = Rect::new(r.x + r.w - 97.0, y + 21.0, 85.0, 27.0);
            if let Some(q)=assets.restoration_quote(entry.id,entry.learned) {
                let cast=Rect::new(button.x,y-3.0,button.w,22.0);
                let remaining=self.skill_remaining(entry.id);
                let command=self.cast_skill(&q);
                draw_rectangle(cast.x,cast.y,cast.w,cast.h,Color::from_rgba(35,60,85,255));
                let label=if remaining>0. {format!("{remaining:.1} s")}else{"Lancer".into()};
                ui.label_in(&label,cast,if command.is_some(){SKYBLUE}else{GRAY});
                if cast.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
                    if let Some(command)=command {out.push(command);}
                }
            }
            if entry.reason == reason::READY {
                draw_rectangle(
                    button.x,
                    button.y,
                    button.w,
                    button.h,
                    Color::from_rgba(40, 75, 110, 255),
                );
                ui.label_in("Apprendre", button, WHITE);
                if button.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
                    if let Some(command) = self.learn_skill(entry.id, entry.next) {
                        out.push(command);
                    }
                }
            }
        }
        let y = r.y + r.h - 54.0;
        ui.label(
            &format!("Page {}/{}", self.skill_page + 1, pages),
            r.x + 170.0,
            y + 18.0,
            WHITE,
        );
        for (label, x, delta) in [
            ("Précédente", r.x + 12.0, -1i32),
            ("Suivante", r.x + r.w - 105.0, 1),
        ] {
            let button = Rect::new(x, y, 93.0, 25.0);
            draw_rectangle(
                button.x,
                button.y,
                button.w,
                button.h,
                Color::from_rgba(35, 50, 75, 255),
            );
            ui.label_in(label, button, WHITE);
            if button.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
                self.skill_page =
                    (self.skill_page as i32 + delta).clamp(0, pages as i32 - 1) as usize;
            }
        }
        if self.is_casting() {
            let cancel=Rect::new(r.x+r.w-115.,r.y+29.,105.,23.);
            ui.label_in("Annuler",cancel,YELLOW);
            if cancel.contains(mouse) && is_mouse_button_pressed(MouseButton::Left){out.push(self.cancel_skill());}
        }
        ui.label(
            "3212 : cadavre de monstre ciblé, puis consommé.",
            r.x + 12.0,
            r.y + r.h - 10.0,
            LIGHTGRAY,
        );
        r.contains(mouse)
    }
}
