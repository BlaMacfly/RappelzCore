//! Modèles 3D Rappelz (`rz-model`) rendus avec macroquad : décors statiques (`.cob` des cartes)
//! et créatures animées (joueurs, PNJ, monstres) avec skinning sur le processeur.
//!
//! Repère des fichiers : 3ds Max, Z vers le haut, personnages tournés vers −Y, matrices en
//! lignes (`p' = p·M`). Repère de rendu : X = est, Y = haut, Z = nord ; on passe de l'un à
//! l'autre en échangeant Y et Z (`swz`).

use std::collections::HashMap;
use std::sync::Arc;

use macroquad::miniquad::TextureWrap;
use macroquad::models::{Mesh as QMesh, Vertex};
use macroquad::prelude::*;
use rz_model::naf::{deform_mesh, skinning_matrices_for_pose, Animation, Pose};
use rz_model::{Cob, Model};

use crate::assets::Assets;

/// Échange Y et Z (Z-haut des fichiers → Y-haut du rendu).
pub fn swz(p: [f32; 3]) -> Vec3 {
    vec3(p[0], p[2], p[1])
}

/// Éclairage simple : ambiant + lumière directionnelle venant du haut-sud-ouest.
fn shade(n: [f32; 3], tint: [u8; 4]) -> [u8; 4] {
    let l = [-0.35f32, -0.45, 0.82];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
    let d = ((n[0] * l[0] + n[1] * l[1] + n[2] * l[2]) / len).max(0.0);
    let k = 0.55 + 0.55 * d;
    [
        (tint[0] as f32 * k).min(255.0) as u8,
        (tint[1] as f32 * k).min(255.0) as u8,
        (tint[2] as f32 * k).min(255.0) as u8,
        tint[3],
    ]
}

/// Texture en mode répétition (les UV des modèles dépassent souvent [0, 1]).
pub fn set_repeat(tex: &Texture2D) {
    unsafe {
        let gl = get_internal_gl();
        gl.quad_context.texture_set_wrap(tex.raw_miniquad_id(), TextureWrap::Repeat, TextureWrap::Repeat);
    }
}

/// Construit des maillages macroquad (indices u16 : découpage si nécessaire).
fn build_meshes(positions: &[[f32; 3]], normals: &[[f32; 3]], uvs: &[[f32; 2]], indices: &[u32], texture: Option<Texture2D>) -> Vec<QMesh> {
    let mut out = Vec::new();
    for tri_chunk in indices.chunks(3 * 20_000) {
        let mut remap: HashMap<u32, u16> = HashMap::new();
        let mut vertices = Vec::new();
        let mut idx = Vec::with_capacity(tri_chunk.len());
        for &i in tri_chunk {
            let new = *remap.entry(i).or_insert_with(|| {
                let p = positions.get(i as usize).copied().unwrap_or_default();
                let n = normals.get(i as usize).copied().unwrap_or([0.0, 0.0, 1.0]);
                let uv = uvs.get(i as usize).copied().unwrap_or_default();
                vertices.push(Vertex { position: swz(p), uv: vec2(uv[0], uv[1]), color: shade(n, [255; 4]), normal: Vec4::ZERO });
                (vertices.len() - 1) as u16
            });
            idx.push(new);
        }
        out.push(QMesh { vertices, indices: idx, texture: texture.clone() });
    }
    out
}

/// Modèle statique prêt à dessiner (décors).
pub struct StaticModel {
    pub meshes: Vec<QMesh>,
}

/// Une partie skinnée ou animée d'une créature.
struct RigMesh {
    model: Arc<Model>,
    mesh_index: usize,
    gpu: QMesh,
}

/// Créature animée : maillages + animations par partie (biped, cheveux…).
pub struct Rig {
    parts: Vec<RigMesh>,
    /// Animations d'attente et de course : (principale, parties attachées).
    idle: Option<(Arc<Animation>, Vec<Arc<Animation>>)>,
    run: Option<(Arc<Animation>, Vec<Arc<Animation>>)>,
    pub height: f32,
}

impl Rig {
    /// Met à jour les sommets pour l'instant `t` (secondes) et dessine avec `transform`.
    pub fn draw(&mut self, transform: Mat4, t: f32, moving: bool) {
        let anim = if moving { self.run.as_ref().or(self.idle.as_ref()) } else { self.idle.as_ref() };
        let pose = anim.map(|(main, parts)| {
            let refs: Vec<&Animation> = parts.iter().map(|a| a.as_ref()).collect();
            Pose::from_animations(main, &refs, t)
        });
        let ticks = t * rz_model::naf::TICKS_PER_SECOND;
        for part in &mut self.parts {
            let mesh = &part.model.meshes[part.mesh_index];
            let mats = match &pose {
                Some(p) if mesh.skin.is_some() => skinning_matrices_for_pose(&part.model, p),
                _ => Vec::new(),
            };
            let (pos, nor) = deform_mesh(mesh, &mats, ticks);
            for v in part.gpu.vertices.iter_mut() {
                // `normal.w` porte l'index du sommet source (voir `build_rig_mesh`).
                let i = v.normal.w as usize;
                if let Some(p) = pos.get(i) {
                    v.position = swz(*p);
                }
                if let Some(n) = nor.get(i) {
                    v.color = shade(*n, [255; 4]);
                }
            }
        }
        unsafe {
            get_internal_gl().quad_gl.push_model_matrix(transform);
        }
        for part in &self.parts {
            draw_mesh(&part.gpu);
        }
        unsafe {
            get_internal_gl().quad_gl.pop_model_matrix();
        }
    }
}

fn build_rig_mesh(mesh: &rz_model::Mesh, texture: Option<Texture2D>) -> Option<QMesh> {
    if mesh.indices.is_empty() || mesh.positions.len() > u16::MAX as usize {
        return None;
    }
    let vertices = mesh
        .positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let uv = mesh.uvs.get(i).copied().unwrap_or_default();
            Vertex { position: swz(*p), uv: vec2(uv[0], uv[1]), color: [255; 4], normal: vec4(0.0, 0.0, 0.0, i as f32) }
        })
        .collect();
    Some(QMesh { vertices, indices: mesh.indices.iter().map(|&i| i as u16).collect(), texture })
}

/// Description d'une créature à assembler.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RigKey {
    /// Joueur : race, sexe, maillages d'équipement en plus du corps nu.
    Player { race: i32, sex: i32 },
    /// PNJ ou monstre : nom du `.cob` (NpcResource.model_file / Monster.model).
    Entity(String),
}

#[derive(Default)]
pub struct ModelCache {
    nx3: HashMap<String, Option<Arc<Model>>>,
    naf: HashMap<String, Option<Arc<Animation>>>,
    cob: HashMap<String, Option<Arc<Cob>>>,
    statics: HashMap<String, Option<Arc<StaticModel>>>,
    textures: HashMap<String, Option<Texture2D>>,
}

impl ModelCache {
    pub fn nx3(&mut self, assets: &mut Assets, name: &str) -> Option<Arc<Model>> {
        let key = name.to_ascii_lowercase();
        self.nx3
            .entry(key.clone())
            .or_insert_with(|| assets.read(&key).and_then(|b| rz_model::parse_nx3(&b).ok()).map(Arc::new))
            .clone()
    }

    pub fn naf(&mut self, assets: &mut Assets, name: &str) -> Option<Arc<Animation>> {
        let key = name.to_ascii_lowercase();
        self.naf
            .entry(key.clone())
            .or_insert_with(|| assets.read(&key).and_then(|b| rz_model::parse_naf(&b).ok()).map(Arc::new))
            .clone()
    }

    pub fn cob(&mut self, assets: &mut Assets, name: &str) -> Option<Arc<Cob>> {
        let key = name.to_ascii_lowercase();
        self.cob
            .entry(key.clone())
            .or_insert_with(|| assets.read(&key).and_then(|b| rz_model::parse_cob(&b).ok()).map(Arc::new))
            .clone()
    }

    fn texture(&mut self, assets: &mut Assets, name: &str) -> Option<Texture2D> {
        if name.is_empty() {
            return None;
        }
        // Le modèle référence un chemin d'atelier : on garde le nom de base, en .dds d'abord
        // (règle du client, 99 % de résolution), puis les autres extensions.
        let key = name.to_ascii_lowercase();
        if let Some(t) = self.textures.get(&key) {
            return t.clone();
        }
        let base = key.rsplit(['\\', '/']).next().unwrap_or(&key).to_string();
        let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(&base).to_string();
        let tex = assets.texture(&format!("{stem}.dds")).or_else(|| assets.texture(&base));
        if let Some(t) = &tex {
            set_repeat(t);
        }
        self.textures.insert(key, tex.clone());
        tex
    }

    /// Décor : `.cob` (liste de maillages de la 1re partie) ou `.nx3` direct.
    pub fn static_model(&mut self, assets: &mut Assets, name: &str) -> Option<Arc<StaticModel>> {
        let key = name.to_ascii_lowercase();
        if let Some(m) = self.statics.get(&key) {
            return m.clone();
        }
        let files: Vec<String> = if key.ends_with(".cob") {
            self.cob(assets, &key)
                .map(|c| {
                    // Décors : le .nx3 figure dans la liste des « animations » de la partie
                    // (modèle à nœuds animés) ; à défaut, le niveau de détail (lod).
                    c.parts
                        .iter()
                        .flat_map(|p| {
                            let mut f: Vec<String> = p
                                .meshes
                                .iter()
                                .cloned()
                                .chain(p.mesh_slots.iter().filter_map(|s| s.files.first().cloned()))
                                .chain(p.animations.iter().filter(|a| a.ends_with(".nx3")).cloned())
                                .collect();
                            if f.is_empty() {
                                f.extend(p.lods.iter().map(|(_, l)| l.clone()));
                            }
                            f
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![key.clone()]
        };
        let mut meshes = Vec::new();
        for f in files {
            let f = if f.ends_with(".nx3") { f } else { format!("{f}.nx3") };
            let Some(model) = self.nx3(assets, &f) else { continue };
            for mesh in model.meshes.iter().filter(|m| m.kind == rz_model::MeshKind::Render) {
                let (pos, nor) = deform_mesh(mesh, &[], 0.0);
                let tex = self.texture(assets, &mesh.texture);
                meshes.extend(build_meshes(&pos, &nor, &mesh.uvs, &mesh.indices, tex));
            }
        }
        let result = (!meshes.is_empty()).then(|| Arc::new(StaticModel { meshes }));
        self.statics.insert(key, result.clone());
        result
    }

    fn load_anim_set(&mut self, assets: &mut Assets, files: &[String]) -> Option<(Arc<Animation>, Vec<Arc<Animation>>)> {
        let mut loaded = files.iter().filter_map(|f| self.naf(assets, f));
        let main = loaded.next()?;
        Some((main, loaded.collect()))
    }

    /// Assemble une créature animée (nouvelle instance : les sommets sont modifiés à chaque image).
    pub fn rig(&mut self, assets: &mut Assets, key: &RigKey, extra_meshes: &[String]) -> Option<Rig> {
        let (mesh_files, idle_files, run_files) = match key {
            RigKey::Player { race, sex } => {
                let files = rz_model::default_character(*race, *sex)?;
                let mut meshes = files.meshes.clone();
                for extra in extra_meshes {
                    // Une pièce d'armure remplace la partie nue correspondante.
                    for (tag, naked) in [("_body", "body_under"), ("_hand", "hand_under"), ("_foot", "foot_under")] {
                        if extra.contains(tag) {
                            meshes.retain(|m| !m.contains(naked));
                        }
                    }
                    meshes.push(extra.clone());
                }
                let run: Vec<String> = (0..2)
                    .filter_map(|part| rz_model::character_animation(*race, *sex, "una", "run", part))
                    .collect();
                (meshes, files.idle, run)
            }
            RigKey::Entity(cob_name) => {
                let cob = self.cob(assets, cob_name)?;
                let mut meshes = Vec::new();
                let (mut idle, mut run) = (Vec::new(), Vec::new());
                for part in &cob.parts {
                    meshes.extend(part.meshes.iter().cloned());
                    meshes.extend(part.mesh_slots.iter().filter_map(|s| s.files.first().cloned()));
                    if let Some(a) = rz_model::cob::pick_idle(&part.animations) {
                        idle.push(a.to_string());
                    }
                    if let Some(a) = part.animations.iter().find(|a| a.contains("_run")).or_else(|| part.animations.iter().find(|a| a.contains("_walk"))) {
                        run.push(a.clone());
                    }
                }
                (meshes, idle, run)
            }
        };
        let mut parts = Vec::new();
        let mut height = 0.0f32;
        for f in mesh_files {
            let f = if f.ends_with(".nx3") { f } else { format!("{f}.nx3") };
            let Some(model) = self.nx3(assets, &f) else { continue };
            for (i, mesh) in model.meshes.iter().enumerate() {
                if mesh.kind != rz_model::MeshKind::Render {
                    continue;
                }
                for p in &mesh.positions {
                    height = height.max(p[2]);
                }
                let tex = self.texture(assets, &mesh.texture);
                if let Some(gpu) = build_rig_mesh(mesh, tex) {
                    parts.push(RigMesh { model: model.clone(), mesh_index: i, gpu });
                }
            }
        }
        if parts.is_empty() {
            return None;
        }
        let idle = self.load_anim_set(assets, &idle_files);
        let run = self.load_anim_set(assets, &run_files);
        Some(Rig { parts, idle, run, height: height.max(10.0) })
    }
}

/// Matrice de rendu d'un objet placé : position (repère monde Z-haut), rotation (radians,
/// autour de X, Y, Z comme dans 3ds Max) et échelle.
pub fn placement(pos: [f32; 3], rot: [f32; 3], scale: [f32; 3]) -> Mat4 {
    let model = Mat4::from_translation(vec3(pos[0], pos[1], pos[2]))
        * Mat4::from_rotation_z(rot[2])
        * Mat4::from_rotation_y(rot[1])
        * Mat4::from_rotation_x(rot[0])
        * Mat4::from_scale(vec3(scale[0], scale[1], scale[2]));
    let s = Mat4::from_cols(Vec4::X, Vec4::Z, Vec4::Y, Vec4::W);
    s * model * s
}

pub fn draw_static(model: &StaticModel, transform: Mat4) {
    unsafe {
        get_internal_gl().quad_gl.push_model_matrix(transform);
    }
    for m in &model.meshes {
        draw_mesh(m);
    }
    unsafe {
        get_internal_gl().quad_gl.pop_model_matrix();
    }
}

impl ModelCache {
    /// Nombre de décors déjà chargés (pour limiter les chargements par image).
    pub fn static_count(&self) -> usize {
        self.statics.len()
    }

    /// Décor uniquement s'il est déjà en cache.
    pub fn static_model_cached(&self, name: &str) -> Option<Arc<StaticModel>> {
        self.statics.get(&name.to_ascii_lowercase()).cloned().flatten()
    }
}
