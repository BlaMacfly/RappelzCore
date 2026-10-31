//! Messages de gameplay du client NoLimits (inventaire, PNJ, personnage, combat, divers).
//! Disposition : `docs/re/network_protocol.md` §9. Les offsets en commentaire comptent depuis le
//! début du paquet (en-tête de 7 octets inclus). [O] = observé dans SFrame.exe, [D] = déduit.

use crate::msg::Message;
use crate::packet::{decode_str, encode_str, Reader, Result, Writer};

pub mod id {
    // Inventaire / équipement
    pub const TS_CS_PUTON_ITEM: u16 = 200;
    pub const TS_CS_PUTOFF_ITEM: u16 = 201;
    pub const TS_SC_WEAR_INFO: u16 = 202;
    pub const TS_CS_DROP_ITEM: u16 = 203;
    pub const TS_CS_TAKE_ITEM: u16 = 204;
    pub const TS_SC_DROP_RESULT: u16 = 205;
    pub const TS_SC_INVENTORY: u16 = 207;
    pub const TS_CS_ERASE_ITEM: u16 = 208;
    pub const TS_SC_ERASE_ITEM: u16 = 209;
    pub const TS_SC_TAKE_ITEM_RESULT: u16 = 210;
    pub const TS_SC_BELT_SLOT_INFO: u16 = 216;
    pub const TS_SC_ITEM_COOL_TIME: u16 = 217;
    pub const TS_CS_CHANGE_ITEM_POSITION: u16 = 218;
    pub const TS_CS_USE_ITEM: u16 = 253;
    pub const TS_SC_DESTROY_ITEM: u16 = 254;
    pub const TS_SC_UPDATE_ITEM_COUNT: u16 = 255;
    pub const TS_SC_ITEM_DROP_INFO: u16 = 282;
    pub const TS_SC_USE_ITEM_RESULT: u16 = 283;
    pub const TS_SC_ITEM_WEAR_INFO: u16 = 287;
    pub const TS_SC_GOLD_UPDATE: u16 = 1001;
    // PNJ / boutique
    pub const TS_SC_MARKET: u16 = 250;
    pub const TS_CS_BUY_ITEM: u16 = 251;
    pub const TS_CS_SELL_ITEM: u16 = 252;
    pub const TS_SC_DIALOG: u16 = 3000;
    pub const TS_CS_DIALOG: u16 = 3001;
    pub const TS_CS_CONTACT: u16 = 3002;
    pub const TS_SC_SHOW_WINDOW: u16 = 3003;
    pub const TS_SC_GENERAL_MESSAGE_BOX: u16 = 3004;
    // Personnage
    pub const TS_SC_LEVEL_UPDATE: u16 = 1002;
    pub const TS_SC_EXP_UPDATE: u16 = 1003;
    pub const TS_SC_STATUS_CHANGE: u16 = 500;
    pub const TS_SC_DEAD: u16 = 504;
    pub const TS_SC_STATE: u16 = 505;
    pub const TS_SC_HPMP: u16 = 509;
    pub const TS_SC_SP: u16 = 514;
    pub const TS_SC_ENERGY: u16 = 515;
    pub const TS_SC_SKILL_LIST: u16 = 403;
    pub const TS_SC_ADDED_SKILL_LIST: u16 = 404;
    pub const TS_SC_QUEST_LIST: u16 = 600;
    pub const TS_SC_QUEST_STATUS: u16 = 601;
    // Combat
    pub const TS_CS_ATTACK_REQUEST: u16 = 100;
    pub const TS_SC_ATTACK_EVENT: u16 = 101;
    pub const TS_SC_CANT_ATTACK: u16 = 102;
    pub const TS_CS_CANCEL_ACTION: u16 = 150;
    pub const TS_CS_SKILL: u16 = 400;
    pub const TS_SC_SKILL: u16 = 401;
    pub const TS_CS_LEARN_SKILL: u16 = 402;
    pub const TS_CS_TARGETING: u16 = 511;
    pub const TS_SC_TARGET: u16 = 512;
    pub const TS_CS_RESURRECTION: u16 = 513;
    pub const TS_SC_REGEN_HPMP: u16 = 516;
    pub const TS_SC_EMOTION: u16 = 1201;
    pub const TS_CS_EMOTION: u16 = 1202;
    // Divers
    pub const TS_CS_QUERY: u16 = 13;
    pub const TS_CS_GAME_TIME: u16 = 1100;
    pub const TS_SC_GAME_TIME: u16 = 1101;
    pub const TS_SC_OPEN_URL: u16 = 9000;
    pub const TS_SC_URL_LIST: u16 = 9001;
}

/// Nombre d'emplacements d'équipement de TS_SC_WEAR_INFO.
pub const WEAR_SLOTS: usize = 28;
/// Position d'objet « non porté » (dans le sac).
pub const WEAR_NONE: i16 = -1;

fn write_i32s(w: &mut Writer, v: &[i32]) {
    for x in v {
        w.i32(*x);
    }
}

fn read_i32s<const N: usize>(r: &mut Reader) -> Result<[i32; N]> {
    let mut a = [0i32; N];
    for x in &mut a {
        *x = r.i32()?;
    }
    Ok(a)
}

/// Chaîne de longueur `u16` (sans NUL final) suivie de ses octets.
fn write_str16(w: &mut Writer, s: &str) {
    let b = encode_str(s);
    w.u16(b.len() as u16).bytes(&b);
}

fn read_str16(r: &mut Reader) -> Result<String> {
    let n = (r.u16()? as usize).min(r.remaining());
    Ok(decode_str(r.bytes(n)?))
}

// ───────────────────────────── Objets ─────────────────────────────

/// TS_ITEM_INFO, 331 octets (0x14b) : enregistrement d'objet de TS_SC_INVENTORY (et du commerce).
/// Offsets relatifs au début de l'enregistrement. Taille, `handle`, `code` (u32 en clair), `count`,
/// `flag`, sockets, `wear_position`, `own_summon_handle` : [O] ; reste : [D] (ordre epic 9.5).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemInfo {
    /// +0x00
    pub handle: u32,
    /// +0x04 code de l'objet (ItemResource), non brouillé.
    pub code: u32,
    /// +0x08
    pub uid: i64,
    /// +0x10
    pub count: i64,
    /// +0x18
    pub ethereal_durability: u32,
    /// +0x1c (durabilité × 100 000 selon le client [D])
    pub endurance: u32,
    /// +0x20
    pub enhance: u8,
    /// +0x21
    pub level: u8,
    /// +0x22
    pub enchant: u16,
    /// +0x24
    pub flag: u32,
    /// +0x28
    pub socket: [i32; 4],
    /// +0x38 awaken : value[5] puis data[5]
    pub awaken_value: [u32; 5],
    /// +0x4c
    pub awaken_data: [i32; 5],
    /// +0x60
    pub random_type: [u32; 10],
    /// +0x88
    pub random_value_1: [i64; 10],
    /// +0xd8
    pub random_value_2: [i64; 10],
    /// +0x128 (> 0 : objet à durée limitée)
    pub remain_time: i32,
    /// +0x12c
    pub elemental_effect_type: u8,
    /// +0x12d
    pub elemental_effect_remain_time: i32,
    /// +0x131
    pub elemental_effect_attack_point: i32,
    /// +0x135
    pub elemental_effect_magic_point: i32,
    /// +0x139
    pub appearance_code: i32,
    /// +0x13d
    pub summon_code: u32,
    /// +0x141 position d'équipement (-1 = sac, -2 = traitement spécial du client)
    pub wear_position: i16,
    /// +0x143 créature (invocation) qui porte l'objet, 0 = le joueur
    pub own_summon_handle: u32,
    /// +0x147 index dans l'inventaire
    pub index: i32,
}

impl ItemInfo {
    pub const SIZE: usize = 331;

    pub fn write(&self, w: &mut Writer) {
        w.u32(self.handle)
            .u32(self.code)
            .i64(self.uid)
            .i64(self.count)
            .u32(self.ethereal_durability)
            .u32(self.endurance)
            .u8(self.enhance)
            .u8(self.level)
            .u16(self.enchant)
            .u32(self.flag);
        write_i32s(w, &self.socket);
        self.awaken_value.iter().for_each(|v| {
            w.u32(*v);
        });
        write_i32s(w, &self.awaken_data);
        self.random_type.iter().for_each(|v| {
            w.u32(*v);
        });
        for v in self.random_value_1.iter().chain(&self.random_value_2) {
            w.i64(*v);
        }
        w.i32(self.remain_time)
            .u8(self.elemental_effect_type)
            .i32(self.elemental_effect_remain_time)
            .i32(self.elemental_effect_attack_point)
            .i32(self.elemental_effect_magic_point)
            .i32(self.appearance_code)
            .u32(self.summon_code)
            .u16(self.wear_position as u16)
            .u32(self.own_summon_handle)
            .i32(self.index);
    }

    pub fn read(r: &mut Reader) -> Result<Self> {
        let mut it = Self {
            handle: r.u32()?,
            code: r.u32()?,
            uid: r.i64()?,
            count: r.i64()?,
            ethereal_durability: r.u32()?,
            endurance: r.u32()?,
            enhance: r.u8()?,
            level: r.u8()?,
            enchant: r.u16()?,
            flag: r.u32()?,
            socket: read_i32s(r)?,
            ..Default::default()
        };
        for v in &mut it.awaken_value {
            *v = r.u32()?;
        }
        it.awaken_data = read_i32s(r)?;
        for v in &mut it.random_type {
            *v = r.u32()?;
        }
        for v in it.random_value_1.iter_mut().chain(it.random_value_2.iter_mut()) {
            *v = r.i64()?;
        }
        it.remain_time = r.i32()?;
        it.elemental_effect_type = r.u8()?;
        it.elemental_effect_remain_time = r.i32()?;
        it.elemental_effect_attack_point = r.i32()?;
        it.elemental_effect_magic_point = r.i32()?;
        it.appearance_code = r.i32()?;
        it.summon_code = r.u32()?;
        it.wear_position = r.u16()? as i16;
        it.own_summon_handle = r.u32()?;
        it.index = r.i32()?;
        Ok(it)
    }
}

/// TS_SC_INVENTORY (207), `9 + 331·n` : `+7 u16 count`, `+9 ItemInfo[count]`. Kind 0x16 [O].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScInventory {
    pub items: Vec<ItemInfo>,
}

impl Message for ScInventory {
    const ID: u16 = id::TS_SC_INVENTORY;
    fn write(&self, w: &mut Writer) {
        w.u16(self.items.len() as u16);
        self.items.iter().for_each(|i| i.write(w));
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let n = r.u16()?;
        Ok(Self { items: (0..n).map(|_| ItemInfo::read(r)).collect::<Result<_>>()? })
    }
}

/// TS_SC_WEAR_INFO (202), 487 octets : apparence équipée d'une créature. Kind 0x11 [O].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScWearInfo {
    /// +7
    pub handle: u32,
    /// +0x0b codes d'objets par emplacement
    pub wear_info: [i32; WEAR_SLOTS],
    /// +0x7b
    pub wear_item_enhance_info: [i32; WEAR_SLOTS],
    /// +0xeb
    pub wear_item_level_info: [i32; WEAR_SLOTS],
    /// +0x15b
    pub wear_item_elemental_type: [u8; WEAR_SLOTS],
    /// +0x177
    pub wear_appearance_code: [i32; WEAR_SLOTS],
}

impl Message for ScWearInfo {
    const ID: u16 = id::TS_SC_WEAR_INFO;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle);
        write_i32s(w, &self.wear_info);
        write_i32s(w, &self.wear_item_enhance_info);
        write_i32s(w, &self.wear_item_level_info);
        w.bytes(&self.wear_item_elemental_type);
        write_i32s(w, &self.wear_appearance_code);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let handle = r.u32()?;
        let wear_info = read_i32s(r)?;
        let wear_item_enhance_info = read_i32s(r)?;
        let wear_item_level_info = read_i32s(r)?;
        let wear_item_elemental_type = r.bytes(WEAR_SLOTS)?.try_into().unwrap();
        let wear_appearance_code = read_i32s(r)?;
        Ok(Self { handle, wear_info, wear_item_enhance_info, wear_item_level_info, wear_item_elemental_type, wear_appearance_code })
    }
}

/// TS_SC_ITEM_WEAR_INFO (287), 26 octets : un objet change de position d'équipement. Kind 0x12.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScItemWearInfo {
    /// +7 [O]
    pub item_handle: u32,
    /// +0x0b [O] (-1 = retiré)
    pub wear_position: i16,
    /// +0x0d [O] porteur (joueur ou invocation)
    pub target_handle: u32,
    /// +0x11 [D] (non recopié par le parseur)
    pub enhance: i32,
    /// +0x15 [D]
    pub elemental_effect_type: u8,
    /// +0x16 [D]
    pub appearance_code: i32,
}

impl Message for ScItemWearInfo {
    const ID: u16 = id::TS_SC_ITEM_WEAR_INFO;
    fn write(&self, w: &mut Writer) {
        w.u32(self.item_handle)
            .u16(self.wear_position as u16)
            .u32(self.target_handle)
            .i32(self.enhance)
            .u8(self.elemental_effect_type)
            .i32(self.appearance_code);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            item_handle: r.u32()?,
            wear_position: r.u16()? as i16,
            target_handle: r.u32()?,
            enhance: r.i32()?,
            elemental_effect_type: r.u8()?,
            appearance_code: r.i32()?,
        })
    }
}

/// TS_CS_PUTON_ITEM (200), 16 octets [O] (FUN_006d7e30) : `+7 u8 position`, `+8 u32 item_handle`,
/// `+0xc u32 target_handle` (porteur : soi-même ou une invocation).
#[derive(Clone, Debug, PartialEq)]
pub struct CsPutOnItem {
    pub position: u8,
    pub item_handle: u32,
    pub target_handle: u32,
}

impl Message for CsPutOnItem {
    const ID: u16 = id::TS_CS_PUTON_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u8(self.position).u32(self.item_handle).u32(self.target_handle);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { position: r.u8()?, item_handle: r.u32()?, target_handle: r.u32()? })
    }
}

/// TS_CS_PUTOFF_ITEM (201), 12 octets [O] : `+7 u8 position`, `+8 u32 target_handle`.
#[derive(Clone, Debug, PartialEq)]
pub struct CsPutOffItem {
    pub position: u8,
    pub target_handle: u32,
}

impl Message for CsPutOffItem {
    const ID: u16 = id::TS_CS_PUTOFF_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u8(self.position).u32(self.target_handle);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { position: r.u8()?, target_handle: r.u32()? })
    }
}

/// TS_CS_DROP_ITEM (203), 15 octets [O] (FUN_006d7cc0) : `+7 u32 item_handle`, `+0xb u32 count`.
#[derive(Clone, Debug, PartialEq)]
pub struct CsDropItem {
    pub item_handle: u32,
    pub count: u32,
}

impl Message for CsDropItem {
    const ID: u16 = id::TS_CS_DROP_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u32(self.item_handle).u32(self.count);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { item_handle: r.u32()?, count: r.u32()? })
    }
}

/// TS_CS_TAKE_ITEM (204), 15 octets [O] (FUN_0096bb80) : `+7 u32 taker_handle` (soi),
/// `+0xb u32 item_handle` (objet au sol).
#[derive(Clone, Debug, PartialEq)]
pub struct CsTakeItem {
    pub taker_handle: u32,
    pub item_handle: u32,
}

impl Message for CsTakeItem {
    const ID: u16 = id::TS_CS_TAKE_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u32(self.taker_handle).u32(self.item_handle);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { taker_handle: r.u32()?, item_handle: r.u32()? })
    }
}

/// TS_CS_ERASE_ITEM (208), `8 + 13·n` [O taille] : `+7 u8 count`, entrées de 13 octets
/// (`u32 handle`, `i64 count`, `u8` [D ordre]).
#[derive(Clone, Debug, PartialEq)]
pub struct CsEraseItem {
    pub items: Vec<(u32, i64, u8)>,
}

impl Message for CsEraseItem {
    const ID: u16 = id::TS_CS_ERASE_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u8(self.items.len() as u8);
        for (h, c, f) in &self.items {
            w.u32(*h).i64(*c).u8(*f);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let n = r.u8()?;
        Ok(Self { items: (0..n).map(|_| Ok((r.u32()?, r.i64()?, r.u8()?))).collect::<Result<_>>()? })
    }
}

/// TS_SC_ERASE_ITEM (209, absent de la table TM_), `8 + 12·n` [O taille] : `+7 u8 count`,
/// entrées `u32 handle`, `i64 count` [D]. Kind 0xaa.
#[derive(Clone, Debug, PartialEq)]
pub struct ScEraseItem {
    pub items: Vec<(u32, i64)>,
}

impl Message for ScEraseItem {
    const ID: u16 = id::TS_SC_ERASE_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u8(self.items.len() as u8);
        for (h, c) in &self.items {
            w.u32(*h).i64(*c);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let n = r.u8()?;
        Ok(Self { items: (0..n).map(|_| Ok((r.u32()?, r.i64()?))).collect::<Result<_>>()? })
    }
}

/// TS_CS_CHANGE_ITEM_POSITION (218), 16 octets [O taille] : `+7 u8 is_storage`,
/// `+8 u32 item_handle_1`, `+0xc u32 item_handle_2` [D].
#[derive(Clone, Debug, PartialEq)]
pub struct CsChangeItemPosition {
    pub is_storage: bool,
    pub item_handle_1: u32,
    pub item_handle_2: u32,
}

impl Message for CsChangeItemPosition {
    const ID: u16 = id::TS_CS_CHANGE_ITEM_POSITION;
    fn write(&self, w: &mut Writer) {
        w.bool(self.is_storage).u32(self.item_handle_1).u32(self.item_handle_2);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { is_storage: r.bool()?, item_handle_1: r.u32()?, item_handle_2: r.u32()? })
    }
}

/// TS_CS_USE_ITEM (253), 47 octets [O] (FUN_0096f9f0) : `+7 u32 item_handle`,
/// `+0xb u32 target_handle`, `+0xf char parameter[32]`.
#[derive(Clone, Debug, PartialEq)]
pub struct CsUseItem {
    pub item_handle: u32,
    pub target_handle: u32,
    pub parameter: String,
}

impl Message for CsUseItem {
    const ID: u16 = id::TS_CS_USE_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u32(self.item_handle).u32(self.target_handle).fixed_str(&self.parameter, 32);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { item_handle: r.u32()?, target_handle: r.u32()?, parameter: r.fixed_str(32)? })
    }
}

/// Messages d'un seul `u32` (handle) : TS_SC_DESTROY_ITEM (254, kind 0x17), TS_CS_CONTACT (3002),
/// TS_CS_CANCEL_ACTION (150), TS_CS_TARGETING (511), TS_SC_TARGET (512), TS_CS_QUERY (13), 11 octets [O].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Handle<const ID: u16>(pub u32);

impl<const I: u16> Message for Handle<I> {
    const ID: u16 = I;
    fn write(&self, w: &mut Writer) {
        w.u32(self.0);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self(r.u32()?))
    }
}

pub type ScDestroyItem = Handle<{ id::TS_SC_DESTROY_ITEM }>;
pub type CsContact = Handle<{ id::TS_CS_CONTACT }>;
pub type CsCancelAction = Handle<{ id::TS_CS_CANCEL_ACTION }>;
pub type CsTargeting = Handle<{ id::TS_CS_TARGETING }>;
pub type ScTarget = Handle<{ id::TS_SC_TARGET }>;
pub type CsQuery = Handle<{ id::TS_CS_QUERY }>;

/// Messages de deux `u32` (15 octets) [O tailles] :
/// TS_SC_TAKE_ITEM_RESULT (210, kind 0x49) `item_handle, taker_handle` ;
/// TS_SC_ITEM_DROP_INFO (282, kind 0x14) `monster_handle, item_handle` [D noms] ;
/// TS_SC_USE_ITEM_RESULT (283, kind 0x1a) `item_handle, target_handle` [D noms] ;
/// TS_SC_STATUS_CHANGE (500, kind 0x31) `handle, status` ;
/// TS_SC_EMOTION (1201, kind 0x51) `handle, emotion` ;
/// TS_SC_GAME_TIME (1101, kind 0x52) `t (horloge serveur), game_time`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pair<const ID: u16>(pub u32, pub u32);

impl<const I: u16> Message for Pair<I> {
    const ID: u16 = I;
    fn write(&self, w: &mut Writer) {
        w.u32(self.0).u32(self.1);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self(r.u32()?, r.u32()?))
    }
}

pub type ScTakeItemResult = Pair<{ id::TS_SC_TAKE_ITEM_RESULT }>;
pub type ScItemDropInfo = Pair<{ id::TS_SC_ITEM_DROP_INFO }>;
pub type ScUseItemResult = Pair<{ id::TS_SC_USE_ITEM_RESULT }>;
pub type ScStatusChange = Pair<{ id::TS_SC_STATUS_CHANGE }>;
pub type ScEmotion = Pair<{ id::TS_SC_EMOTION }>;
pub type ScGameTime = Pair<{ id::TS_SC_GAME_TIME }>;

/// TS_SC_DROP_RESULT (205), 12 octets [O] : `+7 u32 item_handle`, `+0xb u8 is_success`. Kind 0x13.
#[derive(Clone, Debug, PartialEq)]
pub struct ScDropResult {
    pub item_handle: u32,
    pub is_success: bool,
}

impl Message for ScDropResult {
    const ID: u16 = id::TS_SC_DROP_RESULT;
    fn write(&self, w: &mut Writer) {
        w.u32(self.item_handle).bool(self.is_success);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { item_handle: r.u32()?, is_success: r.bool()? })
    }
}

/// TS_SC_UPDATE_ITEM_COUNT (255), 19 octets [O] : `+7 u32 item_handle`, `+0xb i64 count`. Kind 0x18.
#[derive(Clone, Debug, PartialEq)]
pub struct ScUpdateItemCount {
    pub item_handle: u32,
    pub count: i64,
}

impl Message for ScUpdateItemCount {
    const ID: u16 = id::TS_SC_UPDATE_ITEM_COUNT;
    fn write(&self, w: &mut Writer) {
        w.u32(self.item_handle).i64(self.count);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { item_handle: r.u32()?, count: r.i64()? })
    }
}

/// TS_SC_BELT_SLOT_INFO (216), 39 octets [O] : `+7 u32 handle[8]` (objets de ceinture). Kind 0x55.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScBeltSlotInfo {
    pub handles: [u32; 8],
}

impl Message for ScBeltSlotInfo {
    const ID: u16 = id::TS_SC_BELT_SLOT_INFO;
    fn write(&self, w: &mut Writer) {
        self.handles.iter().for_each(|h| {
            w.u32(*h);
        });
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let mut handles = [0u32; 8];
        for h in &mut handles {
            *h = r.u32()?;
        }
        Ok(Self { handles })
    }
}

/// TS_SC_ITEM_COOL_TIME (217), 167 octets [O] : `+7 i32 cool_time[40]`. Kind 0x57.
#[derive(Clone, Debug, PartialEq)]
pub struct ScItemCoolTime {
    pub cool_time: [i32; 40],
}

impl Message for ScItemCoolTime {
    const ID: u16 = id::TS_SC_ITEM_COOL_TIME;
    fn write(&self, w: &mut Writer) {
        write_i32s(w, &self.cool_time);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { cool_time: read_i32s(r)? })
    }
}

/// TS_SC_GOLD_UPDATE (1001), 19 octets [O taille] : `+7 i64 gold`, `+0xf i32 chaos` [D]. Kind 0x20.
#[derive(Clone, Debug, PartialEq)]
pub struct ScGoldUpdate {
    pub gold: i64,
    pub chaos: i32,
}

impl Message for ScGoldUpdate {
    const ID: u16 = id::TS_SC_GOLD_UPDATE;
    fn write(&self, w: &mut Writer) {
        w.i64(self.gold).i32(self.chaos);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { gold: r.i64()?, chaos: r.i32()? })
    }
}

// ───────────────────────────── PNJ ─────────────────────────────

/// Types de TS_SC_DIALOG : les menus ne sont lus que pour 0, 3 et 10 [O].
pub mod dialog_type {
    pub const NPC: i32 = 0;
    pub const MESSAGE: i32 = 1;
    pub const MENU: i32 = 3;
    pub const SPECIAL: i32 = 10;
}

/// TS_SC_DIALOG (3000) [O] (FUN_00a25670, kind SMSG_NPC_DIALOG) : `+7 i32 type`,
/// `+0xb u32 npc_handle`, `+0xf u16 title_len`, `+0x11 u16 text_len`, `+0x13 u16 menu_len`,
/// `+0x15 title`, `text`, `menu` (sans NUL). Le menu est découpé sur `'\t'` :
/// `"\t<texte>\t<trigger>"` répété [D appariement]. Texte `"QUEST|<id>|..."` = dialogue de quête.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScDialog {
    pub dialog_type: i32,
    pub npc_handle: u32,
    pub title: String,
    pub text: String,
    /// (texte affiché, trigger renvoyé par TS_CS_DIALOG)
    pub menu: Vec<(String, String)>,
}

impl ScDialog {
    pub fn menu_string(&self) -> String {
        self.menu.iter().map(|(t, k)| format!("\t{t}\t{k}")).collect()
    }
}

impl Message for ScDialog {
    const ID: u16 = id::TS_SC_DIALOG;
    fn write(&self, w: &mut Writer) {
        let (t, x, m) = (encode_str(&self.title), encode_str(&self.text), encode_str(&self.menu_string()));
        w.i32(self.dialog_type).u32(self.npc_handle).u16(t.len() as u16).u16(x.len() as u16).u16(m.len() as u16);
        w.bytes(&t).bytes(&x).bytes(&m);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let dialog_type = r.i32()?;
        let npc_handle = r.u32()?;
        let (tl, xl, ml) = (r.u16()? as usize, r.u16()? as usize, r.u16()? as usize);
        let title = decode_str(r.bytes(tl)?);
        let text = decode_str(r.bytes(xl)?);
        let menu_s = decode_str(r.bytes(ml)?);
        let parts: Vec<&str> = menu_s.split('\t').skip(1).collect();
        let menu = parts.chunks(2).map(|c| (c[0].to_string(), c.get(1).unwrap_or(&"").to_string())).collect();
        Ok(Self { dialog_type, npc_handle, title, text, menu })
    }
}

/// TS_CS_DIALOG (3001), `9 + len` [O] : `+7 u16 len`, `+9 trigger[len]` (choix de menu).
#[derive(Clone, Debug, PartialEq)]
pub struct CsDialog {
    pub trigger: String,
}

impl Message for CsDialog {
    const ID: u16 = id::TS_CS_DIALOG;
    fn write(&self, w: &mut Writer) {
        write_str16(w, &self.trigger);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { trigger: read_str16(r)? })
    }
}

/// TS_SC_GENERAL_MESSAGE_BOX (3004), `9 + len` [O] : `+7 u16 len`, `+9 text[len]`. Kind 0xb5.
#[derive(Clone, Debug, PartialEq)]
pub struct ScGeneralMessageBox {
    pub text: String,
}

impl Message for ScGeneralMessageBox {
    const ID: u16 = id::TS_SC_GENERAL_MESSAGE_BOX;
    fn write(&self, w: &mut Writer) {
        write_str16(w, &self.text);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { text: read_str16(r)? })
    }
}

/// Entrée de boutique, 20 octets [O taille, D champs].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarketItem {
    pub code: i32,
    pub price: i64,
    pub huntaholic_point: i32,
    pub arena_point: i32,
}

/// TS_SC_MARKET (250), `13 + 20·n` [O] : `+7 u32 npc_handle`, `+0xb u16 count`, `+0xd MarketItem[]`.
/// Kind 0x3c.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScMarket {
    pub npc_handle: u32,
    pub items: Vec<MarketItem>,
}

impl Message for ScMarket {
    const ID: u16 = id::TS_SC_MARKET;
    fn write(&self, w: &mut Writer) {
        w.u32(self.npc_handle).u16(self.items.len() as u16);
        for i in &self.items {
            w.i32(i.code).i64(i.price).i32(i.huntaholic_point).i32(i.arena_point);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let npc_handle = r.u32()?;
        let n = r.u16()?;
        let items = (0..n)
            .map(|_| Ok(MarketItem { code: r.i32()?, price: r.i64()?, huntaholic_point: r.i32()?, arena_point: r.i32()? }))
            .collect::<Result<_>>()?;
        Ok(Self { npc_handle, items })
    }
}

/// TS_CS_BUY_ITEM (251), 13 octets [O] (FUN_006d2e60 kind 0x416) : `+7 i32 item_code`, `+0xb u16 count`.
#[derive(Clone, Debug, PartialEq)]
pub struct CsBuyItem {
    pub item_code: i32,
    pub count: u16,
}

impl Message for CsBuyItem {
    const ID: u16 = id::TS_CS_BUY_ITEM;
    fn write(&self, w: &mut Writer) {
        w.i32(self.item_code).u16(self.count);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { item_code: r.i32()?, count: r.u16()? })
    }
}

/// TS_CS_SELL_ITEM (252), 19 octets [O] (kind UI 0x418) : `+7 u32 item_handle`, `+0xb i64 count`.
#[derive(Clone, Debug, PartialEq)]
pub struct CsSellItem {
    pub item_handle: u32,
    pub count: i64,
}

impl Message for CsSellItem {
    const ID: u16 = id::TS_CS_SELL_ITEM;
    fn write(&self, w: &mut Writer) {
        w.u32(self.item_handle).i64(self.count);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { item_handle: r.u32()?, count: r.i64()? })
    }
}

// ───────────────────────────── Personnage ─────────────────────────────

/// TS_SC_LEVEL_UPDATE (1002), 19 octets [O] : `+7 u32 handle`, `+0xb i32 level`, `+0xf i32 job_level`.
/// Kind 0x21 (log « OnLevelUpdate [handle][lv][jl] »).
#[derive(Clone, Debug, PartialEq)]
pub struct ScLevelUpdate {
    pub handle: u32,
    pub level: i32,
    pub job_level: i32,
}

impl Message for ScLevelUpdate {
    const ID: u16 = id::TS_SC_LEVEL_UPDATE;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).i32(self.level).i32(self.job_level);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()?, level: r.i32()?, job_level: r.i32()? })
    }
}

/// TS_SC_EXP_UPDATE (1003), 27 octets [O taille] : `+7 u32 handle`, `+0xb i64 exp`, `+0x13 i64 jp` [D].
/// Kind 0x22.
#[derive(Clone, Debug, PartialEq)]
pub struct ScExpUpdate {
    pub handle: u32,
    pub exp: i64,
    pub jp: i64,
}

impl Message for ScExpUpdate {
    const ID: u16 = id::TS_SC_EXP_UPDATE;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).i64(self.exp).i64(self.jp);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()?, exp: r.i64()?, jp: r.i64()? })
    }
}

/// TS_SC_HPMP (509), 36 octets [O offsets, D noms] : `+7 handle`, `+0xb add_hp`, `+0xf hp`,
/// `+0x13 max_hp`, `+0x17 add_mp`, `+0x1b mp`, `+0x1f max_mp` (i32), `+0x23 u8 need_to_display`.
/// Kind 0x37.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScHpMp {
    pub handle: u32,
    pub add_hp: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub add_mp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub need_to_display: bool,
}

impl Message for ScHpMp {
    const ID: u16 = id::TS_SC_HPMP;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle)
            .i32(self.add_hp)
            .i32(self.hp)
            .i32(self.max_hp)
            .i32(self.add_mp)
            .i32(self.mp)
            .i32(self.max_mp)
            .bool(self.need_to_display);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            handle: r.u32()?,
            add_hp: r.i32()?,
            hp: r.i32()?,
            max_hp: r.i32()?,
            add_mp: r.i32()?,
            mp: r.i32()?,
            max_mp: r.i32()?,
            need_to_display: r.bool()?,
        })
    }
}

/// TS_SC_REGEN_HPMP (516), 27 octets [O largeurs] : `+7 handle`, `+0xb i32 hp_regen`,
/// `+0xf i32 mp_regen`, `+0x13 i32 hp`, `+0x17 i32 mp` [D noms]. Kind 0x56.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScRegenHpMp {
    pub handle: u32,
    pub hp_regen: i32,
    pub mp_regen: i32,
    pub hp: i32,
    pub mp: i32,
}

impl Message for ScRegenHpMp {
    const ID: u16 = id::TS_SC_REGEN_HPMP;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).i32(self.hp_regen).i32(self.mp_regen).i32(self.hp).i32(self.mp);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()?, hp_regen: r.i32()?, mp_regen: r.i32()?, hp: r.i32()?, mp: r.i32()? })
    }
}

/// TS_SC_SP (514), 15 octets [O] : `+7 handle`, `+0xb i16 sp`, `+0xd i16 max_sp`. Kind 0x48.
#[derive(Clone, Debug, PartialEq)]
pub struct ScSp {
    pub handle: u32,
    pub sp: i16,
    pub max_sp: i16,
}

impl Message for ScSp {
    const ID: u16 = id::TS_SC_SP;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).u16(self.sp as u16).u16(self.max_sp as u16);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()?, sp: r.u16()? as i16, max_sp: r.u16()? as i16 })
    }
}

/// TS_SC_ENERGY (515), 13 octets [O] : `+7 handle`, `+0xb i16 energy`. Kind 0x54.
#[derive(Clone, Debug, PartialEq)]
pub struct ScEnergy {
    pub handle: u32,
    pub energy: i16,
}

impl Message for ScEnergy {
    const ID: u16 = id::TS_SC_ENERGY;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).u16(self.energy as u16);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()?, energy: r.u16()? as i16 })
    }
}

/// TS_SC_STATE (505), 31 octets [O offsets, D noms] : état (buff/debuff). Kind 0x34.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScState {
    /// +7
    pub handle: u32,
    /// +0x0b
    pub state_handle: u16,
    /// +0x0d
    pub state_level: u16,
    /// +0x0f
    pub state_code: u32,
    /// +0x13 (-1 = permanent)
    pub remain_time: i32,
    /// +0x17
    pub start_time: u32,
    /// +0x1b
    pub state_value: i32,
}

impl Message for ScState {
    const ID: u16 = id::TS_SC_STATE;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle)
            .u16(self.state_handle)
            .u16(self.state_level)
            .u32(self.state_code)
            .i32(self.remain_time)
            .u32(self.start_time)
            .i32(self.state_value);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            handle: r.u32()?,
            state_handle: r.u16()?,
            state_level: r.u16()?,
            state_code: r.u32()?,
            remain_time: r.i32()?,
            start_time: r.u32()?,
            state_value: r.i32()?,
        })
    }
}

/// Compétence de TS_SC_SKILL_LIST, 14 octets [O] (FUN_00a2ae90).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillInfo {
    pub skill_id: i32,
    pub base_skill_level: u8,
    pub current_skill_level: u8,
    pub total_cool_time: u32,
    pub remain_cool_time: u32,
}

/// TS_SC_SKILL_LIST (403), `14 + 14·n` [O] : `+7 u32 target`, `+0xb u16 count`,
/// `+0xd u8 modification_type`, `+0xe SkillInfo[count]`. Kind 0x2d.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScSkillList {
    pub target: u32,
    pub modification_type: u8,
    pub skills: Vec<SkillInfo>,
}

impl Message for ScSkillList {
    const ID: u16 = id::TS_SC_SKILL_LIST;
    fn write(&self, w: &mut Writer) {
        w.u32(self.target).u16(self.skills.len() as u16).u8(self.modification_type);
        for s in &self.skills {
            w.i32(s.skill_id).u8(s.base_skill_level).u8(s.current_skill_level).u32(s.total_cool_time).u32(s.remain_cool_time);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let target = r.u32()?;
        let n = r.u16()?;
        let modification_type = r.u8()?;
        let skills = (0..n)
            .map(|_| {
                Ok(SkillInfo {
                    skill_id: r.i32()?,
                    base_skill_level: r.u8()?,
                    current_skill_level: r.u8()?,
                    total_cool_time: r.u32()?,
                    remain_cool_time: r.u32()?,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Self { target, modification_type, skills })
    }
}

/// TS_CS_LEARN_SKILL (402), 21 bytes. Size/three u32 + i16 observed locally;
/// field semantics corroborated by pinned rzu/NGemity >=8.2 headers, not a retail capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsLearnSkill { pub target:u32, pub skill_id:i32, pub base_random_skill_id:u32, pub skill_level:i16 }
impl Message for CsLearnSkill {
    const ID:u16=id::TS_CS_LEARN_SKILL;
    fn write(&self,w:&mut Writer){w.u32(self.target).i32(self.skill_id).u32(self.base_random_skill_id).u16(self.skill_level as u16);}
    fn read(r:&mut Reader)->Result<Self>{
        let request=Self {target:r.u32()?,skill_id:r.i32()?,base_random_skill_id:r.u32()?,skill_level:r.u16()? as i16};
        if r.remaining()!=0{return Err(crate::packet::PacketError::Invalid {id:Self::ID,field:"learn request length"});}Ok(request)
    }
}

/// Quête active de TS_SC_QUEST_LIST, 61 octets [O offsets] (FUN_00a29430).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuestInfo {
    pub code: i32,
    pub start_id: i32,
    pub value: [i32; 6],
    pub status: [i32; 6],
    pub progress: u8,
    pub time_limit: i32,
}

/// TS_SC_QUEST_LIST (600), `11 + 61·a + 8·p` [O] : `+7 u16 active_count`, `+9 u16 pending_count`,
/// `+0xb QuestInfo[active]`, puis `{i32 code, i32 start_id}[pending]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScQuestList {
    pub active: Vec<QuestInfo>,
    pub pending: Vec<(i32, i32)>,
}

impl Message for ScQuestList {
    const ID: u16 = id::TS_SC_QUEST_LIST;
    fn write(&self, w: &mut Writer) {
        w.u16(self.active.len() as u16).u16(self.pending.len() as u16);
        for q in &self.active {
            w.i32(q.code).i32(q.start_id);
            write_i32s(w, &q.value);
            write_i32s(w, &q.status);
            w.u8(q.progress).i32(q.time_limit);
        }
        for (c, s) in &self.pending {
            w.i32(*c).i32(*s);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let (a, p) = (r.u16()?, r.u16()?);
        let active = (0..a)
            .map(|_| {
                Ok(QuestInfo {
                    code: r.i32()?,
                    start_id: r.i32()?,
                    value: read_i32s(r)?,
                    status: read_i32s(r)?,
                    progress: r.u8()?,
                    time_limit: r.i32()?,
                })
            })
            .collect::<Result<_>>()?;
        let pending = (0..p).map(|_| Ok((r.i32()?, r.i32()?))).collect::<Result<_>>()?;
        Ok(Self { active, pending })
    }
}

// ───────────────────────────── Combat ─────────────────────────────

/// TS_CS_ATTACK_REQUEST (100), 15 octets [O] (ctor FUN_00969de0, envoi FUN_0096b6c0) :
/// `+7 u32 handle` (soi), `+0xb u32 target_handle`.
#[derive(Clone, Debug, PartialEq)]
pub struct CsAttackRequest {
    pub handle: u32,
    pub target_handle: u32,
}

impl Message for CsAttackRequest {
    const ID: u16 = id::TS_CS_ATTACK_REQUEST;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).u32(self.target_handle);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()?, target_handle: r.u32()? })
    }
}

/// Résultat d'un coup, 61 octets (0x3d) [O taille, D champs].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttackInfo {
    pub damage: i32,
    pub mp_damage: i32,
    pub flag: u8,
    pub elemental_damage: [i32; 7],
    pub target_hp: i64,
    pub target_mp: i32,
    pub attacker_hp: i64,
    pub attacker_mp: i32,
}

impl AttackInfo {
    pub const SIZE: usize = 61;
}

/// `attack_flag` / `flag` d'ATTACK_INFO [D] : bit 0 = raté, 1 = critique, 2 = bloqué, 3 = parfait.
pub mod attack_action {
    pub const ATTACK: u8 = 0;
    pub const AIMING: u8 = 1;
    pub const END: u8 = 2;
    pub const CANCEL: u8 = 3;
}

/// TS_SC_ATTACK_EVENT (101), `22 + 61·n` [O] : `+7 u32 attacker`, `+0xb u32 target`,
/// `+0xf u16 attack_speed`, `+0x11 u16 attack_delay`, `+0x13 u8 attack_action`, `+0x14 u8 attack_flag`,
/// `+0x15 u8 count`, `+0x16 AttackInfo[count]`. Kind 0xc. Si `attack_action` vaut 1 ou 4 et que
/// `target == 0`, le client prend `target = attacker` [O].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScAttackEvent {
    pub attacker: u32,
    pub target: u32,
    pub attack_speed: u16,
    pub attack_delay: u16,
    pub attack_action: u8,
    pub attack_flag: u8,
    pub attacks: Vec<AttackInfo>,
}

impl Message for ScAttackEvent {
    const ID: u16 = id::TS_SC_ATTACK_EVENT;
    fn write(&self, w: &mut Writer) {
        w.u32(self.attacker)
            .u32(self.target)
            .u16(self.attack_speed)
            .u16(self.attack_delay)
            .u8(self.attack_action)
            .u8(self.attack_flag)
            .u8(self.attacks.len() as u8);
        for a in &self.attacks {
            w.i32(a.damage).i32(a.mp_damage).u8(a.flag);
            write_i32s(w, &a.elemental_damage);
            w.i64(a.target_hp).i32(a.target_mp).i64(a.attacker_hp).i32(a.attacker_mp);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let mut e = Self {
            attacker: r.u32()?,
            target: r.u32()?,
            attack_speed: r.u16()?,
            attack_delay: r.u16()?,
            attack_action: r.u8()?,
            attack_flag: r.u8()?,
            attacks: Vec::new(),
        };
        let n = r.u8()?;
        for _ in 0..n {
            e.attacks.push(AttackInfo {
                damage: r.i32()?,
                mp_damage: r.i32()?,
                flag: r.u8()?,
                elemental_damage: read_i32s(r)?,
                target_hp: r.i64()?,
                target_mp: r.i32()?,
                attacker_hp: r.i64()?,
                attacker_mp: r.i32()?,
            });
        }
        Ok(e)
    }
}

/// TS_SC_CANT_ATTACK (102), 19 octets [O] : `+7 u32 attacker`, `+0xb u32 target`, `+0xf i32 reason`.
/// Kind 0xe.
#[derive(Clone, Debug, PartialEq)]
pub struct ScCantAttack {
    pub attacker: u32,
    pub target: u32,
    pub reason: i32,
}

impl Message for ScCantAttack {
    const ID: u16 = id::TS_SC_CANT_ATTACK;
    fn write(&self, w: &mut Writer) {
        w.u32(self.attacker).u32(self.target).i32(self.reason);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { attacker: r.u32()?, target: r.u32()?, reason: r.i32()? })
    }
}

/// TS_CS_SKILL (400), 31 octets [O] (FUN_0096b840) : `+7 u16 skill_id`, `+9 u32 caster` (soi),
/// `+0xd u32 target`, `+0x11 f32 x`, `+0x15 f32 y`, `+0x19 f32 z`, `+0x1d u8 layer`, `+0x1e u8 skill_level`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CsSkill {
    pub skill_id: u16,
    pub caster: u32,
    pub target: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: u8,
    pub skill_level: u8,
}

impl Message for CsSkill {
    const ID: u16 = id::TS_CS_SKILL;
    fn write(&self, w: &mut Writer) {
        w.u16(self.skill_id)
            .u32(self.caster)
            .u32(self.target)
            .f32(self.x)
            .f32(self.y)
            .f32(self.z)
            .u8(self.layer)
            .u8(self.skill_level);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            skill_id: r.u16()?,
            caster: r.u32()?,
            target: r.u32()?,
            x: r.f32()?,
            y: r.f32()?,
            z: r.f32()?,
            layer: r.u8()?,
            skill_level: r.u8()?,
        })
    }
}

/// `type` de TS_SC_SKILL : 0 = FIRE [O] ; autres valeurs [D].
pub mod skill_type {
    pub const FIRE: u8 = 0;
    pub const CASTING: u8 = 1;
    pub const CASTING_UPDATE: u8 = 2;
    pub const CANCEL: u8 = 3;
    pub const REGION_FIRE: u8 = 4;
    pub const COMPLETE: u8 = 5;
}

/// Types de « hit » de TS_SC_SKILL (FIRE) et taille de chaque entrée [O] : 0/1 → 32, 2 → 45,
/// 10 → 10, 20/21 → 13, 22 → 25, 23 → 23, 30 → 19, 40/41 → 36, 42 → 17.
pub mod skill_hit {
    pub const DAMAGE: u8 = 0;
    pub const MAGIC_DAMAGE: u8 = 1;
    pub const DAMAGE_WITH_KNOCK_BACK: u8 = 2;
    pub const RESULT: u8 = 10;
    pub const ADD_HP: u8 = 20;
    pub const ADD_MP: u8 = 21;
    pub const ADD_HP_MP_SP: u8 = 22;
    pub const REBIRTH: u8 = 23;
    pub const RUSH: u8 = 30;
    pub const CHAIN_DAMAGE: u8 = 40;
    pub const CHAIN_MAGIC_DAMAGE: u8 = 41;
    pub const CHAIN_HEAL: u8 = 42;
}

/// Hit DAMAGE / MAGIC_DAMAGE, 32 octets [O taille, D champs].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillDamageHit {
    pub hit_type: u8,
    pub target: u32,
    pub target_hp: i32,
    pub damage_type: u8,
    pub damage: i32,
    pub flag: i32,
    pub elemental_damage: [u16; 7],
}

impl SkillDamageHit {
    pub fn write(&self, w: &mut Writer) {
        w.u8(self.hit_type).u32(self.target).i32(self.target_hp).u8(self.damage_type).i32(self.damage).i32(self.flag);
        self.elemental_damage.iter().for_each(|v| {
            w.u16(*v);
        });
    }
}

/// TS_SC_SKILL (401) [O] (FUN_00a2a7f0, kind SMSG_SKILL_EVENT) : en-tête de 48 octets
/// `+7 u16 skill_id`, `+9 u8 skill_level`, `+0xa u32 caster`, `+0xe u32 target`, `+0x12 f32 x`,
/// `+0x16 f32 y`, `+0x1a f32 z`, `+0x1e u8 layer`, `+0x1f u8 type`, `+0x20 i32 hp_cost`,
/// `+0x24 i32 mp_cost`, `+0x28 i32 caster_hp`, `+0x2c i32 caster_mp`, puis `payload` selon `type`.
/// FIRE : `+0x30 u8 multiple`, `+0x31 f32 range`, `+0x35 u8 target_count`, `+0x36 u8 fire_count`,
/// `+0x37 u16 count`, `+0x39 hits` (voir [`skill_hit`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScSkill {
    pub skill_id: u16,
    pub skill_level: u8,
    pub caster: u32,
    pub target: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: u8,
    pub skill_type: u8,
    pub hp_cost: i32,
    pub mp_cost: i32,
    pub caster_hp: i32,
    pub caster_mp: i32,
    pub payload: Vec<u8>,
}

impl ScSkill {
    /// Corps FIRE avec des hits de dégâts.
    pub fn fire_payload(multiple: bool, range: f32, target_count: u8, fire_count: u8, hits: &[SkillDamageHit]) -> Vec<u8> {
        let mut w = Writer::new();
        w.bool(multiple).f32(range).u8(target_count).u8(fire_count).u16(hits.len() as u16);
        hits.iter().for_each(|h| h.write(&mut w));
        w.finish(0).body
    }
}

impl Message for ScSkill {
    const ID: u16 = id::TS_SC_SKILL;
    fn write(&self, w: &mut Writer) {
        w.u16(self.skill_id)
            .u8(self.skill_level)
            .u32(self.caster)
            .u32(self.target)
            .f32(self.x)
            .f32(self.y)
            .f32(self.z)
            .u8(self.layer)
            .u8(self.skill_type)
            .i32(self.hp_cost)
            .i32(self.mp_cost)
            .i32(self.caster_hp)
            .i32(self.caster_mp)
            .bytes(&self.payload);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            skill_id: r.u16()?,
            skill_level: r.u8()?,
            caster: r.u32()?,
            target: r.u32()?,
            x: r.f32()?,
            y: r.f32()?,
            z: r.f32()?,
            layer: r.u8()?,
            skill_type: r.u8()?,
            hp_cost: r.i32()?,
            mp_cost: r.i32()?,
            caster_hp: r.i32()?,
            caster_mp: r.i32()?,
            payload: r.rest().to_vec(),
        })
    }
}

/// TS_CS_RESURRECTION (513), 12 octets [O taille] : `+7 u32 handle`, `+0xb u8 type` (0..3).
#[derive(Clone, Debug, PartialEq)]
pub struct CsResurrection {
    pub handle: u32,
    pub res_type: u8,
}

impl Message for CsResurrection {
    const ID: u16 = id::TS_CS_RESURRECTION;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).u8(self.res_type);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()?, res_type: r.u8()? })
    }
}

/// TS_CS_EMOTION (1202), 11 octets [O taille] : `+7 i32 emotion` [D].
#[derive(Clone, Debug, PartialEq)]
pub struct CsEmotion {
    pub emotion: i32,
}

impl Message for CsEmotion {
    const ID: u16 = id::TS_CS_EMOTION;
    fn write(&self, w: &mut Writer) {
        w.i32(self.emotion);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { emotion: r.i32()? })
    }
}

// ───────────────────────────── Divers ─────────────────────────────

/// TS_SC_URL_LIST (9001), `9 + len` [O] : `+7 u16 len`, `+9 "clé|valeur|clé|valeur..."` ; chaque paire
/// est copiée dans l'environnement du client (`web_download_port` converti en entier).
/// Ignoré si le client a l'option `not_use_url`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScUrlList {
    pub entries: Vec<(String, String)>,
}

impl Message for ScUrlList {
    const ID: u16 = id::TS_SC_URL_LIST;
    fn write(&self, w: &mut Writer) {
        let s = self.entries.iter().map(|(k, v)| format!("{k}|{v}")).collect::<Vec<_>>().join("|");
        write_str16(w, &s);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let s = read_str16(r)?;
        let parts: Vec<&str> = s.split('|').collect();
        Ok(Self { entries: parts.chunks(2).filter(|c| c.len() == 2).map(|c| (c[0].into(), c[1].into())).collect() })
    }
}

/// TS_SC_OPEN_URL (9000), `18 + len` [O offsets] : `+7 u8 wait_for_event_scene` [D], `+8 u32 width`,
/// `+0xc u32 height`, `+0x10 u16 len`, `+0x12 url[len]`. Kind 100.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScOpenUrl {
    pub wait_for_event_scene: bool,
    pub width: u32,
    pub height: u32,
    pub url: String,
}

impl Message for ScOpenUrl {
    const ID: u16 = id::TS_SC_OPEN_URL;
    fn write(&self, w: &mut Writer) {
        w.bool(self.wait_for_event_scene).u32(self.width).u32(self.height);
        write_str16(w, &self.url);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { wait_for_event_scene: r.bool()?, width: r.u32()?, height: r.u32()?, url: read_str16(r)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::Empty;

    fn size_of<M: Message>(m: &M) -> usize {
        m.to_packet().to_bytes().len()
    }

    fn roundtrip<M: Message + PartialEq + std::fmt::Debug>(m: &M) {
        assert_eq!(&M::from_packet(&m.to_packet()).unwrap(), m);
    }

    #[test]
    fn observed_sizes() {
        let mut w = Writer::new();
        ItemInfo::default().write(&mut w);
        assert_eq!(w.len(), ItemInfo::SIZE);
        assert_eq!(size_of(&ScInventory { items: vec![ItemInfo::default(); 2] }), 9 + 2 * 331);
        assert_eq!(size_of(&ScWearInfo::default()), 487);
        assert_eq!(size_of(&ScItemWearInfo::default()), 26);
        assert_eq!(size_of(&CsPutOnItem { position: 0, item_handle: 1, target_handle: 2 }), 16);
        assert_eq!(size_of(&CsPutOffItem { position: 0, target_handle: 2 }), 12);
        assert_eq!(size_of(&CsDropItem { item_handle: 1, count: 1 }), 15);
        assert_eq!(size_of(&CsTakeItem { taker_handle: 1, item_handle: 2 }), 15);
        assert_eq!(size_of(&CsEraseItem { items: vec![(1, 1, 0)] }), 8 + 13);
        assert_eq!(size_of(&ScEraseItem { items: vec![(1, 1)] }), 8 + 12);
        assert_eq!(size_of(&CsChangeItemPosition { is_storage: false, item_handle_1: 1, item_handle_2: 2 }), 16);
        assert_eq!(size_of(&CsUseItem { item_handle: 1, target_handle: 0, parameter: String::new() }), 47);
        assert_eq!(size_of(&ScDestroyItem { 0: 1 }), 11);
        assert_eq!(size_of(&ScUpdateItemCount { item_handle: 1, count: 5 }), 19);
        assert_eq!(size_of(&ScDropResult { item_handle: 1, is_success: true }), 12);
        assert_eq!(size_of(&ScTakeItemResult { 0: 1, 1: 2 }), 15);
        assert_eq!(size_of(&ScBeltSlotInfo::default()), 39);
        assert_eq!(size_of(&ScItemCoolTime { cool_time: [0; 40] }), 167);
        assert_eq!(size_of(&ScGoldUpdate { gold: 1, chaos: 0 }), 19);
        assert_eq!(size_of(&CsContact { 0: 1 }), 11);
        assert_eq!(size_of(&CsDialog { trigger: String::new() }), 9);
        assert_eq!(size_of(&ScDialog::default()), 21);
        assert_eq!(size_of(&ScMarket { npc_handle: 1, items: vec![MarketItem::default()] }), 13 + 20);
        assert_eq!(size_of(&CsBuyItem { item_code: 1, count: 1 }), 13);
        assert_eq!(size_of(&CsSellItem { item_handle: 1, count: 1 }), 19);
        assert_eq!(size_of(&ScLevelUpdate { handle: 1, level: 1, job_level: 1 }), 19);
        assert_eq!(size_of(&ScExpUpdate { handle: 1, exp: 1, jp: 1 }), 27);
        assert_eq!(size_of(&ScHpMp::default()), 36);
        assert_eq!(size_of(&ScRegenHpMp::default()), 27);
        assert_eq!(size_of(&ScSp { handle: 1, sp: 0, max_sp: 0 }), 15);
        assert_eq!(size_of(&ScEnergy { handle: 1, energy: 0 }), 13);
        assert_eq!(size_of(&ScState::default()), 31);
        assert_eq!(size_of(&ScStatusChange { 0: 1, 1: 0 }), 15);
        assert_eq!(size_of(&ScSkillList { target: 1, modification_type: 0, skills: vec![SkillInfo::default()] }), 14 + 14);
        let q = ScQuestList { active: vec![QuestInfo::default()], pending: vec![(1, 2)] };
        assert_eq!(size_of(&q), 11 + 61 + 8);
        assert_eq!(size_of(&CsAttackRequest { handle: 1, target_handle: 2 }), 15);
        let ev = ScAttackEvent { attacks: vec![AttackInfo::default()], ..Default::default() };
        assert_eq!(size_of(&ev), 22 + AttackInfo::SIZE);
        assert_eq!(size_of(&ScCantAttack { attacker: 1, target: 2, reason: 0 }), 19);
        assert_eq!(size_of(&CsCancelAction { 0: 1 }), 11);
        assert_eq!(size_of(&CsSkill::default()), 31);
        let fire = ScSkill::fire_payload(false, 0.0, 1, 1, &[SkillDamageHit::default()]);
        assert_eq!(fire.len(), 9 + 32);
        assert_eq!(size_of(&ScSkill { payload: fire, ..Default::default() }), 0x39 + 32);
        assert_eq!(size_of(&CsTargeting { 0: 1 }), 11);
        assert_eq!(size_of(&ScTarget { 0: 1 }), 11);
        assert_eq!(size_of(&CsResurrection { handle: 1, res_type: 0 }), 12);
        assert_eq!(size_of(&ScEmotion { 0: 1, 1: 2 }), 15);
        assert_eq!(size_of(&CsEmotion { emotion: 1 }), 11);
        assert_eq!(size_of(&CsQuery { 0: 1 }), 11);
        assert_eq!(size_of(&Empty::<{ id::TS_CS_GAME_TIME }>), 7);
        assert_eq!(size_of(&ScGameTime { 0: 1, 1: 2 }), 15);
        assert_eq!(size_of(&ScOpenUrl::default()), 18);
        assert_eq!(size_of(&ScUrlList::default()), 9);
        assert_eq!(size_of(&ScGeneralMessageBox { text: "abc".into() }), 12);
    }

    #[test]
    fn item_offsets() {
        let it = ItemInfo { handle: 1, code: 2, count: 3, flag: 4, wear_position: 5, own_summon_handle: 6, index: 7, remain_time: 8, ..Default::default() };
        let mut w = Writer::new();
        it.write(&mut w);
        let b = w.finish(0).body;
        let at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        assert_eq!((at(0), at(4), at(0x10), at(0x24), at(0x128)), (1, 2, 3, 4, 8));
        assert_eq!(i16::from_le_bytes([b[0x141], b[0x142]]), 5);
        assert_eq!((at(0x143), at(0x147)), (6, 7));
    }

    #[test]
    fn roundtrips() {
        let it = ItemInfo { handle: 0x8000_0001, code: 101100, count: 7, enhance: 3, wear_position: WEAR_NONE, socket: [1, 2, 3, 4], random_value_2: [9; 10], ..Default::default() };
        roundtrip(&ScInventory { items: vec![it.clone(), ItemInfo { wear_position: 2, ..it }] });
        roundtrip(&ScWearInfo { handle: 5, wear_info: [3; WEAR_SLOTS], wear_item_elemental_type: [1; WEAR_SLOTS], ..Default::default() });
        roundtrip(&ScItemWearInfo { item_handle: 1, wear_position: -1, target_handle: 3, enhance: 4, elemental_effect_type: 5, appearance_code: 6 });
        roundtrip(&CsPutOnItem { position: 2, item_handle: 10, target_handle: 11 });
        roundtrip(&CsEraseItem { items: vec![(1, 2, 3), (4, 5, 6)] });
        roundtrip(&CsUseItem { item_handle: 1, target_handle: 2, parameter: "x".into() });
        let dlg = ScDialog {
            dialog_type: dialog_type::NPC,
            npc_handle: 0x8000_0010,
            title: "Marchand".into(),
            text: "Bonjour, voyageur.".into(),
            menu: vec![("Acheter".into(), "open_market()".into()), ("Au revoir".into(), String::new())],
        };
        roundtrip(&dlg);
        roundtrip(&CsDialog { trigger: "open_market()".into() });
        roundtrip(&ScMarket { npc_handle: 3, items: vec![MarketItem { code: 1, price: 100, ..Default::default() }] });
        roundtrip(&ScHpMp { handle: 1, hp: 50, max_hp: 100, need_to_display: true, ..Default::default() });
        roundtrip(&ScSkillList { target: 1, modification_type: 0, skills: vec![SkillInfo { skill_id: 6001, base_skill_level: 1, current_skill_level: 1, ..Default::default() }] });
        roundtrip(&ScQuestList { active: vec![QuestInfo { code: 1, progress: 1, status: [1, 0, 0, 0, 0, 0], ..Default::default() }], pending: vec![(2, 3)] });
        let mut ev = ScAttackEvent { attacker: 1, target: 2, attack_speed: 100, attack_delay: 50, ..Default::default() };
        ev.attacks.push(AttackInfo { damage: 12, target_hp: 88, ..Default::default() });
        roundtrip(&ev);
        roundtrip(&CsSkill { skill_id: 6001, caster: 1, target: 2, x: 1.0, y: 2.0, z: 3.0, layer: 0, skill_level: 1 });
        roundtrip(&ScSkill { skill_id: 6001, caster: 1, target: 2, payload: ScSkill::fire_payload(false, 0.0, 1, 1, &[SkillDamageHit::default()]), ..Default::default() });
        roundtrip(&ScUrlList { entries: vec![("help.url".into(), "http://x".into())] });
        roundtrip(&ScOpenUrl { wait_for_event_scene: false, width: 800, height: 600, url: "http://x".into() });
        roundtrip(&ScGameTime { 0: 1234, 1: 5678 });
    }
}
