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
    Command { name: "server", level: 0, help: ".server info — état du serveur" },
    Command { name: "tele", level: 1, help: ".tele <lieu> — téléportation (game_tele)" },
    Command { name: "go", level: 1, help: ".go <x> <y> [z] — téléportation aux coordonnées" },
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
                if let Some(p) = w.players.get_mut(&handle) {
                    p.max_hp = crate::gameplay::max_hp_for(lv);
                    p.max_mp = crate::gameplay::max_mp_for(lv);
                    p.hp = p.max_hp;
                    p.mp = p.max_mp;
                    p.attack = crate::gameplay::attack_for(lv);
                    p.defence = crate::gameplay::defence_for(lv);
                    p.enter.creature.level = lv as i32;
                }
                let level_packet = rz_proto::msg_game::ScLevelUpdate { handle, level: lv as i32, job_level: 1 }.to_packet();
                w.broadcast_around(handle, &level_packet, true);
                w.send_to(handle, msg::ScProperty { handle, name: "lv".into(), value: lv as i64, text: None }.to_packet());
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
    let args: Vec<&str> = line.split_whitespace().collect();
    match args.as_slice() {
        [] => {}
        ["help"] | ["?"] => {
            for l in [
                "account create <compte> <mot_de_passe> [email]",
                "account set password <compte> <mot_de_passe>",
                "account set gmlevel <compte> <0-3> [royaume|-1]",
                "announce <texte>",
                "server info",
                "saveall",
                "server shutdown | exit",
            ] {
                println!("  {l}");
            }
        }
        ["account", "create", user, pass, rest @ ..] => {
            let email = rest.first().copied().unwrap_or("");
            match auth::create_account(&ctx.login, user, pass, email).await {
                Ok(id) => println!("Compte {user} créé (id {id})."),
                Err(e) => println!("Échec : {e}"),
            }
        }
        ["account", "set", "password", user, pass] => match auth::find_account(&ctx.login, user).await? {
            Some(a) => {
                auth::set_password(&ctx.login, a.id, user, pass).await?;
                println!("Mot de passe de {user} modifié.");
            }
            None => println!("Compte inconnu."),
        },
        ["account", "set", "gmlevel", user, level, rest @ ..] => {
            let realm: i32 = rest.first().and_then(|r| r.parse().ok()).unwrap_or(-1);
            match (auth::find_account(&ctx.login, user).await?, level.parse::<u8>()) {
                (Some(a), Ok(lv)) if lv <= 3 => {
                    auth::set_gm_level(&ctx.login, a.id, lv, realm).await?;
                    println!("{user} : niveau GM {lv} (royaume {realm}).");
                }
                _ => println!("Usage : account set gmlevel <compte> <0-3> [royaume|-1]"),
            }
        }
        ["announce", text @ ..] => announce(&ctx, &text.join(" ")),
        ["server", "info"] => println!("{}", server_info(&ctx)),
        ["saveall"] => {
            let _ = ctx.save_all.send(());
            println!("Sauvegarde de tous les joueurs demandée.");
        }
        ["server", "shutdown", ..] | ["exit"] | ["quit"] => return Ok(true),
        _ => println!("Commande inconnue (help)."),
    }
    Ok(false)
}
