//! Modèles et animations Rappelz (moteur nFlavor) : `.nx3` (modèles), `.naf` (animations),
//! `.cob` (objets composés). Voir docs/re/model_format.md.
//!
//! Conventions :
//! - repère du fichier = 3ds Max : **Z vers le haut**, unités du jeu (≈ 10 cm : un asura fait
//!   ~17 unités) ;
//! - matrices [`math::Mat4`] en lignes, vecteurs lignes (`p' = p · M`, translation en `m[3]`),
//!   comme D3DX ; `mul(a, b)` applique `a` puis `b` ;
//! - triangles en sens **horaire** vus de face ; UV bruts au sens D3D (V souvent négatif, à
//!   échantillonner en WRAP) ;
//! - temps d'animation en ticks (4800/s), [`naf::TICKS_PER_SECOND`].

pub mod assembly;
pub mod cob;
pub mod container;
pub mod math;
pub mod naf;
pub mod nx3;
pub mod obj;
pub mod tml;

pub use assembly::{
    character_animation, character_meshes, default_character, entity_cob_name, item_model_file,
    parse_default_model_res, race_prefix, CharacterFiles, DefaultModelRes,
};
pub use cob::{parse_cob, Cob, CobPart, MeshSlot, ANIM_PARTS};
pub use container::{parse_document, Container, Document, Obj, Value};
pub use math::Mat4;
pub use naf::{
    deform_mesh, parse_naf, parse_naf_channels, skinning_matrices, skinning_matrices_for_pose,
    AnimBone, Animation, Pose, TICKS_PER_SECOND,
};
pub use nx3::{normalize_bone_name, parse_nx3, Bone, Material, Mesh, MeshKind, Model};

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("signature inconnue (ni BTRF ni DIFKNIBK)")]
    BadMagic,
    #[error("fichier tronqué (position 0x{0:x})")]
    Truncated(usize),
    #[error("gabarit inconnu {0}")]
    UnknownTemplate(String),
    #[error("objet {0} absent")]
    Missing(&'static str),
    #[error("données invalides : {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests;
