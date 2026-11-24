//! Créatures d'invocation : `Creature.dbc` (db_creature.rdb, docs/re/summon_protocol.md §7).

use std::collections::HashMap;
use std::path::Path;

use crate::dbcstore::{NamedDbc, StoreError};

/// Groupe d'objets des cartes de créature (`Item.group`, +0xc de l'ItemResource) [O].
pub const CARD_ITEM_GROUP: i32 = 13;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CreatureDef {
    pub id: u32,
    pub name_id: i32,
    /// Grade de la carte, 0..5.
    pub rank: u8,
    /// Stade d'évolution 1..3 (0 : entrée technique).
    pub form: u8,
    /// Id de la forme suivante, 0 = aucune.
    pub evolve_target: u32,
    /// Objet « carte vierge » associé ; quatre cartes génériques couvrent la plupart des créatures.
    pub card_id: u32,
    pub walk_speed: i32,
    pub run_speed: i32,
}

impl CreatureDef {
    /// Niveau maximal par stade, d'après la ferme à créatures du client (60 / 115 / 150) [O].
    pub fn max_level(&self) -> u16 {
        match self.form {
            0 | 1 => 60,
            2 => 115,
            _ => 150,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CreatureBook {
    pub creatures: HashMap<u32, CreatureDef>,
}

impl CreatureBook {
    pub fn open(dbc_dir: &Path) -> Result<Self, StoreError> {
        Ok(Self::from_table(&NamedDbc::open(dbc_dir, "creature")?))
    }

    pub fn from_table(table: &NamedDbc) -> Self {
        let mut creatures = HashMap::new();
        for r in table.rows() {
            let Ok(id) = u32::try_from(r.id()) else { continue };
            creatures.entry(id).or_insert_with(|| CreatureDef {
                id,
                name_id: r.int("name_id"),
                rank: r.int("rank").clamp(0, 255) as u8,
                form: r.int("form").clamp(0, 255) as u8,
                evolve_target: r.int("evolve_target").max(0) as u32,
                card_id: r.int("card_id").max(0) as u32,
                walk_speed: r.int("walk_speed"),
                run_speed: r.int("run_speed"),
            });
        }
        Self { creatures }
    }

    pub fn get(&self, id: u32) -> Option<&CreatureDef> {
        self.creatures.get(&id)
    }

    /// Créature par défaut d'un objet carte : `Item.summon_id`, si l'objet est une carte connue.
    pub fn card_creature(&self, item_group: i32, summon_id: i32) -> u32 {
        u32::try_from(summon_id)
            .ok()
            .filter(|id| item_group == CARD_ITEM_GROUP && *id > 0 && self.creatures.contains_key(id))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_cards_give_a_creature() {
        let mut b = CreatureBook::default();
        b.creatures.insert(1801, CreatureDef { id: 1801, form: 1, ..Default::default() });
        assert_eq!(b.card_creature(13, 1801), 1801);
        assert_eq!(b.card_creature(13, 0), 0, "carte vierge");
        assert_eq!(b.card_creature(130, 1801), 0, "autre groupe");
        assert_eq!(b.card_creature(13, 42), 0, "créature inconnue");
        assert_eq!(b.card_creature(13, -5), 0);
        assert_eq!([0u8, 1, 2, 3].map(|form| CreatureDef { form, ..Default::default() }.max_level()), [60, 60, 115, 150]);
    }

    /// Vraies tables converties, si présentes (non publiées).
    #[test]
    fn real_table_when_available() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        let Ok(book) = CreatureBook::open(&dir) else { return };
        assert_eq!(book.creatures.len(), 3543);
        let c = book.get(1401).expect("créature 1401");
        assert_eq!((c.form, c.evolve_target, c.card_id), (1, 1402, 540001));
        // Toute évolution mène à une créature existante.
        assert!(book.creatures.values().filter(|c| c.evolve_target > 0).all(|c| book.get(c.evolve_target).is_some()));
    }
}
