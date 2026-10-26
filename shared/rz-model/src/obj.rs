//! Export Wavefront `.obj` (contrôle visuel). Repère conservé (Z vers le haut, unités du jeu),
//! V retourné (`vt u -v`) et triangles réordonnés en sens antihoraire, la convention OBJ.

use std::fmt::Write as _;

/// Une géométrie à écrire (positions déjà transformées dans le repère voulu).
pub struct ObjPart<'a> {
    pub name: &'a str,
    pub texture: &'a str,
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub uvs: &'a [[f32; 2]],
    pub indices: &'a [u32],
}

/// Produit le texte `.obj` (groupes `g` et `usemtl <texture>`).
pub fn write_obj(parts: &[ObjPart]) -> String {
    let mut s = String::from("# rz-model : export nx3\n");
    let (mut base, mut tbase, mut nbase) = (1usize, 1usize, 1usize);
    for p in parts {
        let _ = writeln!(s, "g {}", p.name.replace(' ', "_"));
        if !p.texture.is_empty() {
            let _ = writeln!(s, "usemtl {}", p.texture);
        }
        for v in p.positions {
            let _ = writeln!(s, "v {} {} {}", v[0], v[1], v[2]);
        }
        let has_uv = p.uvs.len() == p.positions.len();
        let has_n = p.normals.len() == p.positions.len();
        if has_uv {
            for t in p.uvs {
                let _ = writeln!(s, "vt {} {}", t[0], -t[1]);
            }
        }
        if has_n {
            for n in p.normals {
                let _ = writeln!(s, "vn {} {} {}", n[0], n[1], n[2]);
            }
        }
        for t in p.indices.chunks_exact(3) {
            let f = |i: u32| {
                let (k, t, n) = (base + i as usize, tbase + i as usize, nbase + i as usize);
                match (has_uv, has_n) {
                    (true, true) => format!("{k}/{t}/{n}"),
                    (true, false) => format!("{k}/{t}"),
                    (false, true) => format!("{k}//{n}"),
                    _ => k.to_string(),
                }
            };
            // Fichier : sens horaire → OBJ : antihoraire.
            let _ = writeln!(s, "f {} {} {}", f(t[0]), f(t[2]), f(t[1]));
        }
        base += p.positions.len();
        if has_uv {
            tbase += p.uvs.len();
        }
        if has_n {
            nbase += p.normals.len();
        }
    }
    s
}
