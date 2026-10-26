//! rz-dbimport : remplit `acore_world` (schéma AzerothCore, inchangé) à partir des DBC
//! extraits du client RappelzRust (`rz-extractor dbc`).
//!
//! | DBC Rappelz        | Tables AzerothCore                                  |
//! |--------------------|-----------------------------------------------------|
//! | Item.dbc           | `item_template`                                     |
//! | Monster.dbc        | `creature_template`, `creature_template_model`      |
//! | NpcResource.dbc    | `creature_template` (entry + [`NPC_ENTRY_OFFSET`]), `creature` (positions) |
//! | Quest.dbc          | `quest_template`                                    |
//! | WorldLocation.dbc  | `game_tele`                                         |
//!
//! Les colonnes sans équivalent restent à leur valeur par défaut AzerothCore ; le
//! worldserver relit les détails propres à Rappelz directement dans les DBC.

use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use rz_rappelz::dbcstore::{NamedDbc, StringTable};

/// Les ids de PNJ et de monstres se recouvrent dans Rappelz : les PNJ sont décalés.
pub const NPC_ENTRY_OFFSET: u32 = 1_000_000;
/// Carte unique du continent Rappelz.
pub const RAPPELZ_MAP: u32 = 0;
const ROWS_PER_INSERT: usize = 500;

#[derive(Parser)]
#[command(name = "rz-dbimport", version, about = "DBC RappelzRust → acore_world")]
struct Cli {
    /// Dossier contenant `dbc/` (DataDir du worldserver).
    #[arg(long, default_value = ".")]
    data_dir: PathBuf,
    /// Fichier SQL à produire.
    #[arg(long, default_value = "sql/import/db_world/rappelz_world.sql")]
    out: PathBuf,
    /// Applique aussi le SQL dans la base (format AzerothCore « hôte;port;user;mdp;base »).
    #[arg(long)]
    apply: Option<String>,
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        match c {
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\0' => {}
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// Chaîne SQL tronquée à la taille de la colonne AzerothCore (en caractères).
fn esc_max(s: &str, max: usize) -> String {
    esc(&s.chars().take(max).collect::<String>())
}

fn clamp_u8(v: i32) -> i32 {
    v.clamp(0, 255)
}

fn clamp_u16(v: i32) -> i32 {
    v.clamp(0, 65535)
}

/// Accumule des lignes et produit des `INSERT` multi-lignes.
struct Inserts<'a> {
    sql: &'a mut String,
    table: &'static str,
    columns: &'static str,
    rows: Vec<String>,
    total: usize,
}

impl<'a> Inserts<'a> {
    fn new(sql: &'a mut String, table: &'static str, columns: &'static str) -> Self {
        Self { sql, table, columns, rows: Vec::new(), total: 0 }
    }

    fn push(&mut self, values: String) {
        self.rows.push(values);
        self.total += 1;
        if self.rows.len() >= ROWS_PER_INSERT {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let _ = writeln!(
            self.sql,
            "INSERT INTO `{}` ({}) VALUES\n({});",
            self.table,
            self.columns,
            self.rows.join("),\n(")
        );
        self.rows.clear();
    }

    fn finish(mut self) -> usize {
        self.flush();
        self.total
    }
}

/// Bits de race WoW (1 << (race-1)) pour les races Rappelz Gaia (3), Deva (4), Asura (5).
fn race_mask(deva: bool, asura: bool, gaia: bool) -> u32 {
    let mut m = 0;
    if gaia {
        m |= 1 << 2;
    }
    if deva {
        m |= 1 << 3;
    }
    if asura {
        m |= 1 << 4;
    }
    m
}

/// Bits de classe : guerrier, chasseur, mage, invocateur (classes de base Rappelz).
fn class_mask(fighter: bool, hunter: bool, magician: bool, summoner: bool) -> u32 {
    (fighter as u32) | (hunter as u32) << 1 | (magician as u32) << 2 | (summoner as u32) << 3
}

fn import_items(sql: &mut String, items: &NamedDbc, strings: &StringTable) -> usize {
    let _ = writeln!(sql, "\n-- Item.dbc → item_template\nDELETE FROM `item_template`;");
    let mut ins = Inserts::new(
        sql,
        "item_template",
        "`entry`,`class`,`subclass`,`name`,`displayid`,`Quality`,`BuyPrice`,`SellPrice`,`InventoryType`,\
         `AllowableClass`,`AllowableRace`,`ItemLevel`,`RequiredLevel`,`stackable`,`Material`,`MaxDurability`,\
         `description`,`VerifiedBuild`",
    );
    let mut seen = std::collections::HashSet::new();
    for r in items.rows() {
        if !seen.insert(r.id()) {
            continue;
        }
        // Les restrictions Rappelz sont des interdictions : bit à 1 = race/classe exclue.
        let races = race_mask(r.int("limit_deva") == 0, r.int("limit_asura") == 0, r.int("limit_gaia") == 0);
        let classes = class_mask(
            r.int("limit_fighter") == 0,
            r.int("limit_hunter") == 0,
            r.int("limit_magician") == 0,
            r.int("limit_summoner") == 0,
        );
        let name = strings.get(r.int("name_id"));
        let price = r.int("price").max(0);
        ins.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},1,{},{},{},0",
            r.id(),
            clamp_u8(r.int("type")),
            clamp_u8(r.int("group")),
            esc_max(if name.is_empty() { "?" } else { name }, 255),
            r.int("icon_id").max(0),
            clamp_u8(r.int("rank")),
            price,
            price / 10,
            clamp_u8(r.int("wear_type")),
            if classes == 0b1111 { -1i64 } else { classes as i64 },
            if races == 0b11100 { -1i64 } else { races as i64 },
            clamp_u16(r.int("level")),
            clamp_u8(r.int("use_min_level")),
            r.int("material").clamp(-1, 127),
            clamp_u16(r.int("endurance")),
            esc_max(strings.get(r.int("tooltip_id")), 255),
        ));
    }
    ins.finish()
}

fn import_monsters(sql: &mut String, monsters: &NamedDbc, strings: &StringTable) -> usize {
    let _ = writeln!(
        sql,
        "\n-- Monster.dbc → creature_template / creature_template_model\n\
         DELETE FROM `creature_template_model` WHERE `CreatureID` < {NPC_ENTRY_OFFSET};\n\
         DELETE FROM `creature_template` WHERE `entry` < {NPC_ENTRY_OFFSET};"
    );
    let mut models = Vec::new();
    let mut ins = Inserts::new(
        sql,
        "creature_template",
        "`entry`,`name`,`minlevel`,`maxlevel`,`faction`,`speed_walk`,`speed_run`,`detection_range`,`rank`,\
         `lootid`,`mingold`,`maxgold`,`AIName`,`VerifiedBuild`",
    );
    let mut seen = std::collections::HashSet::new();
    for r in monsters.rows() {
        if !seen.insert(r.id()) {
            continue;
        }
        let level = clamp_u8(r.int("level"));
        // Vitesses Rappelz : unité ≈ 1/100 de la vitesse de référence.
        let walk = (r.int("walk_speed").max(1) as f32 / 100.0).max(0.1);
        let run = (r.int("run_speed").max(1) as f32 / 100.0).max(0.1);
        ins.push(format!(
            "{},{},{level},{level},14,{walk:.3},{run:.3},{:.1},{},{},{},{},'',0",
            r.id(),
            esc_max(strings.get(r.int("name_id")), 100),
            r.int("visible_range").max(0) as f32 / 100.0,
            clamp_u8(r.int("monster_type")).min(4),
            r.int("drop_table_link_id").max(0),
            r.int("gold_min").max(0),
            r.int("gold_max").max(0),
        ));
        models.push(format!("{},0,{},{:.3},1,0", r.id(), r.id(), r.float("scale").max(0.01)));
    }
    let n = ins.finish();
    let mut ins = Inserts::new(sql, "creature_template_model", "`CreatureID`,`Idx`,`CreatureDisplayID`,`DisplayScale`,`Probability`,`VerifiedBuild`");
    for m in models {
        ins.push(m);
    }
    ins.finish();
    n
}

fn import_npcs(sql: &mut String, npcs: &NamedDbc, strings: &StringTable) -> (usize, usize) {
    let _ = writeln!(
        sql,
        "\n-- NpcResource.dbc → creature_template (entry + {NPC_ENTRY_OFFSET}) / creature\n\
         DELETE FROM `creature` WHERE `id1` >= {NPC_ENTRY_OFFSET};\n\
         DELETE FROM `creature_template_model` WHERE `CreatureID` >= {NPC_ENTRY_OFFSET};\n\
         DELETE FROM `creature_template` WHERE `entry` >= {NPC_ENTRY_OFFSET};"
    );
    let mut spawns = Vec::new();
    let mut models = Vec::new();
    let mut ins = Inserts::new(
        sql,
        "creature_template",
        "`entry`,`name`,`subname`,`minlevel`,`maxlevel`,`faction`,`npcflag`,`speed_walk`,`speed_run`,`unit_flags`,\
         `AIName`,`ScriptName`,`VerifiedBuild`",
    );
    // Un même PNJ peut figurer plusieurs fois (plusieurs emplacements) : un template par id,
    // un spawn `creature` par ligne, guid séquentiel comme dans AzerothCore.
    let mut seen = std::collections::HashSet::new();
    for (guid, r) in npcs.rows().enumerate() {
        let entry = NPC_ENTRY_OFFSET + r.id() as u32;
        if seen.insert(entry) {
            let level = clamp_u8(r.int("level")).max(1);
            // npcflag 1 = gossip ; non attaquable → UNIT_FLAG_NON_ATTACKABLE (0x2).
            let unit_flags = if r.int("attackable") == 0 { 2 } else { 0 };
            ins.push(format!(
                "{entry},{},{},{level},{level},35,1,1,1.14286,{unit_flags},'',{},0",
                esc_max(strings.get(r.int("name_text_id")), 100),
                esc_max(strings.get(r.int("text_id")), 100),
                esc_max(r.str("contact_script"), 64),
            ));
            models.push(format!("{entry},0,{},1,1,0", entry));
        }
        let orientation = (r.int("face") as f32).to_radians();
        spawns.push(format!(
            "{},{entry},{RAPPELZ_MAP},{},{},{},{orientation:.4},{},0,0,0,'NpcResource {}'",
            guid + 1,
            r.int("x"),
            r.int("y"),
            r.int("z"),
            r.int("regen_time").max(0),
            r.id(),
        ));
    }
    let n = ins.finish();
    let mut ins = Inserts::new(sql, "creature_template_model", "`CreatureID`,`Idx`,`CreatureDisplayID`,`DisplayScale`,`Probability`,`VerifiedBuild`");
    for m in models {
        ins.push(m);
    }
    ins.finish();
    let mut ins = Inserts::new(
        sql,
        "creature",
        "`guid`,`id1`,`map`,`position_x`,`position_y`,`position_z`,`orientation`,`spawntimesecs`,`wander_distance`,\
         `MovementType`,`VerifiedBuild`,`Comment`",
    );
    for s in spawns {
        ins.push(s);
    }
    (n, ins.finish())
}

fn import_quests(sql: &mut String, quests: &NamedDbc, strings: &StringTable) -> usize {
    let _ = writeln!(sql, "\n-- Quest.dbc → quest_template\nDELETE FROM `quest_template`;");
    let mut ins = Inserts::new(
        sql,
        "quest_template",
        "`ID`,`QuestLevel`,`MinLevel`,`RewardMoney`,`TimeAllowed`,`AllowableRaces`,`RewardItem1`,`RewardAmount1`,\
         `RewardChoiceItemID1`,`RewardChoiceItemQuantity1`,`RewardChoiceItemID2`,`RewardChoiceItemQuantity2`,\
         `RewardChoiceItemID3`,`RewardChoiceItemQuantity3`,`RewardChoiceItemID4`,`RewardChoiceItemQuantity4`,\
         `RewardChoiceItemID5`,`RewardChoiceItemQuantity5`,`RewardChoiceItemID6`,`RewardChoiceItemQuantity6`,\
         `LogTitle`,`LogDescription`,`QuestCompletionLog`,`VerifiedBuild`",
    );
    let mut seen = std::collections::HashSet::new();
    for r in quests.rows() {
        if !seen.insert(r.id()) {
            continue;
        }
        let races = race_mask(r.int("limit_deva") != 0, r.int("limit_asura") != 0, r.int("limit_gaia") != 0);
        let mut choices = String::new();
        for i in 1..=6 {
            let _ = write!(
                choices,
                "{},{},",
                r.int(&format!("optional_reward_id{i}")).max(0),
                clamp_u16(r.int(&format!("optional_reward_quantity{i}")))
            );
        }
        ins.push(format!(
            "{},{},{},{},{},{},{},{},{choices}{},{},{},0",
            r.id(),
            r.int("limit_level").clamp(-1, 32767),
            clamp_u8(r.int("limit_level")),
            r.int("gold").max(0),
            r.int("time_limit").max(0),
            if races == 0 || races == 0b11100 { 0 } else { races },
            r.int("default_reward_id").max(0),
            clamp_u16(r.int("default_reward_quantity")),
            esc(strings.get(r.int("text_id_quest"))),
            esc(strings.get(r.int("text_id_summary"))),
            esc(strings.get(r.int("text_id_status"))),
        ));
    }
    ins.finish()
}

fn import_locations(sql: &mut String, locations: &NamedDbc, strings: &StringTable) -> usize {
    let _ = writeln!(sql, "\n-- WorldLocation.dbc → game_tele\nDELETE FROM `game_tele` WHERE `map` = {RAPPELZ_MAP};");
    let mut ins = Inserts::new(sql, "game_tele", "`id`,`position_x`,`position_y`,`position_z`,`orientation`,`map`,`name`");
    let mut seen = std::collections::HashSet::new();
    for r in locations.rows() {
        let name: String = strings.get(r.int("text_id")).chars().filter(|c| !c.is_whitespace()).take(100).collect();
        if name.is_empty() || !seen.insert(name.to_lowercase()) {
            continue;
        }
        ins.push(format!("{},{},{},0,0,{RAPPELZ_MAP},{}", r.id(), r.int("x"), r.int("y"), esc(&name)));
    }
    ins.finish()
}

#[tokio::main]
async fn main() -> Result<()> {
    rz_common::init_logging("info");
    let cli = Cli::parse();
    let dbc = cli.data_dir.join("dbc");
    let strings = StringTable::open(&dbc).context("String.dbc (lancer d'abord « rz-extractor dbc »)")?;
    tracing::info!("{} textes chargés", strings.len());

    let mut sql = String::from(
        "-- Généré par rz-dbimport (RappelzCore) à partir des DBC du client RappelzRust.\n\
         -- Schéma : acore_world d'AzerothCore, non modifié.\n\
         SET NAMES utf8mb4;\nSET FOREIGN_KEY_CHECKS = 0;\n",
    );
    let n = import_items(&mut sql, &NamedDbc::open(&dbc, "item")?, &strings);
    tracing::info!("item_template : {n}");
    let n = import_monsters(&mut sql, &NamedDbc::open(&dbc, "monster")?, &strings);
    tracing::info!("creature_template (monstres) : {n}");
    let (n, s) = import_npcs(&mut sql, &NamedDbc::open(&dbc, "npcresource")?, &strings);
    tracing::info!("creature_template (PNJ) : {n}, creature : {s}");
    let n = import_quests(&mut sql, &NamedDbc::open(&dbc, "quest")?, &strings);
    tracing::info!("quest_template : {n}");
    let n = import_locations(&mut sql, &NamedDbc::open(&dbc, "worldlocation")?, &strings);
    tracing::info!("game_tele : {n}");
    sql.push_str("SET FOREIGN_KEY_CHECKS = 1;\n");

    if let Some(parent) = cli.out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cli.out, &sql)?;
    tracing::info!("{} écrit ({} Kio)", cli.out.display(), sql.len() / 1024);

    if let Some(info) = cli.apply {
        let pool = rz_database::connect(&info, "World", 1).await?;
        let mut tx = pool.begin().await?;
        for stmt in sql.split(";\n").map(str::trim).filter(|s| !s.is_empty() && !s.starts_with("--")) {
            rz_database::sqlx::raw_sql(stmt).execute(&mut *tx).await.with_context(|| {
                format!("requête : {}", stmt.chars().take(120).collect::<String>())
            })?;
        }
        tx.commit().await?;
        tracing::info!("Import appliqué dans la base");
    }
    Ok(())
}
