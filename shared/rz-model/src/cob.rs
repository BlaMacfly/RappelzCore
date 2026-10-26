//! Objets composés `.cob` (gabarits `cob_header_v10` … `v13`) : ce que charge le client pour
//! une race/sexe de joueur (`asm.cob`…), un PNJ (`NpcResource.model_file`) ou un monstre
//! (`Monster.model`) : maillages par emplacement, animations par partie, boîtes, LOD.
//! Voir docs/re/model_format.md §6.

use crate::container::{parse_document, Obj};
use crate::ModelError;

/// Parties d'animation, dans l'ordre de `obj_partindex` (chaînes `_biped.naf`… du client).
pub const ANIM_PARTS: [&str; 11] = [
    "biped",
    "hair",
    "necklace",
    "earring",
    "mantle",
    "fabric",
    "longskirt",
    "middleskirt",
    "shortskirt",
    "weaponl",
    "weaponr",
];

#[derive(Debug, Clone, PartialEq)]
pub struct MeshSlot {
    /// Emplacement (`mesh_part_num`) : 0 visage, 1 cheveux, 2 corps, 3 pieds, 4 mains…
    pub slot: u32,
    /// Fichiers `.nx3` possibles pour cet emplacement (PNJ/monstres : en général un seul).
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AniEvent {
    pub event_type: u32,
    pub detail: u32,
    /// En ticks.
    pub time: u32,
    pub file: String,
    pub direction: u32,
    pub property: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AniEventList {
    pub animation: String,
    pub events: Vec<AniEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CobPart {
    /// Version du gabarit (10 à 13).
    pub version: u32,
    /// Partie d'animation ([`ANIM_PARTS`]).
    pub part_index: u32,
    /// Race (3 gaïa, 4 déva, 5 asura ; 1 = monstre…).
    pub clan: u32,
    /// Sexe (1 femme, 2 homme, 0 sans).
    pub sex: u32,
    pub render_type: u32,
    pub prop_category: u32,
    pub shadow_type: u32,
    /// Boîtes `[xmin, ymin, zmin, xmax, ymax, zmax]`.
    pub sel_cube: [f32; 6],
    pub dead_cube: [f32; 6],
    pub visible_cube: [f32; 6],
    pub collide_cube: [f32; 6],
    pub effect_points: Vec<(u32, String)>,
    pub lods: Vec<(u32, String)>,
    pub height_file: String,
    pub camera_collision_file: String,
    /// Liste plate de maillages (décors).
    pub meshes: Vec<String>,
    pub mesh_slots: Vec<MeshSlot>,
    /// Animations de cette partie (`*_<partie>.naf`).
    pub animations: Vec<String>,
    pub ani_events: Vec<AniEventList>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cob {
    pub parts: Vec<CobPart>,
}

fn cube(o: &Obj, m: &str) -> [f32; 6] {
    let mut c = [0.0; 6];
    for (d, s) in c.iter_mut().zip(o.f32s(m)) {
        *d = *s;
    }
    c
}

fn clean(s: &str) -> String {
    let t = s.trim().to_ascii_lowercase();
    if t == "empty" {
        String::new()
    } else {
        t
    }
}

pub fn parse_cob(bytes: &[u8]) -> Result<Cob, ModelError> {
    let doc = parse_document(bytes)?;
    let mut parts = Vec::new();
    for o in &doc.objects {
        let version = match o.name() {
            "cob_header_v10" => 10,
            "cob_header_v11" => 11,
            "cob_header_v12" => 12,
            "cob_header_v13" => 13,
            _ => continue,
        };
        parts.push(CobPart {
            version,
            part_index: o.u32("obj_partindex").unwrap_or(0),
            clan: o.u32("obj_clan").unwrap_or(0),
            sex: o.u32("obj_sex").unwrap_or(0),
            render_type: o.u32("render_type").unwrap_or(0),
            prop_category: o.u32("prop_category").unwrap_or(0),
            shadow_type: o.u32("shadow_type").unwrap_or(0),
            sel_cube: cube(o, "sel_cube"),
            dead_cube: cube(o, "dead_cube"),
            visible_cube: cube(o, "visible_cube"),
            collide_cube: cube(o, "collide_cube"),
            effect_points: o
                .objs("effpos_list")
                .iter()
                .map(|e| {
                    (
                        e.u32("nIndex").unwrap_or(0),
                        clean(e.str("effect_pos_name")),
                    )
                })
                .collect(),
            lods: o
                .objs("lod_list")
                .iter()
                .map(|e| (e.u32("nLevel").unwrap_or(0), clean(e.str("lod_name"))))
                .collect(),
            height_file: clean(o.str("height_file_name")),
            camera_collision_file: clean(o.str("camera_collision_file_name")),
            meshes: o
                .strs("mesh_list")
                .iter()
                .map(|s| clean(s))
                .filter(|s| !s.is_empty())
                .collect(),
            mesh_slots: o
                .objs("mesh_part_list")
                .iter()
                .map(|p| MeshSlot {
                    slot: p.u32("mesh_part_num").unwrap_or(0),
                    files: p
                        .strs("mesh_part_list")
                        .iter()
                        .map(|s| clean(s))
                        .filter(|s| !s.is_empty())
                        .collect(),
                })
                .filter(|s| !s.files.is_empty())
                .collect(),
            animations: o
                .strs("ani_list")
                .iter()
                .map(|s| clean(s))
                .filter(|s| !s.is_empty())
                .collect(),
            ani_events: o
                .objs("ani_event_list")
                .iter()
                .map(|l| AniEventList {
                    animation: clean(l.str("ani_name")),
                    events: l
                        .objs("event_list")
                        .iter()
                        .map(|e| AniEvent {
                            event_type: e.u32("event_type").unwrap_or(0),
                            detail: e.u32("event_detail").unwrap_or(0),
                            time: e.u32("event_time").unwrap_or(0),
                            file: clean(e.str("event_file_name")),
                            direction: e.u32("event_direction").unwrap_or(0),
                            property: e.u32("event_property").unwrap_or(0),
                        })
                        .collect(),
                })
                .collect(),
        });
    }
    if parts.is_empty() {
        return Err(ModelError::Missing("cob_header"));
    }
    Ok(Cob { parts })
}

impl Cob {
    pub fn part(&self, index: u32) -> Option<&CobPart> {
        self.parts.iter().find(|p| p.part_index == index)
    }

    /// Maillages à charger pour une entité non joueur : la liste plate puis, pour chaque partie
    /// et chaque emplacement, le **premier** fichier (règle déduite : les cob de PNJ/monstres
    /// n'en listent qu'un par emplacement). Renvoie `(partie, fichier)`.
    pub fn default_meshes(&self) -> Vec<(u32, String)> {
        let mut out = Vec::new();
        for p in &self.parts {
            for m in &p.meshes {
                out.push((p.part_index, m.clone()));
            }
            for s in &p.mesh_slots {
                if let Some(f) = s.files.first() {
                    out.push((p.part_index, f.clone()));
                }
            }
        }
        out
    }

    /// Animation d'attente de chaque partie qui en a : `*_default01_*`, sinon `*_idle01_*`,
    /// sinon la première de la liste. Renvoie `(partie, fichier)`.
    pub fn idle_animations(&self) -> Vec<(u32, String)> {
        self.parts
            .iter()
            .filter_map(|p| pick_idle(&p.animations).map(|a| (p.part_index, a.to_string())))
            .collect()
    }
}

/// Choisit l'animation d'attente dans une liste de `.naf`.
pub fn pick_idle(list: &[String]) -> Option<&str> {
    let by = |k: &str| list.iter().find(|a| a.contains(k)).map(|s| s.as_str());
    by("_una_default01_")
        .or_else(|| by("_default01_"))
        .or_else(|| by("_idle01_"))
        .or_else(|| list.first().map(|s| s.as_str()))
}
