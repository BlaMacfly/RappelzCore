//! Server-authoritative fixed self MP restoration; no database work under the world lock.
use crate::{session::Session, skills::learned, world::World};
use rz_proto::{
    msg, msg_game as game,
    restoration::{self as wire, Phase},
    Message,
};
use rz_rappelz::restoration::Quote;

pub(crate) struct PendingCast {
    guid: u32,
    position: (f32, f32, f32, u8),
    end: u64,
    quote: Quote,
    known: Vec<(u32, u8, Option<Quote>)>,
}
impl Session {
    pub fn cast_skill(&mut self, request: game::CsSkill) {
        let Some(g) = &self.in_game else {
            return;
        };
        let levels = learned(&g.character).ok();
        let quote = self
            .ctx
            .data
            .restoration
            .as_ref()
            .and_then(|b| b.quote(request.skill_id, request.skill_level));
        let valid = levels.as_ref().is_some_and(|s| {
            s.get(&u32::from(request.skill_id))
                .is_some_and(|n| request.skill_level > 0 && request.skill_level <= *n)
        });
        let ok = if valid {
            if let (Some(q), Some(levels)) = (quote, levels) {
                let known = levels
                    .into_iter()
                    .map(|(id, level)| {
                        (
                            id,
                            level,
                            self.ctx.data.restoration.as_ref().and_then(|b| {
                                u16::try_from(id).ok().and_then(|id| b.quote(id, level))
                            }),
                        )
                    })
                    .collect();
                let mut world = self.ctx.world.lock().unwrap();
                let now = world.cast_clock();
                if world.players.get(&g.handle).is_some_and(|p| {
                    p.guid == g.character.guid && p.account_id == g.character.account
                }) {
                    world.begin_restoration(g.handle, &request, q, known, now)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        self.send(&msg::ScResult {
            request: game::id::TS_CS_SKILL,
            result: if ok {
                msg::result::SUCCESS
            } else {
                msg::result::ACCESS_DENIED
            },
            value: 0,
        });
        if !ok {
            self.ctx.world.lock().unwrap().notify(
                g.handle,
                "Lancement refusé : compétence, état, PM ou recharge indisponible.",
            );
        }
    }
}
impl World {
    pub(crate) fn begin_restoration(
        &mut self,
        handle: u32,
        r: &game::CsSkill,
        q: Quote,
        known: Vec<(u32, u8, Option<Quote>)>,
        now: u64,
    ) -> bool {
        self.cast_cooldowns.retain(|_, end| *end > now);
        let Some(p) = self.players.get(&handle) else {
            return false;
        };
        if r.caster != handle
            || r.target != handle
            || r.layer != p.layer
            || r.skill_id != q.id
            || r.skill_level != q.level
            || ![r.x, r.y, r.z].iter().all(|v| v.is_finite())
            || p.hp <= 0
            || p.mp < q.mp.max(q.minimum_mp)
            || p.hp < q.minimum_hp
            || p.enter.creature.level < i32::from(q.minimum_level)
            || p.max_mp <= 0
            || p.mp < 0
            || p.mp > p.max_mp
            || !p.motion.remaining_points(self.now()).is_empty()
            || p.attack_target.is_some()
            || self.casts.contains_key(&handle)
            || self.cast_cooldowns.contains_key(&(p.guid, q.group))
            || self.cast_cooldowns.len() >= 8192
            || known.len() > 512
        {
            return false;
        }
        let (x, y) = p.motion.position(self.now());
        let position = (x, y, p.z, p.layer);
        let packet = wire::event(
            q.id,
            q.level,
            handle,
            position,
            p.hp,
            p.mp,
            0,
            Phase::Casting(q.cast),
        )
        .to_packet();
        self.casts.insert(
            handle,
            PendingCast {
                guid: p.guid,
                position,
                end: now + u64::from(q.cast),
                quote: q,
                known,
            },
        );
        self.broadcast_around(handle, &packet, true);
        true
    }
    pub(crate) fn cancel_restoration(&mut self, handle: u32) {
        let Some(c) = self.casts.remove(&handle) else {
            return;
        };
        if let Some(p) = self.players.get(&handle) {
            let packet = wire::event(
                c.quote.id,
                c.quote.level,
                handle,
                c.position,
                p.hp.max(0),
                p.mp.max(0),
                0,
                Phase::Cancel,
            )
            .to_packet();
            self.broadcast_around(handle, &packet, true);
        }
    }
    pub(crate) fn cooldown_remaining(&self, guid: u32, group: u32) -> u32 {
        self.cast_cooldowns.get(&(guid, group)).map_or(0, |end| {
            end.saturating_sub(self.cast_clock())
                .min(u64::from(u32::MAX)) as u32
        })
    }
    pub(crate) fn tick_restorations(&mut self, now: u64) {
        self.cast_cooldowns.retain(|_, end| *end > now);
        let handles: Vec<_> = self.casts.keys().copied().collect();
        for handle in handles {
            let c = &self.casts[&handle];
            let valid = self.players.get(&handle).is_some_and(|p| {
                let (x, y) = p.motion.position(self.now());
                p.guid == c.guid
                    && p.hp > 0
                    && p.layer == c.position.3
                    && p.motion.remaining_points(self.now()).is_empty()
                    && (x - c.position.0).abs() < 0.01
                    && (y - c.position.1).abs() < 0.01
                    && (p.z - c.position.2).abs() < 0.01
                    && p.attack_target.is_none()
                    && p.hp >= c.quote.minimum_hp
                    && p.mp >= c.quote.mp.max(c.quote.minimum_mp)
                    && p.mp <= p.max_mp
                    && p.max_mp > 0
            });
            if !valid {
                self.cancel_restoration(handle);
                continue;
            }
            if now < c.end {
                continue;
            }
            let c = self.casts.remove(&handle).unwrap();
            let p = self.players.get_mut(&handle).unwrap();
            let before = p.mp;
            p.mp = (i64::from(before) - i64::from(c.quote.mp) + i64::from(c.quote.amount))
                .min(i64::from(p.max_mp)) as i32;
            p.last_combat = now as u32;
            p.enter.creature.mp = p.mp;
            let event = wire::event(
                c.quote.id,
                c.quote.level,
                handle,
                c.position,
                p.hp,
                p.mp,
                c.quote.mp,
                Phase::Fire {
                    mana: p.mp,
                    amount: c.quote.amount,
                },
            )
            .to_packet();
            let hpmp = game::ScHpMp {
                handle,
                hp: p.hp,
                max_hp: p.max_hp,
                mp: p.mp,
                max_mp: p.max_mp,
                add_hp: 0,
                add_mp: p.mp - before,
                need_to_display: true,
            }
            .to_packet();
            self.cast_cooldowns
                .insert((c.guid, c.quote.group), now + u64::from(c.quote.cooldown));
            self.broadcast_around(handle, &event, true);
            self.broadcast_around(handle, &hpmp, true);
            let skills = c
                .known
                .into_iter()
                .map(|(id, level, quote)| {
                    let (total, remain) = quote.map_or((0, 0), |q| {
                        (
                            q.cooldown,
                            self.cast_cooldowns
                                .get(&(c.guid, q.group))
                                .map_or(0, |end| end.saturating_sub(now) as u32),
                        )
                    });
                    game::SkillInfo {
                        skill_id: id as i32,
                        base_skill_level: level,
                        current_skill_level: level,
                        total_cool_time: total,
                        remain_cool_time: remain,
                    }
                })
                .collect();
            self.send_to(
                handle,
                game::ScSkillList {
                    target: handle,
                    modification_type: 0,
                    skills,
                }
                .to_packet(),
            );
        }
    }
}
