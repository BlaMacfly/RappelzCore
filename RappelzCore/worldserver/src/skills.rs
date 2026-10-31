//! Skill learning: validate a requested next level, persist JP+level atomically, then publish.
use crate::session::Session;
use anyhow::Result;
use rz_database::characters::{self, Character};
use rz_proto::{
    msg, msg_game as game,
    skills::{self as wire, catalog, reason},
};
use rz_rappelz::skills::{LearnStatus, LearningOffer, MAX_SKILLS, MAX_SKILL_LEVEL};
use std::collections::BTreeMap;

const PREFIX: &str = "learned_skill_";
pub fn learned(c: &Character) -> std::result::Result<BTreeMap<u32, u8>, &'static str> {
    let mut levels = BTreeMap::new();
    for (key, value) in &c.extra {
        if let Some(id) = key.strip_prefix(PREFIX) {
            let id = id
                .parse::<u32>()
                .ok()
                .filter(|id| *id > 0 && *id <= i32::MAX as u32)
                .ok_or("Compétences sauvegardées invalides.")?;
            if key != &format!("{PREFIX}{id}") {
                return Err("Identifiant de compétence sauvegardé invalide.");
            }
            let level = value
                .parse::<u8>()
                .ok()
                .filter(|l| *l > 0 && *l <= MAX_SKILL_LEVEL)
                .ok_or("Niveau de compétence sauvegardé invalide.")?;
            if levels.insert(id, level).is_some() || levels.len() > MAX_SKILLS {
                return Err("Liste de compétences sauvegardée invalide.");
            }
        }
    }
    Ok(levels)
}
fn numbers(c: &Character) -> std::result::Result<(i32, i32, i64), &'static str> {
    let get = |key: &str, default: i64| {
        c.extra.get(key).map_or(Ok(default), |s| {
            s.parse::<i64>()
                .map_err(|_| "Progression sauvegardée invalide.")
        })
    };
    let job = i32::try_from(get("job", i64::from(c.class))?).map_err(|_| "Métier invalide.")?;
    let jlv = i32::try_from(get("jlv", 1)?).map_err(|_| "Niveau de métier invalide.")?;
    let jp = get("jp", 0)?;
    if jlv < 1 || jp < 0 {
        return Err("Progression sauvegardée invalide.");
    }
    Ok((job, jlv, jp))
}
fn status(status: LearnStatus) -> u8 {
    match status {
        LearnStatus::Ready => reason::READY,
        LearnStatus::NotEnoughJp => reason::JP,
        LearnStatus::Prerequisite => reason::PREREQUISITE,
        LearnStatus::JobLevel => reason::JOB_LEVEL,
        LearnStatus::PlayerLevel => reason::PLAYER_LEVEL,
        LearnStatus::Maximum => reason::MAXIMUM,
        LearnStatus::UnsupportedRule => reason::UNSUPPORTED,
        LearnStatus::InvalidData => reason::INVALID_DATA,
        LearnStatus::WrongJob => reason::WRONG_JOB,
        LearnStatus::UnknownSkill => reason::UNKNOWN_SKILL,
    }
}
fn offer(o: LearningOffer) -> wire::SkillOffer {
    wire::SkillOffer {
        id: o.skill,
        learned: o.learned,
        next: o.next,
        maximum: o.maximum,
        cost: o.cost,
        player_level: o.player_level,
        job_level: o.job_level,
        prerequisites: o.prerequisites,
        passive: o.passive,
        reason: status(o.status),
    }
}
fn notice(code: u8) -> &'static str {
    match code {
        reason::JP => "Vous n'avez pas assez de JP.",
        reason::PREREQUISITE => "Compétence préalable requise.",
        reason::JOB_LEVEL => "Niveau de métier insuffisant.",
        reason::PLAYER_LEVEL => "Niveau du personnage insuffisant.",
        reason::MAXIMUM => "Niveau maximal de cette compétence atteint.",
        reason::WRONG_JOB => "Cette compétence n'appartient pas à votre arbre actuel.",
        reason::UNSUPPORTED => "Cette règle d'apprentissage n'est pas encore prise en charge.",
        _ => "Données d'apprentissage indisponibles ou invalides.",
    }
}
impl Session {
    fn learning_catalog(&self) -> Option<wire::ScSkillCatalog> {
        let g = self.in_game.as_ref()?;
        let c = &g.character;
        let mut state = wire::ScSkillCatalog {
            target: g.handle,
            job: 0,
            job_level: 0,
            player_level: c.level,
            jp: 0,
            status: catalog::INVALID_CHARACTER,
            entries: vec![],
        };
        let Ok((job, jlv, jp)) = numbers(c) else {
            return Some(state);
        };
        state.job = job;
        state.job_level = jlv;
        state.jp = jp;
        let Ok(levels) = learned(c) else {
            return Some(state);
        };
        let Some(book) = &self.ctx.data.skills else {
            state.status = catalog::UNAVAILABLE;
            return Some(state);
        };
        if !book.has_job(job) {
            state.status = catalog::UNKNOWN_JOB;
            return Some(state);
        }
        let entries = book.catalog(job, c.level, jlv, jp, &levels);
        if entries.len() > MAX_SKILLS {
            return Some(state);
        }
        state.status = catalog::READY;
        state.entries = entries.into_iter().map(offer).collect();
        Some(state)
    }
    pub fn enable_skills(&mut self, version: u16) {
        if version != wire::VERSION {
            return;
        }
        if let Some(state) = self.learning_catalog() {
            if self.skill_catalog.as_ref() != Some(&state) {
                self.send(&state);
                self.skill_catalog = Some(state);
            }
        }
    }
    pub fn send_skill_catalog(&mut self) {
        if self.skill_catalog.is_some() {
            self.enable_skills(wire::VERSION);
        }
    }
    pub fn send_learned_skills(&self) {
        let Some(g) = &self.in_game else {
            return;
        };
        let Ok(levels) = learned(&g.character) else {
            tracing::warn!(
                guid = g.character.guid,
                "invalid stored skill levels; learning disabled"
            );
            return;
        };
        let world=self.ctx.world.lock().unwrap();
        self.send(&game::ScSkillList {
            target: g.handle,
            modification_type: 0,
            skills: levels
                .into_iter()
                .map(|(skill_id, level)| {
                    let q=self.ctx.data.restoration.as_ref().and_then(|book|u16::try_from(skill_id).ok().and_then(|id|book.quote(id,level)));
                    let (total,remain)=q.map_or((0,0),|q|(q.cooldown,world.cooldown_remaining(g.character.guid,q.group)));
                    game::SkillInfo {
                    skill_id: skill_id as i32,
                    base_skill_level: level,
                    current_skill_level: level,
                    total_cool_time:total,remain_cool_time:remain,
                }})
                .collect(),
        });
    }
    fn learning_result(&mut self, success: bool, message: &str) {
        self.send(&msg::ScResult {
            request: game::id::TS_CS_LEARN_SKILL,
            result: if success {
                msg::result::SUCCESS
            } else {
                msg::result::ACCESS_DENIED
            },
            value: 0,
        });
        if let Some(g) = &self.in_game {
            self.ctx.world.lock().unwrap().notify(g.handle, message);
        }
        self.send_skill_catalog();
    }
    pub async fn learn_skill(&mut self, request: game::CsLearnSkill) -> Result<()> {
        let Some(g) = &self.in_game else {
            return Ok(());
        };
        let active = {
            let world = self.ctx.world.lock().unwrap();
            !world.casts.contains_key(&g.handle) && world.players.get(&g.handle).is_some_and(|p| {
                p.guid == g.character.guid && p.account_id == g.character.account && p.hp > 0
            })
        };
        if !active
            || request.target != g.handle
            || request.skill_id <= 0
            || request.base_random_skill_id != 0
            || request.skill_level <= 0
            || request.skill_level > i16::from(MAX_SKILL_LEVEL)
        {
            self.learning_result(
                false,
                "Requête d'apprentissage refusée : cible, type ou niveau invalide.",
            );
            return Ok(());
        }
        let mut candidate = self.character_snapshot().unwrap();
        let (job, jlv, jp) = match numbers(&candidate) {
            Ok(v) => v,
            Err(message) => {
                self.learning_result(false, message);
                return Ok(());
            }
        };
        let levels = match learned(&candidate) {
            Ok(v) => v,
            Err(message) => {
                self.learning_result(false, message);
                return Ok(());
            }
        };
        let Some(book) = &self.ctx.data.skills else {
            self.learning_result(false, "Données d'apprentissage indisponibles.");
            return Ok(());
        };
        let id = request.skill_id as u32;
        let quote = book.offer(job, candidate.level, jlv, jp, &levels, id);
        if i16::from(quote.next) != request.skill_level {
            self.learning_result(
                false,
                "Requête d'apprentissage périmée : actualisez le niveau demandé.",
            );
            return Ok(());
        }
        if quote.status != LearnStatus::Ready {
            self.learning_result(false, notice(status(quote.status)));
            return Ok(());
        }
        if levels.len() >= MAX_SKILLS && !levels.contains_key(&id) {
            self.learning_result(false, "Nombre maximal de compétences atteint.");
            return Ok(());
        }
        let remaining = jp
            .checked_sub(quote.cost.unwrap())
            .filter(|n| *n >= 0)
            .expect("validated learning quote");
        candidate.set_extra("jp", remaining);
        candidate.set_extra(&format!("{PREFIX}{id}"), quote.next);
        // Existing characters::save writes character+all extras in one SQL transaction.
        // Never mutate memory, publish success, or debit JP before that transaction commits.
        if let Err(error) = characters::save(&self.ctx.chars, &candidate).await {
            tracing::warn!(
                guid = candidate.guid,
                "skill learning could not be saved: {error:#}"
            );
            self.learning_result(false, "Apprentissage non sauvegardé ; aucun JP débité.");
            return Ok(());
        }
        let handle = self.in_game.as_ref().unwrap().handle;
        let xp = candidate.xp;
        self.in_game.as_mut().unwrap().character = candidate;
        self.send(&game::ScExpUpdate {
            handle,
            exp: i64::from(xp),
            jp: remaining,
        });
        self.send_learned_skills();
        self.learning_result(
            true,
            &format!(
                "Compétence {id} apprise au niveau {} ({} JP).",
                quote.next,
                quote.cost.unwrap()
            ),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persisted_skill_levels_are_bounded_and_other_character_extras_are_preserved() {
        let mut c = Character::default();
        c.set_extra("job", 200);
        c.set_extra("jlv", 3);
        c.set_extra("jp", 18);
        c.set_extra("model0", 17);
        c.set_extra("learned_skill_42", 2);
        assert_eq!(learned(&c).unwrap(), BTreeMap::from([(42, 2)]));
        assert_eq!(numbers(&c).unwrap(), (200, 3, 18));
        assert_eq!(c.extra_i64("model0", 0), 17);
        for value in ["0", "51", "-1", "abc"] {
            c.set_extra("learned_skill_42", value);
            assert!(learned(&c).is_err());
        }
        c.set_extra("jp", "not-a-number");
        assert!(numbers(&c).is_err());
    }
}
