//! Modèles `.nx3` : matériaux, maillages (sous-maillages par bloc), os (matrices de liaison
//! inverses) et poids. Voir docs/re/model_format.md §4.

use crate::container::{parse_document, Container, Obj};
use crate::math::{self, Mat4};
use crate::ModelError;

/// Normalise un nom d'os pour comparer nx3 et naf : minuscules, `' '` et `'@'` → `'_'`
/// (`Bip01 R Hand` ≡ `Bip01_R_Hand`, `@HideBip01` ≡ `_HideBip01`).
pub fn normalize_bone_name(n: &str) -> String {
    n.chars()
        .map(|c| match c {
            ' ' | '@' => '_',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

/// Fichier de texture pour un chemin d'outil : dernier composant, en minuscules, extension
/// remplacée par `.dds` (les nx3 citent souvent `.tga`/`.psd`, l'archive ne contient que le
/// `.dds`) : `H:\Graphic\...\fx01.tga` → `fx01.dds`.
pub fn texture_basename(path: &str) -> String {
    let base = path
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if base.is_empty() {
        return base;
    }
    match base.rfind('.') {
        Some(i) => format!("{}.dds", &base[..i]),
        None => format!("{base}.dds"),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    /// Nom de fichier (minuscules, sans chemin) ; vide si aucun.
    pub texture: String,
    /// Chemin tel qu'écrit par l'exporteur.
    pub texture_path: String,
    pub mtl_id: u32,
    pub channel_id: u32,
    pub power: f32,
    pub self_illumination: f32,
    pub smoothing: u8,
    /// Couleurs ARGB (`0xAARRGGBB`).
    pub ambient: u32,
    pub diffuse: u32,
    pub specular: u32,
}

/// Rôle d'un maillage, d'après le préfixe de son nom (FUN_00a7e5a0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshKind {
    Render,
    /// `ep_*` : point d'effet, non rendu.
    EffectPoint,
    /// `ev_*` : point d'événement, non rendu (le client n'en garde que les premiers sommets).
    EventPoint,
}

/// Clé d'animation de nœud (temps en ticks, 4800/s).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeKey {
    pub time: u32,
    pub matrix: Mat4,
}

/// Image de morphing supplémentaire (`mesh_frame_array[1..]`).
#[derive(Debug, Clone, PartialEq)]
pub struct MorphFrame {
    pub time: u32,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

/// Un sous-maillage : un bloc (`nx3_mesh_block`) d'un nœud (`nx3_mesh` / `nx3_new_mesh`).
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub kind: MeshKind,
    /// Index du nœud (parcours en profondeur de l'arbre de maillages).
    pub node: usize,
    /// Nœud parent (`mesh_children_array`).
    pub parent_node: Option<usize>,
    pub block: usize,
    pub material_id: u32,
    pub texture_index: u32,
    pub lightmap_index: u32,
    /// Texture résolue : `materials[material_id][texture_index].texture` (indices bornés).
    pub texture: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// UV bruts, convention D3D (origine en haut à gauche). Les V sont souvent négatifs : le
    /// client échantillonne en mode WRAP. Pour OpenGL (origine en bas) : `(u, -v)`.
    pub uvs: Vec<[f32; 2]>,
    /// Second jeu d'UV (lightmap) quand `texel_array` contient 4 flottants par sommet.
    pub uvs2: Option<Vec<[f32; 2]>>,
    pub colors: Vec<u32>,
    /// Triangles (liste), sommets dans le sens **horaire** vus de face dans le repère du fichier.
    pub indices: Vec<u32>,
    /// Jusqu'à 4 influences par sommet `(index dans Model::bones, poids)`, poids normalisés.
    pub skin: Option<Vec<[(u16, f32); 4]>>,
    /// `mesh_tm` de la première image.
    pub transform: Mat4,
    /// Animation du nœud (`ani_time_array`/`ani_matrix_array`, ou `ani_block_array`).
    pub node_keys: Vec<NodeKey>,
    /// Visibilité animée `(temps, valeur)`.
    pub visibility: Vec<(u32, f32)>,
    /// Notes d'effets `(temps, texte)` (`fx_array`).
    pub fx_notes: Vec<(u32, String)>,
    pub morph_frames: Vec<MorphFrame>,
}

impl Mesh {
    /// Matrice du nœud à l'instant `ticks` : clés interpolées, sinon `transform`. Les sommets
    /// d'un maillage non skinné sont dans le repère local du nœud.
    pub fn node_matrix(&self, ticks: f32) -> Mat4 {
        let k = &self.node_keys;
        if k.is_empty() {
            return self.transform;
        }
        if k.len() == 1 || ticks <= k[0].time as f32 {
            return k[0].matrix;
        }
        let last = k[k.len() - 1];
        if ticks >= last.time as f32 {
            return last.matrix;
        }
        let i = k.partition_point(|x| (x.time as f32) <= ticks).max(1) - 1;
        let (a, b) = (k[i], k[i + 1]);
        let span = (b.time - a.time).max(1) as f32;
        math::lerp_mat(&a.matrix, &b.matrix, (ticks - a.time as f32) / span)
    }
}

/// Os d'un modèle (une entrée de `mesh_tm_array`).
#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    pub name: String,
    /// Le nx3 ne contient pas de hiérarchie : toujours `None` ici (elle est dans le .naf,
    /// voir [`crate::naf::AnimBone::parent`]).
    pub parent: Option<usize>,
    /// Matrice de liaison inverse (modèle → os), telle que stockée.
    pub inverse_bind: Mat4,
    /// Matrice de liaison (os → modèle) = inverse de la précédente.
    pub bind: Mat4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub container: Container,
    /// `nx3_version_header.version` (0x10000 = 1.0) si présent.
    pub version: Option<u32>,
    /// `materials[material_id][texture_index]`.
    pub materials: Vec<Vec<Material>>,
    pub meshes: Vec<Mesh>,
    pub bones: Vec<Bone>,
    pub has_camera: bool,
    pub has_light: bool,
}

impl Model {
    pub fn bone_index(&self, name: &str) -> Option<usize> {
        let n = normalize_bone_name(name);
        self.bones
            .iter()
            .position(|b| normalize_bone_name(&b.name) == n)
    }
    pub fn is_skinned(&self) -> bool {
        self.meshes.iter().any(|m| m.skin.is_some())
    }
}

fn v3(s: &[f32]) -> Vec<[f32; 3]> {
    s.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}
fn v2(s: &[f32]) -> Vec<[f32; 2]> {
    s.chunks_exact(2).map(|c| [c[0], c[1]]).collect()
}

/// Analyse un `.nx3` (BTRF ou KNIB).
pub fn parse_nx3(bytes: &[u8]) -> Result<Model, ModelError> {
    let doc = parse_document(bytes)?;
    let mut model = Model {
        container: doc.container,
        version: doc
            .find("nx3_version_header")
            .and_then(|o| o.u32("version")),
        materials: Vec::new(),
        meshes: Vec::new(),
        bones: Vec::new(),
        has_camera: doc.find("nx3_camera").is_some(),
        has_light: doc.find("nx3_light_header").is_some(),
    };
    if let Some(mh) = doc.find("nx3_mtl_header") {
        for m in mh.objs("mtl_array") {
            model.materials.push(
                m.objs("sub_mtl_block_array")
                    .iter()
                    .map(|s| Material {
                        name: s.str("mtl_name").to_string(),
                        texture: texture_basename(s.str("texture_name")),
                        texture_path: s.str("texture_name").to_string(),
                        mtl_id: s.u32("mtl_id").unwrap_or(0),
                        channel_id: s.u32("channel_id").unwrap_or(0),
                        power: s.f32("power").unwrap_or(0.0),
                        self_illumination: s.f32("self_illumi").unwrap_or(0.0),
                        smoothing: s.u32("smoothing").unwrap_or(0) as u8,
                        ambient: s.u32("ambient").unwrap_or(0),
                        diffuse: s.u32("diffuse").unwrap_or(0),
                        specular: s.u32("specular").unwrap_or(0),
                    })
                    .collect(),
            );
        }
    }
    let header = doc
        .find("nx3_new_mesh_header")
        .or_else(|| doc.find("nx3_mesh_header"))
        .or_else(|| doc.find("old_nx3_mesh_header"));
    if let Some(h) = header {
        for t in h.objs("mesh_tm_array") {
            let inv = math::from_slice16(t.f32s("tm"));
            model.bones.push(Bone {
                name: t.str("name").to_string(),
                parent: None,
                inverse_bind: inv,
                bind: math::inverse(&inv),
            });
        }
        let mut node = 0usize;
        for m in h.objs("mesh_array") {
            add_node(&mut model, m, None, &mut node)?;
        }
    }
    Ok(model)
}

fn add_node(
    model: &mut Model,
    m: &Obj,
    parent: Option<usize>,
    counter: &mut usize,
) -> Result<(), ModelError> {
    let node = *counter;
    *counter += 1;
    let name = m.str("mesh_name").to_string();
    let kind = if name.starts_with("ep_") {
        MeshKind::EffectPoint
    } else if name.starts_with("ev_") {
        MeshKind::EventPoint
    } else {
        MeshKind::Render
    };
    let material_id = m.u32("material_id").unwrap_or(0);
    // Animation de nœud : nx3_new_mesh (tableaux séparés) ou nx3_mesh (blocs).
    let mut node_keys = Vec::new();
    let times = m.u32s("ani_time_array");
    let mats = m.f32s("ani_matrix_array");
    for (i, &t) in times.iter().enumerate() {
        if let Some(s) = mats.get(i * 12..i * 12 + 12) {
            node_keys.push(NodeKey {
                time: t,
                matrix: math::from_slice12(s),
            });
        }
    }
    for b in m.objs("ani_block_array") {
        node_keys.push(NodeKey {
            time: b.u32("time").unwrap_or(0),
            matrix: math::from_slice12(b.f32s("matrix")),
        });
    }
    let mut visibility: Vec<(u32, f32)> = m
        .u32s("visi_time_array")
        .into_iter()
        .zip(m.f32s("visi_value_array").iter().copied())
        .collect();
    for b in m.objs("visi_block_array") {
        visibility.push((
            b.u32("time").unwrap_or(0),
            b.f32("visibility").unwrap_or(1.0),
        ));
    }
    let mut fx_notes = Vec::new();
    for fx in m.objs("fx_array") {
        for b in fx.objs("fx_block_array") {
            fx_notes.push((b.u32("time").unwrap_or(0), b.str("note").to_string()));
        }
    }

    for (bi, blk) in m.objs("mesh_block_array").iter().enumerate() {
        let texture_index = blk.u32("texture_index").unwrap_or(0);
        let frames = blk.objs("mesh_frame_array");
        let Some(f0) = frames.first() else { continue };
        let positions = v3(f0.f32s("vertex_array"));
        let nv = positions.len();
        let normals = v3(f0.f32s("normal_array"));
        let tex = f0.f32s("texel_array");
        let (uvs, uvs2) = if tex.len() >= nv * 4 && nv > 0 {
            (v2(&tex[..nv * 2]), Some(v2(&tex[nv * 2..nv * 4])))
        } else {
            (v2(tex), None)
        };
        let mut indices: Vec<u32> = blk.u32s("index_buffer_array");
        // Quelques fichiers ont un reste (ex. asm_cloak_nephilim_v3.nx3) : le client dessine
        // nombre/3 triangles, le reste est ignoré.
        indices.truncate(indices.len() / 3 * 3);
        if let Some(&bad) = indices.iter().find(|&&i| i as usize >= nv) {
            return Err(ModelError::Invalid(format!(
                "{name} : index {bad} ≥ {nv} sommets"
            )));
        }
        let skin = build_skin(model, f0, nv);
        // `material_id` désigne le `mtl_id` d'un groupe de matériaux (296, 34…), pas sa
        // position ; repli sur la position pour les fichiers où les deux coïncident.
        let texture = model
            .materials
            .iter()
            .find(|g| g.first().is_some_and(|m| m.mtl_id == material_id))
            .or_else(|| model.materials.get(material_id as usize))
            .and_then(|g| g.get(texture_index as usize).or_else(|| g.first()))
            .map(|mt| mt.texture.clone())
            .unwrap_or_default();
        let morph_frames = frames[1..]
            .iter()
            .map(|f| MorphFrame {
                time: f.u32("time_value").unwrap_or(0),
                positions: v3(f.f32s("vertex_array")),
                normals: v3(f.f32s("normal_array")),
            })
            .collect();
        model.meshes.push(Mesh {
            name: name.clone(),
            kind,
            node,
            parent_node: parent,
            block: bi,
            material_id,
            texture_index,
            lightmap_index: blk.u32("lightmap_index").unwrap_or(0),
            texture,
            positions,
            normals,
            uvs,
            uvs2,
            colors: f0.u32s("color_array"),
            indices,
            skin,
            transform: math::from_slice16(f0.f32s("mesh_tm")),
            node_keys: node_keys.clone(),
            visibility: visibility.clone(),
            fx_notes: fx_notes.clone(),
            morph_frames,
        });
    }
    for c in m.objs("mesh_children_array") {
        add_node(model, c, Some(node), counter)?;
    }
    Ok(())
}

/// Poids : un `nx3_weight_frame` par os (position = index d'os), `weight_array` = paires
/// (index de sommet en flottant, poids), `offset_vector_array` = position du sommet dans le
/// repère de l'os (= sommet × liaison inverse, vérifié).
fn build_skin(model: &mut Model, frame: &Obj, nv: usize) -> Option<Vec<[(u16, f32); 4]>> {
    let blocks = frame.objs("bone_block");
    let mut infl: Vec<Vec<(u16, f32)>> = vec![Vec::new(); nv];
    let mut any = false;
    for (pos, bb) in blocks.iter().enumerate() {
        let w = bb.f32s("weight_array");
        if w.is_empty() {
            continue;
        }
        let bname = bb.str("bone_name");
        let nb = normalize_bone_name(bname);
        let bi = match model.bones.get(pos) {
            Some(b) if normalize_bone_name(&b.name) == nb => pos,
            _ => match model
                .bones
                .iter()
                .position(|b| normalize_bone_name(&b.name) == nb)
            {
                Some(i) => i,
                None => {
                    // Os sans matrice de liaison : on le crée avec l'identité.
                    model.bones.push(Bone {
                        name: bname.to_string(),
                        parent: None,
                        inverse_bind: math::IDENTITY,
                        bind: math::IDENTITY,
                    });
                    model.bones.len() - 1
                }
            },
        };
        for p in w.chunks_exact(2) {
            let vi = p[0] as usize;
            if vi < nv && p[1] > 0.0 {
                infl[vi].push((bi as u16, p[1]));
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    Some(
        infl.into_iter()
            .map(|mut v| {
                v.sort_by(|a, b| b.1.total_cmp(&a.1));
                v.truncate(4);
                let s: f32 = v.iter().map(|x| x.1).sum();
                let mut out = [(0u16, 0.0f32); 4];
                for (o, x) in out.iter_mut().zip(v.iter()) {
                    *o = (x.0, if s > 0.0 { x.1 / s } else { 0.0 });
                }
                out
            })
            .collect(),
    )
}
