//! Gabarits (« templates ») du conteneur BTRF / KNIB, au format texte `.tml` du moteur nFlavor.
//!
//! Le client lit ses définitions dans l'archive (`nx3.tml`, `nobj.tml`, `pfx.tml`, `sprite.tml`,
//! `qjtv.tml`). La syntaxe est celle des gabarits DirectX `.x` :
//!
//! ```text
//! template nom
//! {GUID}
//! {
//!     type membre;            // dword, float, char, word, string, ou un autre gabarit
//!     type tableau[N];        // tableau de taille fixe
//!     type tableau[membre];   // tableau dont la taille est donnée par un membre dword
//! }
//! ```
//!
//! [`SCHEMA_TML`] reprend les gabarits de `nx3.tml` et `nobj.tml` (modèles, animations, objets
//! composés). [`schema()`] les analyse une fois pour toutes.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Type primitif d'un membre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prim {
    Char,
    Word,
    Dword,
    Float,
    /// Chaîne : dans un fichier BTRF, index 1-based dans la table de chaînes (u32, 0 = vide).
    Str,
    /// `dict` (qjtv.tml) : bloc opaque.
    Dict,
}

impl Prim {
    /// Taille dans un fichier BTRF (`None` = variable).
    pub fn btrf_size(self) -> Option<usize> {
        match self {
            Prim::Char => Some(1),
            Prim::Word => Some(2),
            Prim::Dword | Prim::Float | Prim::Str => Some(4),
            Prim::Dict => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Prim(Prim),
    /// Index du gabarit dans [`Schema::templates`].
    Template(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dim {
    Scalar,
    Fixed(usize),
    /// Taille donnée par un autre membre (index du membre).
    Var(usize),
}

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub ty: Ty,
    pub dim: Dim,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub guid: [u8; 16],
    pub members: Vec<Member>,
    /// Membres réellement stockés dans un fichier BTRF : tous sauf les compteurs de tableaux
    /// (`dword x_size;` suivi de `t a[x_size]`), qui sont implicites.
    pub stored: Vec<usize>,
}

impl Template {
    pub fn member_index(&self, name: &str) -> Option<usize> {
        self.members.iter().position(|m| m.name == name)
    }
}

#[derive(Debug)]
pub struct Schema {
    pub templates: Vec<Template>,
    by_guid: HashMap<[u8; 16], usize>,
    by_name: HashMap<String, usize>,
}

impl Schema {
    pub fn by_guid(&self, g: &[u8; 16]) -> Option<usize> {
        self.by_guid.get(g).copied()
    }
    pub fn by_name(&self, n: &str) -> Option<usize> {
        self.by_name.get(n).copied()
    }
}

/// Valeur d'un chiffre « hexadécimal » telle que la calcule le moteur : `0-9`, `A-F`/`a-f`
/// normalement, toute autre lettre vaut `toupper(c) - '0'` (ex. `G` = 23). Certains GUID de
/// `nobj.tml` contiennent de telles lettres (`{G2E978BB-...}`), et les fichiers contiennent bien
/// la valeur ainsi calculée (vérifié sur `cob_header_v13` et `mesh_texture_data`).
fn hex_digit(c: u8) -> u32 {
    match c {
        b'0'..=b'9' => (c - b'0') as u32,
        b'A'..=b'F' => (c - b'A' + 10) as u32,
        b'a'..=b'f' => (c - b'a' + 10) as u32,
        _ => (c.to_ascii_uppercase() as u32).wrapping_sub(b'0' as u32),
    }
}

fn hex_num(s: &str) -> u32 {
    s.bytes()
        .fold(0u32, |v, c| v.wrapping_mul(16).wrapping_add(hex_digit(c)))
}

/// GUID texte `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` → 16 octets dans l'ordre du fichier
/// (Data1/2/3 little-endian, Data4 octet par octet, comme un GUID Windows).
pub fn parse_guid(s: &str) -> Option<[u8; 16]> {
    let s = s.trim().trim_start_matches('{').trim_end_matches('}');
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 5
        || p[0].len() != 8
        || p[1].len() != 4
        || p[2].len() != 4
        || p[3].len() != 4
        || p[4].len() != 12
    {
        return None;
    }
    let mut g = [0u8; 16];
    g[0..4].copy_from_slice(&hex_num(p[0]).to_le_bytes());
    g[4..6].copy_from_slice(&(hex_num(p[1]) as u16).to_le_bytes());
    g[6..8].copy_from_slice(&(hex_num(p[2]) as u16).to_le_bytes());
    let rest = format!("{}{}", p[3], p[4]);
    for i in 0..8 {
        g[8 + i] = hex_num(&rest[i * 2..i * 2 + 2]) as u8;
    }
    Some(g)
}

/// Analyse un texte `.tml`. Les gabarits référencés avant d'être définis sont acceptés.
pub fn parse_tml(text: &str) -> Result<Schema, String> {
    // Retire les commentaires `//`.
    let clean: String = text
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");
    struct Raw {
        name: String,
        guid: [u8; 16],
        members: Vec<(String, String, Option<String>)>,
    }
    let mut raws = Vec::new();
    let mut rest = clean.as_str();
    while let Some(i) = rest.find("template") {
        rest = &rest[i + "template".len()..];
        let o1 = rest.find('{').ok_or("gabarit sans GUID")?;
        let name = rest[..o1].trim().to_string();
        let c1 = rest[o1..].find('}').ok_or("GUID non fermé")? + o1;
        let guid =
            parse_guid(&rest[o1..=c1]).ok_or_else(|| format!("GUID invalide pour {name}"))?;
        let after = &rest[c1 + 1..];
        let o2 = after.find('{').ok_or("corps absent")?;
        let c2 = after.find('}').ok_or("corps non fermé")?;
        let body = &after[o2 + 1..c2];
        let mut members = Vec::new();
        for decl in body.split(';') {
            let d = decl.trim();
            if d.is_empty() {
                continue;
            }
            let (head, dim) = match d.find('[') {
                Some(b) => (
                    d[..b].trim(),
                    Some(d[b + 1..].trim_end_matches(']').trim().to_string()),
                ),
                None => (d, None),
            };
            let mut it = head.split_whitespace();
            let ty = it.next().ok_or("membre sans type")?.to_string();
            let mn = it
                .next()
                .ok_or_else(|| format!("membre sans nom dans {name}"))?
                .to_string();
            members.push((ty, mn, dim));
        }
        raws.push(Raw {
            name,
            guid,
            members,
        });
        rest = &after[c2 + 1..];
    }
    let by_name: HashMap<String, usize> = raws
        .iter()
        .enumerate()
        .map(|(i, r)| (r.name.clone(), i))
        .collect();
    let mut templates = Vec::new();
    for r in &raws {
        let mut members = Vec::new();
        for (ty, mn, dim) in &r.members {
            let t = match ty.as_str() {
                "char" | "uchar" | "byte" => Ty::Prim(Prim::Char),
                "word" => Ty::Prim(Prim::Word),
                "dword" | "int" => Ty::Prim(Prim::Dword),
                "float" => Ty::Prim(Prim::Float),
                "string" => Ty::Prim(Prim::Str),
                "dict" => Ty::Prim(Prim::Dict),
                other => Ty::Template(
                    *by_name
                        .get(other)
                        .ok_or_else(|| format!("type inconnu {other} dans {}", r.name))?,
                ),
            };
            let d = match dim {
                None => Dim::Scalar,
                Some(s) if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() => {
                    Dim::Fixed(s.parse().unwrap())
                }
                Some(s) => Dim::Var(
                    r.members
                        .iter()
                        .position(|(_, n, _)| n == s)
                        .ok_or_else(|| format!("dimension {s} inconnue dans {}", r.name))?,
                ),
            };
            members.push(Member {
                name: mn.clone(),
                ty: t,
                dim: d,
            });
        }
        let dims: Vec<usize> = members
            .iter()
            .filter_map(|m| {
                if let Dim::Var(i) = m.dim {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        let stored = (0..members.len()).filter(|i| !dims.contains(i)).collect();
        templates.push(Template {
            name: r.name.clone(),
            guid: r.guid,
            members,
            stored,
        });
    }
    let by_guid = templates
        .iter()
        .enumerate()
        .map(|(i, t)| (t.guid, i))
        .collect();
    Ok(Schema {
        templates,
        by_guid,
        by_name,
    })
}

/// Gabarits de `nx3.tml` et `nobj.tml` du client (commentaires retirés).
pub const SCHEMA_TML: &str = r#"
template nx3_version_header {81BCE021-AD76-346f-9C7D-19885FD118B6} { dword version; }
template nx3_mtl_block {81BCE071-AC76-496f-9C7D-19885FD118B6} {
  string mtl_name; string texture_name; dword mtl_id; dword channel_id; float power; float self_illumi;
  char smoothing; dword ambient; dword diffuse; dword specular; }
template nx3_mtl {52BCCAA6-3C16-4286-8B9E-1A798F9D94DE} { dword sub_mtl_block_size; nx3_mtl_block sub_mtl_block_array[sub_mtl_block_size]; }
template nx3_mtl_header {209BBB41-681F-4b9b-9744-4D88E1413DCC} { dword mtl_size; nx3_mtl mtl_array[mtl_size]; }
template old_nx3_mesh_frame {CB3441E8-5740-4c6e-84AC-455C3AC08DCA} {
  dword time_value; dword vertex_size; float vertex_array[vertex_size]; dword normal_size; float normal_array[normal_size];
  dword texel_size; float texel_array[texel_size]; dword color_size; dword color_array[color_size]; }
template nx3_weight_frame {B513DF30-80BE-44f4-980B-84B9D979A607} {
  string bone_name; dword weight_size; float weight_array[weight_size]; dword offset_vector_size; float offset_vector_array[offset_vector_size]; }
template nx3_mesh_frame {1C77954B-CDD5-4615-B7AD-F23BD3D0C23E} {
  dword time_value; dword vertex_size; float vertex_array[vertex_size]; dword normal_size; float normal_array[normal_size];
  dword texel_size; float texel_array[texel_size]; dword color_size; dword color_array[color_size];
  dword bone_size; nx3_weight_frame bone_block[bone_size]; float mesh_tm[16]; }
template nx3_mesh_block {C817F7B0-E4E7-40fb-97B3-2B97CC000521} {
  dword texture_index; dword mesh_frame_size; nx3_mesh_frame mesh_frame_array[mesh_frame_size];
  dword index_buffer_size; word index_buffer_array[index_buffer_size]; dword lightmap_index; }
template nx3_ani_block {630E91D8-A032-47c1-89F3-2BD5938B336E} { dword time; float matrix[12]; }
template nx3_visi_block {AA9BA163-BCD9-4eb8-831A-0B656C304995} { dword time; float visibility; }
template nx3_fx_block {83A78B40-C401-451d-8AE2-D26BEAB73E5A} { dword time; string note; }
template nx3_fx {22C5CCD2-C9FC-4aa8-A63A-6677296313F8} { dword fx_block_size; nx3_fx_block fx_block_array[fx_block_size]; }
template nx3_mesh {0748DC1A-1DB1-458a-9C7E-B3D46FC4585B} {
  string mesh_name; dword material_id; dword channel_id;
  dword mesh_block_size; nx3_mesh_block mesh_block_array[mesh_block_size];
  dword ani_block_size; nx3_ani_block ani_block_array[ani_block_size];
  dword visi_block_size; nx3_visi_block visi_block_array[visi_block_size];
  dword fx_size; nx3_fx fx_array[fx_size];
  dword mesh_children_size; nx3_mesh mesh_children_array[mesh_children_size]; }
template nx3_new_mesh {1718DC1B-1DB1-458a-9C7E-C3D46FC4585B} {
  string mesh_name; dword material_id; dword channel_id;
  dword mesh_block_size; nx3_mesh_block mesh_block_array[mesh_block_size];
  dword ani_time_size; dword ani_time_array[ani_time_size]; dword ani_matrix_size; float ani_matrix_array[ani_matrix_size];
  dword visi_time_size; dword visi_time_array[visi_time_size]; float visi_value_array[visi_time_size];
  dword fx_size; nx3_fx fx_array[fx_size];
  dword mesh_children_size; nx3_new_mesh mesh_children_array[mesh_children_size]; }
template nx3_mesh_tm {F09C560E-7328-411e-87A3-EEB165D5F929} { string name; float tm[16]; }
template old_nx3_mesh_header {D3699024-DB2C-4855-9CB5-FAA4F1CE20D7} { dword mesh_size; nx3_mesh mesh_array[mesh_size]; }
template nx3_mesh_header {D6D25AEB-A735-4fef-A17D-6E2117498226} {
  dword mesh_size; nx3_mesh mesh_array[mesh_size]; dword mesh_tm_size; nx3_mesh_tm mesh_tm_array[mesh_tm_size]; }
template nx3_new_mesh_header {A6D25AEB-A735-1fef-C17D-EE2117498226} {
  dword mesh_size; nx3_new_mesh mesh_array[mesh_size]; dword mesh_tm_size; nx3_mesh_tm mesh_tm_array[mesh_tm_size]; }
template nx3_camera_block {3D80819B-0CB9-4da4-9773-4C23198FEB7F} {
  float mat_object[12]; float mat_target[12]; char has_target; dword time_value; float fov; float dist;
  float hither; float yon; float near_range; float far_range; }
template nx3_camera {55E7B200-BE6C-4719-85E4-DBB4F31654FD} {
  string block_name; dword camera_block_size; nx3_camera_block camera_block_array[camera_block_size]; }
template nx3_light_block {2542DF01-5429-4ae0-8F3E-3A0EC0841DB2} {
  float mat_object[12]; float mat_target[12]; dword time_value; dword clr_light; float intens;
  float atten_start; float atten_end; char onoff; }
template nx3_light {690414EE-85A9-40d3-892B-58596A6A20C4} {
  string block_name; dword light_type; dword ambient_light; dword light_block_size; nx3_light_block light_block_array[light_block_size]; }
template nx3_light_header {445E49A8-A0F9-4a30-9BF2-4D7FC575B87D} { dword light_size; nx3_light light_array[light_size]; }
template nx3_bone_ani {B84344DE-A1B6-4aa7-AFD6-22CA8A4B0CEE} {
  string szName; dword parent_Index; float base_tm[16]; dword flag;
  dword pos_time_size; dword pos_time_array[pos_time_size]; dword pos_key_size; float pos_key_array[pos_key_size];
  dword rot_time_size; dword rot_time_array[rot_time_size]; dword rot_key_size; float rot_key_array[rot_key_size];
  dword child_size; dword child_Index_array[child_size]; }
template nx3_bone_ani_channel {81B3C137-B2B8-40b0-A3AD-D149467F5932} {
  string channel_name; dword channel_flag; dword channel_time_span; dword channel_frame_rate;
  dword bone_ani_size; nx3_bone_ani bone_ani_array[bone_ani_size]; }
template nx3_bone_ani_header {E8F9296B-B9DD-4080-8BC7-6C69E0AA3FEB} {
  dword bone_count; dword channel_count; nx3_bone_ani_channel channel_array[channel_count]; }

template ani_event {1247357D-E63F-4316-AB58-E2BA90F06ABC} {
  dword event_type; dword event_detail; dword event_time; string event_file_name; dword event_direction; dword event_property; }
template ani_event_list {A77F9538-A44C-42b3-9266-01FDA84E6573} {
  string ani_name; dword event_list_size; ani_event event_list[event_list_size]; }
template cob_mesh_part_list {2F46F4A1-9DC3-4fb4-8A9F-6EED47DD2A39} {
  dword mesh_part_num; dword mesh_part_size; string mesh_part_list[mesh_part_size]; }
template cob_header_v10 {5103BB2E-ECBA-44ad-B67E-C69A3F415C65} {
  dword obj_partindex; dword obj_clan; dword obj_sex;
  dword mesh_part_size; cob_mesh_part_list mesh_part_list[mesh_part_size];
  dword ani_list_size; string ani_list[ani_list_size];
  dword ani_event_size; ani_event_list ani_event_list[ani_event_size]; }
template effect_pos {C72669AC-858A-48ea-89CA-2E7BB9D60EE2} { dword nIndex; string effect_pos_name; }
template lod_data {07BD7F6E-1F3B-45a9-BF27-40CAF95132E6} { dword nLevel; string lod_name; }
template cob_header_v11 {7AA3937F-5183-40d2-B6AF-CCBDE5C96E55} {
  dword obj_partindex; dword obj_clan; dword obj_sex; dword render_type; dword prop_category; dword shadow_type;
  float sel_cube[6]; float dead_cube[6]; float visible_cube[6]; float collide_cube[6];
  dword effpos_list_size; effect_pos effpos_list[effpos_list_size];
  dword lod_list_size; lod_data lod_list[lod_list_size];
  string height_file_name;
  dword mesh_list_size; string mesh_list[mesh_list_size];
  dword mesh_part_size; cob_mesh_part_list mesh_part_list[mesh_part_size];
  dword ani_list_size; string ani_list[ani_list_size];
  dword ani_event_size; ani_event_list ani_event_list[ani_event_size]; }
template cob_header_v12 {F1D867AA-07E3-4246-BFB8-6436ED556775} {
  dword obj_partindex; dword obj_clan; dword obj_sex; dword render_type; dword prop_category; dword shadow_type;
  float sel_cube[6]; float dead_cube[6]; float visible_cube[6]; float collide_cube[6];
  dword effpos_list_size; effect_pos effpos_list[effpos_list_size];
  dword lod_list_size; lod_data lod_list[lod_list_size];
  string height_file_name; string camera_collision_file_name;
  dword mesh_list_size; string mesh_list[mesh_list_size];
  dword mesh_part_size; cob_mesh_part_list mesh_part_list[mesh_part_size];
  dword ani_list_size; string ani_list[ani_list_size];
  dword ani_event_size; ani_event_list ani_event_list[ani_event_size]; }
template seq_texture_data {H591W7G6-8EB1-8AX1-8G2K-2WDJ41EH6K73} {
  dword texture_name_list_size; string texture_name_list[texture_name_list_size]; }
template seq_data {A368K8L5-6AD2-2ET3-3Q5J-8SSG14JK1Q45} {
  string str_seq_name; dword texture_group_size; seq_texture_data seq_texture_list[texture_group_size]; }
template nx3_seq_data {B452E9Y1-5WE7-9GN2-9U0I-5DBK38LK3A67} {
  string str_nx3_name; dword ani_part_index; dword seq_size; seq_data seq_list[seq_size]; }
template mesh_texture_data {3B73E1H8-2CD5-6GD0-5Y8O-9SVN77BB4M99} { dword nx3_size; nx3_seq_data nx3_seq_list[nx3_size]; }
template cob_header_v13 {G2E978BB-18F4-5357-CGC9-7547FE667886} {
  dword obj_partindex; dword obj_clan; dword obj_sex; dword render_type; dword prop_category; dword shadow_type;
  float sel_cube[6]; float dead_cube[6]; float visible_cube[6]; float collide_cube[6];
  dword effpos_list_size; effect_pos effpos_list[effpos_list_size];
  dword lod_list_size; lod_data lod_list[lod_list_size];
  string height_file_name; string camera_collision_file_name;
  dword mesh_list_size; string mesh_list[mesh_list_size];
  dword mesh_part_size; cob_mesh_part_list mesh_part_list[mesh_part_size];
  dword ani_list_size; string ani_list[ani_list_size];
  dword ani_event_size; ani_event_list ani_event_list[ani_event_size];
  dword mesh_texture_size; mesh_texture_data mesh_texture_list[mesh_texture_size]; }
"#;

/// Schéma intégré ([`SCHEMA_TML`]), analysé au premier appel.
pub fn schema() -> &'static Schema {
    static S: OnceLock<Schema> = OnceLock::new();
    S.get_or_init(|| parse_tml(SCHEMA_TML).expect("SCHEMA_TML invalide"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guids_match_files() {
        // Octets lus dans 1handaxe_ice.nx3 et beast_desert wildcat.cob.
        let hex = |s: &str| -> [u8; 16] {
            let mut g = [0u8; 16];
            for i in 0..16 {
                g[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
            }
            g
        };
        let s = schema();
        assert_eq!(
            s.by_guid(&hex("21e0bc8176ad6f349c7d19885fd118b6")),
            s.by_name("nx3_version_header")
        );
        assert_eq!(
            s.by_guid(&hex("bb78e972f4185753d7c97547fe667886")),
            s.by_name("cob_header_v13")
        );
        assert_eq!(
            s.by_guid(&hex("88e2733bd52cd077799fb37e77bb5d99")),
            s.by_name("mesh_texture_data")
        );
        let t = &s.templates[s.by_name("nx3_mesh_block").unwrap()];
        let stored: Vec<&str> = t
            .stored
            .iter()
            .map(|&i| t.members[i].name.as_str())
            .collect();
        assert_eq!(
            stored,
            [
                "texture_index",
                "mesh_frame_array",
                "index_buffer_array",
                "lightmap_index"
            ]
        );
    }
}
