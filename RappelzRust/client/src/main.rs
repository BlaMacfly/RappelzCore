//! RappelzRust — client de jeu Rappelz en Rust.
//!
//! Installation façon World of Warcraft (produite par `rz-extractor client`) :
//! `RappelzRust.exe`, `Data/*.MPQ`, `Data/<locale>/locale-<locale>.MPQ`,
//! `Data/<locale>/realmlist.wtf`, `WTF/Config.wtf`.
//! Options : `-data <dossier>`, `-demo [-pos x,y]` (monde hors ligne), `-screenshot <png>`.

mod assets;
mod addons;
mod audio;
mod config;
mod gui;
mod models;
mod net;
mod online_validation;
mod rzui;
mod terrain;
mod world;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use macroquad::prelude::*;
use rz_proto::msg::{self, LobbyCharacterInfo, ServerInfo};
use tokio::sync::mpsc::UnboundedSender;

use assets::Assets;
use config::ClientConfig;
use gui::{Gui, GOLD};
use net::{NetCmd, NetEvent};

#[derive(PartialEq)]
enum Screen {
    Login,
    Realms,
    Characters,
    Create,
    World,
}

struct CreateForm {
    name: String,
    race: usize,
    sex: usize,
    job: usize,
}

const RACES: [(i32, &str); 3] = [(3, "Gaia"), (4, "Deva"), (5, "Asura")];
const FIELD_ACCOUNT: u32 = 1;
const FIELD_PASSWORD: u32 = 2;
const FIELD_NAME: u32 = 3;

pub struct App {
    combat_trace: Option<online_validation::PacketTrace>,
    addons: addons::Addons,
    addon_realm: String,
    /// Interface d'origine (fenêtres .nui).
    rzui: Option<rzui::RzUi>,
    /// Écran de chargement d'origine (`loading_imageNN.jpg`) affiché à l'entrée en jeu.
    loading: Option<(f64, String)>,
    audio: audio::Audio,
    field_music: &'static str,
    /// Personnage à faire entrer automatiquement (`-enter`, tests).
    auto_enter: Option<String>,
    /// Nombre d'images affichées depuis l'entrée dans le monde (capture de test).
    world_frame: u32,
    root: PathBuf,
    assets: Option<Assets>,
    config: ClientConfig,
    net: UnboundedSender<NetCmd>,
    events: Receiver<NetEvent>,
    gui: Gui,
    screen: Screen,
    status: String,
    error: Option<String>,
    account: String,
    password: String,
    servers: Vec<ServerInfo>,
    characters: Vec<LobbyCharacterInfo>,
    selected: usize,
    create: CreateForm,
    world: Option<world::WorldState>,
    background: Option<Texture2D>,
}

/// Icône de la fenêtre (barre de titre, barre des tâches), générée par `build.rs` depuis le logo.
fn window_icon() -> macroquad::miniquad::conf::Icon {
    macroquad::miniquad::conf::Icon {
        small: *include_bytes!(concat!(env!("OUT_DIR"), "/icon_16.rgba")),
        medium: *include_bytes!(concat!(env!("OUT_DIR"), "/icon_32.rgba")),
        big: *include_bytes!(concat!(env!("OUT_DIR"), "/icon_64.rgba")),
    }
}

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title: "RappelzRust".to_owned(),
            window_width: 1280,
            window_height: 720,
            high_dpi: true,
            icon: Some(window_icon()),
            ..Default::default()
        },
        // Les modèles Rappelz dépassent les tampons par défaut (10 000 sommets / 5 000 index).
        draw_call_vertex_capacity: 65_536,
        draw_call_index_capacity: 196_608,
        ..Default::default()
    }
}

fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned())
}

fn rz_addon_account(account: &str) -> String {
    // Server account identifiers are case-insensitive; use one settings scope per account.
    account.to_uppercase()
}

fn data_root() -> PathBuf {
    if let Some(p) = arg_value("-data") {
        return PathBuf::from(p);
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    match exe_dir {
        Some(d) if d.join("Data").is_dir() => d,
        _ => std::env::current_dir().unwrap_or_default(),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rz_common::init_logging("info");
    prevent_quit(); // Allow PLAYER_LOGOUT and atomic settings saves before the window closes.
    let root = data_root();
    let mut config = ClientConfig::load(&root);
    let locale = config.get("locale", "frFR");
    config.set("locale", &locale);
    rz_proto::packet::set_codepage(&config.get("codepage", "windows-1252"));

    let (ev_tx, ev_rx) = mpsc::channel();
    let net = net::spawn(ev_tx);
    let mut assets = Assets::open(&root.join("Data"), &locale).map_err(|e| tracing::error!("{e:#}")).ok();
    let background = assets.as_mut().and_then(|a| a.texture("lobby_login_image.jpg"));
    let font = assets.as_mut().and_then(|a| a.font("tahoma.ttf"));

    let music_volume = config.get("Sound_MusicVolume", "0.5").parse().unwrap_or(0.5);
    let sfx_volume = config.get("Sound_SFXVolume", "0.7").parse().unwrap_or(0.7);
    let rzui = assets.as_mut().map(|a| rzui::RzUi::new(a, font.clone()));
    let mut app = App {
        combat_trace: ["-validate-combat","-validate-revival","-validate-visibility","-validate-layer","-validate-party","-validate-skills","-validate-casting"].iter().any(|flag|arg_value(flag).is_some()).then(Default::default),
        addons: addons::Addons::new(&root),
        addon_realm: "offline".into(),
        rzui,
        loading: None,
        audio: audio::Audio::new(music_volume, sfx_volume),
        field_music: audio::FIELD_MUSIC[0],
        auto_enter: arg_value("-enter"),
        world_frame: 0,
        root,
        error: assets.is_none().then(|| "Données introuvables : lancez rz-extractor client.".to_string()),
        assets,
        account: config.get("accountName", ""),
        config,
        net,
        events: ev_rx,
        gui: Gui::new(font),
        screen: Screen::Login,
        status: String::new(),
        password: String::new(),
        servers: Vec::new(),
        characters: Vec::new(),
        selected: 0,
        create: CreateForm { name: String::new(), race: 1, sex: 0, job: 0 },
        world: None,
        background,
    };
    app.gui.focus(if app.account.is_empty() { FIELD_ACCOUNT } else { FIELD_PASSWORD });

    if std::env::args().any(|a| a == "-demo") {
        if let Some(a) = &mut app.assets {
            let (x, y) = arg_value("-pos")
                .and_then(|p| p.split_once(',').map(|(x, y)| (x.parse().unwrap_or(8330.0), y.parse().unwrap_or(7167.0))))
                .unwrap_or((8330.0, 7167.0));
            app.world = Some(world::WorldState::demo(a, x, y));
            app.screen = Screen::World;
        }
    }
    // -login compte:mot_de_passe : connexion automatique (tests, raccourci).
    if let Some((account, password)) = arg_value("-login").as_deref().and_then(|l| l.split_once(':')) {
        let (host, port) = config::realmlist(&app.root, &app.config.get("locale", "frFR"));
        app.account = account.to_string();
        app.status = format!("Connexion à {host}:{port}…");
        app.send(NetCmd::Login { host, port, account: account.to_string(), password: password.to_string() });
    }
    let screenshot = arg_value("-screenshot");
    let validation_flags: Vec<_> = ["-validate-addons","-validate-combat","-validate-revival","-validate-visibility","-validate-layer","-validate-party","-validate-skills","-validate-casting"]
        .into_iter().filter_map(|flag|arg_value(flag).map(|path|(flag,path))).collect();
    let validation_args = match validation_flags.as_slice() {
        [] => None,
        [(flag,path)] => {
            let plan=PathBuf::from(path.as_str());
            Some(match *flag {
                "-validate-addons"=>online_validation::OnlineValidation::new(plan,&app),
                "-validate-combat"=>online_validation::OnlineValidation::combat(plan,&app),
                "-validate-revival"=>online_validation::OnlineValidation::revival(plan,&app),
                "-validate-visibility"=>online_validation::OnlineValidation::visibility(plan,&app),
                "-validate-layer"=>online_validation::OnlineValidation::layer(plan,&app),
                "-validate-party"=>online_validation::OnlineValidation::party(plan,&app),
                "-validate-skills"=>online_validation::OnlineValidation::skills(plan,&app),
                "-validate-casting"=>online_validation::OnlineValidation::casting(plan,&app),
                _=>unreachable!("validation flag comes from the fixed list"),
            })
        }
        _ => { tracing::error!("Choose only one validation scenario"); std::process::exit(2) },
    };
    let mut validation = validation_args.map(|result| {
        result
            .unwrap_or_else(|e| { tracing::error!("{e:#}"); std::process::exit(2) })
    });
    let mut frame = 0u32;

    loop {
        app.pump_events();
        app.sync_addons();
        app.addons.begin_frame(app.screen == Screen::World);
        app.gui.begin_frame();
        clear_background(Color::from_rgba(12, 14, 22, 255));
        match app.screen {
            Screen::World => {
                app.world_frame += 1;
                app.draw_world()
            }
            _ => app.draw_glue(),
        }
        app.sync_addons();
        app.addons.draw(&app.gui, app.screen == Screen::World);
        for message in app.addons.runtime.drain_messages() {
            if let Some(validation) = &mut validation { validation.message(&message); }
            tracing::info!("{message}");
            if let Some(world) = &mut app.world { world.addon_message(message); }
        }
        if let Some(validation) = &mut validation {
            match validation.tick(&mut app) {
                Ok(true) => break,
                Ok(false) => {},
                Err(error) => { tracing::error!("{error:#}"); validation.fail(error); break; }
            }
        }
        if let Some(a) = &mut app.assets {
            let wanted = if app.screen == Screen::World { app.field_music } else { audio::LOBBY_MUSIC };
            if screenshot.is_none() {
                app.audio.play_music(a, wanted).await;
            }
            if app.gui.take_clicked() {
                app.audio.effect(a, audio::CLICK).await;
            }
        }
        frame += 1;
        if let Some(path) = &screenshot {
            // Avec -enter, on attend d'être dans le monde (au plus ~20 s).
            let ready = app.auto_enter.is_none() || (app.screen == Screen::World && app.world_frame >= 60);
            let min_frames: u32 = arg_value("-frames").and_then(|f| f.parse().ok()).unwrap_or(30);
            if (frame >= min_frames && ready) || frame > 1200 + min_frames {
                get_screen_data().export_png(path);
                tracing::info!("Capture enregistrée : {path}");
                break;
            }
        }
        next_frame().await;
        if is_quit_requested() { break; }
    }
    if let Err(e) = app.addons.runtime.shutdown() { tracing::error!("addon settings: {e:#}"); }
    if let Some(mut validation) = validation {
        for message in app.addons.runtime.drain_messages() { validation.message(&message); tracing::info!("{message}"); }
        match validation.finish(&app) {
            Ok(true) => {},
            result => { tracing::error!("online validation unsuccessful: {result:?}"); std::process::exit(1); }
        }
    }
}

impl App {
    fn sync_addons(&mut self) {
        let state = self.world.as_ref().map(world::WorldState::addon_state).unwrap_or_default();
        self.addons.runtime.update_state(state);
    }
    fn send(&self, cmd: NetCmd) {
        let _ = self.net.send(cmd);
    }

    fn pump_events(&mut self) {
        while let Ok(ev) = self.events.try_recv() {
            if let Some(trace) = &mut self.combat_trace { trace.observe(&ev); }
            match ev {
                NetEvent::RealmSelected(id) => {
                    let (host, port) = config::realmlist(&self.root, &self.config.get("locale", "frFR"));
                    self.addon_realm = format!("{host}:{port}/{id}");
                }
                NetEvent::Status(s) => self.status = s,
                NetEvent::Error(e) => {
                    self.error = Some(e);
                    self.status.clear();
                    self.screen = Screen::Login;
                    self.world = None;
                }
                NetEvent::Disconnected(s) => {
                    self.status = s;
                    self.screen = Screen::Login;
                    self.world = None;
                }
                NetEvent::ServerList(list) => {
                    self.status.clear();
                    self.servers = list;
                    self.screen = Screen::Realms;
                    if self.servers.len() == 1 {
                        self.status = format!("Connexion à {}…", self.servers[0].name);
                        self.send(NetCmd::SelectServer(self.servers[0].index));
                    }
                }
                NetEvent::CharacterList(list) => {
                    self.status.clear();
                    self.characters = list;
                    self.selected = self.selected.min(self.characters.len().saturating_sub(1));
                    if self.screen != Screen::Create {
                        self.screen = Screen::Characters;
                    }
                    self.world = None;
                    if let Some(name) = self.auto_enter.clone() {
                        if self.characters.iter().any(|c| c.name == name) && self.status.is_empty() {
                            self.status = format!("Chargement de {name}…");
                            self.send(NetCmd::EnterWorld(name));
                        }
                    }
                }
                // Learning refusals are normal gameplay responses; the server supplies a notice/catalog.
                NetEvent::Result {request,..} if matches!(request,rz_proto::msg_game::id::TS_CS_LEARN_SKILL|rz_proto::msg_game::id::TS_CS_SKILL)=>{},
                NetEvent::Result { request, result } => {
                    if result != 0 {
                        self.error = Some(match (request, result) {
                            (msg::id::TS_CS_CREATE_CHARACTER, 9) => "Ce nom est déjà pris.".into(),
                            (msg::id::TS_CS_CREATE_CHARACTER, 0x13) => "Nom invalide (2 à 12 lettres ou chiffres).".into(),
                            (msg::id::TS_CS_CREATE_CHARACTER, 5) => "Nombre maximal de personnages atteint.".into(),
                            _ => format!("Requête {request} refusée (code {result})."),
                        });
                    } else if request == msg::id::TS_CS_CREATE_CHARACTER {
                        self.create.name.clear();
                        self.screen = Screen::Characters;
                    }
                }
                NetEvent::LoginResult(lr) => {
                    self.status.clear();
                    if lr.result == 0 {
                        // Load scoped variables before PLAYER_LOGIN and incoming world events.
                        let account = rz_addon_account(&self.account);
                        self.addons.runtime.update_state(Default::default());
                        if let Err(e) = self.addons.runtime.set_identity(&account, &self.addon_realm, Some(&lr.name)) {
                            self.addons.error = Some(format!("{e:#}"));
                        }
                        self.world = Some(world::WorldState::new(&lr));
                        self.loading = Some((get_time(), format!("loading_image{:02}.jpg", lr.handle % 20 + 1)));
                        self.field_music = audio::FIELD_MUSIC[lr.handle as usize % audio::FIELD_MUSIC.len()];
                        self.screen = Screen::World;
                        self.world_frame = 0;
                    } else {
                        self.error = Some(format!("Entrée en jeu refusée (code {}).", lr.result));
                    }
                }
                other => {
                    let system_message = match &other {
                        NetEvent::Chat { chat_type, message, .. } if *chat_type == msg::chat_type::NOTICE => Some(message.clone()),
                        _ => None,
                    };
                    let party_message=match &other {
                        NetEvent::Chat {chat_type,sender,message} if *chat_type==msg::chat_type::PARTY=>Some((sender.clone(),message.clone())),
                        _=>None,
                    };
                    if let (Some(w), Some(a)) = (&mut self.world, &mut self.assets) {
                        w.on_event(other, a);
                    }
                    self.sync_addons();
                    if let Some(message) = system_message {
                        self.addons.runtime.dispatch(rz_addon::Event::new("CHAT_MSG_SYSTEM", vec![message.into()]));
                    }
                    if let Some((sender,message))=party_message {
                        self.addons.runtime.dispatch(rz_addon::Event::new("CHAT_MSG_PARTY",vec![message.into(),sender.into()]));
                    }
                }
            }
            self.sync_addons();
        }
    }

    fn draw_glue(&mut self) {
        let (sw, sh) = (screen_width(), screen_height());
        if let Some(bg) = &self.background {
            draw_texture_ex(bg, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(sw, sh)), ..Default::default() });
        }
        if self.rzui.is_none() || self.screen == Screen::Create {
            self.gui.text("RappelzRust", 30.0, 60.0, 44, GOLD);
        }
        self.gui.text(concat!("RappelzRust v", env!("CARGO_PKG_VERSION"), " — RappelzCore"), sw - 260.0, sh - 10.0, 14, LIGHTGRAY);
        if !self.status.is_empty() {
            self.gui.text_centered(&self.status, sw / 2.0, sh - 40.0, 22, WHITE);
        }
        if let Some(e) = self.error.clone() {
            let r = Rect::new(sw / 2.0 - 240.0, sh / 2.0 - 70.0, 480.0, 140.0);
            self.gui.panel(r, "Erreur");
            self.gui.text_centered(&e, sw / 2.0, r.y + 70.0, 18, WHITE);
            if self.gui.button(Rect::new(sw / 2.0 - 60.0, r.y + 92.0, 120.0, 32.0), "OK") || is_key_pressed(KeyCode::Enter) {
                self.error = None;
            }
            return;
        }
        match self.screen {
            Screen::Login => self.draw_login(sw, sh),
            Screen::Realms => self.draw_realms(sw, sh),
            Screen::Characters => self.draw_characters(sw, sh),
            Screen::Create => self.draw_create(sw, sh),
            Screen::World => {}
        }
    }

    /// Saisie clavier dans un champ (focus géré par `self.gui`).
    fn edit_text(&mut self, field: u32, max: usize) {
        if self.gui.focused() != Some(field) {
            return;
        }
        let value = if field == FIELD_ACCOUNT { &mut self.account } else { &mut self.password };
        while let Some(c) = get_char_pressed() {
            if !c.is_control() && value.chars().count() < max {
                value.push(c);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            value.pop();
        }
    }

    fn start_login(&mut self) {
        let (host, port) = config::realmlist(&self.root, &self.config.get("locale", "frFR"));
        if self.account.is_empty() || !self.status.is_empty() {
            return;
        }
        self.config.set("accountName", &self.account);
        self.config.save();
        self.status = format!("Connexion à {host}:{port}…");
        let password = std::mem::take(&mut self.password);
        self.send(NetCmd::Login { host, port, account: self.account.clone(), password });
    }

    /// Connexion avec la fenêtre d'origine `window_lobby_login.nui`.
    fn draw_login_rz(&mut self, sw: f32, sh: f32) -> bool {
        let (Some(ui), Some(assets)) = (&mut self.rzui, &mut self.assets) else { return false };
        let size = ui.size(assets, "window_lobby_login.nui");
        if size.x < 50.0 {
            return false;
        }
        let logo = ui.size(assets, "window_lobby_login_logo.nui");
        ui.draw(assets, "window_lobby_login_logo.nui", vec2((sw - logo.x) / 2.0, sh * 0.08), &rz_ui::UiState::default());
        let origin = vec2((sw - size.x) / 2.0, sh * 0.58);
        let mut st = rz_ui::UiState::default();
        st.set_caption("static_text00", "Compte").set_caption("static_text01", "Mot de passe");
        let frame = ui.draw(assets, "window_lobby_login.nui", origin, &st);
        let acc_r = frame.rect("accountedit").unwrap_or(Rect::new(origin.x + 147.0, origin.y + 45.0, 161.0, 20.0));
        let pwd_r = frame.rect("passwordedit").unwrap_or(Rect::new(origin.x + 147.0, origin.y + 74.0, 161.0, 20.0));
        let caret = if (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
        let acc_text = format!("{}{}", self.account, if self.gui.focused() == Some(FIELD_ACCOUNT) { caret } else { "" });
        let pwd_text = format!("{}{}", "*".repeat(self.password.chars().count()), if self.gui.focused() == Some(FIELD_PASSWORD) { caret } else { "" });
        ui.label(&acc_text, acc_r.x + 4.0, acc_r.y + acc_r.h - 5.0, WHITE);
        ui.label(&pwd_text, pwd_r.x + 4.0, pwd_r.y + pwd_r.h - 5.0, WHITE);
        let (host, port) = config::realmlist(&self.root, &self.config.get("locale", "frFR"));
        ui.label(&format!("realmlist : {host}:{port}"), 10.0, sh - 10.0, LIGHTGRAY);
        let (ok, quit) = (frame.clicked("idok"), frame.clicked("button_quitgame"));
        if is_mouse_button_pressed(MouseButton::Left) {
            let m: Vec2 = mouse_position().into();
            if acc_r.contains(m) {
                self.gui.focus(FIELD_ACCOUNT);
            } else if pwd_r.contains(m) {
                self.gui.focus(FIELD_PASSWORD);
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            let next = if self.gui.focused() == Some(FIELD_ACCOUNT) { FIELD_PASSWORD } else { FIELD_ACCOUNT };
            self.gui.focus(next);
        }
        let field = self.gui.focused().unwrap_or(FIELD_ACCOUNT);
        self.edit_text(field, if field == FIELD_ACCOUNT { 32 } else { 16 });
        if ok || is_key_pressed(KeyCode::Enter) {
            self.start_login();
        }
        if quit {
            std::process::exit(0);
        }
        true
    }

    /// Liste des royaumes avec `window_lobby_server_list.nui`.
    fn draw_realms_rz(&mut self, sw: f32, sh: f32) -> bool {
        let (Some(ui), Some(assets)) = (&mut self.rzui, &mut self.assets) else { return false };
        let size = ui.size(assets, "window_lobby_server_list.nui");
        if size.x < 50.0 {
            return false;
        }
        let origin = vec2((sw - size.x) / 2.0, (sh - size.y) / 2.0);
        let mut st = rz_ui::UiState::default();
        st.set_caption("static_text", "Liste des serveurs");
        // Les exemples et la légende hors cadre servent à l'éditeur, pas au jeu.
        for id in ["server_select", "server_list_text_00", "server_list_text_01", "server_select_sample_01", "common_selectclickbackground_01", "static_condition_check_01"] {
            st.hide(id);
        }
        for s in ["a", "b", "c", "d", "e"] {
            st.hide(&format!("server_list_text_sample_{s}"));
            st.hide(&format!("static_condition_check_{s}"));
        }
        if self.servers.is_empty() {
            st.hide("server_select_sample_00").hide("static_condition_check_00").hide("common_selectclickbackground_00");
        }
        let frame = ui.draw(assets, "window_lobby_server_list.nui", origin, &st);
        let row0 = frame.rect("common_selectclickbackground_00").unwrap_or(Rect::new(origin.x + 16.0, origin.y + 46.0, 246.0, 44.0));
        let mut chosen = None;
        let m: Vec2 = mouse_position().into();
        for (i, s) in self.servers.iter().enumerate() {
            let r = Rect::new(row0.x, row0.y + i as f32 * 44.0, row0.w, 42.0);
            if r.contains(m) {
                draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(255, 210, 90, 40));
                if is_mouse_button_pressed(MouseButton::Left) {
                    chosen = Some(s.index);
                }
            }
            ui.label(&s.name, r.x + 40.0, r.y + 18.0, Color::from_rgba(255, 230, 160, 255));
            ui.label(&format!("{}:{}   population {} %", s.ip, s.port, s.user_ratio), r.x + 40.0, r.y + 34.0, LIGHTGRAY);
        }
        if (frame.clicked("button_choice") || is_key_pressed(KeyCode::Enter)) && !self.servers.is_empty() {
            chosen = Some(self.servers[0].index);
        }
        if let Some(idx) = chosen {
            self.status = "Connexion au royaume…".into();
            self.send(NetCmd::SelectServer(idx));
        }
        true
    }

    /// Sélection du personnage avec `window_lobby_character_list.nui` + `_admin.nui`.
    fn draw_characters_rz(&mut self, sw: f32, sh: f32) -> bool {
        let labels: Vec<(String, String, String)> = self
            .characters
            .iter()
            .map(|c| {
                let job = self.assets.as_ref().map(|a| a.job_name(c.job)).unwrap_or_default();
                (c.name.clone(), job, c.level.to_string())
            })
            .collect();
        let (Some(ui), Some(assets)) = (&mut self.rzui, &mut self.assets) else { return false };
        let size = ui.size(assets, "window_lobby_character_list.nui");
        if size.x < 50.0 {
            return false;
        }
        let origin = vec2(sw - size.x - 30.0, (sh - size.y - 110.0).max(10.0) / 2.0);
        let mut st = rz_ui::UiState::default();
        st.set_caption("static_text", "Personnages");
        for i in 0..8 {
            match labels.get(i) {
                Some((name, job, lv)) => {
                    st.set_caption(&format!("select_name_text{i:02}"), name)
                        .set_caption(&format!("select_jobname_text{i:02}"), job)
                        .set_caption(&format!("select_lv_number{i:02}"), lv)
                        .set_caption(&format!("select_jlv_number{i:02}"), "1");
                    if i != self.selected {
                        st.hide(&format!("character_select_{i:02}"));
                    }
                }
                None => {
                    for p in ["character_select_", "select_name_text", "select_jobname_text", "select_lv_number", "select_jlv_number", "select_lv_font", "select_jlv_font", "static_job_icon"] {
                        st.hide(&format!("{p}{i:02}"));
                    }
                }
            }
        }
        let frame = ui.draw(assets, "window_lobby_character_list.nui", origin, &st);
        for i in 0..labels.len().min(8) {
            if frame.clicked(&format!("select_lv_bg{i:02}")) || frame.clicked(&format!("character_select_{i:02}")) {
                self.selected = i;
            }
        }
        let admin = ui.size(assets, "window_lobby_character_admin.nui");
        let mut ast = rz_ui::UiState::default();
        ast.set_caption("static_text", "Gestion");
        if labels.is_empty() {
            ast.set_state("button_character_delete", rz_ui::ControlState::Disabled);
        }
        let aframe = ui.draw(assets, "window_lobby_character_admin.nui", vec2(origin.x, origin.y + size.y + 8.0), &ast);
        let _ = admin;
        if aframe.clicked("button_character_create") {
            self.screen = Screen::Create;
            self.gui.focus(FIELD_NAME);
        }
        if aframe.clicked("button_character_delete") {
            if let Some(c) = self.characters.get(self.selected) {
                self.send(NetCmd::DeleteCharacter(c.name.clone()));
            }
        }
        if (frame.clicked("entergamebtn") || is_key_pressed(KeyCode::Enter)) && self.status.is_empty() {
            if let Some(c) = self.characters.get(self.selected) {
                self.status = format!("Chargement de {}…", c.name);
                self.send(NetCmd::EnterWorld(c.name.clone()));
            }
        }
        true
    }

    fn draw_login(&mut self, sw: f32, sh: f32) {
        if self.draw_login_rz(sw, sh) {
            return;
        }
        let r = Rect::new(sw / 2.0 - 170.0, sh * 0.52, 340.0, 220.0);
        self.gui.panel(r, "");
        self.gui.text_centered("Nom de compte", sw / 2.0, r.y + 28.0, 18, GOLD);
        self.gui.text_field(FIELD_ACCOUNT, Rect::new(r.x + 20.0, r.y + 38.0, 300.0, 30.0), &mut self.account, false, 32);
        self.gui.text_centered("Mot de passe", sw / 2.0, r.y + 98.0, 18, GOLD);
        self.gui.text_field(FIELD_PASSWORD, Rect::new(r.x + 20.0, r.y + 108.0, 300.0, 30.0), &mut self.password, true, 16);
        if is_key_pressed(KeyCode::Tab) {
            let next = if self.gui.focused() == Some(FIELD_ACCOUNT) { FIELD_PASSWORD } else { FIELD_ACCOUNT };
            self.gui.focus(next);
        }
        let mut connect = self.gui.button(Rect::new(sw / 2.0 - 90.0, r.y + 160.0, 180.0, 36.0), "Connexion");
        connect |= is_key_pressed(KeyCode::Enter);
        let (host, port) = config::realmlist(&self.root, &self.config.get("locale", "frFR"));
        self.gui.text(&format!("realmlist : {host}:{port}"), 12.0, sh - 12.0, 14, LIGHTGRAY);
        if connect && !self.account.is_empty() && self.status.is_empty() {
            self.config.set("accountName", &self.account);
            self.config.save();
            self.status = format!("Connexion à {host}:{port}…");
            let password = std::mem::take(&mut self.password);
            self.send(NetCmd::Login { host, port, account: self.account.clone(), password });
        }
    }

    fn draw_realms(&mut self, sw: f32, sh: f32) {
        if self.draw_realms_rz(sw, sh) {
            return;
        }
        let r = Rect::new(sw / 2.0 - 260.0, sh / 2.0 - 180.0, 520.0, 360.0);
        self.gui.panel(r, "Liste des royaumes");
        let mut chosen = None;
        for (i, s) in self.servers.iter().enumerate() {
            let label = format!("{:<20}  {}:{}   population {} %", s.name, s.ip, s.port, s.user_ratio);
            if self.gui.list_item(Rect::new(r.x + 16.0, r.y + 44.0 + i as f32 * 32.0, r.w - 32.0, 30.0), &label, false) {
                chosen = Some(s.index);
            }
        }
        if let Some(idx) = chosen {
            self.status = "Connexion au royaume…".into();
            self.send(NetCmd::SelectServer(idx));
        }
    }

    fn character_label(&self, c: &LobbyCharacterInfo) -> String {
        let job = self.assets.as_ref().map(|a| a.job_name(c.job)).unwrap_or_default();
        let race = RACES.iter().find(|r| r.0 == c.race).map(|r| r.1).unwrap_or("?");
        format!("{}   niv. {}   {race} — {job}", c.name, c.level)
    }

    fn draw_characters(&mut self, sw: f32, sh: f32) {
        if self.draw_characters_rz(sw, sh) {
            return;
        }
        let r = Rect::new(sw - 440.0, 110.0, 410.0, sh - 220.0);
        self.gui.panel(r, "Personnages");
        let labels: Vec<String> = self.characters.iter().map(|c| self.character_label(c)).collect();
        for (i, label) in labels.iter().enumerate() {
            if self.gui.list_item(Rect::new(r.x + 12.0, r.y + 44.0 + i as f32 * 34.0, r.w - 24.0, 32.0), label, i == self.selected) {
                self.selected = i;
            }
        }
        if labels.is_empty() {
            self.gui.text_centered("Aucun personnage", r.x + r.w / 2.0, r.y + 70.0, 18, LIGHTGRAY);
        }
        if self.gui.button(Rect::new(r.x + 12.0, r.y + r.h - 46.0, 190.0, 34.0), "Créer un personnage") {
            self.screen = Screen::Create;
            self.gui.focus(FIELD_NAME);
        }
        if !self.characters.is_empty() && self.gui.button(Rect::new(r.x + r.w - 202.0, r.y + r.h - 46.0, 190.0, 34.0), "Supprimer") {
            if let Some(c) = self.characters.get(self.selected) {
                self.send(NetCmd::DeleteCharacter(c.name.clone()));
            }
        }
        let enter_rect = Rect::new(sw / 2.0 - 130.0, sh - 80.0, 260.0, 42.0);
        if !self.characters.is_empty()
            && (self.gui.button(enter_rect, "Entrer dans le monde") || is_key_pressed(KeyCode::Enter))
            && self.status.is_empty()
        {
            if let Some(c) = self.characters.get(self.selected) {
                self.status = format!("Chargement de {}…", c.name);
                self.send(NetCmd::EnterWorld(c.name.clone()));
            }
        }
    }

    fn draw_create(&mut self, sw: f32, sh: f32) {
        let jobs = self.assets.as_ref().map(|a| a.starting_jobs()).unwrap_or_else(|| vec![(0, "Novice".into())]);
        let job_names: Vec<&str> = jobs.iter().map(|j| j.1.as_str()).collect();
        let r = Rect::new(sw / 2.0 - 220.0, sh / 2.0 - 190.0, 440.0, 380.0);
        self.gui.panel(r, "Création de personnage");
        let x = r.x + 30.0;
        let w = r.w - 60.0;
        self.gui.text("Nom (2 à 12 caractères)", x, r.y + 64.0, 17, GOLD);
        self.gui.text_field(FIELD_NAME, Rect::new(x, r.y + 72.0, w, 30.0), &mut self.create.name, false, 12);
        self.gui.text("Race", x, r.y + 128.0, 17, GOLD);
        self.gui.selector(Rect::new(x, r.y + 136.0, w, 30.0), &["Gaia", "Deva", "Asura"], &mut self.create.race);
        self.gui.text("Sexe", x, r.y + 192.0, 17, GOLD);
        self.gui.selector(Rect::new(x, r.y + 200.0, w, 30.0), &["Homme", "Femme"], &mut self.create.sex);
        self.gui.text("Métier", x, r.y + 256.0, 17, GOLD);
        self.gui.selector(Rect::new(x, r.y + 264.0, w, 30.0), &job_names, &mut self.create.job);
        let create = self.gui.button(Rect::new(x, r.y + r.h - 54.0, 170.0, 36.0), "Créer");
        if self.gui.button(Rect::new(r.x + r.w - 200.0, r.y + r.h - 54.0, 170.0, 36.0), "Retour") || is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Characters;
        }
        if (create || is_key_pressed(KeyCode::Enter)) && !self.create.name.trim().is_empty() {
            let info = LobbyCharacterInfo {
                sex: self.create.sex as i32 + 1,
                race: RACES[self.create.race.min(2)].0,
                level: 1,
                job: jobs.get(self.create.job).map(|j| j.0).unwrap_or(0),
                job_level: 1,
                name: self.create.name.trim().to_string(),
                ..Default::default()
            };
            self.send(NetCmd::CreateCharacter(info));
        }
    }

    fn draw_world(&mut self) {
        let (Some(w), Some(a), Some(ui)) = (&mut self.world, &mut self.assets, &mut self.rzui) else { return };
        let actions = w.update_and_draw(a, ui, self.addons.mouse_captured, self.addons.manager);
        if w.take_reload_ui() { self.addons.reload(); }
        // Écran de chargement : la scène se dessine dessous (textures préchargées), puis disparaît.
        if let Some((since, image)) = &self.loading {
            let t = (get_time() - since) as f32;
            if t > 1.8 {
                self.loading = None;
            } else {
                let (sw, sh) = (screen_width(), screen_height());
                clear_background(BLACK);
                if let Some(tex) = a.texture(image) {
                    let scale = (sw / tex.width()).min(sh / tex.height());
                    let (w2, h2) = (tex.width() * scale, tex.height() * scale);
                    draw_texture_ex(&tex, (sw - w2) / 2.0, (sh - h2) / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(w2, h2)), ..Default::default() });
                }
                let progress = (t / 1.6).min(1.0);
                draw_rectangle(sw * 0.2, sh - 40.0, sw * 0.6, 12.0, Color::from_rgba(0, 0, 0, 180));
                draw_rectangle(sw * 0.2 + 2.0, sh - 38.0, (sw * 0.6 - 4.0) * progress, 8.0, GOLD);
            }
        }
        for cmd in actions {
            if matches!(cmd, NetCmd::ReturnLobby) {
                self.status = "Retour au lobby…".into();
            }
            let _ = self.net.send(cmd);
        }
    }
}
