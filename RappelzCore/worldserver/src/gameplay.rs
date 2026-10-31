//! Gameplay de la session : inventaire et équipement (`item_instance` / `character_inventory`),
//! dialogues et boutiques des PNJ (`npc_vendor`), combat et récompenses.
//! Messages : `rz_proto::msg_game` (docs/re/network_protocol.md §9).

use anyhow::Result;
use rz_database::characters::{self, BAG_SLOT_BASE};
use rz_proto::msg::Message;
use rz_proto::msg_game::{self as game, id as gid};
use rz_proto::RawPacket;

use crate::data::NPC_ENTRY_OFFSET;
use crate::session::Session;
use crate::world::WorldEvent;

/// Les handles d'objets d'inventaire sont dérivés du guid d'`item_instance`.
pub const ITEM_HANDLE_BASE: u32 = 0x6000_0000;
pub const BAG_CAPACITY: u8 = 120;

#[derive(Clone, Debug)]
pub struct InvItem {
    pub guid: u32,
    pub entry: u32,
    pub count: u32,
    /// `character_inventory.slot` : < 28 équipé (wear_position), sinon sac (index + 28).
    pub slot: u8,
}

impl InvItem {
    pub fn handle(&self) -> u32 {
        ITEM_HANDLE_BASE + self.guid
    }

    pub fn worn(&self) -> bool {
        self.slot < BAG_SLOT_BASE
    }
}

/// Identifiants traités par ce module.
pub fn handles(id: u16) -> bool {
    matches!(
        id,
        gid::TS_CS_PUTON_ITEM
            | gid::TS_CS_SKILL
            | gid::TS_CS_LEARN_SKILL
            | gid::TS_CS_PUTOFF_ITEM
            | gid::TS_CS_DROP_ITEM
            | gid::TS_CS_ERASE_ITEM
            | gid::TS_CS_CONTACT
            | gid::TS_CS_DIALOG
            | gid::TS_CS_BUY_ITEM
            | gid::TS_CS_SELL_ITEM
            | gid::TS_CS_ATTACK_REQUEST
            | gid::TS_CS_CANCEL_ACTION
            | gid::TS_CS_TARGETING
            | gid::TS_CS_QUERY
            | gid::TS_CS_GAME_TIME
    )
}

/// Statistiques dérivées du niveau (en attendant les formules de db_BasicStat / db_Exp).
pub fn max_hp_for(level: u8) -> i32 {
    80 + level as i32 * 20
}
pub fn max_mp_for(level: u8) -> i32 {
    40 + level as i32 * 10
}
pub fn attack_for(level: u8) -> i32 {
    12 + level as i32 * 4
}
pub fn defence_for(level: u8) -> i32 {
    4 + level as i32 * 2
}
/// Expérience nécessaire pour passer le niveau suivant.
pub fn exp_to_next(level: u8) -> i64 {
    let l = level as i64;
    100 * l * l + 200 * l
}

/// Build the complete award without publishing or changing live/session state.
/// XP, JP, money and level-up vitals must commit in the same character save.
fn reward_candidate(mut c: characters::Character, exp: i64, jp: i64, gold: i64) -> (characters::Character, bool) {
    let mut total = i64::from(c.xp).saturating_add(exp.max(0));
    let mut leveled = false;
    while c.level < 255 && total >= exp_to_next(c.level) {
        total -= exp_to_next(c.level);
        c.level += 1;
        leveled = true;
    }
    c.xp = total.clamp(0, u32::MAX as i64) as u32;
    c.set_extra("jp", c.extra_i64("jp", 0).saturating_add(jp.max(0)).max(0));
    c.money = i64::from(c.money).saturating_add(gold.max(0)).min(u32::MAX as i64) as u32;
    if leveled {
        c.health = max_hp_for(c.level) as u32;
        c.power1 = max_mp_for(c.level) as u32;
    }
    (c, leveled)
}

impl Session {
    fn item_info(&self, it: &InvItem) -> game::ItemInfo {
        let endurance = self.ctx.data.items.get(&it.entry).map(|i| i.endurance).unwrap_or(0);
        game::ItemInfo {
            handle: it.handle(),
            code: it.entry,
            uid: it.guid as i64,
            count: it.count as i64,
            endurance,
            level: 1,
            wear_position: if it.worn() { it.slot as i16 } else { game::WEAR_NONE },
            index: if it.worn() { 0 } else { (it.slot - BAG_SLOT_BASE) as i32 },
            ..Default::default()
        }
    }

    pub fn wear_array(items: &[InvItem]) -> [i32; game::WEAR_SLOTS] {
        let mut wear = [0i32; game::WEAR_SLOTS];
        for it in items.iter().filter(|i| i.worn()) {
            wear[it.slot as usize] = it.entry as i32;
        }
        wear
    }

    pub async fn load_items(&self, guid: u32) -> Result<Vec<InvItem>> {
        Ok(characters::inventory(&self.ctx.chars, guid)
            .await?
            .into_iter()
            .map(|i| InvItem { guid: i.item_guid, entry: i.entry, count: i.count, slot: i.slot })
            .collect())
    }

    /// Paquets envoyés après TS_SC_LOGIN_RESULT (séquence observée, §9).
    pub fn send_initial_state(&self) {
        let Some(g) = &self.in_game else { return };
        let items: Vec<game::ItemInfo> = g.items.iter().map(|i| self.item_info(i)).collect();
        for chunk in items.chunks(150) {
            self.send(&game::ScInventory { items: chunk.to_vec() });
        }
        self.send(&game::ScWearInfo { handle: g.handle, wear_info: Self::wear_array(&g.items), ..Default::default() });
        self.send(&game::ScGoldUpdate { gold: g.character.money as i64, chaos: 0 });
        self.send_learned_skills();
        self.send(&game::ScQuestList::default());
        let c = &g.character;
        self.send(&game::ScExpUpdate { handle: g.handle, exp: c.xp as i64, jp: c.extra_i64("jp", 0) });
        self.send(&game::ScHpMp {
            handle: g.handle,
            hp: c.health as i32,
            max_hp: max_hp_for(c.level),
            mp: c.power1 as i32,
            max_mp: max_mp_for(c.level),
            ..Default::default()
        });
    }

    fn free_bag_slot(items: &[InvItem]) -> Option<u8> {
        (BAG_SLOT_BASE..BAG_SLOT_BASE.saturating_add(BAG_CAPACITY)).find(|s| !items.iter().any(|i| i.slot == *s))
    }

    /// Crée un objet dans le sac (achat, commande `.additem`, butin).
    pub async fn give_item(&mut self, entry: u32, count: u32) -> Result<bool> {
        let ctx = self.ctx.clone();
        let Some(g) = &mut self.in_game else { return Ok(false) };
        let Some(slot) = Self::free_bag_slot(&g.items) else {
            return Ok(false);
        };
        let guid = ctx.next_item_guid();
        characters::add_item(&ctx.chars, g.character.guid, guid, entry, count.max(1), 0, slot).await?;
        let it = InvItem { guid, entry, count: count.max(1), slot };
        g.items.push(it.clone());
        let info = self.item_info(&it);
        self.send(&game::ScInventory { items: vec![info] });
        Ok(true)
    }

    async fn remove_item(&mut self, handle: u32) -> Result<Option<InvItem>> {
        let ctx = self.ctx.clone();
        let Some(g) = &mut self.in_game else { return Ok(None) };
        let Some(pos) = g.items.iter().position(|i| i.handle() == handle) else { return Ok(None) };
        let it = g.items.remove(pos);
        characters::delete_item(&ctx.chars, g.character.guid, it.guid).await?;
        self.send(&game::Handle::<{ gid::TS_SC_DESTROY_ITEM }>(handle));
        Ok(Some(it))
    }

    fn set_money(&mut self, money: i64) {
        if let Some(g) = &mut self.in_game {
            g.character.money = money.clamp(0, u32::MAX as i64) as u32;
            let gold = g.character.money as i64;
            self.send(&game::ScGoldUpdate { gold, chaos: 0 });
        }
    }

    fn broadcast_wear(&self) {
        let Some(g) = &self.in_game else { return };
        self.ctx.world.lock().unwrap().set_wear(g.handle, Self::wear_array(&g.items));
    }

    pub async fn handle_gameplay(&mut self, p: RawPacket) -> Result<()> {
        let Some(handle) = self.in_game.as_ref().map(|g| g.handle) else { return Ok(()) };
        match p.id {
            gid::TS_CS_LEARN_SKILL=>self.learn_skill(game::CsLearnSkill::from_packet(&p)?).await?,
            gid::TS_CS_PUTON_ITEM => {
                let req = game::CsPutOnItem::from_packet(&p)?;
                self.put_on(handle, req.position, req.item_handle).await?;
            }
            gid::TS_CS_PUTOFF_ITEM => {
                let req = game::CsPutOffItem::from_packet(&p)?;
                self.put_off(handle, req.position).await?;
            }
            gid::TS_CS_DROP_ITEM => {
                let _request = game::CsDropItem::from_packet(&p)?;
                // Ground loot does not exist yet: a drop must never behave like erase.
                self.ctx.world.lock().unwrap().notify(handle,"Le dépôt au sol n'est pas disponible. L'objet reste dans le sac.");
            }
            gid::TS_CS_ERASE_ITEM => {
                for it in game::CsEraseItem::from_packet(&p)?.items {
                    self.remove_item(it.0).await?;
                }
            }
            gid::TS_CS_CONTACT => self.contact(game::CsContact::from_packet(&p)?.0),
            gid::TS_CS_DIALOG => {
                let trigger = game::CsDialog::from_packet(&p)?.trigger;
                self.dialog_choice(&trigger);
            }
            gid::TS_CS_BUY_ITEM => {
                let req = game::CsBuyItem::from_packet(&p)?;
                self.buy(req.item_code as u32, req.count as u32).await?;
            }
            gid::TS_CS_SELL_ITEM => {
                let req = game::CsSellItem::from_packet(&p)?;
                self.sell(req.item_handle, req.count).await?;
            }
            gid::TS_CS_ATTACK_REQUEST => {
                let req = game::CsAttackRequest::from_packet(&p)?;
                self.ctx.world.lock().unwrap().start_attack(handle, req.target_handle);
            }
            gid::TS_CS_SKILL => self.cast_skill(game::CsSkill::from_packet(&p)?),
            gid::TS_CS_CANCEL_ACTION => {
                let mut world=self.ctx.world.lock().unwrap();
                world.cancel_restoration(handle);world.stop_attack(handle);
            },
            gid::TS_CS_TARGETING => {
                let target = game::CsTargeting::from_packet(&p)?.0;
                let accepted = {
                    let world=self.ctx.world.lock().unwrap();
                    world.can_see(handle,target,world.now())
                };
                self.send(&game::Handle::<{ gid::TS_SC_TARGET }>(if accepted {target} else {0}));
            }
            gid::TS_CS_QUERY => {
                let target = game::CsQuery::from_packet(&p)?.0;
                self.ctx.world.lock().unwrap().query(handle, target);
            }
            gid::TS_CS_GAME_TIME => {
                let now = self.ctx.world.lock().unwrap().now();
                self.send(&game::Pair::<{ gid::TS_SC_GAME_TIME }>(now, chrono::Utc::now().timestamp() as u32));
            }
            _ => {}
        }
        Ok(())
    }

    async fn put_on(&mut self, handle: u32, position: u8, item_handle: u32) -> Result<()> {
        if position as usize >= game::WEAR_SLOTS {
            return Ok(());
        }
        let ctx = self.ctx.clone();
        let Some(g) = &mut self.in_game else { return Ok(()) };
        let Some(idx) = g.items.iter().position(|i| i.handle() == item_handle && !i.worn()) else { return Ok(()) };
        let owner = g.character.guid;
        let bag_slot = g.items[idx].slot;
        // L'objet déjà porté à cet emplacement prend la place libérée dans le sac.
        if let Some(prev) = g.items.iter_mut().find(|i| i.slot == position) {
            prev.slot = bag_slot;
            characters::move_item(&ctx.chars, owner, prev.guid, bag_slot).await?;
            let prev_handle = prev.handle();
            self.send(&game::ScItemWearInfo { item_handle: prev_handle, wear_position: game::WEAR_NONE, target_handle: handle, ..Default::default() });
        }
        let g = self.in_game.as_mut().unwrap();
        g.items[idx].slot = position;
        characters::move_item(&ctx.chars, owner, g.items[idx].guid, position).await?;
        self.send(&game::ScItemWearInfo { item_handle, wear_position: position as i16, target_handle: handle, ..Default::default() });
        self.broadcast_wear();
        Ok(())
    }

    async fn put_off(&mut self, handle: u32, position: u8) -> Result<()> {
        let ctx = self.ctx.clone();
        let Some(g) = &mut self.in_game else { return Ok(()) };
        let Some(free) = Self::free_bag_slot(&g.items) else {
            self.ctx.world.lock().unwrap().notify(handle, "Le sac est plein.");
            return Ok(());
        };
        let owner = g.character.guid;
        let Some(it) = g.items.iter_mut().find(|i| i.slot == position) else { return Ok(()) };
        it.slot = free;
        let (guid, item_handle) = (it.guid, it.handle());
        characters::move_item(&ctx.chars, owner, guid, free).await?;
        self.send(&game::ScItemWearInfo { item_handle, wear_position: game::WEAR_NONE, target_handle: handle, ..Default::default() });
        self.broadcast_wear();
        Ok(())
    }

    fn npc_at(&self, npc_handle: u32) -> Option<(u32, String)> {
        let viewer=self.in_game.as_ref()?.handle;
        let w = self.ctx.world.lock().unwrap();
        w.visible_npc(viewer,npc_handle)
    }

    /// TS_CS_CONTACT → TS_SC_DIALOG : salutation et menu (boutique si `npc_vendor`).
    fn contact(&mut self, npc_handle: u32) {
        let Some((npc_id, name)) = self.npc_at(npc_handle) else { return };
        self.dialog_npc = Some(npc_handle);
        let has_shop = self.ctx.data.vendors.read().unwrap().contains_key(&(NPC_ENTRY_OFFSET + npc_id));
        let mut menu = Vec::new();
        if has_shop {
            menu.push(("Commerce".to_string(), "market".to_string()));
        }
        menu.push(("Au revoir".to_string(), "close".to_string()));
        let text = if has_shop {
            "Bienvenue ! Jetez un œil à mes marchandises.".to_string()
        } else {
            "Bonjour, aventurier. Que puis-je pour vous ?".to_string()
        };
        self.send(&game::ScDialog { dialog_type: game::dialog_type::NPC, npc_handle, title: name, text, menu });
    }

    fn dialog_choice(&mut self, trigger: &str) {
        let Some(npc_handle) = self.dialog_npc else { return };
        if trigger != "market" {
            self.dialog_npc = None;
            return;
        }
        let Some((npc_id, _)) = self.npc_at(npc_handle) else { self.dialog_npc=None; return };
        let items = self.ctx.data.vendors.read().unwrap().get(&(NPC_ENTRY_OFFSET + npc_id)).cloned().unwrap_or_default();
        let items = items
            .into_iter()
            .map(|code| game::MarketItem {
                code: code as i32,
                price: self.ctx.data.items.get(&code).map(|i| i.buy_price).unwrap_or(0),
                ..Default::default()
            })
            .collect();
        self.send(&game::ScMarket { npc_handle, items });
    }

    async fn buy(&mut self, code: u32, count: u32) -> Result<()> {
        let Some(npc_handle) = self.dialog_npc else { return Ok(()) };
        let Some((npc_id, _)) = self.npc_at(npc_handle) else { self.dialog_npc=None; return Ok(()) };
        let sold = self.ctx.data.vendors.read().unwrap().get(&(NPC_ENTRY_OFFSET + npc_id)).is_some_and(|v| v.contains(&code));
        let Some(item) = self.ctx.data.items.get(&code).cloned().filter(|_| sold) else { return Ok(()) };
        let count = count.clamp(1, 999);
        let total = item.buy_price * count as i64;
        let money = self.in_game.as_ref().map(|g| g.character.money as i64).unwrap_or(0);
        let handle = self.in_game.as_ref().map(|g| g.handle).unwrap_or(0);
        if money < total {
            self.ctx.world.lock().unwrap().notify(handle, "Vous n'avez pas assez de rupees.");
            return Ok(());
        }
        if self.give_item(code, count).await? {
            self.set_money(money - total);
            self.ctx.world.lock().unwrap().notify(handle, &format!("Acheté : {} ×{count}", item.name));
        } else {
            self.ctx.world.lock().unwrap().notify(handle, "Le sac est plein.");
        }
        Ok(())
    }

    async fn sell(&mut self, item_handle: u32, _count: i64) -> Result<()> {
        if self.dialog_npc.and_then(|npc|self.npc_at(npc)).is_none() {
            self.dialog_npc=None;
            return Ok(());
        }
        let worn = self.in_game.as_ref().is_some_and(|g| g.items.iter().any(|i| i.handle() == item_handle && i.worn()));
        if worn {
            return Ok(());
        }
        if let Some(it) = self.remove_item(item_handle).await? {
            let price = self.ctx.data.items.get(&it.entry).map(|i| i.sell_price.max(i.buy_price / 10)).unwrap_or(0);
            let money = self.in_game.as_ref().map(|g| g.character.money as i64).unwrap_or(0);
            self.set_money(money + price * it.count as i64);
        }
        Ok(())
    }

    /// Événements du monde (récompenses, mort) appliqués au personnage de la session.
    pub async fn handle_world_event(&mut self, ev: WorldEvent) -> Result<()> {
        let ctx = self.ctx.clone();
        let Some(g) = &self.in_game else { return Ok(()) };
        let handle = g.handle;
        match ev {
            WorldEvent::Reward { exp, jp, gold, monster } => {
                let (candidate, leveled) = reward_candidate(self.character_snapshot().unwrap(), exp, jp, gold);
                // Do not hold the world lock over SQL or expose an award before COMMIT.
                if let Err(error) = characters::save(&ctx.chars, &candidate).await {
                    // A failed COMMIT can have an uncertain outcome. Never let normal
                    // disconnect cleanup overwrite it with the older session snapshot.
                    self.detach_world_without_save();
                    return Err(error.context("reward save failed; closing without a stale disconnect save"));
                }
                let (level, xp, money) = (candidate.level, candidate.xp, candidate.money);
                let jp_total = candidate.extra_i64("jp", 0);
                let jlv = candidate.extra_i64("jlv", 1) as i32;
                self.in_game.as_mut().unwrap().character = candidate;
                self.send(&game::ScExpUpdate { handle, exp: xp as i64, jp: jp_total });
                if gold > 0 {
                    self.send(&game::ScGoldUpdate { gold: money as i64, chaos: 0 });
                }
                let mut w = ctx.world.lock().unwrap();
                w.notify(handle, &format!("{monster} vaincu : +{exp} EXP, +{jp} JP, +{gold} rupees."));
                if leveled {
                    w.set_player_level(handle, level, jlv);
                    w.notify(handle, &format!("Niveau {level} atteint !"));
                }
            }
            WorldEvent::Died => {
                let (x, y) = ctx.settings.start_position(self.in_game.as_ref().unwrap().character.race as i32);
                let mut w = ctx.world.lock().unwrap();
                w.notify(handle, "Vous êtes mort. Retour au point de départ.");
                w.revive(handle, x, y);
            }
        }
        self.send_skill_catalog();
        Ok(())
    }
}

#[cfg(test)]
mod reward_tests {
    use super::*;
    use characters::Character;

    #[test]
    fn reward_candidate_preserves_extras_and_contains_level_gold_and_healed_vitals() {
        let mut c = Character { level:2, xp:799, money:10, health:35, power1:6, position_x:42., ..Default::default() };
        c.set_extra("jp",7);c.set_extra("job",100);c.set_extra("unmapped","keep");
        let (next,leveled)=reward_candidate(c.clone(),5,3,9);
        assert!(leveled);assert_eq!((next.level,next.xp,next.money,next.health,next.power1),(3,4,19,140,70));
        assert_eq!(next.extra_i64("jp",0),10);assert_eq!(next.extra["unmapped"],"keep");
        assert_eq!(next.position_x,42.);assert_eq!((c.level,c.xp,c.money,c.health,c.power1),(2,799,10,35,6));
        assert_eq!(c.extra_i64("jp",0),7);
    }

    #[test]
    fn capped_progress_never_wraps_or_turns_negative() {
        let mut c=Character {level:255,xp:u32::MAX,money:u32::MAX,health:7,power1:3,..Default::default()};
        c.set_extra("jp",i64::MAX-1);
        let (next,leveled)=reward_candidate(c,i64::MAX,3,i64::MAX);
        assert!(!leveled);assert_eq!((next.level,next.xp,next.money,next.health,next.power1),(255,u32::MAX,u32::MAX,7,3));
        assert_eq!(next.extra_i64("jp",0),i64::MAX);
        let (same,_)=reward_candidate(next.clone(),-1,-1,-1);
        assert_eq!((same.xp,same.money,same.extra),(next.xp,next.money,next.extra));
    }
}
