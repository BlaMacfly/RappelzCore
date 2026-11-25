//! Compétences actives dont le client donne la formule (docs/re/skill_protocol.md §8) : dégâts à
//! cible unique (30001, 231), à coups multiples (30009, 30016, 232), de zone (30011, 261) et de
//! zone à coups multiples (30012, 263), soins 501, états 301. Tout ce qui demande une règle non
//! établie (coût particulier, bascule, compétence liée…) est refusé.
//!
//! Les formules sont celles des info-bulles : des valeurs **avant** défense et résistance.

use std::collections::HashMap;
use std::path::Path;

use crate::dbcstore::{NamedDbc, Row, StoreError};
use crate::passives::{Held, Weapons};
use crate::states::StateBook;

pub const SUMMON: u32 = 4001;
pub const UNSUMMON: u32 = 4002;

/// `cast_range` × 12 = unités du monde [O].
pub const RANGE_UNIT: f32 = 12.0;

/// Valeurs de db_skill écrasées par l'exécutable après chargement (tables 0xe91128 et 0xe91608,
/// relues dans le binaire), limitées aux familles prises en charge : (compétence, champ, valeur).
/// Champ 1..20 = var ; 21 = delay_cooltime en secondes.
const OVERRIDES: &[(u32, u8, f32)] = &[
    (3151, 21, 40.0),
    (21304, 2, 0.2),
    (21311, 21, 15.0),
    (45101, 1, 0.4),
    (45104, 1, 0.5),
    (45106, 1, 0.5),
    (45107, 1, 0.5),
    (45111, 1, 0.6),
    (45116, 1, 1.5),
    (45117, 1, 0.6),
    (45117, 21, 60.0),
    (45120, 1, 0.5),
    (45121, 1, 1.2),
    (45125, 1, 1.0),
    (45127, 1, 1.2),
    (45135, 1, 0.5),
    (45203, 1, 1.2),
    (45206, 1, 1.2),
    (45208, 21, 25.0),
    (45210, 1, 1.3),
    (45211, 1, 1.3),
    (45217, 1, 1.3),
    (45221, 1, 1.4),
    (45222, 1, 1.3),
    (45223, 21, 30.0),
    (45227, 1, 1.4),
    (45228, 21, 15.0),
    (45315, 1, 0.4),
    (45404, 1, 0.9),
    (45416, 1, 1.1),
    (45418, 1, 1.2),
    (45419, 1, 1.0),
    (45426, 1, 0.6),
    (45428, 1, 1.2),
    (45505, 1, 1.7),
    (45516, 1, 2.0),
    (45517, 1, 1.8),
    (45518, 1, 1.7),
    (45519, 1, 1.8),
    (47016, 1, 2.7),
    (48203, 21, 300.0),
    (48208, 1, 0.35),
    (48208, 2, 0.05),
    (48209, 1, 1.2),
    (48211, 1, 0.6),
    (48212, 1, 0.6),
    (48213, 1, 0.6),
    (48214, 1, 0.5),
    (48258, 1, 0.9),
    (48259, 1, 0.439),
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
    /// 701 : état permanent du lanceur, entretenu en PM ; la même demande l'éteint.
    Toggle,
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

/// Types d'animation (db_skillstage, `stage_type_id`) qui jouent les impacts de cette nature
/// (§3.4). Sans fiche, le client n'envoie pas la compétence : elle est refusée.
fn stage_plays(kind: SpellKind, stage: u16) -> bool {
    match kind {
        SpellKind::Physical | SpellKind::Magical => matches!(stage, 101 | 111 | 122 | 201 | 401 | 404 | 411 | 501 | 502),
        SpellKind::Curse | SpellKind::Bless => matches!(stage, 401 | 501),
        SpellKind::Heal => stage == 401,
        // L'impact RESULT d'une bascule est purement visuel ; FIRE suffit à libérer le client.
        SpellKind::Toggle | SpellKind::Summon | SpellKind::Unsummon => true,
    }
}

/// Entretien d'une bascule, toutes les 5 secondes (texte 1585).
pub const UPKEEP_INTERVAL: u32 = 500;
/// Durée d'un état sans fin : 505 annonce alors −1.
pub const PERMANENT: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleQuote {
    pub upkeep: i32,
    /// Bascules qui s'excluent, 0 = aucune [D].
    pub group: u32,
}

/// Zone autour de la cible principale [D pour le centre].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    /// Unités du monde.
    pub radius: f32,
    pub max_targets: u8,
}

#[derive(Clone, Debug)]
struct SpellDef {
    weapons: Weapons,
    stage: u16,
    state: Option<StateApply>,
    toggle_group: u32,
    kind: SpellKind,
    area: Option<Area>,
    /// Coups : (base, par niveau).
    hits: (f32, f32),
    self_only: bool,
    vars: [f32; 20],
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
    /// État posé en plus de l'effet (ou seul : 301).
    pub state: Option<StateQuote>,
    pub area: Option<Area>,
    /// Coups portés à chaque cible.
    pub hits: u8,
    /// Sans cible à choisir : le lanceur.
    pub self_only: bool,
    pub toggle: Option<ToggleQuote>,
    /// Type d'animation du client, qui commande la forme de FIRE.
    pub stage: u16,
}

impl SpellQuote {
    /// (target_count, fire_count) de FIRE pour `targets` cibles (§3.4). Ces champs commandent le
    /// nombre d'impacts joués ; `target_count` ne doit jamais dépasser le nombre d'impacts, ni
    /// être nul avec des impacts (types 121–123 : boucle sans fin du client).
    pub fn fire_counts(&self, targets: usize) -> (u8, u8) {
        let impacts = (targets * usize::from(self.hits)).min(255) as u8;
        match self.stage {
            201 | 401 | 501 | 502 => (impacts, 1),
            404 => (impacts, impacts),
            _ => (targets.min(255) as u8, 1),
        }
    }

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

/// État de `state_id`, dont tous les effets doivent être connus et de même nature (bonus ou
/// malus) que la compétence.
fn state_of(r: Row<'_>, states: &StateBook) -> Option<(SpellKind, StateApply)> {
    let state = states.get(u32::try_from(r.int("state_id")).ok()?)?;
    let harmful = r.int("is_harmful") == 1;
    if state.harmful != harmful {
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

/// Forme d'une famille : (nature, zone, coups). `flag` (v6 ou v11) non nul : règle inconnue,
/// refusée ; `max` est le nombre de cibles (999 = sans limite) [D, par analogie avec le texte
/// du type 111].
fn shape(effect: i32, v: &[f32; 20]) -> Option<(SpellKind, Option<Area>, (f32, f32))> {
    let var = |n: usize| v[n - 1];
    let area = |radius: f32, flag: f32, max: f32| {
        let ok = flag == 0.0 && (1.0..=50.0).contains(&radius) && max.fract() == 0.0 && (1.0..=999.0).contains(&max);
        ok.then(|| Area { radius: radius * RANGE_UNIT, max_targets: max.min(50.0) as u8 })
    };
    let one = (1.0, 0.0);
    Some(match effect {
        30001 => (SpellKind::Physical, None, one),
        30009 | 30016 => (SpellKind::Physical, None, (var(5), var(6))),
        30011 => (SpellKind::Physical, Some(area(var(5), var(6), var(7))?), one),
        30012 => (SpellKind::Physical, Some(area(var(5), var(6), var(7))?), (var(8), var(9))),
        231 => (SpellKind::Magical, None, one),
        232 if (10..=12).all(|n| var(n) == 0.0) => (SpellKind::Magical, None, (var(7), var(8))),
        261 if (7..=9).all(|n| var(n) == 0.0) => (SpellKind::Magical, Some(area(var(10), var(11), var(12))?), one),
        263 => (SpellKind::Magical, Some(area(var(10), var(11), var(12))?), (var(7), var(8))),
        501 => (SpellKind::Heal, None, one),
        _ => return None,
    })
}

fn definition(r: Row<'_>, states: &StateBook, stages: &HashMap<u32, u16>) -> Option<SpellDef> {
    let id = u32::try_from(r.id()).ok()?;
    // `is_passive` et `is_physical_act` sont inversés dans le schéma : 1 = active, 1 = magique.
    if r.int("is_valid") != 1 || r.int("is_passive") != 1 {
        return None;
    }
    let vars: [f32; 20] = std::array::from_fn(|i| r.float(&format!("var{}", i + 1)));
    let mut state = None;
    let (mut area, mut hits) = (None, (1.0, 0.0));
    let kind = match (id, r.int("effect_type")) {
        // Les états supplémentaires (v1..v5) ne sont pas pris en charge.
        (_, 301) if vars[..5].iter().all(|v| *v == 0.0) => {
            let (kind, apply) = state_of(r, states)?;
            state = Some(apply);
            kind
        }
        // Autres états (v4, v5…) non pris en charge ; v3 ne concerne que l'amélioration.
        (_, 701) if vars[3..].iter().all(|v| *v == 0.0) && r.int("is_toggle") == 1 => {
            state = Some(state_of(r, states)?.1);
            SpellKind::Toggle
        }
        (SUMMON, 601) => SpellKind::Summon,
        (UNSUMMON, 602) => SpellKind::Unsummon,
        (_, effect) => {
            let kind;
            (kind, area, hits) = shape(effect, &vars)?;
            if r.int("state_id") != 0 {
                state = Some(state_of(r, states)?.1);
            }
            kind
        }
    };
    let stage = *stages.get(&id)?;
    if !stage_plays(kind, stage) {
        return None;
    }
    let mut self_only = false;
    if !matches!(kind, SpellKind::Summon | SpellKind::Unsummon) {
        let harmful = !matches!(kind, SpellKind::Heal | SpellKind::Bless | SpellKind::Toggle);
        // Sans cible à choisir (§5.3) : la demande vise le lanceur.
        self_only = !harmful && r.int("is_need_target") == 0;
        let toggle = kind == SpellKind::Toggle;
        let supported = MUST_BE_ZERO.iter().all(|c| (toggle && *c == "is_toggle") || (r.float(c) == 0.0 && r.int(c) == 0))
            && (!toggle || self_only)
            && (r.int("target") == 1 || (area.is_some() && r.int("target") == 2))
            && (self_only || r.int("is_need_target") == 1)
            && r.int("is_harmful") == i32::from(harmful)
            && (area.is_none() || (r.int("uf_self") == 0 && r.int("tf_monster") == 1));
        if !supported {
            return None;
        }
    }
    let mut def = SpellDef {
        weapons: Weapons::of(r),
        stage,
        state,
        toggle_group: r.int("toggle_group").max(0) as u32,
        kind,
        area,
        hits,
        self_only,
        vars,
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
        let stages = NamedDbc::open(dbc_dir, "skillstage")?;
        let stages: HashMap<u32, u16> =
            stages.rows().filter_map(|r| Some((u32::try_from(r.id()).ok()?, u16::try_from(r.int("stage_type_id")).ok()?))).collect();
        Ok(Self { spells: table.rows().filter_map(|r| Some((u32::try_from(r.id()).ok()?, definition(r, states, &stages)?))).collect() })
    }

    pub fn len(&self) -> usize {
        self.spells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spells.is_empty()
    }

    /// Arme exigée par la compétence (§5.2).
    pub fn weapon_fits(&self, id: u32, held: Held) -> bool {
        self.spells.get(&id).is_some_and(|d| d.weapons.fits(held))
    }

    pub fn quote(&self, id: u32, level: u8) -> Option<SpellQuote> {
        let d = self.spells.get(&id)?;
        if level == 0 || level > 50 {
            return None;
        }
        let l = f64::from(level);
        let v = d.vars.map(f64::from);
        let hits = (d.hits.0 + d.hits.1 * f32::from(level) + 1e-4).trunc();
        let hits = (1.0..=20.0).contains(&hits).then_some(hits as u8)?;
        let (ratio, flat, hp_ratio) = match d.kind {
            SpellKind::Physical => (v[0] + l * v[1], v[2] + l * v[3], 0.0),
            SpellKind::Magical => (v[0] + l * v[1], v[3] + l * v[4], 0.0),
            SpellKind::Heal => (v[0] + l * v[1], v[2] + l * v[3], v[4] + l * v[5]),
            _ => (0.0, 0.0, 0.0),
        };
        let state = match d.state {
            None => None,
            Some(s) => {
                let value = s.level.0 + (s.level.1 * f32::from(level) + 1e-4).trunc();
                let duration = match d.kind {
                    SpellKind::Toggle => PERMANENT,
                    _ => centiseconds(s.second, level).filter(|d| *d > 0)?,
                };
                let chance = s.chance.0 + s.chance.1 * f32::from(level);
                // Niveau 0 = retrait côté client : un tel état ne peut pas être posé.
                let value = (1.0..=10_000.0).contains(&value).then_some(value as u16)?;
                Some(StateQuote { id: s.id, level: value, duration, chance })
            }
        };
        let toggle = (d.kind == SpellKind::Toggle)
            .then(|| ToggleQuote { upkeep: (v[0] + l * v[1]).clamp(0.0, 1e6) as i32, group: d.toggle_group });
        Some(SpellQuote {
            toggle,
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
            area: d.area,
            hits,
            self_only: d.self_only,
            stage: d.stage,
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
        // 2032 Triple Assaut : 2,9 + 0,1·L coups ; 2043 : coups et étourdissement.
        assert_eq!(book.quote(2032, 1).map(|q| q.hits), Some(3));
        assert_eq!(book.quote(2043, 1).map(|q| (q.hits, q.state.map(|s| s.id))), Some((3, Some(6006))));
        // 2063 : zone de 5 × 12, 10 cibles. 2053 (v6 = 1) et 3025 (v11 = 1) : règle inconnue.
        assert_eq!(book.quote(2063, 1).and_then(|q| q.area), Some(Area { radius: 60.0, max_targets: 10 }));
        assert!(book.quote(2053, 1).is_none() && book.quote(3025, 1).is_none());
        // 3427 Enchantement d'Arme : bascule du groupe 4, 1 + 2·L PM par 5 s… refusée (second état v4) ;
        // 3426 : feu, 1 + 1·L PM.
        let q = book.quote(3426, 2).unwrap();
        assert_eq!((q.kind, q.toggle, q.state.map(|s| s.duration)), (SpellKind::Toggle, Some(ToggleQuote { upkeep: 3, group: 4 }), Some(PERMANENT)));
        assert!(q.self_only && book.quote(3427, 1).is_none());
        // 3201 : sans cible à choisir, sur soi.
        assert!(book.quote(3201, 1).unwrap().self_only);
        // Forme de FIRE : coups d'arme (122) par vagues de cibles, sorts (401) un acte par impact.
        let q = book.quote(2032, 1).unwrap();
        assert_eq!((q.stage, q.fire_counts(1)), (122, (1, 1)));
        assert_eq!((q.hits, SpellQuote { stage: 401, ..q }.fire_counts(4), SpellQuote { stage: 404, ..q }.fire_counts(2)), (3, (12, 1), (6, 6)));
        assert_eq!(SpellQuote { stage: 501, ..q }.fire_counts(100), (255, 1));
        // 45127 et 45116 (zone) : v1 surchargé par l'exécutable. 2501 : refusée.
        if let (Some(a), Some(b)) = (book.quote(45116, 1), book.quote(45116, 2)) {
            assert!((2.0 * a.ratio - b.ratio - 1.5).abs() < 1e-3 && a.area.is_some());
        }
        if let (Some(a), Some(b)) = (book.quote(45127, 1), book.quote(45127, 2)) {
            assert!((2.0 * a.ratio - b.ratio - 1.2).abs() < 1e-3, "v1 surchargé");
        }
        assert!(book.quote(2501, 1).is_none());
        // Invocation : 8,8 s − 0,8 s par niveau ; 60 + 20 PM par niveau.
        let q = book.quote(SUMMON, 1).unwrap();
        assert_eq!((q.kind, q.cast, q.mp, q.cooldown), (SpellKind::Summon, 800, 80, 1000));
        assert_eq!(book.quote(SUMMON, 11).unwrap().cast, 0);
        assert_eq!(book.quote(UNSUMMON, 1).map(|q| (q.cast, q.mp)), Some((50, 5)));
        assert!(book.quote(2001, 0).is_none() && book.quote(2001, 51).is_none());
    }
}
