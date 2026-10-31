//! Typed skill learning from converted Skill/SkillTree/Job data.
//! This covers learning only; casting costs/effects and inherited job trees are separate.
use crate::dbcstore::{NamedDbc, StoreError};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

pub const MAX_SKILL_LEVEL: u8 = 50;
pub const MAX_SKILLS: usize = 512;

#[derive(Clone, Debug)]
pub struct SkillDefinition {
    pub id: u32,
    pub valid: bool,
    pub passive: bool,
    pub learning_jp: [i64; 50],
}
#[derive(Clone, Debug, PartialEq)]
pub struct LearningRule {
    pub tree: i32,
    pub skill: u32,
    pub group: i32,
    pub min: u8,
    pub max: u8,
    pub player_level: u16,
    pub job_level: u16,
    pub ratio: f32,
    pub prerequisites: [(u32, u8); 3],
    pub enhance_min: i32,
    pub enhance_max: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LearnStatus {
    Ready,
    NotEnoughJp,
    Prerequisite,
    JobLevel,
    PlayerLevel,
    Maximum,
    UnsupportedRule,
    InvalidData,
    WrongJob,
    UnknownSkill,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearningOffer {
    pub skill: u32,
    pub learned: u8,
    pub next: u8,
    pub maximum: u8,
    pub cost: Option<i64>,
    pub player_level: u16,
    pub job_level: u16,
    pub prerequisites: [(u32, u8); 3],
    pub passive: bool,
    pub status: LearnStatus,
}
#[derive(Default)]
pub struct SkillBook {
    definitions: HashMap<u32, SkillDefinition>,
    rules: HashMap<i32, BTreeMap<u32, Vec<LearningRule>>>,
    jobs: HashMap<i32, i32>,
}

impl SkillBook {
    pub fn open(directory: &Path) -> Result<Self, StoreError> {
        let definitions = NamedDbc::open(directory, "skill")?;
        let rules = NamedDbc::open(directory, "skilltree")?;
        let jobs = NamedDbc::open(directory, "job")?;
        let definitions = definitions
            .rows()
            .filter(|r| r.id() > 0)
            .map(|r| SkillDefinition {
                id: r.id() as u32,
                valid: r.int("is_valid") == 1,
                passive: r.int("is_passive") != 0,
                learning_jp: std::array::from_fn(|i| i64::from(r.int(&format!("jp_{:02}", i + 1)))),
            })
            .collect();
        // Never use by_id here: many rows share the same tree and skill (level tiers).
        let rules = rules
            .rows()
            .filter_map(|r| {
                let min = u8::try_from(r.int("min_skill_lv")).ok()?;
                let max = u8::try_from(r.int("max_skill_lv")).ok()?;
                let player_level = u16::try_from(r.int("lv")).ok()?;
                let job_level = u16::try_from(r.int("job_lv")).ok()?;
                let skill = u32::try_from(r.int("skill_id")).ok()?;
                let mut prerequisites = [(0, 0); 3];
                for (i, p) in prerequisites.iter_mut().enumerate() {
                    *p = (
                        u32::try_from(r.int(&format!("need_skill_id_{}", i + 1))).ok()?,
                        u8::try_from(r.int(&format!("need_skill_lv_{}", i + 1))).ok()?,
                    );
                }
                Some(LearningRule {
                    tree: r.int("skill_tree_id"),
                    skill,
                    group: r.int("skill_group_id"),
                    min,
                    max,
                    player_level,
                    job_level,
                    ratio: r.float("jp_ratio"),
                    prerequisites,
                    enhance_min: r.int("cenhance_min"),
                    enhance_max: r.int("cenhance_max"),
                })
            })
            .collect();
        let jobs = jobs
            .rows()
            .map(|r| (r.id(), r.int("skill_tree_id")))
            .collect();
        Ok(Self::from_parts(definitions, rules, jobs))
    }
    pub fn from_parts(
        definitions: Vec<SkillDefinition>,
        rules: Vec<LearningRule>,
        jobs: Vec<(i32, i32)>,
    ) -> Self {
        let mut book = Self {
            definitions: definitions.into_iter().map(|s| (s.id, s)).collect(),
            jobs: jobs.into_iter().collect(),
            ..Default::default()
        };
        for rule in rules {
            let entries = book
                .rules
                .entry(rule.tree)
                .or_default()
                .entry(rule.skill)
                .or_default();
            if !entries.contains(&rule) {
                entries.push(rule);
            }
        }
        book
    }
    pub fn counts(&self) -> (usize, usize, usize) {
        (
            self.definitions.len(),
            self.rules
                .values()
                .flat_map(|t| t.values())
                .map(Vec::len)
                .sum(),
            self.jobs.len(),
        )
    }
    pub fn has_job(&self, job: i32) -> bool {
        self.jobs
            .get(&job)
            .is_some_and(|tree| self.rules.contains_key(tree))
    }
    pub fn offer(
        &self,
        job: i32,
        player_level: u8,
        job_level: i32,
        jp: i64,
        learned: &BTreeMap<u32, u8>,
        skill: u32,
    ) -> LearningOffer {
        let current = learned.get(&skill).copied().unwrap_or(0);
        let mut base = LearningOffer {
            skill,
            learned: current,
            next: current
                .checked_add(1)
                .filter(|n| *n <= MAX_SKILL_LEVEL)
                .unwrap_or(0),
            maximum: current,
            cost: None,
            player_level: 0,
            job_level: 0,
            prerequisites: [(0, 0); 3],
            passive: false,
            status: LearnStatus::UnknownSkill,
        };
        let Some(definition) = self.definitions.get(&skill) else {
            return base;
        };
        base.passive = definition.passive;
        if !definition.valid || jp < 0 || current > MAX_SKILL_LEVEL {
            base.status = LearnStatus::InvalidData;
            return base;
        }
        let Some(rules) = self
            .jobs
            .get(&job)
            .and_then(|tree| self.rules.get(tree))
            .and_then(|tree| tree.get(&skill))
        else {
            base.status = LearnStatus::WrongJob;
            return base;
        };
        base.maximum = rules
            .iter()
            .map(|r| r.max.min(MAX_SKILL_LEVEL))
            .max()
            .unwrap_or(current)
            .max(current);
        if base.next == 0 || base.next > base.maximum {
            base.status = LearnStatus::Maximum;
            return base;
        }
        let mut candidates = Vec::new();
        for rule in rules
            .iter()
            .filter(|r| r.min <= base.next && base.next <= r.max)
        {
            let mut offer = base.clone();
            offer.player_level = rule.player_level;
            offer.job_level = rule.job_level;
            offer.prerequisites = rule.prerequisites;
            if rule
                .prerequisites
                .iter()
                .any(|(id, level)| (*id == 0) != (*level == 0) || *level > MAX_SKILL_LEVEL)
            {
                offer.status = LearnStatus::InvalidData;
            } else if rule.group != 0 || rule.enhance_min > 0 || rule.enhance_max < 0 {
                offer.status = LearnStatus::UnsupportedRule;
            } else {
                let raw = definition.learning_jp[(base.next - 1) as usize];
                let scaled = raw as f64 * f64::from(rule.ratio);
                if raw < 0
                    || !rule.ratio.is_finite()
                    || rule.ratio < 0.0
                    || !scaled.is_finite()
                    || scaled >= i64::MAX as f64
                {
                    offer.status = LearnStatus::InvalidData;
                } else {
                    offer.cost = Some(scaled.trunc() as i64);
                    offer.status = if u16::from(player_level) < rule.player_level {
                        LearnStatus::PlayerLevel
                    } else if job_level < i32::from(rule.job_level) {
                        LearnStatus::JobLevel
                    } else if rule.prerequisites.iter().any(|(id, level)| {
                        *id != 0 && learned.get(id).copied().unwrap_or(0) < *level
                    }) {
                        LearnStatus::Prerequisite
                    } else if jp < offer.cost.unwrap() {
                        LearnStatus::NotEnoughJp
                    } else {
                        LearnStatus::Ready
                    };
                }
            }
            candidates.push(offer);
        }
        candidates.sort_by_key(|o| {
            (
                o.status,
                o.player_level,
                o.job_level,
                o.cost,
                o.prerequisites,
            )
        });
        if let Some(offer) = candidates.first() {
            let mut result = offer.clone();
            // Conflicting eligible costs are ambiguous data; never pick a cheaper branch silently.
            if result.status == LearnStatus::Ready
                && candidates
                    .iter()
                    .any(|o| o.status == LearnStatus::Ready && o.cost != result.cost)
            {
                result.status = LearnStatus::InvalidData;
                result.cost = None;
            }
            result
        } else {
            base.status = LearnStatus::UnsupportedRule;
            base
        }
    }
    pub fn catalog(
        &self,
        job: i32,
        player_level: u8,
        job_level: i32,
        jp: i64,
        learned: &BTreeMap<u32, u8>,
    ) -> Vec<LearningOffer> {
        let mut ids: BTreeSet<u32> = self
            .jobs
            .get(&job)
            .and_then(|t| self.rules.get(t))
            .map(|t| t.keys().copied().collect())
            .unwrap_or_default();
        ids.extend(learned.keys().copied());
        ids.into_iter()
            .take(MAX_SKILLS + 1)
            .map(|id| self.offer(job, player_level, job_level, jp, learned, id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(skill: u32) -> LearningRule {
        LearningRule {
            tree: 10,
            skill,
            group: 0,
            min: 1,
            max: 3,
            player_level: 1,
            job_level: 1,
            ratio: 1.0,
            prerequisites: [(0, 0); 3],
            enhance_min: 0,
            enhance_max: 5,
        }
    }
    fn book(rules: Vec<LearningRule>) -> SkillBook {
        SkillBook::from_parts(
            vec![
                SkillDefinition {
                    id: 42,
                    valid: true,
                    passive: false,
                    learning_jp: [10; 50],
                },
                SkillDefinition {
                    id: 43,
                    valid: true,
                    passive: true,
                    learning_jp: [3; 50],
                },
            ],
            rules,
            vec![(100, 10)],
        )
    }
    #[test]
    fn learning_cost_and_requirements_are_data_backed_with_zero_and_wide_jp() {
        let mut r = rule(42);
        r.ratio = 1.25;
        r.player_level = 2;
        r.job_level = 3;
        r.prerequisites[0] = (43, 1);
        let b = book(vec![r]);
        let mut known = BTreeMap::new();
        assert_eq!(
            b.offer(100, 1, 3, 100, &known, 42).status,
            LearnStatus::PlayerLevel
        );
        assert_eq!(
            b.offer(100, 2, 1, 100, &known, 42).status,
            LearnStatus::JobLevel
        );
        assert_eq!(
            b.offer(100, 2, 3, 100, &known, 42).status,
            LearnStatus::Prerequisite
        );
        known.insert(43, 1);
        let o = b.offer(100, 2, 3, 11, &known, 42);
        assert_eq!((o.status, o.cost), (LearnStatus::NotEnoughJp, Some(12)));
        assert_eq!(
            b.offer(100, 2, 3, i64::MAX, &known, 42).status,
            LearnStatus::Ready
        );
        let mut free = rule(43);
        free.ratio = 0.0;
        let b = book(vec![free]);
        assert_eq!(b.offer(100, 1, 1, 0, &BTreeMap::new(), 43).cost, Some(0));
    }
    #[test]
    fn all_tier_rows_are_retained_instead_of_last_row_wins() {
        let mut low = rule(42);
        low.max = 1;
        let mut high = rule(42);
        high.max = 3;
        high.player_level = 10;
        let b = book(vec![high.clone(), low.clone(), low.clone()]);
        assert_eq!(b.counts().1, 2);
        assert_eq!(
            b.offer(100, 1, 1, 100, &BTreeMap::new(), 42).status,
            LearnStatus::Ready
        );
        assert_eq!(
            b.offer(100, 1, 1, 100, &BTreeMap::from([(42, 1)]), 42)
                .status,
            LearnStatus::PlayerLevel
        );
        assert_eq!(
            b.offer(100, 10, 1, 100, &BTreeMap::from([(42, 1)]), 42)
                .status,
            LearnStatus::Ready
        );
        assert_eq!(
            b.offer(100, 10, 1, 100, &BTreeMap::from([(42, 3)]), 42)
                .status,
            LearnStatus::Maximum
        );
    }
    #[test]
    fn unsupported_and_ambiguous_rules_and_invalid_costs_fail_closed() {
        let mut bad = rule(42);
        bad.group = 1;
        assert_eq!(
            book(vec![bad])
                .offer(100, 1, 1, 100, &BTreeMap::new(), 42)
                .status,
            LearnStatus::UnsupportedRule
        );
        let mut alternate = rule(42);
        alternate.ratio = 2.0;
        assert_eq!(
            book(vec![rule(42), alternate])
                .offer(100, 1, 1, 100, &BTreeMap::new(), 42)
                .status,
            LearnStatus::InvalidData
        );
        for ratio in [f32::NAN, f32::INFINITY, -1.0, f32::MAX] {
            let mut r = rule(42);
            r.ratio = ratio;
            assert_eq!(
                book(vec![r])
                    .offer(100, 1, 1, 100, &BTreeMap::new(), 42)
                    .status,
                LearnStatus::InvalidData
            );
        }
        assert_eq!(
            book(vec![rule(42)])
                .offer(200, 1, 1, 100, &BTreeMap::new(), 42)
                .status,
            LearnStatus::WrongJob
        );
    }
    #[test]
    fn available_local_skill_tables_load_and_keep_starter_job_mapping() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        if !path.join("Skill.dbc").exists() {
            return;
        }
        let b = SkillBook::open(&path).unwrap();
        assert!(b.counts().0 > 2000);
        assert!(b.counts().1 > 1000);
        let known = BTreeMap::new();
        let offer = b.offer(200, 1, 1, 4, &known, 1004);
        assert_eq!((offer.cost, offer.status), (Some(4), LearnStatus::Ready));
        assert_eq!(
            b.offer(200, 1, 3, 100, &known, 1003).status,
            LearnStatus::Prerequisite
        );
        assert_eq!(
            b.offer(200, 1, 3, 4, &BTreeMap::from([(1004, 1)]), 1003)
                .status,
            LearnStatus::Ready
        );
    }
}
