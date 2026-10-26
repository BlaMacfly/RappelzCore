//! Conversion des bases `.rdb` vers des fichiers WDBC (`DBFilesClient\*.dbc`).
//!
//! Chaque champ RDB devient une colonne DBC de 4 octets, comme dans les DBC de WoW :
//! entiers et bits → `i`, flottants et décimaux ×100 → `f`, chaînes fixes → `s` (UTF-8).
//! La première colonne (`id`/`code`) est l'index `n`. La chaîne de format obtenue est
//! celle qu'utilisent RappelzCore et RappelzRust pour relire le fichier (cf. `DBCfmt.h`).

use rz_dbc::{DbcWriter, Value};

use crate::rdb::{FieldType, RdbTable, RdbValue, StringEntry};

/// Nom de fichier DBC façon WoW pour une table Rappelz.
pub fn dbc_name(table: &str) -> String {
    let known = [
        ("item", "Item"),
        ("item.ref", "ItemResource"),
        ("monster", "Monster"),
        ("skill", "Skill"),
        ("job", "Job"),
        ("npcresource", "NpcResource"),
        ("quest", "Quest"),
        ("questlink", "QuestLink"),
        ("worldlocation", "WorldLocation"),
        ("dropgroup", "DropGroup"),
        ("skilltree", "SkillTree"),
        ("skillstage", "SkillStage"),
        ("skilltreedisplay", "SkillTreeDisplay"),
        ("itemeffectresource", "ItemEffectResource"),
        ("enhanceeffect", "EnhanceEffect"),
        ("fieldpropresource", "FieldPropResource"),
        ("monstermotionset", "MonsterMotionSet"),
        ("monsterspeciesresource", "MonsterSpeciesResource"),
        ("motionfxset", "MotionFxSet"),
        ("soundresource", "SoundResource"),
        ("titleresource", "TitleResource"),
        ("creatureenhance", "CreatureEnhance"),
        ("tenacity", "Tenacity"),
        ("combineres", "CombineRes"),
        ("string", "String"),
    ];
    let base = known
        .iter()
        .find(|(t, _)| *t == table)
        .map(|(_, n)| n.to_string())
        .unwrap_or_else(|| {
            let mut c = table.chars();
            c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
        });
    format!("DBFilesClient\\{base}.dbc")
}

/// Chaîne de format DBC d'une table convertie.
pub fn dbc_format(table: &RdbTable) -> String {
    table
        .layout
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| match f.ty {
            _ if i == 0 => 'n',
            FieldType::F32 | FieldType::F64 | FieldType::Dec2 => 'f',
            FieldType::Char(_) => 's',
            _ => 'i',
        })
        .collect()
}

pub fn table_to_dbc(table: &RdbTable) -> Vec<u8> {
    let format = dbc_format(table);
    let mut w = DbcWriter::new(&format);
    for row in &table.rows {
        let values: Vec<Value> = row
            .iter()
            .zip(format.chars())
            .map(|(v, c)| match (c, v) {
                ('f', v) => Value::F32(v.as_f64() as f32),
                ('s', RdbValue::Str(s)) => Value::Str(s.clone()),
                ('s', _) => Value::Str(String::new()),
                (_, v) => Value::I32(v.as_i64() as i32),
            })
            .collect();
        w.push(&values).expect("format cohérent");
    }
    w.to_bytes()
}

/// `String.dbc` : `code` (n), `group_id` (i), `name` (s), `value` (s).
pub const STRING_DBC_FORMAT: &str = "niss";

pub fn strings_to_dbc(entries: &[StringEntry]) -> Vec<u8> {
    let mut w = DbcWriter::new(STRING_DBC_FORMAT);
    for e in entries {
        w.push(&[
            Value::I32(e.code),
            Value::I32(e.group_id),
            Value::Str(e.name.clone()),
            Value::Str(e.value.clone()),
        ])
        .expect("format String.dbc");
    }
    w.to_bytes()
}

/// `ItemResource.dbc` : index (n) → nom de fichier (s), pour les champs i16 de `Item.dbc`.
pub const ITEM_RESOURCE_DBC_FORMAT: &str = "ns";

pub fn item_ref_to_dbc(names: &[String]) -> Vec<u8> {
    let mut w = DbcWriter::new(ITEM_RESOURCE_DBC_FORMAT);
    for (i, n) in names.iter().enumerate() {
        w.push(&[Value::U32(i as u32), Value::Str(n.clone())]).expect("format ItemResource.dbc");
    }
    w.to_bytes()
}
