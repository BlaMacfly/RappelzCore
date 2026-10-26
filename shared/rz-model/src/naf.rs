//! Animations squelettiques `.naf` (gabarit `nx3_bone_ani_header`) et skinning.
//! Voir docs/re/model_format.md §5.
//!
//! Points clés (vérifiés sur les fichiers) :
//! - temps en **ticks 3ds Max : 4800 par seconde** (le client joue à `vitesse × 4.8` par ms,
//!   FUN_00986b10) ;
//! - les clés (position, quaternion `(x,y,z,w)`) de chaque os sont **absolues dans le repère
//!   racine du fichier**, pas relatives au parent : repère modèle pour `*_biped.naf`, repère de
//!   l'os d'attache (`Head`, `Bip01 R Hand`…) pour les parties (`*_hair.naf`, `*_weaponr.naf`…) ;
//! - `base_tm` = inverse de la pose de la première image.

use crate::container::parse_document;
use crate::math::{self, Mat4};
use crate::nx3::{normalize_bone_name, Mesh, Model};
use crate::ModelError;

pub const TICKS_PER_SECOND: f32 = 4800.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosKey {
    pub time: u32,
    pub pos: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotKey {
    pub time: u32,
    /// Quaternion `(x, y, z, w)`.
    pub rot: [f32; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimBone {
    pub name: String,
    /// Index du parent dans [`Animation::bones`] (`parent_Index`, -1 → `None`).
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    /// Inverse de la pose initiale (repère racine → os).
    pub base_tm: Mat4,
    /// `flag` (0x101 partout dans les fichiers observés ; sens inconnu).
    pub flag: u32,
    pub pos_keys: Vec<PosKey>,
    pub rot_keys: Vec<RotKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub name: String,
    pub flag: u32,
    /// Durée en ticks (`channel_time_span`, typiquement multiple de 160 + 1).
    pub time_span: u32,
    /// `channel_frame_rate` (30).
    pub frame_rate: u32,
    /// `bone_count` de l'en-tête (taille du squelette complet, ≥ bones.len()).
    pub bone_count: u32,
    pub bones: Vec<AnimBone>,
}

/// Analyse un `.naf` et renvoie le premier canal (tous les fichiers observés n'en ont qu'un).
pub fn parse_naf(bytes: &[u8]) -> Result<Animation, ModelError> {
    parse_naf_channels(bytes)?
        .into_iter()
        .next()
        .ok_or(ModelError::Missing("nx3_bone_ani_channel"))
}

/// Analyse tous les canaux d'un `.naf`.
pub fn parse_naf_channels(bytes: &[u8]) -> Result<Vec<Animation>, ModelError> {
    let doc = parse_document(bytes)?;
    let h = doc
        .find("nx3_bone_ani_header")
        .ok_or(ModelError::Missing("nx3_bone_ani_header"))?;
    let bone_count = h.u32("bone_count").unwrap_or(0);
    let mut out = Vec::new();
    for ch in h.objs("channel_array") {
        let src = ch.objs("bone_ani_array");
        let mut bones = Vec::with_capacity(src.len());
        for b in src {
            let pt = b.u32s("pos_time_array");
            let pk = b.f32s("pos_key_array");
            let rt = b.u32s("rot_time_array");
            let rk = b.f32s("rot_key_array");
            let parent = b
                .u32("parent_Index")
                .map(|p| p as i32)
                .filter(|&p| p >= 0 && (p as usize) < src.len())
                .map(|p| p as usize);
            bones.push(AnimBone {
                name: b.str("szName").to_string(),
                parent,
                children: b
                    .u32s("child_Index_array")
                    .into_iter()
                    .map(|c| c as usize)
                    .filter(|&c| c < src.len())
                    .collect(),
                base_tm: math::from_slice16(b.f32s("base_tm")),
                flag: b.u32("flag").unwrap_or(0),
                pos_keys: pt
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &t)| {
                        pk.get(i * 3..i * 3 + 3).map(|p| PosKey {
                            time: t,
                            pos: [p[0], p[1], p[2]],
                        })
                    })
                    .collect(),
                rot_keys: rt
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &t)| {
                        rk.get(i * 4..i * 4 + 4).map(|q| RotKey {
                            time: t,
                            rot: [q[0], q[1], q[2], q[3]],
                        })
                    })
                    .collect(),
            });
        }
        out.push(Animation {
            name: ch.str("channel_name").to_string(),
            flag: ch.u32("channel_flag").unwrap_or(0),
            time_span: ch.u32("channel_time_span").unwrap_or(0),
            frame_rate: ch.u32("channel_frame_rate").unwrap_or(30),
            bone_count,
            bones,
        });
    }
    Ok(out)
}

fn seg<T: Copy>(keys: &[T], time: impl Fn(&T) -> u32, t: f32) -> Option<(T, T, f32)> {
    let first = *keys.first()?;
    if keys.len() == 1 || t <= time(&first) as f32 {
        return Some((first, first, 0.0));
    }
    let last = keys[keys.len() - 1];
    if t >= time(&last) as f32 {
        return Some((last, last, 0.0));
    }
    let i = keys.partition_point(|k| (time(k) as f32) <= t).max(1) - 1;
    let (a, b) = (keys[i], keys[i + 1]);
    let span = time(&b).saturating_sub(time(&a)).max(1) as f32;
    Some((a, b, ((t - time(&a) as f32) / span).clamp(0.0, 1.0)))
}

impl Animation {
    /// Durée en secondes.
    pub fn duration(&self) -> f32 {
        self.time_span.saturating_sub(1) as f32 / TICKS_PER_SECOND
    }

    pub fn bone_index(&self, name: &str) -> Option<usize> {
        let n = normalize_bone_name(name);
        self.bones
            .iter()
            .position(|b| normalize_bone_name(&b.name) == n)
    }

    /// Os d'attache d'une animation de partie : racine `Dummy_<partie>_<os>` → `<os>`
    /// (`Dummy_hair_Head` → `Head`, `Dummy_weaponR_Bip01 R Hand` → `Bip01 R Hand`).
    pub fn attach_bone(&self) -> Option<String> {
        let root = &self.bones.iter().find(|b| b.parent.is_none())?.name;
        let rest = root.strip_prefix("Dummy_")?;
        let i = rest.find('_')?;
        Some(rest[i + 1..].to_string())
    }

    /// Matrice de chaque os à `ticks` (non bouclé : bornes répétées), **dans le repère racine
    /// du fichier** (voir l'en-tête du module). Os sans clé : inverse de `base_tm`.
    pub fn sample_ticks(&self, ticks: f32) -> Vec<Mat4> {
        self.bones
            .iter()
            .map(|b| {
                let base = math::inverse(&b.base_tm);
                let pos = match seg(&b.pos_keys, |k| k.time, ticks) {
                    Some((a, c, t)) => math::lerp3(a.pos, c.pos, t),
                    None => [base[3][0], base[3][1], base[3][2]],
                };
                match seg(&b.rot_keys, |k| k.time, ticks) {
                    Some((a, c, t)) => math::from_quat_pos(math::slerp(a.rot, c.rot, t), pos),
                    None => {
                        let mut m = base;
                        m[3] = [pos[0], pos[1], pos[2], 1.0];
                        m
                    }
                }
            })
            .collect()
    }

    /// Comme [`Animation::sample_ticks`], `time_s` en secondes, **bouclé** sur la durée.
    /// Renvoie des matrices **globales** (repère racine du fichier), une par os de `bones`.
    pub fn sample(&self, time_s: f32) -> Vec<[[f32; 4]; 4]> {
        let d = self.duration();
        let t = if d > 0.0 { time_s.rem_euclid(d) } else { 0.0 };
        self.sample_ticks(t * TICKS_PER_SECOND)
    }
}

/// Pose complète d'un personnage : matrices globales (repère modèle) indexées par nom d'os.
#[derive(Debug, Clone, Default)]
pub struct Pose {
    names: Vec<String>,
    pub world: Vec<Mat4>,
}

impl Pose {
    /// `main` = animation `*_biped.naf` (repère modèle) ; `parts` = animations de parties
    /// (cheveux, armes, cape…) attachées par leur os racine (voir [`Animation::attach_bone`]).
    /// `time_s` en secondes, bouclé sur la durée de `main`.
    pub fn from_animations(main: &Animation, parts: &[&Animation], time_s: f32) -> Pose {
        let d = main.duration();
        let t = if d > 0.0 { time_s.rem_euclid(d) } else { 0.0 } * TICKS_PER_SECOND;
        let mut pose = Pose::default();
        for (b, m) in main.bones.iter().zip(main.sample_ticks(t)) {
            pose.names.push(normalize_bone_name(&b.name));
            pose.world.push(m);
        }
        for p in parts {
            let attach = p
                .attach_bone()
                .and_then(|a| pose.get(&a))
                .unwrap_or(math::IDENTITY);
            for (b, m) in p.bones.iter().zip(p.sample_ticks(t)) {
                pose.names.push(normalize_bone_name(&b.name));
                pose.world.push(math::mul(&m, &attach));
            }
        }
        pose
    }

    pub fn get(&self, bone: &str) -> Option<Mat4> {
        let n = normalize_bone_name(bone);
        self.names
            .iter()
            .position(|x| *x == n)
            .map(|i| self.world[i])
    }
}

/// Matrices de skinning d'un modèle pour une pose : `inverse_bind × monde(os)` pour chaque
/// entrée de `model.bones` ; identité si l'os est absent de la pose (sommet laissé en pose de
/// liaison). Un sommet skinné vaut `Σ wᵢ · (v × M[osᵢ])`.
pub fn skinning_matrices_for_pose(model: &Model, pose: &Pose) -> Vec<Mat4> {
    model
        .bones
        .iter()
        .map(|b| match pose.get(&b.name) {
            Some(w) => math::mul(&b.inverse_bind, &w),
            None => math::IDENTITY,
        })
        .collect()
}

/// Raccourci : skinning avec une seule animation (`*_biped.naf` ou animation de monstre).
pub fn skinning_matrices(model: &Model, anim: &Animation, time_s: f32) -> Vec<Mat4> {
    skinning_matrices_for_pose(model, &Pose::from_animations(anim, &[], time_s))
}

/// Applique le skinning (ou, pour un maillage non skinné, la matrice de nœud à `ticks`) et
/// renvoie positions et normales dans le repère modèle.
pub fn deform_mesh(mesh: &Mesh, skin_mats: &[Mat4], ticks: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    match &mesh.skin {
        Some(skin) => {
            let mut pos = Vec::with_capacity(mesh.positions.len());
            let mut nor = Vec::with_capacity(mesh.positions.len());
            for (i, (p, w)) in mesh.positions.iter().zip(skin).enumerate() {
                let n = mesh.normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]);
                let (mut ap, mut an, mut tw) = ([0.0f32; 3], [0.0f32; 3], 0.0f32);
                for &(b, wt) in w.iter().filter(|x| x.1 > 0.0) {
                    let m = skin_mats.get(b as usize).copied().unwrap_or(math::IDENTITY);
                    let q = math::transform_point(*p, &m);
                    let r = math::transform_vector(n, &m);
                    for k in 0..3 {
                        ap[k] += q[k] * wt;
                        an[k] += r[k] * wt;
                    }
                    tw += wt;
                }
                if tw <= 0.0 {
                    ap = *p;
                    an = n;
                }
                pos.push(ap);
                let l = (an[0] * an[0] + an[1] * an[1] + an[2] * an[2])
                    .sqrt()
                    .max(1e-12);
                nor.push(an.map(|c| c / l));
            }
            (pos, nor)
        }
        None => {
            let m = mesh.node_matrix(ticks);
            (
                mesh.positions
                    .iter()
                    .map(|&p| math::transform_point(p, &m))
                    .collect(),
                mesh.normals
                    .iter()
                    .map(|&n| math::transform_vector(n, &m))
                    .collect(),
            )
        }
    }
}
