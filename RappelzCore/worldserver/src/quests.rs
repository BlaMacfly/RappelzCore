//! Objectifs de quêtes : interprétation des `value1..12` de QuestResource selon le type et
//! progression des six compteurs Rappelz. Logique pure (sans réseau ni SQL), testée ici.
//!
//! Interprétation des types, déduite des données (Quest.dbc) [D] — à confronter à
//! docs/re/quest_protocol.md :
//! - 101 tuer au total `value2` monstres parmi `value1/3/5` ;
//! - 102 tuer des monstres précis : paires (monstre, nombre) en `value1..6` ;
//! - 103 / 107 / 109 posséder des objets : paires (objet, nombre) en `value1..12` ;
//! - 106 objet `value1` ×`value2` obtenu sur les monstres `value7..12` ;
//! - 401 parler au PNJ de fin (aucun compteur) ;
//! - 501 atteindre le niveau de métier `value2`.
//! Les autres types (201, 301, 302, 601, 701, 901…) ne sont pas encore pris en charge :
//! la quête reste en cours, sans progression automatique.

use std::collections::HashSet;

use rz_database::quests::{QuestStatus, PROGRESS_SLOTS};
use rz_rappelz::quests::QuestDef;

/// Nombre maximal de quêtes en cours (limite du journal) [D].
pub const MAX_ACTIVE: usize = 20;

/// Journal de quêtes du personnage en jeu.
#[derive(Clone, Debug, Default)]
pub struct QuestLog {
    pub active: Vec<QuestStatus>,
    pub rewarded: HashSet<u32>,
}

impl QuestLog {
    pub fn get(&self, quest: u32) -> Option<&QuestStatus> {
        self.active.iter().find(|q| q.quest == quest)
    }

    pub fn get_mut(&mut self, quest: u32) -> Option<&mut QuestStatus> {
        self.active.iter_mut().find(|q| q.quest == quest)
    }

    /// La quête peut-elle être proposée à ce personnage ?
    pub fn can_start(&self, q: &QuestDef, level: u8, job_level: u16, race: u8) -> bool {
        let id = q.id as u32;
        self.get(id).is_none()
            && (q.repeatable || !self.rewarded.contains(&id))
            && self.active.len() < MAX_ACTIVE
            && (q.limit_level <= 0 || i32::from(level) >= q.limit_level)
            && (q.limit_max_level <= 0 || i32::from(level) <= q.limit_max_level)
            && (q.limit_job_level <= 0 || i32::from(job_level) >= q.limit_job_level)
            && q.allows_race(race)
            && q.prerequisites_met(|f| self.rewarded.contains(&(f as u32)))
            && goals(q).is_some()
    }
}

/// Ce que demande une quête, pour un compteur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goal {
    /// Tuer `count` monstres dont l'id est dans `monsters` (0 = case vide).
    Kill { monsters: [u32; 3], count: u16 },
    /// Posséder `count` exemplaires de l'objet.
    Have { item: u32, count: u16 },
    /// Objet `item` ×`count` donné en tuant un des `monsters`.
    Hunt { item: u32, count: u16, monsters: [u32; 6] },
    /// Atteindre le niveau de métier.
    JobLevel { level: u16 },
    /// Paire vide : le compteur à cette position reste à 0 (le client compare par position).
    Unused,
}

fn pos(v: i32) -> u32 {
    v.max(0) as u32
}

fn cnt(v: i32) -> u16 {
    v.clamp(0, u16::MAX as i32) as u16
}

/// Objectifs par compteur (au plus six), ou `None` si le type n'est pas pris en charge.
pub fn goals(q: &QuestDef) -> Option<Vec<Goal>> {
    let v = &q.values;
    // valueK < 0 : le client lit la cible dans le bloc aléatoire `value[]` du paquet
    // (docs/re/quest_protocol.md §4.2), que le serveur ne tire pas encore.
    if v.iter().any(|x| *x < 0) {
        return None;
    }
    // status[i] correspond à la paire (value(2i+1), value(2i+2)) : on garde les positions.
    let pair = |i: usize| (v[2 * i] > 0 && v[2 * i + 1] > 0).then(|| (pos(v[2 * i]), cnt(v[2 * i + 1])));
    let trim = |mut g: Vec<Goal>| {
        while g.last() == Some(&Goal::Unused) {
            g.pop();
        }
        g
    };
    Some(match q.kind {
        101 => vec![Goal::Kill { monsters: [pos(v[0]), pos(v[2]), pos(v[4])], count: cnt(v[1]).max(1) }],
        102 => trim((0..3).map(|i| pair(i).map_or(Goal::Unused, |(m, c)| Goal::Kill { monsters: [m, 0, 0], count: c })).collect()),
        103 | 107 | 109 => trim((0..6).map(|i| pair(i).map_or(Goal::Unused, |(item, count)| Goal::Have { item, count })).collect()),
        106 => vec![Goal::Hunt {
            item: pos(v[0]),
            count: cnt(v[1]).max(1),
            monsters: [pos(v[6]), pos(v[7]), pos(v[8]), pos(v[9]), pos(v[10]), pos(v[11])],
        }],
        401 => Vec::new(),
        501 => vec![Goal::JobLevel { level: cnt(v[1]) }],
        _ => return None,
    })
}

/// État du joueur utile à l'évaluation.
pub trait Holder {
    fn item_count(&self, item: u32) -> u32;
    fn job_level(&self) -> u16;
}

/// Compteurs recalculés à partir de l'inventaire/métier (objectifs Have, Hunt, JobLevel) ;
/// les compteurs de monstres tués sont conservés. Renvoie vrai si un compteur a changé.
pub fn refresh(goals: &[Goal], counts: &mut [u16; PROGRESS_SLOTS], who: &impl Holder) -> bool {
    let mut changed = false;
    for (i, g) in goals.iter().enumerate().take(PROGRESS_SLOTS) {
        let value = match *g {
            Goal::Kill { .. } | Goal::Unused => continue,
            Goal::Have { item, count } | Goal::Hunt { item, count, .. } => who.item_count(item).min(count as u32) as u16,
            Goal::JobLevel { level } => who.job_level().min(level),
        };
        if counts[i] != value {
            counts[i] = value;
            changed = true;
        }
    }
    changed
}

/// Monstre tué : incrémente les compteurs concernés. Renvoie vrai si un compteur a changé.
pub fn on_kill(goals: &[Goal], counts: &mut [u16; PROGRESS_SLOTS], monster: u32) -> bool {
    let mut changed = false;
    for (i, g) in goals.iter().enumerate().take(PROGRESS_SLOTS) {
        if let Goal::Kill { monsters, count } = *g {
            if monster != 0 && monsters.contains(&monster) && counts[i] < count {
                counts[i] += 1;
                changed = true;
            }
        }
    }
    changed
}

/// Objet de chasse à donner pour ce monstre (quête 106), si l'objectif n'est pas atteint.
pub fn hunt_drop(goals: &[Goal], counts: &[u16; PROGRESS_SLOTS], monster: u32) -> Option<u32> {
    goals.iter().enumerate().take(PROGRESS_SLOTS).find_map(|(i, g)| match *g {
        Goal::Hunt { item, count, monsters } if item != 0 && monster != 0 && monsters.contains(&monster) && counts[i] < count => {
            Some(item)
        }
        _ => None,
    })
}

/// Tous les objectifs sont-ils remplis ?
pub fn is_complete(goals: &[Goal], counts: &[u16; PROGRESS_SLOTS]) -> bool {
    goals.iter().enumerate().take(PROGRESS_SLOTS).all(|(i, g)| {
        let need = match *g {
            Goal::Kill { count, .. } | Goal::Have { count, .. } | Goal::Hunt { count, .. } => count,
            Goal::JobLevel { level } => level,
            Goal::Unused => 0,
        };
        counts[i] >= need
    })
}

/// Objets à retirer de l'inventaire en rendant la quête (objets de quête collectés).
pub fn consumed_items(goals: &[Goal]) -> Vec<(u32, u32)> {
    goals
        .iter()
        .filter_map(|g| match *g {
            Goal::Have { item, count } | Goal::Hunt { item, count, .. } => Some((item, count as u32)),
            _ => None,
        })
        .collect()
}

// ───────────────────────── Session : dialogue, progression, rendu ─────────────────────────
//
// Protocole : docs/re/quest_protocol.md. Le client renvoie le trigger du menu tel quel
// (TS_CS_DIALOG) ; `end_quest( code, index )` est formaté par sa fenêtre de récompense.

use anyhow::Result;
use rz_database::characters;
use rz_proto::msg_game::{self as game, quest_progress, QuestTrigger};

use crate::gameplay::InvItem;
use crate::session::Session;

/// Préfixe des triggers « afficher la quête » des menus de PNJ (format libre côté serveur).
const SHOW_PREFIX: &str = "quest_show|";

struct Inventory<'a> {
    items: &'a [InvItem],
    job_level: u16,
}

impl Holder for Inventory<'_> {
    fn item_count(&self, item: u32) -> u32 {
        self.items.iter().filter(|i| i.entry == item && i.in_bag()).map(|i| i.count).sum()
    }
    fn job_level(&self) -> u16 {
        self.job_level
    }
}

/// État d'une quête du point de vue d'un PNJ, pour la couleur du menu (0 / 1 / 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MenuState {
    Offer = 0,
    InProgress = 1,
    Finished = 2,
}

fn progress_of(goals: &[Goal], st: &QuestStatus) -> u8 {
    if is_complete(goals, &st.counts) {
        quest_progress::FINISHED
    } else {
        quest_progress::IN_PROGRESS
    }
}

impl Session {
    fn quest_info(&self, st: &QuestStatus) -> game::QuestInfo {
        let goals = self.ctx.data.quests.get(st.quest as i32).and_then(goals).unwrap_or_default();
        game::QuestInfo {
            code: st.quest as i32,
            start_id: 0,
            value: [0; 6],
            status: st.counts.map(i32::from),
            progress: progress_of(&goals, st),
            time_limit: 0,
        }
    }

    /// TS_SC_QUEST_LIST complet (le client vide puis reconstruit sa liste à chaque envoi).
    pub fn send_quest_list(&self) {
        let Some(g) = &self.in_game else { return };
        let active = g.quests.active.iter().map(|st| self.quest_info(st)).collect();
        self.send(&game::ScQuestList { active, pending: Vec::new() });
    }

    fn holder_state(&self) -> Option<(Vec<InvItem>, u16)> {
        let g = self.in_game.as_ref()?;
        Some((g.items.clone(), g.character.extra_i64("jlv", 1).clamp(0, u16::MAX as i64) as u16))
    }

    /// Recalcule les objectifs liés à l'inventaire / au métier ; envoie 601 et sauvegarde si besoin.
    pub async fn refresh_quests(&mut self) -> Result<()> {
        let Some((items, job_level)) = self.holder_state() else { return Ok(()) };
        let who = Inventory { items: &items, job_level };
        let book = &self.ctx.data.quests;
        let mut changed = Vec::new();
        if let Some(g) = &mut self.in_game {
            for st in &mut g.quests.active {
                let Some(goals) = book.get(st.quest as i32).and_then(goals) else { continue };
                if refresh(&goals, &mut st.counts, &who) {
                    changed.push((st.clone(), progress_of(&goals, st)));
                }
            }
        }
        self.publish_quest_changes(changed).await
    }

    async fn publish_quest_changes(&mut self, changed: Vec<(QuestStatus, u8)>) -> Result<()> {
        let Some(guid) = self.in_game.as_ref().map(|g| g.character.guid) else { return Ok(()) };
        for (mut st, progress) in changed {
            st.status = if progress == quest_progress::FINISHED {
                rz_database::quests::STATUS_COMPLETE
            } else {
                rz_database::quests::STATUS_INCOMPLETE
            };
            if let Some(live) = self.in_game.as_mut().and_then(|g| g.quests.get_mut(st.quest)) {
                live.status = st.status;
            }
            rz_database::quests::upsert(&self.ctx.chars, guid, &st).await?;
            self.send(&game::ScQuestStatus { code: st.quest as i32, status: st.counts.map(i32::from), progress });
        }
        Ok(())
    }

    /// Monstre tué par ce joueur : compteurs de chasse et objets de quête (type 106).
    pub async fn quest_kill(&mut self, monster: u32) -> Result<()> {
        let book = &self.ctx.data.quests;
        let mut changed = Vec::new();
        let mut drops = Vec::new();
        if let Some(g) = &mut self.in_game {
            for st in &mut g.quests.active {
                let Some(goals) = book.get(st.quest as i32).and_then(goals) else { continue };
                if let Some(item) = hunt_drop(&goals, &st.counts, monster) {
                    drops.push(item);
                }
                if on_kill(&goals, &mut st.counts, monster) {
                    changed.push((st.clone(), progress_of(&goals, st)));
                }
            }
        }
        self.publish_quest_changes(changed).await?;
        for item in drops {
            // Butin de quête remis directement dans le sac (pas encore d'objets au sol).
            if !self.give_item(item, 1).await? {
                let handle = self.in_game.as_ref().map(|g| g.handle).unwrap_or(0);
                self.ctx.world.lock().unwrap().notify(handle, "Le sac est plein : objet de quête perdu.");
            }
        }
        Ok(())
    }

    /// Entrées de menu « quête » d'un PNJ (`NpcResource.id`) pour ce personnage.
    pub fn quest_menu(&self, npc_id: u32) -> Vec<(String, String)> {
        let Some(g) = &self.in_game else { return Vec::new() };
        let c = &g.character;
        let job_level = c.extra_i64("jlv", 1).clamp(0, u16::MAX as i64) as u16;
        let book = &self.ctx.data.quests;
        let mut entries: Vec<(i32, MenuState)> = Vec::new();
        for link in book.links_of(npc_id as i32) {
            let Some(q) = book.get(link.quest_id) else { continue };
            let state = match g.quests.get(q.id as u32) {
                Some(st) => {
                    let done = goals(q).is_some_and(|gl| is_complete(&gl, &st.counts));
                    if done && link.end {
                        MenuState::Finished
                    } else if link.start || link.progress || link.end {
                        MenuState::InProgress
                    } else {
                        continue;
                    }
                }
                None if link.start && g.quests.can_start(q, c.level, job_level, c.race) => MenuState::Offer,
                None => continue,
            };
            match entries.iter_mut().find(|(id, _)| *id == q.id) {
                Some(e) => e.1 = e.1.max(state),
                None => entries.push((q.id, state)),
            }
        }
        // Quêtes à rendre d'abord, puis en cours, puis proposées.
        entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        entries
            .into_iter()
            .filter_map(|(id, state)| {
                let q = book.get(id)?;
                Some((format!("QUEST|{}|{}", q.text_quest, state as u8), format!("{SHOW_PREFIX}{id}")))
            })
            .collect()
    }

    fn npc_links(&self, npc_handle: u32, quest: i32) -> Option<rz_rappelz::quests::QuestLink> {
        let (npc_id, _) = self.npc_at(npc_handle)?;
        self.ctx.data.quests.links_of(npc_id as i32).find(|l| l.quest_id == quest).copied()
    }

    /// Trigger de dialogue lié aux quêtes ; `false` s'il ne s'agit pas d'une quête.
    pub async fn quest_dialog(&mut self, trigger: &str) -> Result<bool> {
        let Some(npc_handle) = self.dialog_npc else { return Ok(false) };
        if let Some(code) = trigger.strip_prefix(SHOW_PREFIX) {
            if let Ok(code) = code.trim().parse::<i32>() {
                self.show_quest(npc_handle, code);
            }
            return Ok(true);
        }
        match QuestTrigger::parse(trigger) {
            Some(QuestTrigger::Start(code)) => self.start_quest(npc_handle, code).await?,
            Some(QuestTrigger::End { code, reward_index }) => self.end_quest(npc_handle, code, reward_index).await?,
            None => return Ok(false),
        }
        Ok(true)
    }

    /// Dialogue d'une quête : proposition (START/REJECT), suivi (OK) ou rendu (REWARD).
    fn show_quest(&mut self, npc_handle: u32, code: i32) {
        let Some(link) = self.npc_links(npc_handle, code) else { return };
        let Some(q) = self.ctx.data.quests.get(code).cloned() else { return };
        let Some(g) = &self.in_game else { return };
        let c = &g.character;
        let job_level = c.extra_i64("jlv", 1).clamp(0, u16::MAX as i64) as u16;
        let title = self.npc_at(npc_handle).map(|(_, n)| n).unwrap_or_default();
        let (text_id, menu) = match g.quests.get(code as u32) {
            Some(st) if link.end && goals(&q).is_some_and(|gl| is_complete(&gl, &st.counts)) => {
                // REWARD : le client lit le trigger comme le code de quête et ouvre la fenêtre
                // de récompense, qui renvoie `end_quest( code, index )`.
                (q.text_summary, vec![("REWARD".to_string(), code.to_string()), ("REJECT".to_string(), "close".to_string())])
            }
            Some(_) => (q.text_summary, vec![("OK".to_string(), "close".to_string())]),
            None if link.start && g.quests.can_start(&q, c.level, job_level, c.race) => (
                q.text_summary,
                vec![("START".to_string(), QuestTrigger::start(code)), ("REJECT".to_string(), "close".to_string())],
            ),
            None => return,
        };
        self.send(&game::ScDialog {
            dialog_type: game::dialog_type::NPC,
            npc_handle,
            title,
            text: format!("QUEST|{code}|{text_id}"),
            menu,
        });
    }

    async fn start_quest(&mut self, npc_handle: u32, code: i32) -> Result<()> {
        if !self.npc_links(npc_handle, code).is_some_and(|l| l.start) {
            return Ok(());
        }
        let Some(q) = self.ctx.data.quests.get(code).cloned() else { return Ok(()) };
        let Some(gl) = goals(&q) else { return Ok(()) };
        let Some((items, job_level)) = self.holder_state() else { return Ok(()) };
        let (guid, handle, allowed) = {
            let g = self.in_game.as_ref().unwrap();
            let c = &g.character;
            (c.guid, g.handle, g.quests.can_start(&q, c.level, job_level, c.race))
        };
        if !allowed {
            return Ok(());
        }
        self.dialog_npc = None;
        self.add_quest(&q, &gl, &items, job_level, guid, handle).await
    }

    async fn add_quest(&mut self, q: &QuestDef, gl: &[Goal], items: &[InvItem], job_level: u16, guid: u32, handle: u32) -> Result<()> {
        let mut st = QuestStatus { quest: q.id as u32, ..Default::default() };
        refresh(gl, &mut st.counts, &Inventory { items, job_level });
        st.status = if is_complete(gl, &st.counts) {
            rz_database::quests::STATUS_COMPLETE
        } else {
            rz_database::quests::STATUS_INCOMPLETE
        };
        rz_database::quests::upsert(&self.ctx.chars, guid, &st).await?;
        self.in_game.as_mut().unwrap().quests.active.push(st);
        self.send_quest_list();
        let title = self.ctx.data.strings.get(q.text_quest).to_string();
        self.ctx.world.lock().unwrap().notify(handle, &format!("Quête acceptée : {title}"));
        Ok(())
    }

    /// `.quest add` : ajoute une quête sans PNJ ni conditions (niveau, race, quêtes préalables).
    pub async fn gm_add_quest(&mut self, code: i32) -> Result<std::result::Result<(), &'static str>> {
        let Some(q) = self.ctx.data.quests.get(code).cloned() else { return Ok(Err("inconnue")) };
        let Some(gl) = goals(&q) else { return Ok(Err("type de quête non pris en charge")) };
        let Some((items, job_level)) = self.holder_state() else { return Ok(Err("hors du monde")) };
        let g = self.in_game.as_ref().unwrap();
        if g.quests.get(code as u32).is_some() {
            return Ok(Err("déjà en cours"));
        }
        if g.quests.active.len() >= MAX_ACTIVE {
            return Ok(Err("journal plein"));
        }
        let (guid, handle) = (g.character.guid, g.handle);
        self.add_quest(&q, &gl, &items, job_level, guid, handle).await?;
        Ok(Ok(()))
    }

    /// TS_CS_DROP_QUEST : abandon d'une quête en cours.
    pub async fn drop_quest(&mut self, code: i32) -> Result<()> {
        let Some(g) = &self.in_game else { return Ok(()) };
        let guid = g.character.guid;
        if g.quests.get(code as u32).is_none() {
            return Ok(());
        }
        rz_database::quests::remove(&self.ctx.chars, guid, code as u32).await?;
        self.in_game.as_mut().unwrap().quests.active.retain(|q| q.quest != code as u32);
        self.send_quest_list();
        Ok(())
    }

    /// Rendu : vérifie les objectifs, consomme les objets de quête, verse les récompenses.
    async fn end_quest(&mut self, npc_handle: u32, code: i32, reward_index: i32) -> Result<()> {
        if !self.npc_links(npc_handle, code).is_some_and(|l| l.end) {
            return Ok(());
        }
        self.refresh_quests().await?;
        let Some(q) = self.ctx.data.quests.get(code).cloned() else { return Ok(()) };
        let Some(gl) = goals(&q) else { return Ok(()) };
        let Some(g) = &self.in_game else { return Ok(()) };
        let handle = g.handle;
        let Some(st) = g.quests.get(code as u32) else { return Ok(()) };
        if !is_complete(&gl, &st.counts) {
            return Ok(());
        }
        // Objets remis : récompense fixe + récompense optionnelle choisie (index 0..5).
        let mut rewards = Vec::new();
        if q.default_reward.code > 0 {
            rewards.push((q.default_reward.code as u32, q.default_reward.quantity.max(1) as u32));
        }
        if let Some(r) = usize::try_from(reward_index).ok().and_then(|i| q.optional_rewards.get(i)).filter(|r| r.code > 0) {
            rewards.push((r.code as u32, r.quantity.max(1) as u32));
        }
        let free = (crate::gameplay::BAG_CAPACITY as usize).saturating_sub(g.items.iter().filter(|i| i.in_bag()).count());
        if free < rewards.len() {
            self.ctx.world.lock().unwrap().notify(handle, "Le sac est plein : libérez de la place pour rendre la quête.");
            return Ok(());
        }
        let Some(snapshot) = self.character_snapshot() else {
            self.detach_world_without_save();
            anyhow::bail!("quest reward lost world ownership; closing without a stale save");
        };
        let (candidate, leveled) = crate::gameplay::reward_candidate(snapshot, q.exp, q.jp, q.gold);
        // Récompense et passage en « rendue » validés ensemble (une seule transaction).
        if let Err(error) = rz_database::quests::complete(&self.ctx.chars, &candidate, code as u32).await {
            self.detach_world_without_save();
            return Err(error.context("quest completion save failed; closing without a stale disconnect save"));
        }
        let (level, xp, money, jp) = (candidate.level, candidate.xp, candidate.money, candidate.extra_i64("jp", 0));
        let jlv = candidate.extra_i64("jlv", 1) as i32;
        {
            let g = self.in_game.as_mut().unwrap();
            g.character = candidate;
            g.quests.active.retain(|s| s.quest != code as u32);
            g.quests.rewarded.insert(code as u32);
        }
        self.dialog_npc = None;
        for (item, count) in consumed_items(&gl) {
            self.consume_item(item, count).await?;
        }
        for (item, count) in rewards {
            self.give_item(item, count).await?;
        }
        self.send(&game::ScExpUpdate { handle, exp: xp as i64, jp });
        self.send(&game::ScGoldUpdate { gold: money as i64, chaos: 0 });
        self.send_quest_list();
        let title = self.ctx.data.strings.get(q.text_quest).to_string();
        let mut w = self.ctx.world.lock().unwrap();
        w.notify(handle, &format!("Quête terminée : {title} (+{} EXP, +{} JP, +{} rupees)", q.exp, q.jp, q.gold));
        if leveled {
            w.set_player_level(handle, level, jlv);
            w.notify(handle, &format!("Niveau {level} atteint !"));
        }
        Ok(())
    }

    /// Retire `count` exemplaires d'un objet du sac (piles entamées ou supprimées).
    async fn consume_item(&mut self, item: u32, mut count: u32) -> Result<()> {
        let ctx = self.ctx.clone();
        let Some(g) = &mut self.in_game else { return Ok(()) };
        let owner = g.character.guid;
        let mut updates = Vec::new();
        for it in g.items.iter_mut().filter(|i| i.entry == item && i.in_bag()) {
            if count == 0 {
                break;
            }
            let take = it.count.min(count);
            it.count -= take;
            count -= take;
            updates.push((it.guid, it.handle(), it.count));
        }
        g.items.retain(|i| i.count > 0);
        for (guid, handle, left) in updates {
            if left == 0 {
                characters::delete_item(&ctx.chars, owner, guid).await?;
                self.send(&game::Handle::<{ game::id::TS_SC_DESTROY_ITEM }>(handle));
            } else {
                characters::set_item_count(&ctx.chars, guid, left).await?;
                self.send(&game::ScUpdateItemCount { item_handle: handle, count: left as i64 });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quest(kind: i32, values: &[i32]) -> QuestDef {
        let mut q = QuestDef { kind, ..Default::default() };
        q.values[..values.len()].copy_from_slice(values);
        q
    }

    struct Bag(Vec<(u32, u32)>, u16);
    impl Holder for Bag {
        fn item_count(&self, item: u32) -> u32 {
            self.0.iter().filter(|(i, _)| *i == item).map(|(_, c)| c).sum()
        }
        fn job_level(&self) -> u16 {
            self.1
        }
    }

    #[test]
    fn kill_individual_pairs_from_quest_1034() {
        let g = goals(&quest(102, &[11007, 2, 12006, 3, 14004, 5])).unwrap();
        assert_eq!(g.len(), 3);
        let mut c = [0; PROGRESS_SLOTS];
        assert!(on_kill(&g, &mut c, 12006));
        assert!(!on_kill(&g, &mut c, 99));
        for _ in 0..10 {
            on_kill(&g, &mut c, 11007);
        }
        assert_eq!(&c[..3], &[2, 1, 0]);
        assert!(!is_complete(&g, &c));
        for _ in 0..2 {
            on_kill(&g, &mut c, 12006);
        }
        for _ in 0..5 {
            on_kill(&g, &mut c, 14004);
        }
        assert!(is_complete(&g, &c));
    }

    #[test]
    fn empty_middle_pair_keeps_counter_positions() {
        // status[i] ↔ (value(2i+1), value(2i+2)) côté client : la 3e paire reste en position 2.
        let g = goals(&quest(102, &[11007, 2, 0, 0, 14004, 1])).unwrap();
        assert_eq!(g.len(), 3);
        assert_eq!(g[1], Goal::Unused);
        let mut c = [0; PROGRESS_SLOTS];
        on_kill(&g, &mut c, 14004);
        assert_eq!(&c[..3], &[0, 0, 1]);
        on_kill(&g, &mut c, 11007);
        on_kill(&g, &mut c, 11007);
        assert!(is_complete(&g, &c));
        let h = goals(&quest(103, &[1, 1, 0, 0, 0, 0])).unwrap();
        assert_eq!(h.len(), 1, "paires vides finales retirées");
    }

    #[test]
    fn kill_total_counts_any_listed_monster() {
        let g = goals(&quest(101, &[1003, 3, 2003, 0, 3004, 0])).unwrap();
        let mut c = [0; PROGRESS_SLOTS];
        on_kill(&g, &mut c, 1003);
        on_kill(&g, &mut c, 2003);
        on_kill(&g, &mut c, 3004);
        on_kill(&g, &mut c, 3004);
        assert_eq!(c[0], 3);
        assert!(is_complete(&g, &c));
    }

    #[test]
    fn collect_follows_inventory_and_is_consumed() {
        let g = goals(&quest(103, &[1000072, 1, 1000073, 2])).unwrap();
        let mut c = [0; PROGRESS_SLOTS];
        assert!(refresh(&g, &mut c, &Bag(vec![(1000072, 5), (1000073, 1)], 1)));
        assert_eq!(&c[..2], &[1, 1]);
        assert!(!is_complete(&g, &c));
        refresh(&g, &mut c, &Bag(vec![(1000072, 1), (1000073, 2)], 1));
        assert!(is_complete(&g, &c));
        assert_eq!(consumed_items(&g), vec![(1000072, 1), (1000073, 2)]);
    }

    #[test]
    fn hunt_drops_only_from_listed_monsters_until_done() {
        let g = goals(&quest(106, &[1000000, 3, 0, 0, 0, 0, 1003, 2003, 3004])).unwrap();
        let mut c = [0; PROGRESS_SLOTS];
        assert_eq!(hunt_drop(&g, &c, 2003), Some(1000000));
        assert_eq!(hunt_drop(&g, &c, 42), None);
        refresh(&g, &mut c, &Bag(vec![(1000000, 3)], 1));
        assert_eq!(hunt_drop(&g, &c, 2003), None);
        assert!(is_complete(&g, &c));
    }

    #[test]
    fn offer_rules_level_rewarded_repeatable_and_prerequisites() {
        let mut q = quest(103, &[1000050, 10]);
        q.id = 1193;
        q.limit_level = 17;
        let mut log = QuestLog::default();
        assert!(!log.can_start(&q, 16, 1, 4), "niveau insuffisant");
        assert!(log.can_start(&q, 17, 1, 4));
        log.rewarded.insert(1193);
        assert!(!log.can_start(&q, 17, 1, 4), "déjà rendue");
        q.repeatable = true;
        assert!(log.can_start(&q, 17, 1, 4), "répétable");
        log.active.push(QuestStatus { quest: 1193, ..Default::default() });
        assert!(!log.can_start(&q, 17, 1, 4), "déjà en cours");
        let mut chained = quest(102, &[11007, 2]);
        chained.id = 1034;
        chained.forequests = [1024, 0, 0];
        assert!(!log.can_start(&chained, 17, 1, 4));
        log.rewarded.insert(1024);
        assert!(log.can_start(&chained, 17, 1, 4));
    }

    #[test]
    fn contact_is_immediately_complete_and_unknown_types_unsupported() {
        let g = goals(&quest(401, &[])).unwrap();
        assert!(is_complete(&g, &[0; PROGRESS_SLOTS]));
        assert!(goals(&quest(901, &[101, 5, 15, 100])).is_none());
    }
}
