//! États (bonus, malus, effets périodiques) : `Tenacity.dbc` (db_tenacity.rdb,
//! docs/re/skill_protocol.md §4.6 et §8.5).
//!
//! Seuls les états dont **tous** les effets sont établis et calculés par le serveur sont
//! retenus : modificateurs de statistiques (types 1 et 2, banque A), dégâts ou soins
//! périodiques (bases 1, 3, 11, 12) et contrôles décrits par leur texte ([`Control`]). Un état
//! partiellement compris est refusé.

use std::collections::HashMap;
use std::path::Path;

use crate::dbcstore::{NamedDbc, Row, StoreError};

/// Statistiques modifiables, dans l'ordre de [`Mods`]. Bits de la banque A [O].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Stat {
    Attack,
    MagicAttack,
    Defence,
    MagicDefence,
    AttackSpeed,
    MoveSpeed,
}

pub const STATS: usize = 6;
const BITS: [(u32, Stat); STATS] = [
    (7, Stat::Attack),
    (8, Stat::MagicAttack),
    (9, Stat::Defence),
    (10, Stat::MagicDefence),
    (11, Stat::AttackSpeed),
    (13, Stat::MoveSpeed),
];

/// Cumul de modificateurs : parts fixes puis pourcentages (additifs entre eux [D]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mods {
    flat: [f64; STATS],
    ratio: [f64; STATS],
}

impl Mods {
    pub fn add(&mut self, other: &Mods) {
        for i in 0..STATS {
            self.flat[i] += other.flat[i];
            self.ratio[i] += other.ratio[i];
        }
    }

    pub fn is_neutral(&self) -> bool {
        *self == Self::default()
    }

    /// Valeur modifiée, jamais négative.
    pub fn apply(&self, stat: Stat, base: i32) -> i32 {
        let i = stat as usize;
        ((f64::from(base) + self.flat[i]) * (1.0 + self.ratio[i])).round().clamp(0.0, 1e9) as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pulse {
    PhysicalDamage,
    MagicDamage,
    HealHp,
    HealMp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Periodic {
    pub pulse: Pulse,
    /// Intervalle en centièmes de seconde (le client l'affiche en secondes).
    pub interval: u32,
    amplify: (f64, f64),
    add: (f64, f64),
}

impl Periodic {
    /// (amplify_base + amplify_per_skl·lv)·puissance + add_base + add_per·lv.
    pub fn amount(&self, level: u16, power: i32) -> i32 {
        let lv = f64::from(level);
        let v = (self.amplify.0 + self.amplify.1 * lv) * f64::from(power) + self.add.0 + self.add.1 * lv;
        (v + 1e-3).clamp(0.0, 1e9) as i32
    }
}

/// Contrôle de la cible, d'après le texte du client (aucun champ de calcul).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Control {
    pub no_move: bool,
    pub no_action: bool,
    /// Tombe dès que le porteur subit des dégâts.
    pub wakes: bool,
}

/// Tous les malus de compétence dont `state_time_type` porte ce bit contrôlent leur cible [D] :
/// un tel état absent de [`control_of`] est refusé.
const TIME_TYPE_CONTROL: i32 = 8;

fn control_of(id: u32) -> Option<Control> {
    let c = |no_move, no_action, wakes| Some(Control { no_move, no_action, wakes });
    match id {
        // Endormissement : « inapte à toute action », se réveille sur dégâts.
        6005 => c(true, true, true),
        // Étourdissement : « inapte à toute action ».
        6006 => c(true, true, false),
        // Immobilisation ; Prison de Givre et Enracinement (dégâts périodiques en plus).
        6007 | 13003 | 13023 => c(true, false, false),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateDef {
    pub id: u32,
    pub harmful: bool,
    pub control: Control,
    pub periodic: Option<Periodic>,
    /// (masque de bits, base, par niveau, pourcentage).
    changes: Vec<(u32, f64, f64, bool)>,
    /// Groupes d'exclusion (`duplicate_group_*`) : un état en remplace un autre du même groupe [D].
    pub exclusive: [i32; 3],
}

impl StateDef {
    pub fn mods(&self, level: u16) -> Mods {
        let mut m = Mods::default();
        for (mask, base, per, percent) in &self.changes {
            let v = base + per * f64::from(level);
            for (bit, stat) in BITS {
                if mask & (1 << bit) != 0 {
                    let slot = if *percent { &mut m.ratio } else { &mut m.flat };
                    slot[stat as usize] += v;
                }
            }
        }
        m
    }

    pub fn excludes(&self, other: &StateDef) -> bool {
        self.exclusive.iter().any(|g| *g > 0 && other.exclusive.contains(g))
    }
}

fn definition(r: Row<'_>) -> Option<StateDef> {
    let id = u32::try_from(r.id()).ok().filter(|i| *i > 0)?;
    let harmful = r.int("is_harmful") == 1;
    let pulse = match r.int("base_effect_id") {
        0 => None,
        1 => Some(Pulse::PhysicalDamage),
        3 => Some(Pulse::MagicDamage),
        11 => Some(Pulse::HealHp),
        12 => Some(Pulse::HealMp),
        _ => return None,
    };
    let periodic = match pulse {
        None => None,
        Some(pulse) => {
            // Un bonus qui blesse ou un malus qui soigne n'est pas compris.
            if matches!(pulse, Pulse::PhysicalDamage | Pulse::MagicDamage) != harmful {
                return None;
            }
            let interval = u32::try_from(r.int("fire_interval")).ok().filter(|s| (1..=600).contains(s))? * 100;
            let f = |c: &str| f64::from(r.float(c));
            Some(Periodic {
                pulse,
                interval,
                amplify: (f("amplify_base"), f("amplify_per_skl")),
                // Colonnes entières, contrairement aux coefficients.
                add: (f64::from(r.int("add_damage_base")), f64::from(r.int("add_damage_per_skl"))),
            })
        }
    };
    let control = match r.int("state_time_type") & TIME_TYPE_CONTROL {
        0 => Control::default(),
        _ if !harmful => Control::default(),
        _ => control_of(id)?,
    };
    let mut changes = Vec::new();
    match r.int("effect_type") {
        -1 | 0 => {}
        kind @ (1 | 2) => {
            // Banque B (résistances, dégâts élémentaires…) : non calculée.
            if [6, 9].iter().any(|o| r.float(&format!("value_{o}")) != 0.0) {
                return None;
            }
            let known: u32 = BITS.iter().map(|(b, _)| 1 << b).sum();
            for o in [0, 3, 12, 15] {
                let mask = f64::from(r.float(&format!("value_{o}")));
                if mask == 0.0 {
                    continue;
                }
                if mask < 0.0 || mask.fract() != 0.0 || mask > f64::from(u32::MAX) || (mask as u32) & !known != 0 {
                    return None;
                }
                let v = |n: i32| f64::from(r.float(&format!("value_{n}")));
                changes.push((mask as u32, v(o + 1), v(o + 2), kind == 2));
            }
        }
        _ => return None,
    }
    if periodic.is_none() && changes.is_empty() && control == Control::default() {
        return None;
    }
    let exclusive = ["duplicate_group_1", "duplicate_group_2", "duplicate_group_3"].map(|c| r.int(c));
    Some(StateDef { id, harmful, control, periodic, changes, exclusive })
}

#[derive(Clone, Debug, Default)]
pub struct StateBook {
    states: HashMap<u32, StateDef>,
}

impl StateBook {
    pub fn open(dbc_dir: &Path) -> Result<Self, StoreError> {
        let table = NamedDbc::open(dbc_dir, "tenacity")?;
        Ok(Self { states: table.rows().filter_map(|r| Some((u32::try_from(r.id()).ok()?, definition(r)?))).collect() })
    }

    pub fn get(&self, id: u32) -> Option<&StateDef> {
        self.states.get(&id)
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

impl StateDef {
    /// État de test : modificateur en pourcentage et effet périodique facultatif.
    #[doc(hidden)]
    pub fn fixture(id: u32, harmful: bool, change: Option<(Stat, f64)>, periodic: Option<(Pulse, u32, f64)>) -> Self {
        let bit = |s: Stat| BITS.iter().find(|(_, x)| *x == s).map_or(0, |(b, _)| 1 << b);
        Self {
            id,
            harmful,
            control: Control::default(),
            periodic: periodic.map(|(pulse, interval, add)| Periodic { pulse, interval, amplify: (0.0, 0.0), add: (add, 0.0) }),
            changes: change.map(|(s, v)| (bit(s), v, 0.0, true)).into_iter().collect(),
            exclusive: [0; 3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mods_combine_and_never_go_negative() {
        let mut m = Mods::default();
        assert!(m.is_neutral());
        m.add(&StateDef::fixture(1, true, Some((Stat::Defence, -0.3)), None).mods(1));
        m.add(&StateDef::fixture(2, true, Some((Stat::Defence, -0.2)), None).mods(1));
        assert_eq!(m.apply(Stat::Defence, 100), 50);
        assert_eq!(m.apply(Stat::Attack, 100), 100);
        m.add(&StateDef::fixture(3, true, Some((Stat::Defence, -5.0)), None).mods(1));
        assert_eq!(m.apply(Stat::Defence, 100), 0);
    }

    /// Vraies tables converties, si présentes (non publiées).
    #[test]
    fn real_states_when_available() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        let Ok(book) = StateBook::open(&dir) else { return };
        assert!(book.len() > 100, "{}", book.len());
        // 5005 Peau Fragile : DEF −30 % − 3 %/niveau.
        let s = book.get(5005).unwrap();
        assert_eq!((s.harmful, s.periodic, s.mods(1).apply(Stat::Defence, 100)), (true, None, 67));
        // 201006 : DEF −100 − 20/niveau, en valeur.
        assert_eq!(book.get(201006).unwrap().mods(2).apply(Stat::Defence, 500), 360);
        // 13023 Enracinement : dégâts magiques chaque seconde, 0,05·lv·mag + 16·lv.
        let p = book.get(13023).unwrap().periodic.unwrap();
        assert_eq!((p.pulse, p.interval, p.amount(2, 100)), (Pulse::MagicDamage, 100, 42));
        // 13222 Eau Régénératrice : soin toutes les 3 s.
        let p = book.get(13222).unwrap().periodic.unwrap();
        assert_eq!((p.pulse, p.interval, p.amount(28, 100)), (Pulse::HealHp, 300, 34));
        assert!(book.get(13023).unwrap().control.no_move, "Enracinement immobilise aussi");
        // Contrôles : étourdissement et sommeil ; refusés : peur (6008), gel (6009), cauchemar.
        let c = book.get(6006).unwrap().control;
        assert_eq!((c.no_move, c.no_action, c.wakes), (true, true, false));
        assert!(book.get(6005).unwrap().control.wakes);
        assert!([6008, 6009, 6012, 6014, 6019, 13601].iter().all(|id| book.get(*id).is_none()));
    }
}
