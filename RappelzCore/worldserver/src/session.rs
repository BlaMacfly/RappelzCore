//! Session joueur sur le worldserver (équivalent de `WorldSession` d'AzerothCore) :
//! authentification par clé one-time, lobby (liste, création, suppression de personnages),
//! entrée dans le monde, déplacements, chat.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use rz_database::characters::{self, Character, MAX_NAME_LEN};
use rz_database::{auth, sqlx};
use rz_proto::conn::{PacketReader, PacketWriter};
use rz_proto::msg::{self, chat_type, id, result, CreatureInfo, LobbyCharacterInfo, Message, PlayerEnterInfo};
use rz_proto::RawPacket;
use tokio::sync::mpsc;

use crate::world::{Motion, PlayerEntity, WorldEvent};
use crate::{commands, Ctx};

/// Personnage en jeu porté par la session.
pub struct InGame {
    pub handle: u32,
    pub character: Character,
    pub items: Vec<crate::gameplay::InvItem>,
}

pub struct Session {
    pub ctx: Arc<Ctx>,
    pub addr: SocketAddr,
    pub tx: mpsc::UnboundedSender<RawPacket>,
    pub account: Option<auth::Account>,
    pub gm_level: u8,
    pub in_game: Option<InGame>,
    /// Événements du monde (récompenses, mort) pour le personnage en jeu.
    pub events_rx: Option<mpsc::UnboundedReceiver<WorldEvent>>,
    /// PNJ avec lequel un dialogue est ouvert.
    pub dialog_npc: Option<u32>,
    /// None until the native client opts into the learning-catalog extension.
    pub skill_catalog:Option<rz_proto::skills::ScSkillCatalog>,
}

pub async fn run(ctx: Arc<Ctx>, addr: SocketAddr, mut reader: PacketReader, mut writer: PacketWriter) {
    let (tx, mut rx) = mpsc::unbounded_channel::<RawPacket>();
    let writer_task = tokio::spawn(async move {
        while let Some(p) = rx.recv().await {
            if writer.send_raw(&p).await.is_err() {
                break;
            }
        }
        let _ = writer.shutdown().await;
    });

    let mut session = Session { ctx: ctx.clone(), addr, tx, account: None, gm_level: 0, in_game: None, events_rx: None, dialog_npc: None,skill_catalog:None };
    let mut save_timer = tokio::time::interval(ctx.settings.save_interval);
    save_timer.tick().await;
    let mut save_all = ctx.save_all.subscribe();
    let mut shutdown = ctx.shutdown.subscribe();

    let outcome: Result<()> = async {
        loop {
            tokio::select! {
                packet = reader.recv() => {
                    let Some(packet) = packet? else { break };
                    session.handle(packet).await?;
                }
                ev = async {
                    match session.events_rx.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if let Some(ev) = ev {
                        session.handle_world_event(ev).await?;
                    }
                }
                _ = save_timer.tick() => session.save().await?,
                _ = save_all.recv() => session.save().await?,
                _ = shutdown.changed() => {
                    let _ = session.tx.send(msg::ScDisconnectDesc { code: 0 }.to_packet());
                    break;
                }
            }
        }
        Ok(())
    }
    .await;
    if let Err(e) = outcome {
        tracing::warn!("{addr} : {e:#}");
    }
    if let Err(e) = session.leave_world().await {
        tracing::error!("{addr} : sauvegarde à la déconnexion : {e:#}");
        session.detach_world_without_save();
    }
    let account_id=session.account.as_ref().map(|a|a.id);
    // Remove all world/socket senders before a possibly unavailable auth database.
    // Otherwise SQL recovery delays EOF and can leave a ghost player indefinitely.
    drop(session);
    let _ = writer_task.await;
    if let Some(account_id)=account_id {
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2),auth::set_online(&ctx.login,account_id,0)).await;
    }
}

impl Session {
    pub fn send<M: Message>(&self, m: &M) {
        let _ = self.tx.send(m.to_packet());
    }

    fn sc_result(&self, request: u16, code: u16, value: i32) {
        self.send(&msg::ScResult { request, result: code, value });
    }

    async fn handle(&mut self, p: RawPacket) -> Result<()> {
        match p.id {
            id::TS_CS_VERSION | id::TS_CS_REPORT | id::TS_CS_CHECK_ILLEGAL_USER | id::TS_CS_GAME_GUARD_AUTH_ANSWER => {}
            id::TS_TIMESYNC | id::TS_CS_GAME_TIME | id::TS_CS_REGION_UPDATE | id::TS_CS_UPDATE | id::TS_CS_GET_REGION_INFO => {}
            id::TS_CS_ACCOUNT_WITH_AUTH => self.handle_auth(msg::CsAccountWithAuth::from_packet(&p)?).await?,
            _ if self.account.is_none() => {
                tracing::debug!("{} : paquet {} avant authentification", self.addr, p.id);
            }
            id::TS_CS_CHARACTER_LIST => self.send_character_list().await?,
            id::TS_CS_CHECK_CHARACTER_NAME => {
                let req = msg::CsCheckCharacterName::from_packet(&p)?;
                let code = self.check_name(&req.name).await?;
                self.sc_result(id::TS_CS_CHECK_CHARACTER_NAME, code, 0);
            }
            id::TS_CS_CREATE_CHARACTER => self.create_character(msg::CsCreateCharacter::from_packet(&p)?.info).await?,
            id::TS_CS_DELETE_CHARACTER => self.delete_character(msg::CsDeleteCharacter::from_packet(&p)?).await?,
            id::TS_CS_LOGIN => self.login(msg::CsLogin::from_packet(&p)?.name).await?,
            id::TS_CS_MOVE_REQUEST => self.handle_move(msg::CsMoveRequest::from_packet(&p)?),
            id::TS_CS_CHAT_REQUEST => self.handle_chat(msg::CsChatRequest::from_packet(&p)?).await?,
            id::TS_CS_RETURN_LOBBY | id::TS_CS_REQUEST_RETURN_LOBBY => {
                self.leave_world().await?;
                self.sc_result(p.id, result::SUCCESS, 0);
            }
            id::TS_CS_LOGOUT => {
                self.leave_world().await?;
                self.sc_result(p.id, result::SUCCESS, 0);
            }
            rz_proto::party::CS_PARTY_HELLO if self.in_game.is_some() => {
                let version=rz_proto::party::CsPartyHello::from_packet(&p)?.0;
                let handle=self.in_game.as_ref().unwrap().handle;
                self.ctx.world.lock().unwrap().enable_party(handle,version);
            }
            rz_proto::skills::CS_SKILLS_HELLO if self.in_game.is_some()=>{
                let version=rz_proto::skills::CsSkillsHello::from_packet(&p)?.0;
                self.enable_skills(version);
            }
            id if crate::gameplay::handles(id) => self.handle_gameplay(p).await?,
            other => tracing::trace!("{} : paquet {other} non géré", self.addr),
        }
        Ok(())
    }

    async fn handle_auth(&mut self, req: msg::CsAccountWithAuth) -> Result<()> {
        let ctx = self.ctx.clone();
        let account = auth::find_account(&ctx.login, &req.account).await?;
        let ok = match &account {
            Some(a) => {
                let key = auth::session_key(&ctx.login, a.id).await?.unwrap_or_default();
                key.len() >= 8 && u64::from_le_bytes(key[..8].try_into().unwrap()) == req.one_time_key
            }
            None => false,
        };
        let Some(account) = account.filter(|_| ok) else {
            tracing::info!("{} : clé de session refusée pour {}", self.addr, req.account);
            self.sc_result(id::TS_CS_ACCOUNT_WITH_AUTH, result::ACCESS_DENIED, 0);
            return Ok(());
        };
        if auth::is_banned(&ctx.login, account.id, &self.addr.ip().to_string()).await? {
            self.sc_result(id::TS_CS_ACCOUNT_WITH_AUTH, result::BLOCKED, 0);
            return Ok(());
        }
        // La clé one-time ne sert qu'une fois.
        sqlx::query("UPDATE account SET session_key = NULL WHERE id = ?").bind(account.id).execute(&ctx.login).await?;
        auth::set_online(&ctx.login, account.id, ctx.settings.realm_id).await?;
        self.gm_level = auth::gm_level(&ctx.login, account.id, ctx.settings.realm_id as i32).await?;
        tracing::info!("{} : compte {} (GM {}) sur le royaume", self.addr, account.username, self.gm_level);
        self.account = Some(account);
        self.sc_result(id::TS_CS_ACCOUNT_WITH_AUTH, result::SUCCESS, 0);
        Ok(())
    }

    fn account_id(&self) -> u32 {
        self.account.as_ref().map(|a| a.id).unwrap_or(0)
    }

    fn lobby_info(c: &Character) -> LobbyCharacterInfo {
        let mut info = LobbyCharacterInfo {
            sex: c.gender as i32 + 1,
            race: c.race as i32,
            hair_color_index: c.extra_i64("hair_color_index", 0) as i32,
            hair_color_rgb: c.extra_i64("hair_color_rgb", 0) as u32,
            texture_id: c.extra_i64("texture_id", 0) as i32,
            level: c.level as i32,
            job: c.extra_i64("job", c.class as i64) as i32,
            job_level: c.extra_i64("jlv", 1) as i32,
            hp: c.health as i32,
            mp: c.power1 as i32,
            name: c.name.clone(),
            skin_color: c.extra_i64("skin_color", c.skin as i64) as u32,
            ..Default::default()
        };
        for (i, m) in info.model_id.iter_mut().enumerate() {
            *m = c.extra_i64(&format!("model{i}"), 0) as i32;
        }
        info
    }

    async fn send_character_list(&mut self) -> Result<()> {
        let list = characters::list_for_account(&self.ctx.chars, self.account_id()).await?;
        let infos = list.iter().map(Self::lobby_info).collect();
        let now = chrono::Utc::now().timestamp() as u32;
        self.send(&msg::ScCharacterList { current_server_time: now, last_login_index: 0, characters: infos });
        Ok(())
    }

    /// Règles de nom d'AzerothCore (`MinPlayerName`, longueur de `characters.name`).
    fn valid_name(name: &str) -> bool {
        let n = name.chars().count();
        (2..=MAX_NAME_LEN).contains(&n) && name.chars().all(|c| c.is_alphanumeric())
    }

    async fn check_name(&self, name: &str) -> Result<u16> {
        if !Self::valid_name(name) {
            return Ok(result::INVALID_TEXT);
        }
        if characters::name_exists(&self.ctx.chars, name).await? {
            return Ok(result::ALREADY_EXIST);
        }
        Ok(result::SUCCESS)
    }

    async fn create_character(&mut self, info: LobbyCharacterInfo) -> Result<()> {
        const REQ: u16 = id::TS_CS_CREATE_CHARACTER;
        let ctx = self.ctx.clone();
        let code = self.check_name(&info.name).await?;
        if code != result::SUCCESS {
            self.sc_result(REQ, code, 0);
            return Ok(());
        }
        if !(3..=5).contains(&info.race) || !(1..=2).contains(&info.sex) {
            self.sc_result(REQ, result::ACCESS_DENIED, 0);
            return Ok(());
        }
        let existing = characters::list_for_account(&ctx.chars, self.account_id()).await?;
        if existing.len() >= ctx.settings.characters_per_realm {
            self.sc_result(REQ, result::TOO_MUCH, 0);
            return Ok(());
        }
        let job = ctx.data.jobs.get(&info.job);
        let (sx, sy) = ctx.settings.start_position(info.race);
        let mut c = Character {
            guid: ctx.next_character_guid(),
            account: self.account_id(),
            name: info.name.clone(),
            race: info.race as u8,
            class: job.map(|j| j.job_class.clamp(0, 255) as u8).unwrap_or(0),
            gender: (info.sex - 1) as u8,
            level: ctx.settings.start_level,
            money: ctx.settings.start_money,
            skin: (info.skin_color & 0xFF) as u8,
            hair_style: info.model_id[0].clamp(0, 255) as u8,
            face: info.model_id[1].clamp(0, 255) as u8,
            hair_color: info.hair_color_index.clamp(0, 255) as u8,
            position_x: sx,
            position_y: sy,
            health: 100,
            power1: 100,
            ..Default::default()
        };
        c.set_extra("job", info.job);
        c.set_extra("jlv", 1);
        c.set_extra("jp", 0);
        c.set_extra("layer", 0);
        c.set_extra("hair_color_index", info.hair_color_index);
        c.set_extra("hair_color_rgb", info.hair_color_rgb);
        c.set_extra("texture_id", info.texture_id);
        c.set_extra("skin_color", info.skin_color);
        for (i, m) in info.model_id.iter().enumerate() {
            c.set_extra(&format!("model{i}"), m);
        }
        characters::insert(&ctx.chars, &c).await?;
        auth::update_realm_character_count(&ctx.login, ctx.settings.realm_id, self.account_id(), existing.len() as u8 + 1)
            .await?;
        tracing::info!("{} : personnage {} créé (guid {}, race {}, métier {})", self.addr, c.name, c.guid, c.race, info.job);
        self.sc_result(REQ, result::SUCCESS, 0);
        Ok(())
    }

    async fn delete_character(&mut self, req: msg::CsDeleteCharacter) -> Result<()> {
        const REQ: u16 = id::TS_CS_DELETE_CHARACTER;
        let ctx = self.ctx.clone();
        let Some(c) = characters::by_name(&ctx.chars, &req.name).await?.filter(|c| c.account == self.account_id()) else {
            self.sc_result(REQ, result::NOT_EXIST, 0);
            return Ok(());
        };
        characters::soft_delete(&ctx.chars, c.guid, self.account_id()).await?;
        let left = characters::list_for_account(&ctx.chars, self.account_id()).await?.len();
        auth::update_realm_character_count(&ctx.login, ctx.settings.realm_id, self.account_id(), left as u8).await?;
        tracing::info!("{} : personnage {} supprimé", self.addr, c.name);
        self.sc_result(REQ, result::SUCCESS, 0);
        Ok(())
    }

    pub fn enter_info(c: &Character, gm: u8) -> PlayerEnterInfo {
        PlayerEnterInfo {
            creature: CreatureInfo {
                face_direction: c.orientation,
                hp: c.health as i32,
                max_hp: crate::gameplay::max_hp_for(c.level),
                mp: c.power1 as i32,
                max_mp: crate::gameplay::max_mp_for(c.level),
                level: c.level as i32,
                race: c.race,
                skin_color: c.extra_i64("skin_color", c.skin as i64) as u32,
                ..Default::default()
            },
            sex: c.gender + 1,
            face_id: c.extra_i64("model1", 0) as u32,
            face_texture_id: c.extra_i64("texture_id", 0) as u32,
            hair_id: c.extra_i64("model0", 0) as u32,
            hair_color_index: c.extra_i64("hair_color_index", 0) as u32,
            hair_color_rgb: c.extra_i64("hair_color_rgb", 0) as u32,
            name: if gm > 0 { format!("<GM>{}", c.name) } else { c.name.clone() },
            job_id: c.extra_i64("job", c.class as i64) as u16,
            ..Default::default()
        }
    }

    async fn login(&mut self, name: String) -> Result<()> {
        if self.in_game.is_some() {
            return Ok(());
        }
        let ctx = self.ctx.clone();
        let Some(mut c) = characters::by_name(&ctx.chars, &name).await?.filter(|c| c.account == self.account_id()) else {
            self.send(&msg::ScLoginResult { result: result::NOT_EXIST, ..Default::default() });
            return Ok(());
        };
        c.online = true;
        characters::set_online(&ctx.chars, c.guid, true).await?;
        let s = &ctx.settings;
        let layer = c.extra_i64("layer", 0) as u8;
        let items = self.load_items(c.guid).await?;
        let wear = Self::wear_array(&items);
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let (max_hp, max_mp) = (crate::gameplay::max_hp_for(c.level), crate::gameplay::max_mp_for(c.level));
        if c.health == 0 || c.health as i32 > max_hp {
            c.health = max_hp as u32;
        }
        if c.power1 as i32 > max_mp {
            c.power1 = max_mp as u32;
        }

        let handle = {
            let mut world = ctx.world.lock().unwrap();
            if let Some(dup) = world.players.values().find(|p| p.guid == c.guid).map(|p| p.handle) {
                world.remove_player(dup);
            }
            let handle = world.allocate_handle();
            self.send(&msg::ScLoginResult {
                result: result::SUCCESS,
                handle,
                x: c.position_x,
                y: c.position_y,
                z: c.position_z,
                layer,
                face_direction: c.orientation,
                region_size: s.region_size,
                hp: c.health as i32,
                mp: c.power1 as i32,
                max_hp,
                max_mp,
                havoc: 0,
                max_havoc: 0,
                sex: c.gender as i32 + 1,
                race: c.race as i32,
                skin_color: c.extra_i64("skin_color", c.skin as i64) as u32,
                name: c.name.clone(),
                cell_size: s.cell_size,
                guild_id: 0,
            });
            let stats = msg::BasicStats {
                attributes: [10; 7],
                attack: 10 + c.level as i32 * 2,
                defence: 5 + c.level as i32,
                magic_point: 10 + c.level as i32 * 2,
                magic_defence: 5 + c.level as i32,
                move_speed: s.move_speed as i16,
                attack_speed: 100,
                attack_range: 50,
            };
            self.send(&msg::ScStatInfo::from_stats(handle, &stats, 0));
            for (prop, value) in [
                ("lv", c.level as i64),
                ("job", c.extra_i64("job", 0)),
                ("jlv", c.extra_i64("jlv", 1)),
                ("jp", c.extra_i64("jp", 0)),
                ("exp", c.xp as i64),
                ("gold", c.money as i64),
                ("permission", self.gm_level as i64),
            ] {
                self.send(&msg::ScProperty { handle, name: prop.into(), value, text: None });
            }
            self.send(&msg::TimeSync { time: world.now() });
            world.add_player(PlayerEntity {
                handle,
                guid: c.guid,
                account_id: c.account,
                name: c.name.clone(),
                gm_level: self.gm_level,
                z: c.position_z,
                layer,
                motion: Motion::at(c.position_x, c.position_y),
                enter: Self::enter_info(&c, self.gm_level),
                wear,
                tx: self.tx.clone(),
                events: events_tx,
                visible: Default::default(),
                hp: c.health as i32,
                max_hp,
                mp: c.power1 as i32,
                max_mp,
                attack: crate::gameplay::attack_for(c.level),
                defence: crate::gameplay::defence_for(c.level),
                attack_target: None,
                next_attack: 0,
                last_combat: 0,
            });
            world.update_visibility();
            handle
        };
        tracing::info!("{} : {} entre dans le monde (handle {handle})", self.addr, c.name);
        if !s.motd.is_empty() {
            ctx.world.lock().unwrap().notify(handle, &s.motd);
        }
        self.in_game = Some(InGame { handle, character: c, items });
        self.events_rx = Some(events_rx);
        self.send_initial_state();
        Ok(())
    }

    /// Met à jour le personnage depuis l'état du monde et le sauvegarde (Player::SaveToDB).
    pub async fn save(&mut self) -> Result<()> {
        let Some(character)=self.character_snapshot() else {return Ok(())};
        if let Err(error)=characters::save(&self.ctx.chars,&character).await {
            // No retry with an older snapshot after an uncertain SQL outcome.
            self.detach_world_without_save();
            return Err(error);
        }
        if let Some(g)=&mut self.in_game {g.character=character;}
        Ok(())
    }

    /// Capture a candidate without changing session state before a durable write succeeds.
    pub fn character_snapshot(&self)->Option<Character> {
        let g=self.in_game.as_ref()?;
        let mut character=g.character.clone();
        {
            let w = self.ctx.world.lock().unwrap();
            if let Some(p) = w.players.get(&g.handle).filter(|p|p.guid==g.character.guid && p.account_id==g.character.account) {
                let (x, y) = p.motion.position(w.now());
                character.position_x = x;
                character.position_y = y;
                character.position_z = p.z;
                character.set_extra("layer",p.layer);
                character.health = p.hp.max(1) as u32;
                character.power1 = p.mp.max(0) as u32;
            }
        }
        Some(character)
    }

    /// Detach after an uncertain durable write without ever saving stale progress.
    /// The caller ends the connection; startup recovery handles stale online flags.
    pub(crate) fn detach_world_without_save(&mut self) {
        if let Some(g) = self.in_game.take() {
            let mut world = self.ctx.world.lock().unwrap();
            if world.players.get(&g.handle).is_some_and(|p| p.guid==g.character.guid && p.account_id==g.character.account) {
                world.remove_player(g.handle);
            }
        }
        self.events_rx=None;
        self.dialog_npc=None;
        self.skill_catalog=None;
    }

    pub async fn leave_world(&mut self) -> Result<()> {
        if self.in_game.is_none() {
            return Ok(());
        }
        self.save().await?;
        let g = self.in_game.take().unwrap();
        self.events_rx = None;
        self.dialog_npc = None;
        self.skill_catalog=None;
        let mut c = g.character;
        c.online = false;
        self.ctx.world.lock().unwrap().remove_player(g.handle);
        characters::save(&self.ctx.chars, &c).await?;
        tracing::info!("{} : {} quitte le monde", self.addr, c.name);
        Ok(())
    }

    fn handle_move(&mut self, req: msg::CsMoveRequest) {
        let Some(g) = &self.in_game else { return };
        if req.handle != g.handle || req.points.len() > 64 {
            return;
        }
        let s = &self.ctx.settings;
        let mut world = self.ctx.world.lock().unwrap();
        let now = world.now();
        world.cancel_restoration(g.handle);
        let Some(p) = world.players.get_mut(&g.handle) else { return };
        // Anti-téléportation : le départ annoncé doit être proche de la position serveur.
        let (sx, sy) = p.motion.position(now);
        let (x, y) = if (sx - req.x).powi(2) + (sy - req.y).powi(2) <= s.move_tolerance.powi(2) {
            (req.x, req.y)
        } else {
            (sx, sy)
        };
        p.motion = Motion { x, y, start_time: now, points: req.points.clone(), speed: s.move_speed as f32 * s.speed_scale };
        let layer = p.layer;
        let packet = msg::ScMove { start_time: now, handle: g.handle, tlayer: layer, speed: s.move_speed, points: req.points }.to_packet();
        world.broadcast_around(g.handle, &packet, true);
    }

    async fn handle_chat(&mut self, req: msg::CsChatRequest) -> Result<()> {
        let Some(g) = &self.in_game else { return Ok(()) };
        let handle = g.handle;
        let name = g.character.name.clone();
        let text = req.message.trim().to_string();
        if text.starts_with('.') || (text.starts_with('/') && self.gm_level > 0) {
            commands::handle(self, &text[1..]).await?;
            self.send_skill_catalog();
            return Ok(());
        }
        let world = self.ctx.world.lock().unwrap();
        match req.chat_type {
            chat_type::PARTY => world.party_chat(handle,&text),
            chat_type::WHISPER => match world.find_by_name(&req.target) {
                Some(target) => {
                    let packet = msg::ScChat { sender: name, chat_type: chat_type::WHISPER, message: text }.to_packet();
                    let _ = target.tx.send(packet);
                }
                None => world.notify(handle, &format!("{} n'est pas connecté.", req.target)),
            },
            chat_type::GLOBAL | chat_type::YELL | chat_type::ADV => {
                world.broadcast_all(&msg::ScChat { sender: name, chat_type: req.chat_type, message: text }.to_packet());
            }
            _ => {
                let packet = msg::ScChatLocal { handle, chat_type: chat_type::NORMAL, message: text }.to_packet();
                world.broadcast_around(handle, &packet, true);
            }
        }
        Ok(())
    }
}
