//! Butin des monstres avec les tables AzerothCore inchangées : `creature_loot_template`
//! (par entry de monstre) et `reference_loot_template` (DropGroup.dbc importé par rz-dbimport).
//!
//! Règles d'AzerothCore reprises : une ligne `GroupId` 0 est tirée indépendamment selon `Chance`
//! (en %) ; dans un groupe `GroupId` > 0, **au plus un** élément est retenu (chances cumulées,
//! puis tirage uniforme parmi les lignes à chance 0 si aucune n'est sortie) ; `Reference` ≠ 0
//! renvoie vers `reference_loot_template`, tiré `MinCount..=MaxCount` fois.

use std::collections::HashMap;

/// Profondeur maximale des références imbriquées (protège d'un cycle dans les données).
const MAX_DEPTH: u8 = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct LootRow {
    pub item: u32,
    pub reference: u32,
    pub chance: f32,
    pub group: u8,
    pub min: u8,
    pub max: u8,
}

#[derive(Clone, Debug, Default)]
pub struct LootStore {
    pub creature: HashMap<u32, Vec<LootRow>>,
    pub reference: HashMap<u32, Vec<LootRow>>,
}

/// Source d'aléa : `roll()` dans [0, 100), `range(a, b)` dans [a, b].
pub trait Dice {
    fn roll(&mut self) -> f32;
    fn range(&mut self, lo: u32, hi: u32) -> u32;
}

pub struct ThreadDice;

impl Dice for ThreadDice {
    fn roll(&mut self) -> f32 {
        rand::random::<f32>() * 100.0
    }
    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo { lo } else { lo + rand::random::<u32>() % (hi - lo + 1) }
    }
}

impl LootStore {
    pub async fn load(db: &rz_database::sqlx::MySqlPool) -> anyhow::Result<Self> {
        let mut store = Self::default();
        for (table, map) in [("creature_loot_template", &mut store.creature), ("reference_loot_template", &mut store.reference)] {
            for (entry, item, reference, chance, group, min, max) in rz_database::world::loot_rows(db, table).await? {
                let row = LootRow { item, reference: reference.unsigned_abs(), chance, group, min, max };
                map.entry(entry).or_default().push(row);
            }
        }
        Ok(store)
    }

    /// Objets (code, nombre) lâchés par le monstre `entry`.
    pub fn roll(&self, entry: u32, dice: &mut impl Dice) -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        if let Some(rows) = self.creature.get(&entry) {
            self.roll_rows(rows, dice, 0, &mut out);
        }
        out
    }

    fn roll_rows(&self, rows: &[LootRow], dice: &mut impl Dice, depth: u8, out: &mut Vec<(u32, u32)>) {
        if depth >= MAX_DEPTH {
            return;
        }
        let mut groups: Vec<u8> = Vec::new();
        for row in rows {
            if row.group == 0 {
                if dice.roll() < row.chance {
                    self.emit(row, dice, depth, out);
                }
            } else if !groups.contains(&row.group) {
                groups.push(row.group);
            }
        }
        for g in groups {
            let members: Vec<&LootRow> = rows.iter().filter(|r| r.group == g).collect();
            let r = dice.roll();
            let mut cumulative = 0.0;
            let mut picked = None;
            for m in members.iter().filter(|m| m.chance > 0.0) {
                cumulative += m.chance;
                if r < cumulative {
                    picked = Some(*m);
                    break;
                }
            }
            if picked.is_none() {
                let equal: Vec<&&LootRow> = members.iter().filter(|m| m.chance <= 0.0).collect();
                if !equal.is_empty() {
                    picked = Some(equal[dice.range(0, equal.len() as u32 - 1) as usize]);
                }
            }
            if let Some(m) = picked {
                self.emit(m, dice, depth, out);
            }
        }
    }

    fn emit(&self, row: &LootRow, dice: &mut impl Dice, depth: u8, out: &mut Vec<(u32, u32)>) {
        let times = dice.range(row.min.max(1) as u32, row.max.max(row.min).max(1) as u32);
        if row.reference != 0 {
            if let Some(sub) = self.reference.get(&row.reference) {
                for _ in 0..times {
                    self.roll_rows(sub, dice, depth + 1, out);
                }
            }
        } else if row.item != 0 {
            out.push((row.item, times));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dés prévisibles : suite de tirages fixée, `range` renvoie le minimum.
    struct Fixed(Vec<f32>);
    impl Dice for Fixed {
        fn roll(&mut self) -> f32 {
            if self.0.is_empty() { 99.9 } else { self.0.remove(0) }
        }
        fn range(&mut self, lo: u32, _hi: u32) -> u32 {
            lo
        }
    }

    fn row(item: u32, reference: u32, chance: f32, group: u8) -> LootRow {
        LootRow { item, reference, chance, group, min: 1, max: 1 }
    }

    #[test]
    fn independent_rows_follow_their_chance() {
        let mut s = LootStore::default();
        s.creature.insert(1, vec![row(10, 0, 50.0, 0), row(11, 0, 5.0, 0)]);
        assert_eq!(s.roll(1, &mut Fixed(vec![10.0, 10.0])), vec![(10, 1)]);
        assert_eq!(s.roll(1, &mut Fixed(vec![60.0, 1.0])), vec![(11, 1)]);
        assert!(s.roll(2, &mut Fixed(vec![])).is_empty());
    }

    #[test]
    fn group_picks_exactly_one_like_a_dropgroup() {
        // DropGroup -510000 : six objets à ~16,66 % chacun.
        let mut s = LootStore::default();
        s.reference.insert(510000, (0..6).map(|i| row(502001 + i, 0, 100.0 / 6.0, 1)).collect());
        s.creature.insert(7, vec![row(510000, 510000, 100.0, 0)]);
        assert_eq!(s.roll(7, &mut Fixed(vec![0.0, 40.0])), vec![(502003, 1)]);
        assert_eq!(s.roll(7, &mut Fixed(vec![0.0, 99.0])), vec![(502006, 1)]);
    }

    #[test]
    fn nested_references_and_cycles_are_bounded() {
        let mut s = LootStore::default();
        s.reference.insert(1, vec![row(2, 2, 100.0, 0)]);
        s.reference.insert(2, vec![row(1, 1, 100.0, 0), row(77, 0, 100.0, 0)]);
        s.creature.insert(5, vec![row(1, 1, 100.0, 0)]);
        let got = s.roll(5, &mut Fixed(vec![0.0; 64]));
        assert!(!got.is_empty() && got.iter().all(|&(i, _)| i == 77), "{got:?}");
        assert!(got.len() <= MAX_DEPTH as usize);
    }
}
