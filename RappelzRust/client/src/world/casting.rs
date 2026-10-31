use super::*;
use rz_proto::restoration::{decode, Phase};

impl WorldState {
    pub fn focus_skill(&mut self, id: u32) {
        self.show_skills = true;
        let per_page = (((screen_height() - 190.).clamp(300., 530.) - 135.) / 62.)
            .floor()
            .max(1.) as usize;
        if let Some(index) = self
            .skill_catalog
            .as_ref()
            .and_then(|s| s.entries.iter().position(|e| e.id == id))
        {
            self.skill_page = index / per_page;
        }
    }
    pub fn skill_remaining(&self, id: u32) -> f64 {
        self.skill_cooldowns
            .get(&id)
            .map_or(0., |(end, _)| (*end - self.now()).max(0.))
    }
    pub fn is_casting(&self) -> bool {
        self.casting.contains_key(&self.handle)
    }
    pub fn cast_skill(&self, q: &rz_rappelz::restoration::Quote) -> Option<NetCmd> {
        if self.offline
            || self.hp <= 0
            || self.mp < q.mp.max(q.minimum_mp)
            || self.is_casting()
            || self.skill_remaining(u32::from(q.id)) > 0.
            || self
                .learned_skills
                .as_ref()?
                .get(&u32::from(q.id))
                .copied()
                .unwrap_or(0)
                < q.level
        {
            return None;
        }
        let (x, y) = self.me.motion.position(self.now());
        Some(NetCmd::Raw(
            game::CsSkill {
                skill_id: q.id,
                caster: self.handle,
                target: self.handle,
                x,
                y,
                z: 0.,
                layer: self.layer,
                skill_level: q.level,
            }
            .to_packet(),
        ))
    }
    pub fn cancel_skill(&self) -> NetCmd {
        NetCmd::Raw(game::Handle::<{ gid::TS_CS_CANCEL_ACTION }>(self.handle).to_packet())
    }
    pub(super) fn apply_restoration(&mut self, s: game::ScSkill, now: f64) {
        if s.layer != self.layer
            || (s.caster != self.handle
                && !self
                    .entities
                    .get(&s.caster)
                    .is_some_and(|e| e.layer == Some(s.layer)))
        {
            return;
        }
        let Ok(phase) = decode(&s) else {
            return;
        };
        let (name, amount) = match phase {
            Phase::Casting(duration) => {
                self.casting.retain(|_, (_, _, end)| *end + 5. > now);
                if self.casting.contains_key(&s.caster) || self.casting.len() >= 256 {
                    return;
                }
                self.casting.insert(
                    s.caster,
                    (s.skill_id, s.skill_level, now + f64::from(duration) / 100.),
                );
                ("CASTING", 0)
            }
            Phase::Cancel | Phase::Fire { .. } => {
                if !self
                    .casting
                    .get(&s.caster)
                    .is_some_and(|(id, level, _)| *id == s.skill_id && *level == s.skill_level)
                {
                    return;
                }
                if matches!(phase, Phase::Fire { .. }) {
                    if s.caster == self.handle {
                        if s.caster_hp > self.max_hp || s.caster_mp > self.max_mp {
                            return;
                        }
                        // FIRE already carries authoritative post-effect vitals. Make them
                        // readable before the Lua event; the following HPMP is a duplicate.
                        self.hp = s.caster_hp;
                        self.mp = s.caster_mp;
                    } else if let Some(e) = self.entities.get_mut(&s.caster) {
                        let Some((_, max_mp)) = e.mana else {
                            return;
                        };
                        if !e.health_known || s.caster_hp > e.max_hp || s.caster_mp > max_mp {
                            return;
                        }
                        e.apply_hpmp(s.caster_hp, e.max_hp, s.caster_mp, max_mp, now);
                    }
                }
                self.casting.remove(&s.caster);
                if let Phase::Fire { amount, .. } = phase {
                    if let Some((x, y)) = self.unit_position(s.caster) {
                        self.floats.push(FloatText {
                            pos: vec3(x, 0., y),
                            text: format!("+{amount} PM"),
                            color: SKYBLUE,
                            start: now,
                        });
                    }
                    ("FIRE", amount)
                } else {
                    ("CANCEL", 0)
                }
            }
        };
        let sequence = self
            .skill_event
            .as_ref()
            .map_or(1, |e| e.sequence.saturating_add(1));
        self.skill_event = Some(rz_addon::SkillEventState {
            sequence,
            skill: s.skill_id,
            phase: name.into(),
            caster: s.caster,
            amount,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepted_cast_phases_do_not_optimistically_change_mana_and_reject_hidden_or_malformed_events(
    ) {
        let mut w = WorldState::from_login(
            &msg::ScLoginResult {
                handle: 7,
                hp: 100,
                max_hp: 100,
                mp: 20,
                max_mp: 50,
                ..Default::default()
            },
            Vec2::ZERO,
        );
        w.on_packet(
            game::ScSkillList {
                target: 7,
                modification_type: 0,
                skills: vec![game::SkillInfo {
                    skill_id: 3212,
                    base_skill_level: 1,
                    current_skill_level: 1,
                    ..Default::default()
                }],
            }
            .to_packet(),
            0.,
        );
        let d = rz_rappelz::restoration::Restoration {
            id: 3212,
            mp_base: 0,
            mp_per_level: 5,
            cast_base: 150,
            cast_per_level: -5,
            cooldown: 2500,
            group: 103,
            minimum_level: 0,
            minimum_hp: 0,
            minimum_mp: 0,
            amount_base: 2000.,
            amount_per_level: 1500.,
        };
        let q = d.quote(1).unwrap();
        assert!(w.cast_skill(&q).is_some());
        assert_eq!(w.mp, 20);
        let cast = rz_proto::restoration::event(
            3212,
            1,
            7,
            (0., 0., 0., 0),
            100,
            20,
            0,
            Phase::Casting(145),
        );
        w.on_packet(cast.to_packet(), 0.);
        assert!(w.is_casting());
        assert!(w.cast_skill(&q).is_none());
        assert_eq!(w.mp, 20);
        w.on_packet(cast.to_packet(), 0.);
        assert_eq!(w.skill_event.as_ref().unwrap().sequence, 1);
        let mut fire = rz_proto::restoration::event(
            3212,
            1,
            7,
            (0., 0., 0., 0),
            100,
            50,
            5,
            Phase::Fire {
                mana: 50,
                amount: 3500,
            },
        );
        fire.target = 8;
        w.on_packet(fire.to_packet(), 1.5);
        assert!(w.is_casting());
        fire.target = 7;
        w.on_packet(fire.to_packet(), 1.5);
        assert!(!w.is_casting());
        assert_eq!(w.mp, 50);
        assert_eq!(w.skill_event.as_ref().unwrap().amount, 3500);
        w.on_packet(fire.to_packet(), 1.5);
        assert_eq!(w.skill_event.as_ref().unwrap().sequence, 2);
        w.on_packet(
            game::ScHpMp {
                handle: 7,
                hp: 100,
                max_hp: 100,
                mp: 50,
                max_mp: 50,
                ..Default::default()
            }
            .to_packet(),
            1.5,
        );
        assert_eq!(w.mp, 50);
        let mut hidden = cast;
        hidden.caster = 8;
        hidden.target = 8;
        w.on_packet(hidden.to_packet(), 2.);
        assert_eq!(w.skill_event.as_ref().unwrap().sequence, 2);
        let mut observer = Entity::new("Witness".into(), Kind::Player, 0., 0., 1, None);
        observer.layer = Some(0);
        observer.set_vitals(100, 100, 20, 50);
        w.entities.insert(8, observer);
        let start = rz_proto::restoration::event(
            3212,
            1,
            8,
            (0., 0., 0., 0),
            100,
            20,
            0,
            Phase::Casting(145),
        );
        w.on_packet(start.to_packet(), 3.);
        let fire = rz_proto::restoration::event(
            3212,
            1,
            8,
            (0., 0., 0., 0),
            100,
            50,
            5,
            Phase::Fire {
                mana: 50,
                amount: 3500,
            },
        );
        w.on_packet(fire.to_packet(), 4.5);
        assert_eq!(w.unit(8).unwrap().mana, Some(50));
        assert_eq!(w.addon_state().skill_event.unwrap().phase, "FIRE");
    }
}
