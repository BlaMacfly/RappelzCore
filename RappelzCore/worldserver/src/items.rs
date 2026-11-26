//! Utilisation des objets consommables (TS_CS_USE_ITEM 253, docs/re/item_protocol.md) : soins,
//! états et paquets. Les effets viennent de `rz_rappelz::consumables`.

use anyhow::Result;
use rz_database::characters;
use rz_proto::msg::{self, Message};
use rz_proto::msg_game::{self as game, id as gid};
use rz_rappelz::consumables::UseEffect;
use rz_rappelz::spells::StateQuote;

use crate::session::Session;
use crate::world::World;

/// Recharge propre à un objet sans groupe.
const OWN_GROUP: u32 = 0x8000_0000;
/// Groupes affichés par le client : TS_SC_ITEM_COOL_TIME, indice = groupe − 1 (FUN_006e6fa0).
const SHOWN_GROUPS: u32 = 40;

impl World {
    #[cfg(test)]
    pub(crate) fn test_restore(&mut self, handle: u32, hp: i32, mp: i32) -> (i32, i32) {
        self.restore(handle, hp, mp)
    }

    #[cfg(test)]
    pub(crate) fn test_item_cooldown(&mut self, guid: u32, key: u32, duration: u32) {
        self.start_item_cooldown(guid, key, duration)
    }

    /// Reste de recharge (centièmes) d'un groupe d'objets du personnage.
    pub(crate) fn item_cooldown(&self, guid: u32, key: u32) -> u32 {
        let now = self.cast_clock();
        self.item_cooldowns.get(&(guid, key)).map_or(0, |end| end.saturating_sub(now) as u32)
    }

    /// Restes de recharge (centièmes) des groupes 1 à 40. Le client ne démarre aucune recharge
    /// seul, et n'accepte une valeur que pour un emplacement libre.
    pub(crate) fn item_cool_times(&self, guid: u32) -> game::ScItemCoolTime {
        game::ScItemCoolTime { cool_time: std::array::from_fn(|i| self.item_cooldown(guid, i as u32 + 1) as i32) }
    }

    fn start_item_cooldown(&mut self, guid: u32, key: u32, duration: u32) {
        let now = self.cast_clock();
        self.item_cooldowns.retain(|_, end| *end > now);
        if duration > 0 && self.item_cooldowns.len() < 8192 {
            self.item_cooldowns.insert((guid, key), now + u64::from(duration));
        }
    }

    /// Soin instantané plafonné ; renvoie le gain réel (PV, PM).
    fn restore(&mut self, handle: u32, hp: i32, mp: i32) -> (i32, i32) {
        let Some(p) = self.players.get_mut(&handle).filter(|p| p.hp > 0) else { return (0, 0) };
        let (hp_before, mp_before) = (p.hp, p.mp);
        p.hp = p.hp.saturating_add(hp).min(p.max_hp);
        p.mp = p.mp.saturating_add(mp).min(p.max_mp);
        (p.enter.creature.hp, p.enter.creature.mp) = (p.hp, p.mp);
        let gained = (p.hp - hp_before, p.mp - mp_before);
        let packet = game::ScHpMp {
            handle,
            add_hp: gained.0,
            hp: p.hp,
            max_hp: p.max_hp,
            add_mp: gained.1,
            mp: p.mp,
            max_mp: p.max_mp,
            need_to_display: true,
        }
        .to_packet();
        self.broadcast_around(handle, &packet, true);
        gained
    }
}

impl Session {
    /// TS_CS_USE_ITEM : objet du sac, sur soi. Réponse 283 en cas de succès, TS_SC_RESULT sinon.
    pub(crate) async fn use_item(&mut self, req: game::CsUseItem) -> Result<()> {
        let outcome = self.try_use_item(&req).await?;
        if let Err(why) = outcome {
            self.send(&msg::ScResult { request: gid::TS_CS_USE_ITEM, result: msg::result::ACCESS_DENIED, value: 0 });
            if let Some(g) = &self.in_game {
                self.ctx.world.lock().unwrap().notify(g.handle, why);
            }
        }
        Ok(())
    }

    async fn try_use_item(&mut self, req: &game::CsUseItem) -> Result<std::result::Result<(), &'static str>> {
        let ctx = self.ctx.clone();
        let Some(g) = &self.in_game else { return Ok(Err("Hors du monde.")) };
        let (handle, guid, level) = (g.handle, g.character.guid, i32::from(g.character.level));
        let Some(it) = g.items.iter().find(|i| i.handle() == req.item_handle && i.in_bag()).cloned() else {
            return Ok(Err("Objet introuvable."));
        };
        let Some(item) = ctx.data.consumables.get(it.entry) else { return Ok(Err("Cet objet ne peut pas encore être utilisé.")) };
        // Effets sur soi seulement (target = soi, ou 0).
        if req.target_handle != handle && req.target_handle != 0 {
            return Ok(Err("Cible non prise en charge."));
        }
        if level < item.min_level || (item.max_level > 0 && level > item.max_level) {
            return Ok(Err("Niveau insuffisant."));
        }
        let key = if item.group != 0 { item.group } else { OWN_GROUP | it.entry };
        // États connus et paquets possibles avant de rien consommer.
        let mut states = Vec::new();
        let mut unpack = None;
        for e in &item.effects {
            match *e {
                UseEffect::State { id, level, duration } => match ctx.data.states.get(id).filter(|d| !d.harmful) {
                    Some(def) => states.push((StateQuote { id, level, duration, chance: 100.0 }, def.clone())),
                    None => return Ok(Err("Effet de cet objet non pris en charge.")),
                },
                UseEffect::Unpack { item, count } => {
                    if !ctx.data.items.contains_key(&item) {
                        return Ok(Err("Contenu inconnu."));
                    }
                    unpack = Some((item, count));
                }
                _ => {}
            }
        }
        {
            let w = ctx.world.lock().unwrap();
            if w.players.get(&handle).is_none_or(|p| p.hp <= 0) {
                return Ok(Err("Vous êtes mort."));
            }
            if w.item_cooldown(guid, key) > 0 {
                return Ok(Err("Objet en recharge."));
            }
        }
        // Contenu d'abord : si le sac est plein, rien n'est consommé.
        if let Some((entry, count)) = unpack {
            if !self.give_item(entry, count).await? {
                return Ok(Err("Sac plein."));
            }
        }
        self.consume_stack(it.guid).await?;
        let mut w = ctx.world.lock().unwrap();
        w.start_item_cooldown(guid, key, item.cooldown);
        for e in &item.effects {
            match *e {
                UseEffect::Heal(n) => drop(w.restore(handle, n, 0)),
                UseEffect::Mana(n) => drop(w.restore(handle, 0, n)),
                _ => {}
            }
        }
        for (quote, def) in states {
            w.apply_state(handle, handle, &quote, def, 0);
        }
        let cool = (item.group > 0 && item.group <= SHOWN_GROUPS).then(|| w.item_cool_times(guid));
        drop(w);
        if let Some(cool) = cool {
            self.send(&cool);
        }
        self.send(&game::Pair::<{ gid::TS_SC_USE_ITEM_RESULT }>(req.item_handle, handle));
        Ok(Ok(()))
    }

    /// Retire un exemplaire d'une pile précise.
    async fn consume_stack(&mut self, item_guid: u32) -> Result<()> {
        let ctx = self.ctx.clone();
        let Some(g) = &mut self.in_game else { return Ok(()) };
        let owner = g.character.guid;
        let Some(it) = g.items.iter_mut().find(|i| i.guid == item_guid) else { return Ok(()) };
        it.count = it.count.saturating_sub(1);
        let (handle, left) = (it.handle(), it.count);
        if left == 0 {
            g.items.retain(|i| i.guid != item_guid);
            characters::delete_item(&ctx.chars, owner, item_guid).await?;
            self.send(&game::Handle::<{ gid::TS_SC_DESTROY_ITEM }>(handle));
        } else {
            characters::set_item_count(&ctx.chars, item_guid, left).await?;
            self.send(&game::ScUpdateItemCount { item_handle: handle, count: i64::from(left) });
        }
        self.refresh_quests().await?;
        Ok(())
    }
}
