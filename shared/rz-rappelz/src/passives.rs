//! Passifs (`Skill.dbc`, docs/re/skill_protocol.md §8.3) et armes requises (§5.2).
//!
//! Comme pour les états, un passif dont un seul effet n'est pas établi est refusé.

use std::collections::HashMap;
use std::path::Path;

use crate::dbcstore::{NamedDbc, Row, StoreError};
use crate::states::{Mods, Stat};

/// Classes d'objet (`Item.dbc`, colonne `class`) lues par le contrôle d'arme du client.
const SHIELD: i32 = 210;
/// (colonne, classe simple, classe « double » admise par deux armes de cette classe).
const WEAPON_COLUMNS: [(&str, i32); 16] = [
    ("vf_one_hand_sword", 101),
    ("vf_two_hand_sword", 102),
    ("vf_double_sword", -101),
    ("vf_dagger", 103),
    ("vf_double_dagger", -103),
    ("vf_spear", 104),
    ("vf_axe", 105),
    ("vf_one_hand_axe", 113),
    ("vf_double_axe", -113),
    ("vf_one_hand_mace", 106),
    ("vf_two_hand_mace", 107),
    ("vf_lightbow", 109),
    ("vf_heavybow", 108),
    ("vf_crossbow", 110),
    ("vf_one_hand_staff", 111),
    ("vf_two_hand_staff", 112),
];

/// Classes des objets tenus (0 = rien).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Held {
    pub right: i32,
    pub left: i32,
}

impl Held {
    /// Arc ou arbalète : l'attaque dépend de l'agilité et de la dextérité (texte 1417).
    pub fn ranged(&self) -> bool {
        (108..=110).contains(&self.right)
    }
}

/// Arme exigée par une compétence.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Weapons {
    /// Sans condition.
    any: bool,
    shield: bool,
    /// Classes admises ; négative : deux armes de cette classe.
    classes: Vec<i32>,
}

impl Weapons {
    pub fn of(r: Row<'_>) -> Self {
        let classes: Vec<i32> = WEAPON_COLUMNS.iter().filter(|(c, _)| r.int(c) != 0).map(|(_, class)| *class).collect();
        let shield = r.int("vf_shield_only") != 0;
        Self { any: r.int("vf_is_not_need_weapon") != 0, shield, classes }
    }

    pub fn fits(&self, held: Held) -> bool {
        // Ordre du client (0x6d20db) : sans condition, sinon bouclier seul, sinon arme.
        if self.any || self.shield {
            return self.any || held.left == SHIELD;
        }
        self.classes.iter().any(|c| match *c {
            c if c < 0 => held.right == -c && held.left == -c,
            c => held.right == c,
        })
    }
}

#[derive(Clone, Debug)]
struct PassiveDef {
    weapons: Weapons,
    /// (statistique, base, par niveau, pourcentage).
    effects: Vec<(Stat, f64, f64, bool)>,
}

fn definition(r: Row<'_>) -> Option<PassiveDef> {
    // `is_passive` est inversé : 0 = passive.
    if r.int("is_valid") != 1 || r.int("is_passive") != 0 || r.int("need_state_id") != 0 || r.int("skill_enchant_link_id") != 0 {
        return None;
    }
    let v: [f64; 20] = std::array::from_fn(|i| f64::from(r.float(&format!("var{}", i + 1))));
    let var = |n: usize| v[n - 1];
    let zero = |from: usize| (from..=20).all(|n| var(n) == 0.0);
    let mut effects = Vec::new();
    use Stat::*;
    // L·v1, L·v2… dans cet ordre.
    const COMBAT: [Stat; 10] =
        [Attack, Defence, MagicAttack, MagicDefence, AttackSpeed, MoveSpeed, Accuracy, MagicAccuracy, Evasion, MagicResistance];
    const PRIMARY: [Stat; 7] = [Strength, Vitality, Agility, Dexterity, Intelligence, Wisdom, Luck];
    match r.int("effect_type") {
        kind @ (3 | 4) => {
            // Banque B (résistances, dégâts élémentaires…) : non calculée.
            if var(7) != 0.0 || var(10) != 0.0 || !zero(19) {
                return None;
            }
            for n in [1, 4, 13, 16] {
                for stat in Stat::in_mask(var(n))? {
                    effects.push((stat, var(n + 1), var(n + 2), kind == 4));
                }
            }
        }
        10001 if zero(13) => {
            for (k, stat) in [Attack, AttackSpeed, Accuracy, MagicAttack, Critical].into_iter().enumerate() {
                effects.push((stat, var(2 * k + 1), var(2 * k + 2), false));
            }
            effects.push((Attack, var(11), var(12), true));
        }
        kind @ (10008 | 10011) if zero(11) => {
            for (k, stat) in COMBAT.into_iter().enumerate() {
                effects.push((stat, 0.0, var(k + 1), kind == 10011));
            }
        }
        // Dix couples dans l'ordre de 10008, en pourcentage.
        10043 => {
            for (k, stat) in COMBAT.into_iter().enumerate() {
                effects.push((stat, var(2 * k + 1), var(2 * k + 2), true));
            }
        }
        kind @ (10015 | 10042) if zero(15) => {
            for (k, stat) in PRIMARY.into_iter().enumerate() {
                effects.push((stat, var(2 * k + 1), var(2 * k + 2), kind == 10042));
            }
        }
        10021 if var(5) == 0.0 && var(6) == 0.0 && zero(11) => {
            for (n, stat) in [(1, MaxHp), (3, MaxMp), (7, HpRecovery), (9, MpRecovery)] {
                effects.push((stat, var(n), var(n + 1), false));
            }
        }
        10045 if zero(5) => {
            effects.push((MaxHp, var(1), var(2), true));
            effects.push((MaxMp, var(3), var(4), true));
        }
        10009 if var(8) == 0.0 && zero(10) => {
            for (n, stat) in
                [(1, HpRegen), (2, HpRecovery), (3, MpRegen), (4, MpRecovery), (5, BlockChance), (6, Block), (7, Critical), (9, CastSpeed)]
            {
                effects.push((stat, 0.0, var(n), false));
            }
        }
        _ => return None,
    }
    effects.retain(|(_, base, per, _)| *base != 0.0 || *per != 0.0);
    (!effects.is_empty()).then(|| PassiveDef { weapons: Weapons::of(r), effects })
}

#[derive(Clone, Debug, Default)]
pub struct PassiveBook {
    passives: HashMap<u32, PassiveDef>,
}

impl PassiveBook {
    pub fn open(dbc_dir: &Path) -> Result<Self, StoreError> {
        let table = NamedDbc::open(dbc_dir, "skill")?;
        Ok(Self { passives: table.rows().filter_map(|r| Some((u32::try_from(r.id()).ok()?, definition(r)?))).collect() })
    }

    pub fn len(&self) -> usize {
        self.passives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.passives.is_empty()
    }

    /// Modificateurs d'un passif appris, ou `None` s'il est inconnu ou si l'arme ne convient pas.
    pub fn mods(&self, id: u32, level: u8, held: Held) -> Option<Mods> {
        let d = self.passives.get(&id).filter(|d| level > 0 && d.weapons.fits(held))?;
        let mut m = Mods::default();
        for (stat, base, per, percent) in &d.effects {
            m.push(*stat, base + per * f64::from(level), *percent);
        }
        Some(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::Attributes;

    #[test]
    fn weapons_accept_single_double_and_shield() {
        let w = |classes: &[i32], shield| Weapons { any: false, shield, classes: classes.to_vec() };
        let held = |right, left| Held { right, left };
        assert!(w(&[101], false).fits(held(101, 0)) && !w(&[101], false).fits(held(102, 0)));
        assert!(w(&[-101], false).fits(held(101, 101)) && !w(&[-101], false).fits(held(101, 210)));
        assert!(w(&[], true).fits(held(0, 210)) && !w(&[], true).fits(held(101, 0)));
        assert!(Weapons { any: true, ..Default::default() }.fits(Held::default()));
        assert!(held(109, 0).ranged() && !held(111, 0).ranged());
    }

    /// Vraies tables converties, si présentes (non publiées).
    #[test]
    fn real_passives_when_available() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        let Ok(book) = PassiveBook::open(&dir) else { return };
        assert!(book.len() > 200, "{}", book.len());
        let none = Held::default();
        let plain = Attributes::default();
        // 1003 Entraînement Défensif : DEF +3 par niveau. 1004 : PV max +1 500 par niveau.
        assert_eq!(book.mods(1003, 2, none).unwrap().derive(Stat::Defence, 10, &plain), 16);
        assert_eq!(book.mods(1004, 1, none).unwrap().derive(Stat::MaxHp, 100, &plain), 1600);
        // 1202 Défense Physique : DEF +0,47 % par niveau, en pourcentage.
        assert_eq!(book.mods(1202, 10, none).unwrap().derive(Stat::Defence, 1000, &plain), 1047);
        // 1111 Maîtrise de l'Arc : ATT +10 +150·L et +1 %·L, arc seulement.
        assert!(book.mods(1111, 1, none).is_none());
        assert_eq!(book.mods(1111, 1, Held { right: 109, left: 0 }).unwrap().derive(Stat::Attack, 0, &plain), 162);
        // 40902 Bénédiction de Force : FOR +1 par niveau, soit 2,8 d'attaque de mêlée par point.
        let m = book.mods(40902, 10, none).unwrap();
        assert_eq!(m.derive(Stat::Attack, 100, &plain), 128);
        assert_eq!(m.derive(Stat::Attack, 100, &Attributes { ranged: true, ..plain }), 100);
        // 21109 (banque B) et 40221 (condition v1) : refusés.
        assert!(book.mods(21109, 1, none).is_none() && book.mods(40221, 1, none).is_none());
    }
}
