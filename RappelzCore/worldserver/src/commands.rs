//! Commandes de chat (`.commande`) et de console, dans l'esprit d'AzerothCore.
//! Niveaux : 0 joueur, 1 modérateur, 2 GM, 3 administrateur (`account_access.gmlevel`).

use std::sync::Arc;

use anyhow::Result;
use rz_database::auth;
use rz_proto::msg::{self, Message};

use crate::session::Session;
use crate::Ctx;

struct Command {
    name: &'static str,
    level: u8,
    help: &'static str,
}

const COMMANDS: &[Command] = &[
    Command { name: "help", level: 0, help: ".help — liste des commandes" },
    Command { name: "gps", level: 0, help: ".gps — position actuelle" },
    Command { name: "online", level: 0, help: ".online — joueurs connectés" },
    Command { name: "save", level: 0, help: ".save — sauvegarde le personnage" },
    Command { name: "party", level: 0, help: ".party invite <joueur> | accept | decline | leave | kick <joueur> | leader <joueur> | disband" },
    Command { name: "server", level: 0, help: ".server info — état du serveur" },
    Command { name: "tele", level: 1, help: ".tele <lieu> — téléportation (game_tele)" },
    Command { name: "go", level: 1, help: ".go <x> <y> [z] — téléportation aux coordonnées" },
    Command { name: "layer", level: 2, help: ".layer <0-255> — change la couche du personnage" },
    Command { name: "summon", level: 2, help: ".summon <joueur> — fait venir un joueur" },
    Command { name: "appear", level: 1, help: ".appear <joueur> — rejoint un joueur" },
    Command { name: "announce", level: 2, help: ".announce <texte> — annonce à tous" },
    Command { name: "level", level: 2, help: ".level <n> — change le niveau" },
    Command { name: "money", level: 2, help: ".money <n> — ajoute des rupees" },
    Command { name: "additem", level: 2, help: ".additem <id> [nombre] — ajoute un objet (Item.dbc)" },
    Command { name: "spawn", level: 2, help: ".spawn <id_monstre> [respawn_s] — place un monstre ici (table creature)" },
    Command { name: "vendor", level: 3, help: ".vendor add <id_objet> — ajoute l'objet au PNJ ciblé le plus proche (npc_vendor)" },
];

/// Exécute une commande de chat ; la réponse part sur le canal @NOTICE du joueur.
pub async fn handle(session: &mut Session, line: &str) -> Result<()> {
    let Some(g) = &session.in_game else { return Ok(()) };
    let (handle, char_name, char_z) = (g.handle, g.character.name.clone(), g.character.position_z);
    let ctx = session.ctx.clone();
    let mut parts = line.split_whitespace();
    let name = parts.next().unwrap_or("").to_lowercase();
    let args: Vec<&str> = parts.collect();
    let reply = |text: &str| ctx.world.lock().unwrap().notify(handle, text);

    let Some(cmd) = COMMANDS.iter().find(|c| c.name == name) else {
        reply("Commande inconnue. Tapez .help");
        return Ok(());
    };
    if session.gm_level < cmd.level {
        reply("Vous n'avez pas le niveau requis pour cette commande.");
        return Ok(());
    }

    match cmd.name {
        "party" => ctx.world.lock().unwrap().party_command(handle,&args),
        "help" => {
            for c in COMMANDS.iter().filter(|c| c.level <= session.gm_level) {
                reply(c.help);
            }
        }
        "gps" => {
            let pos = ctx.world.lock().unwrap().player_position(handle);
            if let Some((x, y)) = pos {
                reply(&format!("X: {x:.1} Y: {y:.1} Z: {:.1}", char_z));
            }
        }
        "online" => {
            let names: Vec<String> = ctx.world.lock().unwrap().players.values().map(|p| p.name.clone()).collect();
            reply(&format!("{} joueur(s) : {}", names.len(), names.join(", ")));
        }
        "save" => {
            session.save().await?;
            reply("Personnage sauvegardé.");
        }
        "server" => reply(&server_info(&ctx)),
        "tele" => match ctx.data.find_teleport(&args.join(" ")) {
            Some(t) => ctx.world.lock().unwrap().teleport(handle, t.x, t.y, t.z),
            None => reply("Lieu inconnu (table game_tele)."),
        },
        "go" => {
            let nums: Vec<f32> = args.iter().filter_map(|a| a.parse().ok()).collect();
            if nums.len() >= 2 {
                ctx.world.lock().unwrap().teleport(handle, nums[0], nums[1], nums.get(2).copied().unwrap_or(0.0));
            } else {
                reply("Usage : .go <x> <y> [z]");
            }
        }
        "summon" | "appear" => {
            let mut world = ctx.world.lock().unwrap();
            let target = args.first().and_then(|n| world.find_by_name(n)).map(|p| p.handle);
            match target {
                Some(t) => {
                    let (from, to) = if cmd.name == "summon" { (handle, t) } else { (t, handle) };
                    if let Some((x, y)) = world.player_position(from) {
                        world.teleport(to, x, y, 0.0);
                    }
                }
                None => world.notify(handle, "Joueur introuvable."),
            }
        }
        "layer" => {
            let layer = if args.len() == 1 { args[0].parse::<u8>().ok() } else { None };
            if let Some(layer) = layer {
                let (exists,switched) = {
                    let mut world=ctx.world.lock().unwrap();
                    let switched=world.players.get(&handle).is_some_and(|p|p.layer!=layer);
                    (world.change_layer(handle,layer),switched)
                };
                if exists {
                    if switched {session.dialog_npc=None;}
                    reply(&format!("Couche {layer}."));
                }
            } else {
                reply("Usage : .layer <0-255>");
            }
        }
        "announce" => {
            let text = format!("[{char_name}] : {}", args.join(" "));
            announce(&ctx, &text);
        }
        "level" => match args.first().and_then(|a| a.parse::<u8>().ok()) {
            Some(lv) if lv >= 1 => {
                if let Some(g) = &mut session.in_game {
                    g.character.level = lv;
                }
                let mut w = ctx.world.lock().unwrap();
                let job_level = session.in_game.as_ref().map(|g| g.character.extra_i64("jlv", 1) as i32).unwrap_or(1);
                w.set_player_level(handle, lv, job_level);
                w.notify(handle, &format!("Niveau {lv}."));
            }
            _ => reply("Usage : .level <n>"),
        },
        "additem" => match args.first().and_then(|a| a.parse::<u32>().ok()) {
            Some(item) if ctx.data.items.contains_key(&item) => {
                let count = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(1);
                let name = ctx.data.items[&item].name.clone();
                if session.give_item(item, count).await? {
                    reply(&format!("Ajouté : {name} ×{count}"));
                } else {
                    reply("Le sac est plein.");
                }
            }
            _ => reply("Usage : .additem <id> [nombre] (id d'Item.dbc)"),
        },
        "spawn" => match args.first().and_then(|a| a.parse::<u32>().ok()).and_then(|id| ctx.data.monsters.get(&id).cloned()) {
            Some(info) => {
                let respawn = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(30u32);
                let pos = ctx.world.lock().unwrap().player_position(handle);
                if let Some((x, y)) = pos {
                    let guid = ctx.next_creature_guid();
                    rz_database::world::add_creature_spawn(&ctx.world_db, guid, info.id, x + 60.0, y, 0.0, respawn).await?;
                    let name = info.name.clone();
                    ctx.world.lock().unwrap().spawn_monster(info, x + 60.0, y, 0.0, respawn);
                    reply(&format!("{name} placé (creature guid {guid})."));
                }
            }
            None => reply("Usage : .spawn <id_monstre> [respawn_s] (id de Monster.dbc)"),
        },
        "vendor" => match (args.first(), args.get(1).and_then(|a| a.parse::<u32>().ok())) {
            (Some(&"add"), Some(item)) if ctx.data.items.contains_key(&item) => {
                let nearest = {
                    let w = ctx.world.lock().unwrap();
                    let (px, py) = w.player_position(handle).unwrap_or_default();
                    w.npcs
                        .values()
                        .min_by(|a, b| {
                            let da = (a.x - px).powi(2) + (a.y - py).powi(2);
                            let db = (b.x - px).powi(2) + (b.y - py).powi(2);
                            da.total_cmp(&db)
                        })
                        .map(|n| (n.npc_id, n.name.clone()))
                };
                if let Some((npc_id, npc_name)) = nearest {
                    let entry = crate::data::NPC_ENTRY_OFFSET + npc_id;
                    rz_database::world::add_vendor_item(&ctx.world_db, entry, item).await?;
                    ctx.data.vendors.write().unwrap().entry(entry).or_default().push(item);
                    reply(&format!("{} ajouté à la boutique de {npc_name}.", ctx.data.items[&item].name));
                }
            }
            _ => reply("Usage : .vendor add <id_objet>"),
        },
        "money" => match args.first().and_then(|a| a.parse::<i64>().ok()) {
            Some(n) => {
                let total = if let Some(g) = &mut session.in_game {
                    g.character.money = (g.character.money as i64 + n).clamp(0, u32::MAX as i64) as u32;
                    g.character.money
                } else {
                    0
                };
                let w = ctx.world.lock().unwrap();
                w.send_to(handle, msg::ScProperty { handle, name: "gold".into(), value: total as i64, text: None }.to_packet());
                w.notify(handle, &format!("Rupees : {total}."));
            }
            None => reply("Usage : .money <n>"),
        },
        _ => {}
    }
    Ok(())
}

pub fn announce(ctx: &Ctx, text: &str) {
    let packet = msg::ScChat { sender: "@NOTICE".into(), chat_type: msg::chat_type::NOTICE, message: text.to_string() }.to_packet();
    ctx.world.lock().unwrap().broadcast_all(&packet);
}

fn server_info(ctx: &Ctx) -> String {
    let w = ctx.world.lock().unwrap();
    let up = w.uptime_secs();
    format!(
        "RappelzCore {} — joueurs : {} (max {}), PNJ : {}, uptime : {}h{:02}m{:02}s",
        env!("CARGO_PKG_VERSION"),
        w.players.len(),
        w.max_online,
        w.npcs.len(),
        up / 3600,
        up / 60 % 60,
        up % 60
    )
}

/// Commandes de la console du worldserver (stdin), comme `worldserver>` d'AzerothCore.
pub async fn console(ctx: Arc<Ctx>, line: &str) -> Result<bool> {
    let result = execute_console(&ctx, line).await?;
    if !result.output.is_empty() { println!("{}", result.output); }
    Ok(result.shutdown)
}

pub struct ConsoleResult {
    pub output: String,
    pub success: bool,
    pub shutdown: bool,
}

fn gm_access_arguments(level: &str, realms: &[&str]) -> Option<(u8, i32)> {
    let level = level.parse::<u8>().ok().filter(|v| *v <= 3)?;
    let realm = match realms { [] => -1, [realm] => realm.parse::<i32>().ok()?, _ => return None };
    (realm == -1 || realm > 0).then_some((level, realm))
}

/// Shared console/SOAP dispatcher, called serially by the world loop.
pub async fn execute_console(ctx: &Ctx, line: &str) -> Result<ConsoleResult> {
    let mut result = ConsoleResult { output: String::new(), success: true, shutdown: false };
    let args: Vec<&str> = line.split_whitespace().collect();
    match args.as_slice() {
        [] => {}
        ["help"] | ["?"] => {
            result.output = [
                "account create <compte> <mot_de_passe> [email]",
                "account set password <compte> <mot_de_passe>",
                "account set gmlevel <compte> <0-3> [royaume|-1]",
                "announce <texte>",
                "server info",
                "saveall",
                "server shutdown | exit",
            ].join("\n");
        }
        ["account", "create", user, pass, rest @ ..] if rest.len() <= 1 => {
            let email = rest.first().copied().unwrap_or("");
            match auth::create_account(&ctx.login, user, pass, email).await {
                Ok(id) => result.output = format!("Compte {user} créé (id {id})."),
                Err(_) => { result.output = "Échec de création du compte.".into(); result.success = false; }
            }
        }
        ["account", "set", "password", user, pass] if pass.chars().count() <= 16 => match auth::find_account(&ctx.login, user).await? {
            Some(a) => {
                auth::set_password(&ctx.login, a.id, user, pass).await?;
                result.output = format!("Mot de passe de {user} modifié.");
            }
            None => { result.output = "Compte inconnu.".into(); result.success = false; }
        },
        ["account", "set", "gmlevel", user, level, rest @ ..] => {
            let Some((lv, realm)) = gm_access_arguments(level, rest) else {
                result.output = "Usage : account set gmlevel <compte> <0-3> [royaume|-1]".into(); result.success = false;
                return Ok(result);
            };
            match auth::find_account(&ctx.login, user).await? {
                Some(a) => {
                    auth::set_gm_level(&ctx.login, a.id, lv, realm).await?;
                    result.output = format!("{user} : niveau GM {lv} (royaume {realm}).");
                }
                _ => { result.output = "Usage : account set gmlevel <compte> <0-3> [royaume|-1]".into(); result.success = false; }
            }
        }
        ["announce", text @ ..] if !text.is_empty() => { announce(ctx, &text.join(" ")); result.output = "Annonce envoyée.".into(); }
        ["server", "info"] => result.output = server_info(ctx),
        ["saveall"] => {
            let _ = ctx.save_all.send(());
            result.output = "Sauvegarde de tous les joueurs demandée.".into();
        }
        ["server", "shutdown"] | ["exit"] | ["quit"] => { result.shutdown = true; result.output = "Arrêt du serveur demandé.".into(); }
        _ => { result.output = "Commande inconnue (help).".into(); result.success = false; }
    }
    Ok(result)
}

#[cfg(test)]
mod console_tests {
    use super::*;
    #[test] fn gm_access_accepts_only_known_levels_and_explicit_valid_realm_scope() {
        assert_eq!(gm_access_arguments("3", &[]), Some((3, -1)));
        assert_eq!(gm_access_arguments("0", &["-1"]), Some((0, -1)));
        assert_eq!(gm_access_arguments("2", &["1"]), Some((2, 1)));
        assert_eq!(gm_access_arguments("3", &["2147483647"]), Some((3, i32::MAX)));
    }
    #[test] fn malformed_realm_never_falls_back_to_global_access() {
        for realm in ["bad", "", "-2", "0", "2147483648"] { assert_eq!(gm_access_arguments("3", &[realm]), None); }
        assert_eq!(gm_access_arguments("3", &["1", "-1"]), None);
        for level in ["-1", "4", "256", "admin"] { assert_eq!(gm_access_arguments(level, &["1"]), None); }
    }
}
