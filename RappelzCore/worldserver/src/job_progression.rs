//! Narrow current-job advancement using the recovered archive JP costs.
//! No guessed effective-stat, old-job inheritance or EXP conversion is performed.
use anyhow::{anyhow, Result};
use rz_database::characters::{self, Character};
use rz_proto::{
    job::{self as wire, status, ScJobProgress},
    msg, msg_game as game, Message,
};

use crate::{data::GameData, session::Session, skills::numbers, world::World};

pub(crate) fn preview(data: &GameData, target: u32, character: &Character) -> ScJobProgress {
    let mut state = ScJobProgress {
        target,
        job: 0,
        job_level: 0,
        jp: 0,
        cost: None,
        status: status::INVALID_CHARACTER,
        base: None,
    };
    let Ok((job, jlv, jp)) = numbers(character) else {
        return state;
    };
    state.job = job;
    state.job_level = jlv;
    state.jp = jp;
    if job < 0 {
        return state;
    }
    let Some(book) = &data.progression else {
        state.status = status::UNAVAILABLE;
        return state;
    };
    let Some(definition) = data.jobs.get(&job) else {
        state.status = status::UNKNOWN_JOB;
        return state;
    };
    let Some(base) = book.base(definition.stat_id) else {
        state.status = status::UNSUPPORTED;
        return state;
    };
    state.base = Some((definition.stat_id, base));
    let Ok(depth) = usize::try_from(definition.job_depth) else {
        state.status = status::UNSUPPORTED;
        return state;
    };
    if depth > 3 {
        state.status = status::UNSUPPORTED;
        return state;
    }
    // The recovered reference supports purchase from JLV 1..49 only. The longer
    // archive column for depth 3 is retained as data, not an invented level policy.
    if jlv >= 50 {
        state.status = status::MAXIMUM;
        return state;
    }
    let Some(row) = book.experience(jlv as u16) else {
        state.status = status::UNSUPPORTED;
        return state;
    };
    let cost = row.jp[depth];
    if cost == 0 {
        state.status = status::MAXIMUM;
        return state;
    }
    state.cost = Some(cost);
    state.status = if jp < cost { status::JP } else { status::READY };
    state
}

fn notice(code: u8) -> &'static str {
    match code {
        status::JP => "Vous n'avez pas assez de JP pour augmenter le niveau de métier.",
        status::MAXIMUM => "Limite de niveau de métier atteinte pour cette progression.",
        status::UNKNOWN_JOB => "Métier sauvegardé inconnu ; sa progression est conservée.",
        status::UNSUPPORTED => {
            "Données de progression de ce métier indisponibles ou non prises en charge."
        }
        status::INVALID_CHARACTER => {
            "Progression sauvegardée invalide ; aucune valeur n'a été modifiée."
        }
        _ => "Profil de progression du métier indisponible.",
    }
}

impl Session {
    pub fn send_job_progress(&self) {
        if self.skill_catalog.is_none() || self.ctx.data.progression.is_none() {
            return;
        }
        let Some(g) = &self.in_game else {
            return;
        };
        let owned = self
            .ctx
            .world
            .lock()
            .unwrap()
            .players
            .get(&g.handle)
            .is_some_and(|p| p.guid == g.character.guid && p.account_id == g.character.account);
        if owned {
            self.send(&preview(&self.ctx.data, g.handle, &g.character));
        }
    }

    fn job_result(&mut self, success: bool, message: &str) {
        self.send(&msg::ScResult {
            request: wire::CS_JOB_LEVEL_UP,
            result: if success {
                msg::result::SUCCESS
            } else {
                msg::result::ACCESS_DENIED
            },
            value: 0,
        });
        // Address this session directly, never a possibly replaced numeric world handle.
        self.send(&msg::ScChat {
            sender: "@NOTICE".into(),
            chat_type: msg::chat_type::NOTICE,
            message: message.into(),
        });
        self.send_skill_catalog();
    }

    pub async fn level_up_job(&mut self, request: wire::CsJobLevelUp) -> Result<()> {
        let Some(g) = &self.in_game else {
            self.job_result(false, "Aucun personnage propriétaire en jeu.");
            return Ok(());
        };
        let active = {
            let world = self.ctx.world.lock().unwrap();
            request.target == g.handle
                && !world.casts.contains_key(&g.handle)
                && world.players.get(&g.handle).is_some_and(|p| {
                    p.guid == g.character.guid && p.account_id == g.character.account && p.hp > 0
                })
        };
        if !active {
            self.job_result(
                false,
                "Progression refusée : propriétaire, vie ou incantation invalide.",
            );
            return Ok(());
        }
        let Some(mut candidate) = self.character_snapshot() else {
            self.detach_world_without_save();
            return Err(anyhow!(
                "job progression lost world ownership; closing without a stale save"
            ));
        };
        let state = preview(&self.ctx.data, request.target, &candidate);
        if state.status != status::READY {
            self.job_result(false, notice(state.status));
            return Ok(());
        }
        let cost = state.cost.expect("validated job quote");
        let remaining = state
            .jp
            .checked_sub(cost)
            .filter(|v| *v >= 0)
            .expect("validated JP balance");
        let next = state.job_level + 1;
        candidate.set_extra("jp", remaining);
        candidate.set_extra("jlv", next);
        // Existing SQL transaction atomically saves the full character and all extras.
        // An uncertain COMMIT must never be followed by a stale disconnect snapshot.
        if let Err(error) = characters::save(&self.ctx.chars, &candidate).await {
            self.detach_world_without_save();
            return Err(error
                .context("job progression save failed; closing without a stale disconnect save"));
        }
        let still_owned = self
            .ctx
            .world
            .lock()
            .unwrap()
            .players
            .get(&request.target)
            .is_some_and(|p| p.guid == candidate.guid && p.account_id == candidate.account);
        if !still_owned {
            self.detach_world_without_save();
            return Err(anyhow!("job progression committed after world ownership changed; closing without stale publication"));
        }
        let level = candidate.level;
        let xp = candidate.xp;
        self.in_game.as_mut().unwrap().character = candidate;
        self.send(&game::ScExpUpdate {
            handle: request.target,
            exp: i64::from(xp),
            jp: remaining,
        });
        self.send(&msg::ScProperty {
            handle: request.target,
            name: "job_level".into(),
            value: i64::from(next),
            text: None,
        });
        self.ctx
            .world
            .lock()
            .unwrap()
            .publish_job_level(request.target, level, next);
        self.job_result(
            true,
            &format!("Niveau de métier {next} atteint ({cost} JP)."),
        );
        Ok(())
    }
}

impl World {
    /// Publish only the confirmed job level. Player level, effective stats and vitals stay put.
    pub(crate) fn publish_job_level(&self, handle: u32, level: u8, job_level: i32) {
        if !self.players.contains_key(&handle) {
            return;
        }
        let packet = game::ScLevelUpdate {
            handle,
            level: i32::from(level),
            job_level,
        }
        .to_packet();
        for viewer in self.players.keys().copied() {
            if self.can_see(viewer, handle, self.now()) {
                self.send_to(viewer, packet.clone());
            }
        }
    }
}
