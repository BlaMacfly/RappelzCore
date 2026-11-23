//! Quêtes : `Quest.dbc` (QuestResource, db_quest.rdb) et `QuestLink.dbc` (PNJ ↔ quête).
//! Champs : `rdb_schemas.rs` ; usage client et protocole : `docs/re/quest_protocol.md`.

use std::collections::HashMap;
use std::path::Path;

use crate::dbcstore::{NamedDbc, Row, StoreError};

/// Nombre de valeurs de paramétrage (`value1..12`) d'une quête.
pub const QUEST_VALUES: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RewardItem {
    pub code: i32,
    pub level: i32,
    pub quantity: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuestDef {
    pub id: i32,
    pub text_quest: i32,
    pub text_summary: i32,
    pub text_status: i32,
    pub limit_level: i32,
    pub limit_job_level: i32,
    pub limit_max_level: i32,
    pub limit_max_job_level: i32,
    /// Races autorisées (deva, asura, gaia) ; toutes à `false` = aucune restriction.
    pub races: [bool; 3],
    /// Familles de métier autorisées (fighter, hunter, magician, summoner).
    pub job_kinds: [bool; 4],
    pub limit_job: i32,
    pub repeatable: bool,
    pub time_limit_type: i32,
    pub time_limit: i32,
    /// Type de quête (101, 102, 103, 106, 401, 701, 901…).
    pub kind: i32,
    pub values: [i32; QUEST_VALUES],
    pub drop_group_id: i32,
    pub exp: i64,
    pub jp: i64,
    pub gold: i64,
    pub default_reward: RewardItem,
    pub optional_rewards: [RewardItem; 6],
    pub forequests: [i32; 3],
    pub or_flag: bool,
}

impl QuestDef {
    fn from_row(r: Row<'_>) -> Self {
        // Colonnes `default_reward_id` / `optional_reward_id1` : suffixe numérique après le nom.
        let reward = |p: &str, n: &str| RewardItem {
            code: r.int(&format!("{p}_id{n}")),
            level: r.int(&format!("{p}_level{n}")),
            quantity: r.int(&format!("{p}_quantity{n}")),
        };
        Self {
            id: r.id(),
            text_quest: r.int("text_id_quest"),
            text_summary: r.int("text_id_summary"),
            text_status: r.int("text_id_status"),
            limit_level: r.int("limit_level"),
            limit_job_level: r.int("limit_job_level"),
            limit_max_level: r.int("limit_max_level"),
            limit_max_job_level: r.int("limit_max_job_level"),
            races: [r.int("limit_deva") != 0, r.int("limit_asura") != 0, r.int("limit_gaia") != 0],
            job_kinds: [
                r.int("limit_fighter") != 0,
                r.int("limit_hunter") != 0,
                r.int("limit_magician") != 0,
                r.int("limit_summoner") != 0,
            ],
            limit_job: r.int("limit_job"),
            repeatable: r.int("repeatable") != 0,
            time_limit_type: r.int("time_limit_type"),
            time_limit: r.int("time_limit"),
            kind: r.int("type"),
            values: std::array::from_fn(|i| r.int(&format!("value{}", i + 1))),
            drop_group_id: r.int("drop_group_id"),
            exp: r.int("exp") as i64,
            jp: r.int("jp") as i64,
            gold: r.int("gold") as i64,
            default_reward: reward("default_reward", ""),
            optional_rewards: std::array::from_fn(|i| reward("optional_reward", &(i + 1).to_string())),
            forequests: [r.int("forequest1"), r.int("forequest2"), r.int("forequest3")],
            or_flag: r.int("or_flag") != 0,
        }
    }

    /// Race autorisée ? `race` au sens Rappelz (3 gaia, 4 deva, 5 asura).
    pub fn allows_race(&self, race: u8) -> bool {
        if self.races.iter().all(|b| !b) || self.races.iter().all(|b| *b) {
            return true;
        }
        match race {
            4 => self.races[0],
            5 => self.races[1],
            3 => self.races[2],
            _ => true,
        }
    }

    /// Quêtes préalables remplies (`or_flag` : une seule suffit) ?
    pub fn prerequisites_met(&self, rewarded: impl Fn(i32) -> bool) -> bool {
        let required: Vec<i32> = self.forequests.iter().copied().filter(|q| *q > 0).collect();
        if required.is_empty() {
            return true;
        }
        if self.or_flag {
            required.iter().any(|q| rewarded(*q))
        } else {
            required.iter().all(|q| rewarded(*q))
        }
    }
}

/// Lien PNJ ↔ quête : le PNJ propose (`start`), suit (`progress`) ou reçoit (`end`) la quête.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuestLink {
    pub npc_id: i32,
    pub quest_id: i32,
    pub start: bool,
    pub progress: bool,
    pub end: bool,
}

#[derive(Clone, Debug, Default)]
pub struct QuestBook {
    pub quests: HashMap<i32, QuestDef>,
    pub links: Vec<QuestLink>,
}

impl QuestBook {
    pub fn open(dbc_dir: &Path) -> Result<Self, StoreError> {
        let quests = NamedDbc::open(dbc_dir, "quest")?;
        let links = NamedDbc::open(dbc_dir, "questlink")?;
        Ok(Self::from_tables(&quests, &links))
    }

    pub fn from_tables(quests: &NamedDbc, links: &NamedDbc) -> Self {
        let mut map = HashMap::new();
        for r in quests.rows() {
            map.entry(r.id()).or_insert_with(|| QuestDef::from_row(r));
        }
        let links = links
            .rows()
            .map(|r| QuestLink {
                npc_id: r.int("npc_id"),
                quest_id: r.int("quest_id"),
                start: r.int("flag_start") != 0,
                progress: r.int("flag_progress") != 0,
                end: r.int("flag_end") != 0,
            })
            .collect();
        Self { quests: map, links }
    }

    pub fn get(&self, id: i32) -> Option<&QuestDef> {
        self.quests.get(&id)
    }

    /// Liens d'un PNJ (`NpcResource.id`).
    pub fn links_of(&self, npc_id: i32) -> impl Iterator<Item = &QuestLink> {
        self.links.iter().filter(move |l| l.npc_id == npc_id)
    }

    pub fn is_empty(&self) -> bool {
        self.quests.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(forequests: [i32; 3], or_flag: bool) -> QuestDef {
        QuestDef { forequests, or_flag, ..Default::default() }
    }

    #[test]
    fn prerequisites_and_or_semantics() {
        let done = |q: i32| q == 10;
        assert!(def([0, 0, 0], false).prerequisites_met(done));
        assert!(def([10, 0, 0], false).prerequisites_met(done));
        assert!(!def([10, 11, 0], false).prerequisites_met(done));
        assert!(def([10, 11, 0], true).prerequisites_met(done));
        assert!(!def([12, 11, 0], true).prerequisites_met(done));
    }

    #[test]
    fn race_filter() {
        let mut q = QuestDef::default();
        assert!(q.allows_race(3) && q.allows_race(4) && q.allows_race(5));
        q.races = [true, false, false];
        assert!(q.allows_race(4) && !q.allows_race(5) && !q.allows_race(3));
    }

    /// Lit les vraies tables converties si elles sont présentes (non publiées).
    #[test]
    fn real_tables_when_available() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        let Ok(book) = QuestBook::open(&dir) else { return };
        let q = book.get(1034).expect("quête 1034");
        assert_eq!(q.kind, 102);
        assert_eq!(&q.values[..6], &[11007, 2, 12006, 3, 14004, 5]);
        assert!(book.links.len() > 1000);
    }
}
