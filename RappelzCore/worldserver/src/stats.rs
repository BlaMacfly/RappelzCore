//! Statistiques d'un personnage : caractéristiques du métier (BasicStat), passifs appris et
//! armes tenues (docs/re/skill_protocol.md §8.3 et §17).
//!
//! La part due au niveau reste la règle provisoire de `gameplay.rs` : le client ne la donne pas.

use rz_database::characters::Character;
use rz_proto::msg;
use rz_rappelz::passives::Held;
use rz_rappelz::states::{Attributes, Mods, Stat};

use crate::data::GameData;
use crate::gameplay::InvItem;
use crate::session::Session;
use crate::world::World;

/// Emplacements d'équipement.
const RIGHT_HAND: u8 = 0;
const LEFT_HAND: u8 = 1;

/// Ce qui ne change qu'avec le métier, les compétences apprises ou l'équipement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Profile {
    pub attributes: Attributes,
    pub passives: Mods,
    pub held: Held,
}

impl Default for Profile {
    fn default() -> Self {
        Self { attributes: Attributes { base: [10; 7], ranged: false }, passives: Mods::default(), held: Held::default() }
    }
}

pub(crate) fn profile(data: &GameData, c: &Character, items: &[InvItem]) -> Profile {
    let mut p = Profile::default();
    let class_at = |slot: u8| {
        let item = items.iter().find(|i| i.worn() && i.slot == slot);
        item.and_then(|i| data.items.get(&i.entry)).map_or(0, |i| i.class)
    };
    p.held = Held { right: class_at(RIGHT_HAND), left: class_at(LEFT_HAND) };
    p.attributes.ranged = p.held.ranged();
    let base = crate::skills::numbers(c)
        .ok()
        .and_then(|(job, _, _)| data.jobs.get(&job))
        .and_then(|j| data.progression.as_ref()?.base(j.stat_id));
    if let Some([str, vit, dex, agi, int, men, luk]) = base {
        p.attributes.base = [str, vit, agi, dex, int, men, luk].map(i32::from);
    }
    for (id, level) in crate::skills::learned(c).unwrap_or_default() {
        if let Some(m) = data.passives.mods(id, level, p.held) {
            p.passives.add(&m);
        }
    }
    p
}

impl Profile {
    /// PV et PM max sans état en cours.
    pub fn vitals(&self, base_hp: i32, base_mp: i32) -> (i32, i32) {
        let of = |s, base| self.passives.derive(s, base, &self.attributes).max(1);
        (of(Stat::MaxHp, base_hp), of(Stat::MaxMp, base_mp))
    }
}

impl World {
    /// Nouveau métier, équipement ou passif : statistiques et PV/PM max republiés s'ils changent.
    pub(crate) fn set_profile(&mut self, handle: u32, profile: Profile) {
        match self.players.get_mut(&handle) {
            Some(p) if p.profile != profile => p.profile = profile,
            _ => return,
        }
        self.refresh_player(handle);
    }

    /// Recalcule les PV/PM max (annoncés s'ils changent) puis envoie les statistiques.
    pub(crate) fn refresh_player(&mut self, handle: u32) {
        let Some(p) = self.players.get(&handle) else { return };
        let (hp, mp) = (p.base_max_hp, p.base_max_mp);
        let (max_hp, max_mp) = (self.stat(handle, Stat::MaxHp, hp).max(1), self.stat(handle, Stat::MaxMp, mp).max(1));
        let p = self.players.get_mut(&handle).unwrap();
        if (p.max_hp, p.max_mp) != (max_hp, max_mp) {
            (p.max_hp, p.max_mp) = (max_hp, max_mp);
            (p.hp, p.mp) = (p.hp.min(max_hp), p.mp.min(max_mp));
            let creature = &mut p.enter.creature;
            (creature.max_hp, creature.max_mp, creature.hp, creature.mp) = (max_hp, max_mp, p.hp, p.mp);
            let packet = Self::hpmp_packet(handle, p.hp, max_hp, p.mp, max_mp, 0);
            self.broadcast_around(handle, &packet, true);
        }
        self.publish_stats(handle);
    }

    /// TS_SC_STAT_INFO complet (type 0) d'un joueur.
    pub(crate) fn stat_snapshot(&self, owner: u32) -> Option<msg::ScStatInfo> {
        let player = self.players.get(&owner)?;
        let level = player.enter.creature.level;
        let mods = self.mods(owner);
        let a = &player.profile.attributes;
        let of = |s, base| mods.derive(s, base, a);
        let short = |v: i32| v.clamp(0, i32::from(i16::MAX)) as i16;
        // Ordre du paquet : FOR, VIT, DEX, AGI, INT, SAG, CHA.
        let attributes = [0, 1, 3, 2, 4, 5, 6].map(|n| short(mods.attribute(n, a)));
        let mut s = msg::ScStatInfo::from_stats(
            owner,
            &msg::BasicStats {
                attributes,
                // Emplacement gauche conservé tel quel ; droite et défense sont écrites ensuite.
                attack: 10 + level * 2,
                defence: of(Stat::Defence, player.defence),
                magic_point: of(Stat::MagicAttack, 10 + level * 2),
                magic_defence: of(Stat::MagicDefence, 5 + level),
                move_speed: of(Stat::MoveSpeed, i32::from(player.move_speed)).min(255) as i16,
                attack_speed: of(Stat::AttackSpeed, crate::states::BASE_ATTACK_SPEED).min(1000) as i16,
                attack_range: 50,
            },
            0,
        );
        s.stats[0x1f - 0x0b..0x23 - 0x0b].copy_from_slice(&of(Stat::Attack, player.attack).to_le_bytes());
        for (offset, stat, base) in SHOWN {
            let o = offset - 0x0b;
            s.stats[o..o + 2].copy_from_slice(&short(of(*stat, *base)).to_le_bytes());
        }
        // Ces deux-là sont des i32.
        for (o, stat, base) in [(0x2b - 0x0b, Stat::Block, 0), (0x49 - 0x0b, Stat::MaxWeight, 1000)] {
            s.stats[o..o + 4].copy_from_slice(&of(stat, base).to_le_bytes());
        }
        Some(s)
    }
}

/// Statistiques affichées mais sans règle de combat côté client : (décalage dans le paquet,
/// statistique, valeur sans bonus, comme avant ce module).
const SHOWN: &[(usize, Stat, i32)] = &[
    (0x1b, Stat::Critical, 0),
    (0x37, Stat::Accuracy, 100),
    (0x39, Stat::Accuracy, 100),
    (0x3b, Stat::MagicAccuracy, 100),
    (0x3d, Stat::Evasion, 0),
    (0x3f, Stat::MagicResistance, 0),
    (0x41, Stat::BlockChance, 0),
    (0x4d, Stat::CastSpeed, 100),
    (0x53, Stat::HpRegen, 0),
    (0x55, Stat::HpRecovery, 0),
    (0x57, Stat::MpRegen, 0),
    (0x59, Stat::MpRecovery, 0),
];

impl Session {
    /// À appeler quand le métier, les compétences apprises ou l'équipement changent.
    pub(crate) fn refresh_profile(&self) {
        let Some(g) = &self.in_game else { return };
        let profile = profile(&self.ctx.data, &g.character, &g.items);
        self.ctx.world.lock().unwrap().set_profile(g.handle, profile);
    }
}
