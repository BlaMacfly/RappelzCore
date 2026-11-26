//! Objets consommables (`Item.dbc`, type 3) : effets établis par l'info-bulle et les données
//! (docs/re/item_protocol.md). Un objet dont un seul effet n'est pas compris est refusé.

use std::collections::HashMap;
use std::path::Path;

use crate::dbcstore::{NamedDbc, Row, StoreError};

/// `type` de db_item des consommables.
const CONSUMABLE: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseEffect {
    /// « Restaure instantanément N HP ».
    Heal(i32),
    /// « Restaure instantanément N MP ».
    Mana(i32),
    /// État de l'objet (`state_id`, niveau, durée en centièmes).
    State { id: u32, level: u16, duration: u32 },
    /// Paquet : N exemplaires d'un autre objet.
    Unpack { item: u32, count: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Consumable {
    pub effects: Vec<UseEffect>,
    /// Centièmes de seconde.
    pub cooldown: u32,
    /// Recharge commune (`cool_time_group`), 0 = propre à l'objet.
    pub group: u32,
    pub min_level: i32,
    pub max_level: i32,
}

fn definition(r: Row<'_>) -> Option<Consumable> {
    if r.int("type") != CONSUMABLE || (0..4).any(|i| r.int(&format!("base_type_{i}")) != 0) {
        return None;
    }
    let mut effects = Vec::new();
    for i in 0..4 {
        let (kind, v1, v2) = (r.int(&format!("opt_type_{i}")), r.float(&format!("opt_var1_{i}")), r.float(&format!("opt_var2_{i}")));
        let whole = |v: f32| (v.fract() == 0.0 && v > 0.0 && v < 1e9).then_some(v as i64);
        effects.push(match kind {
            0 => continue,
            1 if v2 == 0.0 => UseEffect::Heal(whole(v1)? as i32),
            2 if v2 == 0.0 => UseEffect::Mana(whole(v1)? as i32),
            6 => {
                let id = u32::try_from(r.int("state_id")).ok().filter(|i| *i > 0)?;
                let level = u16::try_from(r.int("state_level")).ok().filter(|l| *l > 0)?;
                let seconds = u32::try_from(r.int("state_time")).ok().filter(|s| (1..=86_400).contains(s))?;
                UseEffect::State { id, level, duration: seconds * 100 }
            }
            94 => UseEffect::Unpack { item: whole(v1)? as u32, count: whole(v2)?.min(100_000) as u32 },
            _ => return None,
        });
    }
    // Sans effet, ou un état sans l'option 6 : rien n'est établi.
    if effects.is_empty() || (r.int("state_id") != 0 && !effects.iter().any(|e| matches!(e, UseEffect::State { .. }))) {
        return None;
    }
    Some(Consumable {
        effects,
        cooldown: u32::try_from(r.int("cool_time")).ok()?.min(86_400) * 100,
        group: u32::try_from(r.int("cool_time_group")).ok()?,
        min_level: r.int("use_min_level"),
        max_level: r.int("use_max_level"),
    })
}

#[derive(Clone, Debug, Default)]
pub struct ConsumableBook {
    items: HashMap<u32, Consumable>,
}

impl ConsumableBook {
    pub fn open(dbc_dir: &Path) -> Result<Self, StoreError> {
        let table = NamedDbc::open(dbc_dir, "item")?;
        Ok(Self { items: table.rows().filter_map(|r| Some((u32::try_from(r.id()).ok()?, definition(r)?))).collect() })
    }

    pub fn get(&self, id: u32) -> Option<&Consumable> {
        self.items.get(&id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vraies tables converties, si présentes (non publiées).
    #[test]
    fn real_consumables_when_available() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        let Ok(book) = ConsumableBook::open(&dir) else { return };
        assert!(book.len() > 500, "{}", book.len());
        // 602004 Potion Rouge Niv.4 : 900 PV, recharge 10 s du groupe 1.
        let p = book.get(602004).unwrap();
        assert_eq!((p.effects.as_slice(), p.cooldown, p.group), (&[UseEffect::Heal(900)][..], 1000, 1));
        assert_eq!(book.get(602201).unwrap().effects, vec![UseEffect::Mana(150)]);
        assert_eq!(book.get(2011110).unwrap().effects, vec![UseEffect::Heal(700), UseEffect::Mana(700)]);
        // 601102 : paquet de 1 000 flèches.
        assert_eq!(book.get(601102).unwrap().effects, vec![UseEffect::Unpack { item: 601001, count: 1000 }]);
        // 491008 (compétence d'apprivoisement, type 5) et 960023 (coloration) : refusés.
        assert!(book.get(491008).is_none() && book.get(960023).is_none());
    }
}
