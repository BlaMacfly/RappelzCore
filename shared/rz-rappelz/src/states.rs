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

/// Statistiques modifiables : numéro de bit dans la banque A du client [O].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Stat {
    Strength,
    Vitality,
    Agility,
    Dexterity,
    Intelligence,
    Wisdom,
    Luck,
    Attack,
    MagicAttack,
    Defence,
    MagicDefence,
    AttackSpeed,
    CastSpeed,
    MoveSpeed,
    Accuracy,
    MagicAccuracy,
    Critical,
    BlockChance,
    Block,
    Evasion,
    MagicResistance,
    MaxHp,
    MaxMp,
    HpRecovery = 24,
    MpRecovery = 25,
    HpRegen = 27,
    MpRegen = 28,
    MaxWeight = 30,
}

/// Bits 23 et 26 (SP), 29 et 31 (inconnus) : absents, donc refusés.
const ALL: [Stat; 28] = [
    Stat::Strength,
    Stat::Vitality,
    Stat::Agility,
    Stat::Dexterity,
    Stat::Intelligence,
    Stat::Wisdom,
    Stat::Luck,
    Stat::Attack,
    Stat::MagicAttack,
    Stat::Defence,
    Stat::MagicDefence,
    Stat::AttackSpeed,
    Stat::CastSpeed,
    Stat::MoveSpeed,
    Stat::Accuracy,
    Stat::MagicAccuracy,
    Stat::Critical,
    Stat::BlockChance,
    Stat::Block,
    Stat::Evasion,
    Stat::MagicResistance,
    Stat::MaxHp,
    Stat::MaxMp,
    Stat::HpRecovery,
    Stat::MpRecovery,
    Stat::HpRegen,
    Stat::MpRegen,
    Stat::MaxWeight,
];
const SLOTS: usize = 31;

impl Stat {
    /// Statistiques d'un masque (colonne décimale) ; `None` si un bit est inconnu.
    pub fn in_mask(mask: f64) -> Option<Vec<Stat>> {
        if mask < 0.0 || mask.fract() != 0.0 || mask > f64::from(u32::MAX) {
            return None;
        }
        let mask = mask as u32;
        let found: Vec<Stat> = ALL.into_iter().filter(|s| mask & (1 << *s as u32) != 0).collect();
        (found.len() as u32 == mask.count_ones()).then_some(found)
    }
}

/// Caractéristiques de base d'un personnage (métier) et nature de son arme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Attributes {
    /// Dans l'ordre des bits : FOR, VIT, AGI, DEX, INT, SAG, CHA.
    pub base: [i32; 7],
    pub ranged: bool,
}

/// Effet d'un point de caractéristique, d'après les textes 1410 à 1416 du client. Taux de
/// butin (CHA) : non appliqué.
fn per_point(attribute: usize, stat: Stat, ranged: bool) -> f64 {
    match (attribute, stat) {
        (0, Stat::Attack) if !ranged => 2.8,
        (0, Stat::MaxWeight) => 10.0,
        (1, Stat::MaxHp) => 33.0,
        (1, Stat::Defence) => 1.6,
        (2, Stat::Attack) if ranged => 1.2,
        (2, Stat::Evasion) => 0.5,
        (2, Stat::AttackSpeed) => 0.1,
        (3, Stat::Attack) if ranged => 2.2,
        (3, Stat::Accuracy) => 0.5,
        (4, Stat::MagicAttack) => 2.0,
        (4, Stat::MaxMp) => 30.0,
        (5, Stat::MagicDefence) => 2.0,
        (5, Stat::MagicResistance | Stat::MagicAccuracy) => 0.5,
        (5, Stat::MpRegen) => 4.1,
        (6, Stat::Critical) => 0.2,
        _ => 0.0,
    }
}

/// Cumul de modificateurs : parts fixes puis pourcentages (additifs entre eux [D]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mods {
    flat: [f64; SLOTS],
    ratio: [f64; SLOTS],
}

impl Mods {
    pub fn add(&mut self, other: &Mods) {
        for i in 0..SLOTS {
            self.flat[i] += other.flat[i];
            self.ratio[i] += other.ratio[i];
        }
    }

    pub fn push(&mut self, stat: Stat, value: f64, percent: bool) {
        let slot = if percent { &mut self.ratio } else { &mut self.flat };
        slot[stat as usize] += value;
    }

    pub fn is_neutral(&self) -> bool {
        *self == Self::default()
    }

    /// Valeur modifiée, jamais négative.
    pub fn apply(&self, stat: Stat, base: i32) -> i32 {
        self.value(stat, f64::from(base))
    }

    fn value(&self, stat: Stat, base: f64) -> i32 {
        let i = stat as usize;
        // Les coefficients sont en simple précision (0,8 × 100 = 79,9999…).
        ((base + self.flat[i]) * (1.0 + self.ratio[i])).round().clamp(0.0, 1e9) as i32
    }

    /// Caractéristique effective (0 = FOR … 6 = CHA).
    pub fn attribute(&self, n: usize, attributes: &Attributes) -> i32 {
        self.value(ALL[n], f64::from(attributes.base[n]))
    }

    /// Statistique dérivée d'un personnage : les points **gagnés ou perdus** par rapport au
    /// métier valent ce que dit le client ; la base reste celle du serveur.
    pub fn derive(&self, stat: Stat, base: i32, attributes: &Attributes) -> i32 {
        let gained: f64 = (0..7)
            .map(|n| per_point(n, stat, attributes.ranged) * f64::from(self.attribute(n, attributes) - attributes.base[n]))
            .sum();
        self.value(stat, f64::from(base) + gained)
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
/// Lu par les fenêtres d'états (FUN_007fed40, FUN_00815110) [O].
const TIME_TYPE_REMOVABLE: i32 = 0x20;

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
    /// Le joueur peut l'ôter lui-même (paquet 408).
    pub removable: bool,
    pub control: Control,
    pub periodic: Option<Periodic>,
    /// (statistique, base, par niveau, pourcentage).
    changes: Vec<(Stat, f64, f64, bool)>,
    /// Groupes d'exclusion (`duplicate_group_*`) : un état en remplace un autre du même groupe [D].
    pub exclusive: [i32; 3],
}

impl StateDef {
    pub fn mods(&self, level: u16) -> Mods {
        let mut m = Mods::default();
        for (stat, base, per, percent) in &self.changes {
            m.push(*stat, base + per * f64::from(level), *percent);
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
            let v = |n: i32| f64::from(r.float(&format!("value_{n}")));
            for o in [0, 3, 12, 15] {
                for stat in Stat::in_mask(v(o))? {
                    changes.push((stat, v(o + 1), v(o + 2), kind == 2));
                }
            }
        }
        _ => return None,
    }
    if periodic.is_none() && changes.is_empty() && control == Control::default() {
        return None;
    }
    let exclusive = ["duplicate_group_1", "duplicate_group_2", "duplicate_group_3"].map(|c| r.int(c));
    Some(StateDef { id, harmful, removable: !harmful && r.int("state_time_type") & TIME_TYPE_REMOVABLE != 0, control, periodic, changes, exclusive })
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
        Self {
            id,
            harmful,
            removable: false,
            control: Control::default(),
            periodic: periodic.map(|(pulse, interval, add)| Periodic { pulse, interval, amplify: (0.0, 0.0), add: (add, 0.0) }),
            changes: change.map(|(s, v)| (s, v, 0.0, true)).into_iter().collect(),
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

    #[test]
    fn gained_attributes_follow_the_client_texts() {
        let mut m = Mods::default();
        let a = Attributes { base: [20, 20, 10, 10, 10, 10, 10], ranged: false };
        assert_eq!(m.derive(Stat::Attack, 92, &a), 92, "la base du métier ne change rien");
        m.push(Stat::Strength, 10.0, false);
        m.push(Stat::Vitality, 0.5, true);
        m.push(Stat::Attack, 0.1, true);
        assert_eq!((m.attribute(0, &a), m.attribute(1, &a)), (30, 30));
        // (100 + 28) × 1,1 ; 10 points de vitalité = 330 PV et 16 de défense.
        assert_eq!(m.derive(Stat::Attack, 100, &a), 141);
        assert_eq!((m.derive(Stat::MaxHp, 480, &a), m.derive(Stat::Defence, 44, &a)), (810, 60));
        assert_eq!(m.derive(Stat::Attack, 100, &Attributes { ranged: true, ..a }), 110);
        assert_eq!(Stat::in_mask(f64::from(1u32 << 23)), None);
        assert_eq!(Stat::in_mask(532_480.0), Some(vec![Stat::MoveSpeed, Stat::Evasion]));
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
