//! Compétences actives à cible unique dont le client donne la formule (docs/re/skill_protocol.md
//! §8) : dégâts physiques 30001, dégâts magiques 231, soins 501. Tout ce qui demande une règle
//! non établie (zone, état, coût particulier, bascule, compétence liée…) est refusé.
//!
//! Les formules sont celles des info-bulles : des valeurs **avant** défense et résistance.

use std::collections::HashMap;
use std::path::Path;

use crate::dbcstore::{NamedDbc, Row, StoreError};
use crate::states::StateBook;

pub const SUMMON: u32 = 4001;
pub const UNSUMMON: u32 = 4002;

/// `cast_range` × 12 = unités du monde [O].
pub const RANGE_UNIT: f32 = 12.0;

/// Valeurs de db_skill écrasées par l'exécutable après chargement (tables 0xe91128 et 0xe91608,
/// relues dans le binaire), limitées aux familles prises en charge : (compétence, champ, valeur).
/// Champ 1..20 = var ; 21 = delay_cooltime en secondes.
const OVERRIDES: &[(u32, u8, f32)] = &[
    (21304, 2, 0.2),
    (45127, 1, 1.2),
    (45208, 21, 25.0),
    (45223, 21, 30.0),
    (45228, 21, 15.0),
    (48209, 1, 1.2),
    (48211, 1, 0.6),
    (48212, 1, 0.6),
    (48213, 1, 0.6),
    (48214, 1, 0.5),
    (48258, 1, 0.9),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellKind {
    /// att·(v1+L·v2)+v3+L·v4
    Physical,
    /// mag·(v1+L·v2)+v4+L·v5
    Magical,
    /// mag·(v1+L·v2)+v3+L·v4, plus (v5+L·v6) des PV max de la cible [D pour l'assiette].
    Heal,
    /// 301 : pose un état sur un monstre (malus) ou un joueur (bonus).
    Curse,
    Bless,
    /// 4001 / 4002 : la cible est une carte (docs/re/summon_protocol.md).
    Summon,
    Unsummon,
}

/// État posé, évalué pour un niveau de compétence (§7).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateQuote {
    pub id: u32,
    pub level: u16,
    /// Centièmes de seconde.
    pub duration: u32,
    /// Pourcentage.
    pub chance: f32,
}

#[derive(Clone, Copy, Debug)]
struct StateApply {
    id: u32,
    level: (f32, f32),
    second: (f32, f32),
    chance: (f32, f32),
}

#[derive(Clone, Copy, Debug)]
struct SpellDef {
    state: Option<StateApply>,
    kind: SpellKind,
    vars: [f32; 6],
    mp: (i32, i32),
    /// Secondes : (base, par niveau).
    cast: (f32, f32),
    cooldown: (f32, f32),
    group: u32,
    /// −1 = portée d'attaque.
    range: i32,
}

/// Compétence évaluée pour un niveau. Durées en centièmes de seconde.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpellQuote {
    pub id: u32,
    pub level: u8,
    pub kind: SpellKind,
    pub mp: i32,
    pub cast: u32,
    pub cooldown: u32,
    /// Clé de recharge : groupe commun, sinon propre à la compétence.
    pub group: u32,
    /// Unités du monde ; `None` = portée d'attaque.
    pub range: Option<f32>,
    /// Multiplicateur d'attaque (physique ou magique) et part fixe.
    pub ratio: f64,
    pub flat: f64,
    /// Part des PV max (soin).
    pub hp_ratio: f64,
    pub state: Option<StateQuote>,
}

impl SpellQuote {
    /// Valeur affichée par l'info-bulle, arrondie comme elle (`int`).
    pub fn nominal(&self, power: i32, target_max_hp: i32) -> i32 {
        let v = f64::from(power) * self.ratio + self.flat + f64::from(target_max_hp) * self.hp_ratio;
        // Les coefficients sont en simple précision : ne pas perdre une unité sur 109,99999.
        (v + 1e-3).clamp(0.0, f64::from(i32::MAX)) as i32
    }
}

fn centiseconds((base, per): (f32, f32), level: u8) -> Option<u32> {
    // Les colonnes dec2 ont une précision au centième : l'arrondir avant de multiplier.
    let cs = |v: f32| (f64::from(v) * 100.0).round();
    let total = cs(base) + cs(per) * f64::from(level);
    (0.0..=360_000.0).contains(&total.max(0.0)).then(|| total.max(0.0) as u32)
}

/// Colonnes qui doivent être nulles : coût, condition ou comportement non pris en charge.
const MUST_BE_ZERO: &[&str] = &[
    "cost_hp", "cost_hp_per_skl", "cost_hp_per", "cost_hp_per_skl_per", "cost_mp_per", "cost_mp_per_skl_per",
    "cost_havoc", "cost_havoc_per_skl", "cost_energy", "cost_energy_per_skl", "cost_exp", "cost_jp", "cost_item",
    "cost_item_count", "need_state_id", "need_havoc", "is_toggle", "is_corpse", "skill_enchant_link_id",
];

/// 301 : état de `state_id`, dont tous les effets doivent être connus ; les états
/// supplémentaires (v1..v5) ne sont pas pris en charge.
fn state_of(r: Row<'_>, states: &StateBook) -> Option<(SpellKind, StateApply)> {
    let state = states.get(u32::try_from(r.int("state_id")).ok()?)?;
    let harmful = r.int("is_harmful") == 1;
    if state.harmful != harmful || (1..=5).any(|n| r.float(&format!("var{n}")) != 0.0) {
        return None;
    }
    let apply = StateApply {
        id: state.id,
        level: (r.int("state_level_base") as f32, r.float("state_level_per_skl")),
        second: (r.float("state_second"), r.float("state_second_per_level")),
        chance: (r.int("probability_on_hit") as f32, r.int("probability_inc_by_slv") as f32),
    };
    (apply.chance.0 > 0.0).then_some((if harmful { SpellKind::Curse } else { SpellKind::Bless }, apply))
}

fn definition(r: Row<'_>, states: &StateBook) -> Option<SpellDef> {
    let id = u32::try_from(r.id()).ok()?;
    // `is_passive` et `is_physical_act` sont inversés dans le schéma : 1 = active, 1 = magique.
    if r.int("is_valid") != 1 || r.int("is_passive") != 1 {
        return None;
    }
    let mut state = None;
    let kind = match (id, r.int("effect_type")) {
        (_, 301) => {
            let (kind, apply) = state_of(r, states)?;
            state = Some(apply);
            kind
        }
        (SUMMON, 601) => SpellKind::Summon,
        (UNSUMMON, 602) => SpellKind::Unsummon,
        (_, 30001) => SpellKind::Physical,
        (_, 231) => SpellKind::Magical,
        (_, 501) => SpellKind::Heal,
        _ => return None,
    };
    if !matches!(kind, SpellKind::Summon | SpellKind::Unsummon) {
        let harmful = i32::from(!matches!(kind, SpellKind::Heal | SpellKind::Bless));
        let supported = MUST_BE_ZERO.iter().all(|c| r.float(c) == 0.0 && r.int(c) == 0)
            && (state.is_some() || r.int("state_id") == 0)
            && r.int("target") == 1
            && r.int("is_need_target") == 1
            && r.int("is_harmful") == harmful;
        if !supported {
            return None;
        }
    }
    let mut def = SpellDef {
        state,
        kind,
        vars: std::array::from_fn(|i| r.float(&format!("var{}", i + 1))),
        mp: (r.int("cost_mp"), r.int("cost_mp_per_skl")),
        cast: (r.float("delay_cast"), r.float("delay_cast_per_skl")),
        cooldown: (r.float("delay_cooltime"), r.float("delay_cooltime_per_skl")),
        group: r.int("cool_time_group_id").max(0) as u32,
        range: r.int("cast_range"),
    };
    for (_, field, value) in OVERRIDES.iter().filter(|o| o.0 == id) {
        match field {
            21 => def.cooldown.0 = *value,
            n => def.vars[usize::from(*n) - 1] = *value,
        }
    }
    def.vars.iter().all(|v| v.is_finite()).then_some(def)
}

#[derive(Clone, Debug, Default)]
pub struct SpellBook {
    spells: HashMap<u32, SpellDef>,
}

impl SpellBook {
    pub fn open(dbc_dir: &Path, states: &StateBook) -> Result<Self, StoreError> {
        let table = NamedDbc::open(dbc_dir, "skill")?;
        Ok(Self { spells: table.rows().filter_map(|r| Some((u32::try_from(r.id()).ok()?, definition(r, states)?))).collect() })
    }

    pub fn len(&self) -> usize {
        self.spells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spells.is_empty()
    }

    pub fn quote(&self, id: u32, level: u8) -> Option<SpellQuote> {
        let d = self.spells.get(&id)?;
        if level == 0 || level > 50 {
            return None;
        }
        let l = f64::from(level);
        let v = d.vars.map(f64::from);
        let (ratio, flat, hp_ratio) = match d.kind {
            SpellKind::Physical => (v[0] + l * v[1], v[2] + l * v[3], 0.0),
            SpellKind::Magical => (v[0] + l * v[1], v[3] + l * v[4], 0.0),
            SpellKind::Heal => (v[0] + l * v[1], v[2] + l * v[3], v[4] + l * v[5]),
            SpellKind::Curse | SpellKind::Bless | SpellKind::Summon | SpellKind::Unsummon => (0.0, 0.0, 0.0),
        };
        let state = match d.state {
            None => None,
            Some(s) => {
                let value = s.level.0 + (s.level.1 * f32::from(level) + 1e-4).trunc();
                let duration = centiseconds(s.second, level).filter(|d| *d > 0)?;
                let chance = s.chance.0 + s.chance.1 * f32::from(level);
                // Niveau 0 = retrait côté client : un tel état ne peut pas être posé.
                let value = (1.0..=10_000.0).contains(&value).then_some(value as u16)?;
                Some(StateQuote { id: s.id, level: value, duration, chance })
            }
        };
        Some(SpellQuote {
            id,
            level,
            kind: d.kind,
            mp: d.mp.0.saturating_add(d.mp.1.saturating_mul(i32::from(level))).max(0),
            cast: centiseconds(d.cast, level)?,
            cooldown: centiseconds(d.cooldown, level)?,
            group: if d.group != 0 { d.group } else { 0x8000_0000 | id },
            range: (d.range >= 0).then(|| d.range as f32 * RANGE_UNIT),
            ratio,
            flat,
            hp_ratio,
            state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vraies tables converties, si présentes (non publiées).
    #[test]
    fn real_skills_follow_the_client_formulas() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        let states = StateBook::open(&dir).unwrap_or_default();
        let Ok(book) = SpellBook::open(&dir, &states) else { return };
        assert!(book.len() > 40, "{}", book.len());
        // 3603 Peau Fragile : état 5005, niveau = niveau de compétence, 30 s.
        let q = book.quote(3603, 2).unwrap();
        assert_eq!((q.kind, q.state.map(|s| (s.id, s.level, s.duration))), (SpellKind::Curse, Some((5005, 2, 3000))));
        // 3222 Eau Régénératrice : niveau d'état 28 par niveau.
        assert_eq!(book.quote(3222, 1).and_then(|q| q.state).map(|s| (s.level, s.duration)), Some((28, 1600)));
        // 2001 Frappe Centrée : att·(0,795+0,12·L)+0+L·103,5 ; portée d'attaque ; instantanée.
        let q = book.quote(2001, 2).unwrap();
        assert_eq!((q.kind, q.mp, q.cast, q.cooldown, q.range), (SpellKind::Physical, 36, 0, 1000, None));
        assert_eq!(q.nominal(100, 0), (100.0_f64 * 1.035 + 207.0) as i32);
        // 3011 Éclat de Lumière : mag·(0,9+0,05·L)+0+L·15 … portée 16 × 12.
        let q = book.quote(3011, 1).unwrap();
        assert_eq!((q.kind, q.mp, q.cast, q.cooldown, q.group, q.range), (SpellKind::Magical, 12, 50, 80, 107, Some(192.0)));
        assert_eq!(q.nominal(100, 0), (100.0_f64 * 0.95 + 15.0) as i32);
        // 3202 Soins : mag·(0,2+0,024·L)+80+L·140 + 0,15 %·L des PV max.
        let q = book.quote(3202, 1).unwrap();
        assert_eq!((q.kind, q.cast), (SpellKind::Heal, 250));
        assert_eq!(q.nominal(100, 1000), 243);
        // 45127 : surchargée par l'exécutable. 45116 et 2501 : refusées.
        if let (Some(a), Some(b)) = (book.quote(45127, 1), book.quote(45127, 2)) {
            assert!((2.0 * a.ratio - b.ratio - 1.2).abs() < 1e-3, "v1 surchargé");
        }
        assert!(book.quote(45116, 1).is_none() && book.quote(2501, 1).is_none());
        // Invocation : 8,8 s − 0,8 s par niveau ; 60 + 20 PM par niveau.
        let q = book.quote(SUMMON, 1).unwrap();
        assert_eq!((q.kind, q.cast, q.mp, q.cooldown), (SpellKind::Summon, 800, 80, 1000));
        assert_eq!(book.quote(SUMMON, 11).unwrap().cast, 0);
        assert_eq!(book.quote(UNSUMMON, 1).map(|q| (q.cast, q.mp)), Some((50, 5)));
        assert!(book.quote(2001, 0).is_none() && book.quote(2001, 51).is_none());
    }
}
