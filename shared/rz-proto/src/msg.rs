//! Messages TS_* du client NoLimits (offsets observés : `docs/re/network_protocol.md` §5).
//! Les offsets en commentaire comptent depuis le début du paquet (en-tête de 7 octets inclus).

use crate::packet::{RawPacket, Reader, Result, Writer};

/// Version envoyée par le client NoLimits dans TS_CA_VERSION / TS_CS_VERSION.
pub const CLIENT_VERSION: &str = "201507080";

pub mod id {
    // Auth
    pub const TS_AC_RESULT: u16 = 10000;
    pub const TS_CA_VERSION: u16 = 10001;
    pub const TS_AC_RESULT_WITH_STRING: u16 = 10002;
    pub const TS_CA_ACCOUNT: u16 = 10010;
    pub const TS_CA_SERVER_LIST: u16 = 10021;
    pub const TS_AC_SERVER_LIST: u16 = 10022;
    pub const TS_CA_SELECT_SERVER: u16 = 10023;
    pub const TS_AC_SELECT_SERVER: u16 = 10024;
    pub const TS_CA_RSA_PUBLIC_KEY: u16 = 71;
    pub const TS_AC_AES_KEY_IV: u16 = 72;
    // Jeu
    pub const TS_SC_RESULT: u16 = 0;
    pub const TS_TIMESYNC: u16 = 2;
    pub const TS_SC_ENTER: u16 = 3;
    pub const TS_SC_MOVE: u16 = 8;
    pub const TS_SC_LEAVE: u16 = 9;
    pub const TS_SC_SET_TIME: u16 = 10;
    pub const TS_SC_REGION_ACK: u16 = 11;
    pub const TS_SC_WARP: u16 = 12;
    pub const TS_CS_QUERY: u16 = 13;
    pub const TS_CS_CHAT_REQUEST: u16 = 20;
    pub const TS_SC_CHAT_LOCAL: u16 = 21;
    pub const TS_SC_CHAT: u16 = 22;
    pub const TS_CS_RETURN_LOBBY: u16 = 23;
    pub const TS_CS_REQUEST_RETURN_LOBBY: u16 = 25;
    pub const TS_CS_LOGOUT: u16 = 27;
    pub const TS_SC_DISCONNECT_DESC: u16 = 28;
    pub const TS_CS_VERSION: u16 = 51;
    pub const TS_CS_GAME_GUARD_AUTH_ANSWER: u16 = 56;
    pub const TS_CS_CHECK_ILLEGAL_USER: u16 = 57;
    pub const TS_CS_LOGIN: u16 = 62;
    pub const TS_CS_MOVE_REQUEST: u16 = 63;
    pub const TS_SC_LOGIN_RESULT: u16 = 64;
    pub const TS_SC_PROPERTY: u16 = 507;
    pub const TS_CS_TARGETING: u16 = 511;
    pub const TS_SC_STAT_INFO: u16 = 1000;
    pub const TS_CS_REGION_UPDATE: u16 = 67;
    pub const TS_CS_UPDATE: u16 = 503;
    pub const TS_CS_GET_REGION_INFO: u16 = 550;
    pub const TS_CS_GAME_TIME: u16 = 1100;
    pub const TS_SC_GAME_TIME: u16 = 1101;
    pub const TS_SC_GOLD_UPDATE: u16 = 1001;
    pub const TS_SC_LEVEL_UPDATE: u16 = 1002;
    pub const TS_SC_EXP_UPDATE: u16 = 1003;
    pub const TS_CS_CHARACTER_LIST: u16 = 2001;
    pub const TS_CS_CREATE_CHARACTER: u16 = 2002;
    pub const TS_CS_DELETE_CHARACTER: u16 = 2003;
    pub const TS_SC_CHARACTER_LIST: u16 = 2004;
    pub const TS_CS_ACCOUNT_WITH_AUTH: u16 = 2005;
    pub const TS_CS_CHECK_CHARACTER_NAME: u16 = 2006;
    pub const TS_CS_REPORT: u16 = 8000;
}

/// Codes de résultat (TS_AC_RESULT / TS_SC_RESULT) utilisés par RappelzCore.
pub mod result {
    pub const SUCCESS: u16 = 0;
    pub const NOT_EXIST: u16 = 1;
    pub const TOO_MUCH: u16 = 5;
    pub const ACCESS_DENIED: u16 = 6;
    pub const ALREADY_EXIST: u16 = 9;
    pub const INVALID_PASSWORD: u16 = 10;
    pub const INVALID_TEXT: u16 = 0x13;
    pub const DB_ERROR: u16 = 0x16;
    pub const BLOCKED: u16 = 0x3c;
}

pub trait Message: Sized {
    const ID: u16;
    fn write(&self, w: &mut Writer);
    fn read(r: &mut Reader) -> Result<Self>;

    fn to_packet(&self) -> RawPacket {
        let mut w = Writer::new();
        self.write(&mut w);
        w.finish(Self::ID)
    }

    fn from_packet(p: &RawPacket) -> Result<Self> {
        Self::read(&mut p.reader())
    }
}

// ───────────────────────────── Auth ─────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    pub version: String,
}

pub struct CaVersion(pub Version);
pub struct CsVersion(pub Version);

impl Message for CaVersion {
    const ID: u16 = id::TS_CA_VERSION;
    fn write(&self, w: &mut Writer) {
        w.fixed_str(&self.0.version, 20);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self(Version { version: r.fixed_str(20)? }))
    }
}

impl Message for CsVersion {
    const ID: u16 = id::TS_CS_VERSION;
    fn write(&self, w: &mut Writer) {
        w.fixed_str(&self.0.version, 20);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self(Version { version: r.fixed_str(20)? }))
    }
}

/// TS_CA_RSA_PUBLIC_KEY (71) : `+7 u32 n`, `+11 PEM[n]`.
pub struct CaRsaPublicKey {
    pub pem: Vec<u8>,
}

impl Message for CaRsaPublicKey {
    const ID: u16 = id::TS_CA_RSA_PUBLIC_KEY;
    fn write(&self, w: &mut Writer) {
        w.u32(self.pem.len() as u32).bytes(&self.pem);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let n = r.u32()? as usize;
        Ok(Self { pem: r.bytes(n)?.to_vec() })
    }
}

/// TS_AC_AES_KEY_IV (72) : `+7 u32 n`, `+11 RSA(clé ‖ IV)[n]`.
pub struct AcAesKeyIv {
    pub data: Vec<u8>,
}

impl Message for AcAesKeyIv {
    const ID: u16 = id::TS_AC_AES_KEY_IV;
    fn write(&self, w: &mut Writer) {
        w.u32(self.data.len() as u32).bytes(&self.data);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let n = r.u32()? as usize;
        Ok(Self { data: r.bytes(n)?.to_vec() })
    }
}

/// TS_CA_ACCOUNT (10010), 149 octets : `account[61]`, `u32 pwd_size`, `pwd_aes[77]`.
pub struct CaAccount {
    pub account: String,
    pub password: Vec<u8>,
}

impl Message for CaAccount {
    const ID: u16 = id::TS_CA_ACCOUNT;
    fn write(&self, w: &mut Writer) {
        let mut pwd = self.password.clone();
        pwd.resize(77, 0);
        w.fixed_str(&self.account, 61).u32(self.password.len().min(77) as u32).bytes(&pwd);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let account = r.fixed_str(61)?;
        let size = (r.u32()? as usize).min(77);
        let raw = r.bytes(77)?;
        Ok(Self { account, password: raw[..size].to_vec() })
    }
}

/// TS_AC_RESULT (10000) : `u16 request`, `u16 result`, `i32 login_flag`.
pub struct AcResult {
    pub request: u16,
    pub result: u16,
    pub login_flag: i32,
}

impl Message for AcResult {
    const ID: u16 = id::TS_AC_RESULT;
    fn write(&self, w: &mut Writer) {
        w.u16(self.request).u16(self.result).i32(self.login_flag);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { request: r.u16()?, result: r.u16()?, login_flag: r.i32()? })
    }
}

pub struct CaServerList;

impl Message for CaServerList {
    const ID: u16 = id::TS_CA_SERVER_LIST;
    fn write(&self, _: &mut Writer) {}
    fn read(_: &mut Reader) -> Result<Self> {
        Ok(Self)
    }
}

/// SERVER_INFO, 302 octets.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerInfo {
    pub index: u16,
    pub name: String,
    pub is_adult: bool,
    pub screenshot_url: String,
    pub ip: String,
    pub port: i32,
    pub user_ratio: u16,
}

impl ServerInfo {
    fn write(&self, w: &mut Writer) {
        w.u16(self.index)
            .fixed_str(&self.name, 21)
            .bool(self.is_adult)
            .fixed_str(&self.screenshot_url, 256)
            .fixed_str(&self.ip, 16)
            .i32(self.port)
            .u16(self.user_ratio);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            index: r.u16()?,
            name: r.fixed_str(21)?,
            is_adult: r.bool()?,
            screenshot_url: r.fixed_str(256)?,
            ip: r.fixed_str(16)?,
            port: r.i32()?,
            user_ratio: r.u16()?,
        })
    }
}

/// TS_AC_SERVER_LIST (10022) : `u16 last_login_server_idx`, `u16 count`, `SERVER_INFO[count]`.
pub struct AcServerList {
    pub last_login_server_idx: u16,
    pub servers: Vec<ServerInfo>,
}

impl Message for AcServerList {
    const ID: u16 = id::TS_AC_SERVER_LIST;
    fn write(&self, w: &mut Writer) {
        w.u16(self.last_login_server_idx).u16(self.servers.len() as u16);
        for s in &self.servers {
            s.write(w);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let last = r.u16()?;
        let n = r.u16()?;
        let servers = (0..n).map(|_| ServerInfo::read(r)).collect::<Result<_>>()?;
        Ok(Self { last_login_server_idx: last, servers })
    }
}

pub struct CaSelectServer {
    pub server_idx: u16,
}

impl Message for CaSelectServer {
    const ID: u16 = id::TS_CA_SELECT_SERVER;
    fn write(&self, w: &mut Writer) {
        w.u16(self.server_idx);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { server_idx: r.u16()? })
    }
}

/// TS_AC_SELECT_SERVER (10024), 41 octets : `u16 result`, `i32 enc_size`, `enc[16]`
/// (AES de la clé one-time u64), `u64` (non lu par le client), `u32 pending_time`.
pub struct AcSelectServer {
    pub result: u16,
    pub encrypted_key: Vec<u8>,
    pub pending_time: u32,
}

impl Message for AcSelectServer {
    const ID: u16 = id::TS_AC_SELECT_SERVER;
    fn write(&self, w: &mut Writer) {
        let mut enc = self.encrypted_key.clone();
        enc.resize(16, 0);
        w.u16(self.result).i32(self.encrypted_key.len().min(16) as i32).bytes(&enc).u64(0).u32(self.pending_time);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let result = r.u16()?;
        let n = (r.i32()?.max(0) as usize).min(16);
        let enc = r.bytes(16)?[..n].to_vec();
        r.u64()?;
        Ok(Self { result, encrypted_key: enc, pending_time: r.u32()? })
    }
}

// ───────────────────────────── Lobby (game server) ─────────────────────────────

/// TS_CS_ACCOUNT_WITH_AUTH (2005), 76 octets : `account[61]`, `u64 one_time_key`.
pub struct CsAccountWithAuth {
    pub account: String,
    pub one_time_key: u64,
}

impl Message for CsAccountWithAuth {
    const ID: u16 = id::TS_CS_ACCOUNT_WITH_AUTH;
    fn write(&self, w: &mut Writer) {
        w.fixed_str(&self.account, 61).u64(self.one_time_key);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { account: r.fixed_str(61)?, one_time_key: r.u64()? })
    }
}

/// TS_SC_RESULT (0) : `u16 request`, `u16 result`, `i32 value`.
pub struct ScResult {
    pub request: u16,
    pub result: u16,
    pub value: i32,
}

impl Message for ScResult {
    const ID: u16 = id::TS_SC_RESULT;
    fn write(&self, w: &mut Writer) {
        w.u16(self.request).u16(self.result).i32(self.value);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { request: r.u16()?, result: r.u16()?, value: r.i32()? })
    }
}

pub struct CsCharacterList {
    pub account: String,
}

impl Message for CsCharacterList {
    const ID: u16 = id::TS_CS_CHARACTER_LIST;
    fn write(&self, w: &mut Writer) {
        w.fixed_str(&self.account, 61);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { account: r.fixed_str(61)? })
    }
}

pub const WEAR_SLOTS: usize = 28;

/// LOBBY_CHARACTER_INFO, 632 octets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LobbyCharacterInfo {
    pub sex: i32,
    pub race: i32,
    pub model_id: [i32; 5],
    pub hair_color_index: i32,
    pub hair_color_rgb: u32,
    pub hide_equip_flag: u32,
    pub texture_id: i32,
    pub wear_info: [i32; WEAR_SLOTS],
    pub level: i32,
    pub job: i32,
    pub job_level: i32,
    pub exp_percentage: i32,
    pub hp: i32,
    pub mp: i32,
    pub permission: i32,
    pub is_banned: bool,
    pub name: String,
    pub skin_color: u32,
    pub create_time: String,
    pub delete_time: String,
    pub wear_item_enhance_info: [i32; WEAR_SLOTS],
    pub wear_item_level_info: [i32; WEAR_SLOTS],
    pub wear_item_elemental_type: [u8; WEAR_SLOTS],
    pub wear_appearance_code: [i32; WEAR_SLOTS],
}

impl LobbyCharacterInfo {
    pub const SIZE: usize = 632;

    pub fn write(&self, w: &mut Writer) {
        w.i32(self.sex).i32(self.race);
        self.model_id.iter().for_each(|&v| {
            w.i32(v);
        });
        w.i32(self.hair_color_index).u32(self.hair_color_rgb).u32(self.hide_equip_flag).i32(self.texture_id);
        self.wear_info.iter().for_each(|&v| {
            w.i32(v);
        });
        w.i32(self.level)
            .i32(self.job)
            .i32(self.job_level)
            .i32(self.exp_percentage)
            .i32(self.hp)
            .i32(self.mp)
            .i32(self.permission)
            .bool(self.is_banned)
            .fixed_str(&self.name, 19)
            .u32(self.skin_color)
            .fixed_str(&self.create_time, 30)
            .fixed_str(&self.delete_time, 30);
        for v in self.wear_item_enhance_info.iter().chain(&self.wear_item_level_info) {
            w.i32(*v);
        }
        w.bytes(&self.wear_item_elemental_type);
        self.wear_appearance_code.iter().for_each(|&v| {
            w.i32(v);
        });
    }

    pub fn read(r: &mut Reader) -> Result<Self> {
        let mut c = Self { sex: r.i32()?, race: r.i32()?, ..Default::default() };
        for v in &mut c.model_id {
            *v = r.i32()?;
        }
        c.hair_color_index = r.i32()?;
        c.hair_color_rgb = r.u32()?;
        c.hide_equip_flag = r.u32()?;
        c.texture_id = r.i32()?;
        for v in &mut c.wear_info {
            *v = r.i32()?;
        }
        c.level = r.i32()?;
        c.job = r.i32()?;
        c.job_level = r.i32()?;
        c.exp_percentage = r.i32()?;
        c.hp = r.i32()?;
        c.mp = r.i32()?;
        c.permission = r.i32()?;
        c.is_banned = r.bool()?;
        c.name = r.fixed_str(19)?;
        c.skin_color = r.u32()?;
        c.create_time = r.fixed_str(30)?;
        c.delete_time = r.fixed_str(30)?;
        for v in &mut c.wear_item_enhance_info {
            *v = r.i32()?;
        }
        for v in &mut c.wear_item_level_info {
            *v = r.i32()?;
        }
        c.wear_item_elemental_type.copy_from_slice(r.bytes(WEAR_SLOTS)?);
        for v in &mut c.wear_appearance_code {
            *v = r.i32()?;
        }
        Ok(c)
    }
}

/// TS_SC_CHARACTER_LIST (2004) : `u32 time`, `u16 last_login_index`, `u16 count`, infos.
pub struct ScCharacterList {
    pub current_server_time: u32,
    pub last_login_index: u16,
    pub characters: Vec<LobbyCharacterInfo>,
}

impl Message for ScCharacterList {
    const ID: u16 = id::TS_SC_CHARACTER_LIST;
    fn write(&self, w: &mut Writer) {
        w.u32(self.current_server_time).u16(self.last_login_index).u16(self.characters.len() as u16);
        for c in &self.characters {
            c.write(w);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let time = r.u32()?;
        let last = r.u16()?;
        let n = r.u16()?;
        let characters = (0..n).map(|_| LobbyCharacterInfo::read(r)).collect::<Result<_>>()?;
        Ok(Self { current_server_time: time, last_login_index: last, characters })
    }
}

pub struct CsCreateCharacter {
    pub info: LobbyCharacterInfo,
}

impl Message for CsCreateCharacter {
    const ID: u16 = id::TS_CS_CREATE_CHARACTER;
    fn write(&self, w: &mut Writer) {
        self.info.write(w);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { info: LobbyCharacterInfo::read(r)? })
    }
}

pub struct CsDeleteCharacter {
    pub name: String,
    pub security_no: String,
}

impl Message for CsDeleteCharacter {
    const ID: u16 = id::TS_CS_DELETE_CHARACTER;
    fn write(&self, w: &mut Writer) {
        w.fixed_str(&self.name, 19).fixed_str(&self.security_no, 19);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { name: r.fixed_str(19)?, security_no: r.fixed_str(19).unwrap_or_default() })
    }
}

pub struct CsCheckCharacterName {
    pub name: String,
}

impl Message for CsCheckCharacterName {
    const ID: u16 = id::TS_CS_CHECK_CHARACTER_NAME;
    fn write(&self, w: &mut Writer) {
        w.fixed_str(&self.name, 19);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { name: r.fixed_str(19)? })
    }
}

/// TS_CS_LOGIN (62), 287 octets : `name[24]`, 256 octets réservés.
pub struct CsLogin {
    pub name: String,
}

impl Message for CsLogin {
    const ID: u16 = id::TS_CS_LOGIN;
    fn write(&self, w: &mut Writer) {
        w.fixed_str(&self.name, 24).zeros(256);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { name: r.fixed_str(24)? })
    }
}

// ───────────────────────────── Monde ─────────────────────────────

/// TS_SC_LOGIN_RESULT (64), 97 octets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScLoginResult {
    pub result: u16,
    pub handle: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: u8,
    pub face_direction: f32,
    pub region_size: i32,
    pub hp: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub max_mp: i32,
    pub havoc: i32,
    pub max_havoc: i32,
    pub sex: i32,
    pub race: i32,
    pub skin_color: u32,
    pub name: String,
    pub cell_size: i32,
    pub guild_id: i32,
}

impl Message for ScLoginResult {
    const ID: u16 = id::TS_SC_LOGIN_RESULT;
    fn write(&self, w: &mut Writer) {
        w.u16(self.result)
            .u32(self.handle)
            .f32(self.x)
            .f32(self.y)
            .f32(self.z)
            .u8(self.layer)
            .f32(self.face_direction)
            .i32(self.region_size)
            .i32(self.hp)
            .i32(self.mp)
            .i32(self.max_hp)
            .i32(self.max_mp)
            .i32(self.havoc)
            .i32(self.max_havoc)
            .i32(self.sex)
            .i32(self.race)
            .u32(self.skin_color)
            .fixed_str(&self.name, 19)
            .i32(self.cell_size)
            .i32(self.guild_id);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            result: r.u16()?,
            handle: r.u32()?,
            x: r.f32()?,
            y: r.f32()?,
            z: r.f32()?,
            layer: r.u8()?,
            face_direction: r.f32()?,
            region_size: r.i32()?,
            hp: r.i32()?,
            mp: r.i32()?,
            max_hp: r.i32()?,
            max_mp: r.i32()?,
            havoc: r.i32()?,
            max_havoc: r.i32()?,
            sex: r.i32()?,
            race: r.i32()?,
            skin_color: r.u32()?,
            name: r.fixed_str(19)?,
            cell_size: r.i32()?,
            guild_id: r.i32()?,
        })
    }
}

/// TS_TIMESYNC (2) : `u32 time` (centièmes de seconde), dans les deux sens.
pub struct TimeSync {
    pub time: u32,
}

impl Message for TimeSync {
    const ID: u16 = id::TS_TIMESYNC;
    fn write(&self, w: &mut Writer) {
        w.u32(self.time);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { time: r.u32()? })
    }
}

pub mod enter_type {
    pub const PLAYER: u8 = 0;
    pub const CREATURE: u8 = 1;
    pub const STATIC: u8 = 2;
}

pub mod obj_type {
    pub const PLAYER: u8 = 0;
    pub const NPC: u8 = 1;
    pub const ITEM: u8 = 2;
    pub const MONSTER: u8 = 3;
    pub const SUMMON: u8 = 4;
    pub const SKILL_PROP: u8 = 5;
    pub const FIELD_PROP: u8 = 6;
    pub const PET: u8 = 7;
}

/// Bloc « creature info » commun aux joueurs, PNJ et monstres (38 octets, +0x1a..+0x40).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreatureInfo {
    pub status: u32,
    pub face_direction: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub level: i32,
    pub race: u8,
    pub skin_color: u32,
    pub is_first_enter: bool,
    pub energy: i32,
}

impl CreatureInfo {
    fn write(&self, w: &mut Writer) {
        w.u32(self.status)
            .f32(self.face_direction)
            .i32(self.hp)
            .i32(self.max_hp)
            .i32(self.mp)
            .i32(self.max_mp)
            .i32(self.level)
            .u8(self.race)
            .u32(self.skin_color)
            .bool(self.is_first_enter)
            .i32(self.energy);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self {
            status: r.u32()?,
            face_direction: r.f32()?,
            hp: r.i32()?,
            max_hp: r.i32()?,
            mp: r.i32()?,
            max_mp: r.i32()?,
            level: r.i32()?,
            race: r.u8()?,
            skin_color: r.u32()?,
            is_first_enter: r.bool()?,
            energy: r.i32()?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerEnterInfo {
    pub creature: CreatureInfo,
    pub sex: u8,
    pub face_id: u32,
    pub face_texture_id: u32,
    pub hair_id: u32,
    pub hair_color_index: u32,
    pub hair_color_rgb: u32,
    pub hide_equip_flag: u32,
    pub name: String,
    pub job_id: u16,
    pub ride_handle: u32,
    pub guild_id: u32,
    pub title_id: i32,
}

/// Données spécifiques après le bloc commun de TS_SC_ENTER.
#[derive(Clone, Debug, PartialEq)]
pub enum EnterKind {
    /// Joueur : 122 octets au total.
    Player(PlayerEnterInfo),
    /// PNJ : 72 octets ; `npc_id` = id de NpcResource.
    Npc { creature: CreatureInfo, npc_id: u32 },
    /// Monstre : 73 octets ; `monster_id` = id de Monster.
    Monster { creature: CreatureInfo, is_tamed: bool, monster_id: u32 },
    /// Autre type : octets bruts après le bloc commun.
    Other { obj_type: u8, data: Vec<u8> },
}

/// TS_SC_ENTER (3) : `u8 type`, `u32 handle`, `x, y, z`, `u8 layer`, `u8 obj_type`, données.
#[derive(Clone, Debug, PartialEq)]
pub struct ScEnter {
    pub enter_type: u8,
    pub handle: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: u8,
    pub kind: EnterKind,
}

impl Message for ScEnter {
    const ID: u16 = id::TS_SC_ENTER;
    fn write(&self, w: &mut Writer) {
        let obj_type = match &self.kind {
            EnterKind::Player(_) => obj_type::PLAYER,
            EnterKind::Npc { .. } => obj_type::NPC,
            EnterKind::Monster { .. } => obj_type::MONSTER,
            EnterKind::Other { obj_type, .. } => *obj_type,
        };
        // L'octet `type` est imposé par le client : 0 joueur, 1 créature, 2 objet statique.
        let enter_type = match &self.kind {
            EnterKind::Player(_) => enter_type::PLAYER,
            EnterKind::Npc { .. } | EnterKind::Monster { .. } => enter_type::CREATURE,
            EnterKind::Other { .. } => self.enter_type,
        };
        w.u8(enter_type).u32(self.handle).f32(self.x).f32(self.y).f32(self.z).u8(self.layer).u8(obj_type);
        match &self.kind {
            EnterKind::Player(p) => {
                p.creature.write(w);
                w.u8(p.sex)
                    .u32(p.face_id)
                    .u32(p.face_texture_id)
                    .u32(p.hair_id)
                    .u32(p.hair_color_index)
                    .u32(p.hair_color_rgb)
                    .u32(p.hide_equip_flag)
                    .fixed_str(&p.name, 19)
                    .u16(p.job_id)
                    .u32(p.ride_handle)
                    .u32(p.guild_id)
                    .i32(p.title_id);
            }
            EnterKind::Npc { creature, npc_id } => {
                creature.write(w);
                w.bytes(&crate::entity_id::encode(*npc_id));
            }
            EnterKind::Monster { creature, is_tamed, monster_id } => {
                creature.write(w);
                w.bytes(&crate::entity_id::encode(*monster_id)).bool(*is_tamed);
            }
            EnterKind::Other { data, .. } => {
                w.bytes(data);
            }
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let enter_type = r.u8()?;
        let handle = r.u32()?;
        let (x, y, z) = (r.f32()?, r.f32()?, r.f32()?);
        let layer = r.u8()?;
        let ot = r.u8()?;
        let kind = match ot {
            obj_type::PLAYER => {
                let creature = CreatureInfo::read(r)?;
                EnterKind::Player(PlayerEnterInfo {
                    creature,
                    sex: r.u8()?,
                    face_id: r.u32()?,
                    face_texture_id: r.u32()?,
                    hair_id: r.u32()?,
                    hair_color_index: r.u32()?,
                    hair_color_rgb: r.u32()?,
                    hide_equip_flag: r.u32()?,
                    name: r.fixed_str(19)?,
                    job_id: r.u16()?,
                    ride_handle: r.u32()?,
                    guild_id: r.u32()?,
                    title_id: r.i32()?,
                })
            }
            obj_type::NPC => {
                let creature = CreatureInfo::read(r)?;
                let npc_id = crate::entity_id::decode(r.bytes(8)?.try_into().unwrap());
                EnterKind::Npc { creature, npc_id }
            }
            obj_type::MONSTER => {
                let creature = CreatureInfo::read(r)?;
                let monster_id = crate::entity_id::decode(r.bytes(8)?.try_into().unwrap());
                EnterKind::Monster { creature, is_tamed: r.bool()?, monster_id }
            }
            other => EnterKind::Other { obj_type: other, data: r.rest().to_vec() },
        };
        Ok(Self { enter_type, handle, x, y, z, layer, kind })
    }
}

pub struct ScLeave {
    pub handle: u32,
}

impl Message for ScLeave {
    const ID: u16 = id::TS_SC_LEAVE;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { handle: r.u32()? })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovePoint {
    pub x: f32,
    pub y: f32,
}

/// TS_CS_MOVE_REQUEST (63) : `handle`, `x`, `y`, `cur_time`, `u8 speed_sync`, `u16 count`, points.
#[derive(Clone, Debug, PartialEq)]
pub struct CsMoveRequest {
    pub handle: u32,
    pub x: f32,
    pub y: f32,
    pub cur_time: u32,
    pub speed_sync: bool,
    pub points: Vec<MovePoint>,
}

impl Message for CsMoveRequest {
    const ID: u16 = id::TS_CS_MOVE_REQUEST;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).f32(self.x).f32(self.y).u32(self.cur_time).bool(self.speed_sync).u16(self.points.len() as u16);
        for p in &self.points {
            w.f32(p.x).f32(p.y);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let (handle, x, y, cur_time, speed_sync) = (r.u32()?, r.f32()?, r.f32()?, r.u32()?, r.bool()?);
        let n = r.u16()?;
        let points = (0..n).map(|_| Ok(MovePoint { x: r.f32()?, y: r.f32()? })).collect::<Result<_>>()?;
        Ok(Self { handle, x, y, cur_time, speed_sync, points })
    }
}

/// TS_SC_MOVE (8) : `start_time`, `handle`, `u8 tlayer`, `u8 speed`, `u16 count`, points.
#[derive(Clone, Debug, PartialEq)]
pub struct ScMove {
    pub start_time: u32,
    pub handle: u32,
    pub tlayer: u8,
    pub speed: u8,
    pub points: Vec<MovePoint>,
}

impl Message for ScMove {
    const ID: u16 = id::TS_SC_MOVE;
    fn write(&self, w: &mut Writer) {
        w.u32(self.start_time).u32(self.handle).u8(self.tlayer).u8(self.speed).u16(self.points.len() as u16);
        for p in &self.points {
            w.f32(p.x).f32(p.y);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let (start_time, handle, tlayer, speed) = (r.u32()?, r.u32()?, r.u8()?, r.u8()?);
        let n = r.u16()?;
        let points = (0..n).map(|_| Ok(MovePoint { x: r.f32()?, y: r.f32()? })).collect::<Result<_>>()?;
        Ok(Self { start_time, handle, tlayer, speed, points })
    }
}

/// TS_SC_WARP (12) : téléportation du joueur.
pub struct ScWarp {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: u8,
}

impl Message for ScWarp {
    const ID: u16 = id::TS_SC_WARP;
    fn write(&self, w: &mut Writer) {
        w.f32(self.x).f32(self.y).f32(self.z).u8(self.layer);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { x: r.f32()?, y: r.f32()?, z: r.f32()?, layer: r.u8()? })
    }
}

pub mod chat_type {
    pub const NORMAL: u8 = 0;
    pub const YELL: u8 = 1;
    pub const ADV: u8 = 2;
    pub const WHISPER: u8 = 3;
    pub const GLOBAL: u8 = 4;
    pub const EMOTION: u8 = 5;
    pub const GM: u8 = 6;
    pub const GM_WHISPER: u8 = 7;
    pub const PARTY: u8 = 10;
    pub const GUILD: u8 = 11;
    pub const NOTICE: u8 = 20;
}

/// TS_CS_CHAT_REQUEST (20) : `target[21]`, `u8 request_id`, `u8 len`, `u8 type`, `msg[len]`.
pub struct CsChatRequest {
    pub target: String,
    pub request_id: u8,
    pub chat_type: u8,
    pub message: String,
}

impl Message for CsChatRequest {
    const ID: u16 = id::TS_CS_CHAT_REQUEST;
    fn write(&self, w: &mut Writer) {
        let mut msg = crate::packet::encode_str(&self.message);
        msg.truncate(170);
        w.fixed_str(&self.target, 21).u8(self.request_id).u8(msg.len() as u8).u8(self.chat_type).bytes(&msg);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let target = r.fixed_str(21)?;
        let request_id = r.u8()?;
        let len = r.u8()? as usize;
        let chat_type = r.u8()?;
        let len = len.min(r.remaining());
        Ok(Self { target, request_id, chat_type, message: crate::packet::decode_str(r.bytes(len)?) })
    }
}

/// TS_SC_CHAT (22) : `sender[21]`, `u16 len`, `u8 type`, `msg[len]`.
pub struct ScChat {
    pub sender: String,
    pub chat_type: u8,
    pub message: String,
}

impl Message for ScChat {
    const ID: u16 = id::TS_SC_CHAT;
    fn write(&self, w: &mut Writer) {
        let msg = crate::packet::encode_str(&self.message);
        w.fixed_str(&self.sender, 21).u16(msg.len() as u16).u8(self.chat_type).bytes(&msg);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let sender = r.fixed_str(21)?;
        let len = r.u16()? as usize;
        let chat_type = r.u8()?;
        let len = len.min(r.remaining());
        Ok(Self { sender, chat_type, message: crate::packet::decode_str(r.bytes(len)?) })
    }
}

/// TS_SC_CHAT_LOCAL (21) : `u32 handle`, `u8 len`, `u8 type`, `msg[len]`.
pub struct ScChatLocal {
    pub handle: u32,
    pub chat_type: u8,
    pub message: String,
}

impl Message for ScChatLocal {
    const ID: u16 = id::TS_SC_CHAT_LOCAL;
    fn write(&self, w: &mut Writer) {
        let mut msg = crate::packet::encode_str(&self.message);
        msg.truncate(255);
        w.u32(self.handle).u8(msg.len() as u8).u8(self.chat_type).bytes(&msg);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let handle = r.u32()?;
        let len = r.u8()? as usize;
        let chat_type = r.u8()?;
        let len = len.min(r.remaining());
        Ok(Self { handle, chat_type, message: crate::packet::decode_str(r.bytes(len)?) })
    }
}

/// TS_SC_PROPERTY (507) : `handle`, `u8 is_number`, `name[16]`, `i64 value`, chaîne sinon.
pub struct ScProperty {
    pub handle: u32,
    pub name: String,
    pub value: i64,
    pub text: Option<String>,
}

impl Message for ScProperty {
    const ID: u16 = id::TS_SC_PROPERTY;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).bool(self.text.is_none()).fixed_str(&self.name, 16).i64(self.value);
        if let Some(t) = &self.text {
            w.bytes(&crate::packet::encode_str(t)).u8(0);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let handle = r.u32()?;
        let is_number = r.bool()?;
        let name = r.fixed_str(16)?;
        let value = r.i64()?;
        let text = (!is_number).then(|| crate::packet::decode_str(r.rest()));
        Ok(Self { handle, name, value, text })
    }
}

/// TS_SC_STAT_INFO (1000), 110 octets : `handle`, 98 octets de statistiques, `u8 type`
/// (disposition : `docs/re/network_protocol.md` §8.3 ; offsets ci-dessous depuis +0x0b).
pub struct ScStatInfo {
    pub handle: u32,
    pub stats: [u8; 98],
    pub info_type: u8,
}

/// Statistiques de base d'un personnage, pour remplir TS_SC_STAT_INFO.
#[derive(Clone, Debug)]
pub struct BasicStats {
    /// str, vit, dex, agi, int, men, luk.
    pub attributes: [i16; 7],
    pub attack: i32,
    pub defence: i32,
    pub magic_point: i32,
    pub magic_defence: i32,
    pub move_speed: i16,
    pub attack_speed: i16,
    pub attack_range: i16,
}

impl ScStatInfo {
    pub fn from_stats(handle: u32, s: &BasicStats, info_type: u8) -> Self {
        let mut st = [0u8; 98];
        let mut put16 = |off: usize, v: i16| st[off - 0x0b..off - 0x0b + 2].copy_from_slice(&v.to_le_bytes());
        for (i, v) in s.attributes.iter().enumerate() {
            put16(0x0d + i * 2, *v);
        }
        put16(0x37, 100); // précision droite
        put16(0x39, 100); // précision gauche
        put16(0x3b, 100); // précision magique
        put16(0x43, s.move_speed);
        put16(0x45, s.attack_speed);
        put16(0x47, s.attack_range);
        put16(0x49, 1000); // poids maximal
        put16(0x4b, 100); // vitesse d'incantation
        put16(0x4d, 100); // vitesse de recharge
        let mut put32 = |off: usize, v: i32| st[off - 0x0b..off - 0x0b + 4].copy_from_slice(&v.to_le_bytes());
        put32(0x1f, s.attack);
        put32(0x23, s.attack);
        put32(0x27, s.defence);
        put32(0x2f, s.magic_point);
        put32(0x33, s.magic_defence);
        Self { handle, stats: st, info_type }
    }
}

impl Message for ScStatInfo {
    const ID: u16 = id::TS_SC_STAT_INFO;
    fn write(&self, w: &mut Writer) {
        w.u32(self.handle).bytes(&self.stats).u8(self.info_type);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let handle = r.u32()?;
        let stats = r.bytes(98)?.try_into().unwrap();
        Ok(Self { handle, stats, info_type: r.u8()? })
    }
}

pub struct ScDisconnectDesc {
    pub code: u8,
}

impl Message for ScDisconnectDesc {
    const ID: u16 = id::TS_SC_DISCONNECT_DESC;
    fn write(&self, w: &mut Writer) {
        w.u8(self.code);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        Ok(Self { code: r.u8()? })
    }
}

/// Paquets sans corps (retour au lobby, déconnexion, liste des serveurs).
pub struct Empty<const ID: u16>;

impl<const I: u16> Message for Empty<I> {
    const ID: u16 = I;
    fn write(&self, _: &mut Writer) {}
    fn read(_: &mut Reader) -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size_of<M: Message>(m: &M) -> usize {
        m.to_packet().to_bytes().len()
    }

    #[test]
    fn observed_sizes() {
        assert_eq!(size_of(&CaVersion(Version { version: CLIENT_VERSION.into() })), 27);
        assert_eq!(size_of(&CaAccount { account: "a".into(), password: vec![0; 16] }), 149);
        assert_eq!(size_of(&AcResult { request: 0, result: 0, login_flag: 0 }), 15);
        assert_eq!(size_of(&CaSelectServer { server_idx: 1 }), 9);
        assert_eq!(size_of(&AcSelectServer { result: 0, encrypted_key: vec![0; 16], pending_time: 0 }), 41);
        assert_eq!(size_of(&CsAccountWithAuth { account: "a".into(), one_time_key: 1 }), 76);
        assert_eq!(size_of(&CsCharacterList { account: "a".into() }), 68);
        assert_eq!(size_of(&CsCreateCharacter { info: LobbyCharacterInfo::default() }), 639);
        assert_eq!(size_of(&CsDeleteCharacter { name: "a".into(), security_no: String::new() }), 45);
        assert_eq!(size_of(&CsCheckCharacterName { name: "a".into() }), 26);
        assert_eq!(size_of(&CsLogin { name: "a".into() }), 287);
        assert_eq!(size_of(&ScLoginResult::default()), 97);
        assert_eq!(size_of(&ScStatInfo { handle: 0, stats: [0; 98], info_type: 0 }), 110);
        assert_eq!(size_of(&ScLeave { handle: 0 }), 11);
        assert_eq!(size_of(&ScProperty { handle: 0, name: "lv".into(), value: 1, text: None }), 36);
        let list = AcServerList { last_login_server_idx: 0, servers: vec![ServerInfo { index: 1, name: "x".into(), is_adult: false, screenshot_url: String::new(), ip: "127.0.0.1".into(), port: 4514, user_ratio: 0 }] };
        assert_eq!(size_of(&list), 11 + 302);
        let chars = ScCharacterList { current_server_time: 0, last_login_index: 0, characters: vec![LobbyCharacterInfo::default()] };
        assert_eq!(size_of(&chars), 15 + 632);
        let base = |kind| ScEnter { enter_type: 0, handle: 1, x: 0.0, y: 0.0, z: 0.0, layer: 0, kind };
        assert_eq!(size_of(&base(EnterKind::Player(PlayerEnterInfo::default()))), 122);
        assert_eq!(size_of(&base(EnterKind::Npc { creature: CreatureInfo::default(), npc_id: 1 })), 72);
        assert_eq!(size_of(&base(EnterKind::Monster { creature: CreatureInfo::default(), is_tamed: false, monster_id: 1 })), 73);
        let mv = CsMoveRequest { handle: 1, x: 0.0, y: 0.0, cur_time: 0, speed_sync: false, points: vec![MovePoint { x: 1.0, y: 2.0 }] };
        assert_eq!(size_of(&mv), 26 + 8);
    }

    #[test]
    fn roundtrips() {
        let info = LobbyCharacterInfo { sex: 2, race: 4, level: 7, job: 100, name: "Élise".into(), ..Default::default() };
        let back = CsCreateCharacter::from_packet(&CsCreateCharacter { info: info.clone() }.to_packet()).unwrap();
        assert_eq!(back.info, info);
        let chat = CsChatRequest { target: String::new(), request_id: 0, chat_type: chat_type::NORMAL, message: "Salut à tous".into() };
        let back = CsChatRequest::from_packet(&chat.to_packet()).unwrap();
        assert_eq!(back.message, "Salut à tous");
        let enter = ScEnter { enter_type: enter_type::CREATURE, handle: 9, x: 1.0, y: 2.0, z: 3.0, layer: 0, kind: EnterKind::Npc { creature: CreatureInfo::default(), npc_id: 1001 } };
        assert_eq!(ScEnter::from_packet(&enter.to_packet()).unwrap(), enter);
    }
}
