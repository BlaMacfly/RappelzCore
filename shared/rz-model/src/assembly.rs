//! Règles d'assemblage des apparences : joueurs (race/sexe + `db_DefaultModelRes.rdb`), PNJ
//! (`NpcResource.model_file`) et monstres (`Monster.model`). Voir docs/re/model_format.md §7.

use crate::cob::ANIM_PARTS;

/// Préfixe de fichiers d'une race/sexe : race 3 gaïa, 4 déva, 5 asura ; sexe 1 femme, 2 homme
/// (valeurs `obj_clan`/`obj_sex` des `.cob` de race, et `LOBBY_CHARACTER_INFO.race/sex`).
pub fn race_prefix(race: i32, sex: i32) -> Option<&'static str> {
    Some(match (race, sex) {
        (3, 2) => "gam",
        (3, 1) => "gaf",
        (4, 2) => "dem",
        (4, 1) => "def",
        (5, 2) => "asm",
        (5, 1) => "asf",
        _ => return None,
    })
}

/// Colonne de `db_DefaultModelRes` pour une race/sexe (ordre du fichier : dem, def, asm, asf,
/// gam, gaf).
pub fn default_model_column(race: i32, sex: i32) -> Option<usize> {
    ["dem", "def", "asm", "asf", "gam", "gaf"]
        .iter()
        .position(|p| Some(*p) == race_prefix(race, sex))
}

/// Une ligne de `db_DefaultModelRes.rdb` (enregistrement de 1552 octets).
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultModelRes {
    pub id: i32,
    /// Type de pièce : 14 cheveux, 15 visage, 16 corps nu, 2 corps, 5 mains, 6 pieds,
    /// 100 visage de PNJ (FUN_00643a30 range 14/15/16 dans des listes dédiées).
    pub part_type: i32,
    /// 1 = défaut, 2 = variante (types 2/5/6).
    pub sub_type: i32,
    /// Noms de modèle (sans `.nx3`) pour dem, def, asm, asf, gam, gaf.
    pub models: [String; 6],
    /// Dernier champ (index croissant, rôle inconnu).
    pub index: i32,
}

impl DefaultModelRes {
    /// Fichier `.nx3` pour une race/sexe.
    pub fn file(&self, race: i32, sex: i32) -> Option<String> {
        let m = &self.models[default_model_column(race, sex)?];
        (!m.is_empty()).then(|| format!("{}.nx3", m.to_ascii_lowercase()))
    }
}

/// Analyse `db_DefaultModelRes.rdb` : en-tête 128 octets, u32 nombre, enregistrements de 1552
/// octets (`i32 id, i32 type, i32 sous-type, char[256] × 6, i32 index`).
pub fn parse_default_model_res(bytes: &[u8]) -> Option<Vec<DefaultModelRes>> {
    const RS: usize = 1552;
    let n = u32::from_le_bytes(bytes.get(0x80..0x84)?.try_into().ok()?) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let r = bytes.get(0x84 + i * RS..0x84 + (i + 1) * RS)?;
        let int = |o: usize| i32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        let s = |k: usize| {
            let f = &r[12 + k * 256..12 + (k + 1) * 256];
            let e = f.iter().position(|&c| c == 0).unwrap_or(256);
            f[..e].iter().map(|&c| c as char).collect::<String>()
        };
        out.push(DefaultModelRes {
            id: int(0),
            part_type: int(4),
            sub_type: int(8),
            models: [s(0), s(1), s(2), s(3), s(4), s(5)],
            index: int(RS - 4),
        });
    }
    Some(out)
}

/// Fichiers d'un personnage joueur.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterFiles {
    /// Objet composé de la race (`asm.cob`…) : listes d'emplacements et d'animations.
    pub cob: String,
    /// Maillages : visage, cheveux, corps, mains, pieds.
    pub meshes: Vec<String>,
    /// Animation d'attente sans arme, une par partie animée (biped en premier).
    pub idle: Vec<String>,
}

/// Parties qui ont une animation pour un personnage nu : squelette et cheveux (les autres
/// parties — cape, jupes, armes — n'existent que si l'équipement les fournit).
const NAKED_PARTS: [&str; 2] = ["biped", "hair"];

/// Personnage nu par défaut d'une race/sexe, d'après `db_DefaultModelRes` (ids 101 cheveux,
/// 201 visage, 301 corps nu, 401 mains, 501 pieds — sous-type 1) et l'attente
/// `<préfixe>_una_default01_<partie>.naf`.
pub fn default_character(race: i32, sex: i32) -> Option<CharacterFiles> {
    let p = race_prefix(race, sex)?;
    Some(CharacterFiles {
        cob: format!("{p}.cob"),
        meshes: [
            "newface_common001",
            "newhair_common001",
            "body_under001",
            "hand_under001",
            "foot_under001",
        ]
        .iter()
        .map(|m| format!("{p}_{m}.nx3"))
        .collect(),
        idle: NAKED_PARTS
            .iter()
            .map(|part| format!("{p}_una_default01_{part}.naf"))
            .collect(),
    })
}

/// Variante qui lit les noms dans la table : ids `model_ids` (ex. `LOBBY_CHARACTER_INFO.model_id`)
/// résolus par `id` ; les ids absents de la table sont ignorés.
pub fn character_meshes(
    table: &[DefaultModelRes],
    race: i32,
    sex: i32,
    model_ids: &[i32],
) -> Vec<String> {
    model_ids
        .iter()
        .filter_map(|id| table.iter().find(|r| r.id == *id))
        .filter_map(|r| r.file(race, sex))
        .collect()
}

/// Modèle d'un objet d'équipement : `db_item.model_00..17` (chaînes de `db_item.ref`) forment
/// 6 colonnes race/sexe (dem, def, asm, asf, gam, gaf) × 3 variantes ; colonne = `col × 3 +
/// variant`. La variante 0 est la valeur courante (les 3 sont identiques pour 99 % des objets ;
/// la règle de choix des variantes 1-2 est inconnue). `"0"` ou vide = aucun modèle.
pub fn item_model_file<S: AsRef<str>>(
    models: &[S],
    race: i32,
    sex: i32,
    variant: usize,
) -> Option<String> {
    let m = models
        .get(default_model_column(race, sex)? * 3 + variant.min(2))?
        .as_ref()
        .trim();
    (!m.is_empty() && m != "0").then(|| format!("{}.nx3", m.to_ascii_lowercase()))
}

/// Animation d'une partie pour une race/sexe, une classe d'arme (`una` mains nues, `swo`, `swt`,
/// `bwa`, `cba`, `dgo`, `sfa`, `spa`, `swa`, `axa`) et un mouvement (`default01`, `run`…).
pub fn character_animation(
    race: i32,
    sex: i32,
    weapon: &str,
    motion: &str,
    part: usize,
) -> Option<String> {
    Some(format!(
        "{}_{weapon}_{motion}_{}.naf",
        race_prefix(race, sex)?,
        ANIM_PARTS.get(part)?
    ))
}

/// Nom du `.cob` d'un PNJ ou d'un monstre : `"%s.cob"` (chaîne `0x00d52c88`) appliqué au champ
/// `NpcResource.model_file` / `Monster.model`, en minuscules ; certaines valeurs contiennent déjà
/// `.cob`.
pub fn entity_cob_name(model: &str) -> Option<String> {
    let m = model.trim().to_ascii_lowercase();
    if m.is_empty() {
        None
    } else if m.ends_with(".cob") {
        Some(m)
    } else {
        Some(format!("{m}.cob"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(race_prefix(5, 2), Some("asm"));
        assert_eq!(default_model_column(3, 1), Some(5));
        assert_eq!(
            entity_cob_name("npc_def_common004").as_deref(),
            Some("npc_def_common004.cob")
        );
        assert_eq!(
            entity_cob_name("npcdem_old.cob").as_deref(),
            Some("npcdem_old.cob")
        );
        assert_eq!(
            character_animation(4, 1, "una", "run", 0).as_deref(),
            Some("def_una_run_biped.naf")
        );
        let cols = [
            "a0", "a1", "a2", "b0", "b1", "b2", "c0", "c1", "c2", "d0", "d1", "d2", "e0", "e1",
            "e2", "f0", "f1", "f2",
        ];
        assert_eq!(item_model_file(&cols, 5, 1, 0).as_deref(), Some("d0.nx3"));
        assert_eq!(item_model_file(&["0"; 18], 5, 1, 0), None);
        let c = default_character(3, 2).unwrap();
        assert_eq!(c.meshes[2], "gam_body_under001.nx3");
        assert_eq!(c.idle[0], "gam_una_default01_biped.naf");
    }
}
