//! Tests sur les vrais fichiers du client (`Rappelz NoLimits/` à la racine du dépôt) ; retour
//! immédiat si le client est absent. `RZ_MODEL_OBJ_DIR=<dossier>` exporte aussi des `.obj`.

use std::path::Path;

use rz_rappelz::DataArchive;

use crate::*;

fn archive() -> Option<DataArchive> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Rappelz NoLimits");
    if !dir.join("data.000").is_file() {
        return None;
    }
    DataArchive::open(dir).ok()
}

/// Fichiers connus pour être inexploitables, y compris par le client : gabarit absent de
/// nx3.tml (GUID {A35E3135-…}, 9 fichiers d'un autre exporteur) ou fichier tronqué.
fn known_bad(e: &ModelError) -> bool {
    matches!(e, ModelError::UnknownTemplate(g) if g.starts_with("{A35E3135"))
        || matches!(e, ModelError::Truncated(_))
}

fn names(ar: &DataArchive, ext: &str) -> Vec<String> {
    let mut v: Vec<String> = ar
        .entries()
        .iter()
        .map(|e| e.name.to_ascii_lowercase())
        .filter(|n| n.ends_with(ext))
        .collect();
    v.sort();
    v
}

fn check_model(name: &str, m: &Model) {
    for mesh in &m.meshes {
        let nv = mesh.positions.len();
        assert!(
            mesh.indices.iter().all(|&i| (i as usize) < nv),
            "{name}/{} : index",
            mesh.name
        );
        assert!(
            mesh.indices.len() % 3 == 0,
            "{name}/{} : triangles",
            mesh.name
        );
        assert!(
            mesh.positions.iter().flatten().all(|c| c.is_finite()),
            "{name}/{} : positions",
            mesh.name
        );
        assert!(
            mesh.uvs.iter().flatten().all(|c| c.is_finite()),
            "{name}/{} : UV",
            mesh.name
        );
        assert!(
            mesh.uvs.is_empty() || mesh.uvs.len() == nv,
            "{name}/{} : nb UV",
            mesh.name
        );
        assert!(
            mesh.normals.is_empty() || mesh.normals.len() == nv,
            "{name}/{} : nb normales",
            mesh.name
        );
        if let Some(skin) = &mesh.skin {
            assert_eq!(skin.len(), nv);
            for w in skin {
                let s: f32 = w.iter().map(|x| x.1).sum();
                assert!(
                    s == 0.0 || (s - 1.0).abs() < 1e-3,
                    "{name}/{} : somme des poids {s}",
                    mesh.name
                );
                assert!(
                    w.iter().all(|x| (x.0 as usize) < m.bones.len()),
                    "{name} : os hors limites"
                );
            }
        }
    }
}

fn check_anim(name: &str, a: &Animation) {
    for (i, b) in a.bones.iter().enumerate() {
        // Parents valides et sans cycle.
        let mut p = b.parent;
        let mut depth = 0;
        while let Some(q) = p {
            assert!(q < a.bones.len() && q != i, "{name} : parent de {}", b.name);
            p = a.bones[q].parent;
            depth += 1;
            assert!(depth <= a.bones.len(), "{name} : cycle");
        }
        assert!(
            b.pos_keys.windows(2).all(|w| w[0].time <= w[1].time),
            "{name} : temps de position"
        );
        assert!(
            b.rot_keys.windows(2).all(|w| w[0].time <= w[1].time),
            "{name} : temps de rotation"
        );
        for k in &b.rot_keys {
            let n: f32 = k.rot.iter().map(|c| c * c).sum::<f32>().sqrt();
            assert!(
                (n - 1.0).abs() < 0.02,
                "{name}/{} : quaternion de norme {n}",
                b.name
            );
        }
        assert!(b.pos_keys.iter().flat_map(|k| k.pos).all(|c| c.is_finite()));
    }
    let m = a.sample(a.duration() * 0.37);
    assert_eq!(m.len(), a.bones.len());
    assert!(
        m.iter().flatten().flatten().all(|c| c.is_finite()),
        "{name} : échantillon"
    );
}

#[test]
fn real_nx3_files() {
    let Some(mut ar) = archive() else { return };
    let all = names(&ar, ".nx3");
    let mut picked: Vec<String> = all.iter().step_by(37).cloned().collect();
    for n in [
        "asm_body_common001.nx3",
        "1handaxe_ice.nx3",
        "desertwildcat_body.nx3",
        "npc_def_face001.nx3",
        "zone005_prop_rail_02.nx3",
        "asf_newhair_common001.nx3",
        "sky_zone001.nx3",
    ] {
        picked.push(n.to_string());
    }
    let (mut ok, mut bad, mut btrf, mut knib, mut skinned, mut tex_ok, mut tex_all) =
        (0, 0, 0, 0, 0, 0, 0);
    for n in &picked {
        let bytes = ar.read_name(n).unwrap();
        match parse_nx3(&bytes) {
            Ok(m) => {
                check_model(n, &m);
                ok += 1;
                match m.container {
                    Container::Knib => knib += 1,
                    _ => btrf += 1,
                }
                skinned += m.is_skinned() as usize;
                for mesh in m
                    .meshes
                    .iter()
                    .filter(|x| x.kind == MeshKind::Render && !x.texture.is_empty())
                {
                    tex_all += 1;
                    tex_ok += ar.find(&mesh.texture).is_some() as usize;
                }
            }
            Err(e) if known_bad(&e) => bad += 1,
            Err(e) => panic!("{n} : {e}"),
        }
    }
    eprintln!("nx3 : {ok} lus ({btrf} BTRF, {knib} KNIB, {skinned} skinnés), {bad} inexploitables connus ; textures trouvées {tex_ok}/{tex_all}");
    assert!(ok >= 200 && knib > 0 && btrf > 0);
    assert!(tex_ok * 10 >= tex_all * 8, "trop de textures introuvables");
}

#[test]
fn real_naf_files() {
    let Some(mut ar) = archive() else { return };
    let all = names(&ar, ".naf");
    let mut picked: Vec<String> = all.iter().step_by(97).cloned().collect();
    picked.push("asm_bwa_default01_biped.naf".into());
    picked.push("desertwildcat_una_idle01_biped.naf".into());
    let (mut ok, mut multi) = (0, 0);
    for n in &picked {
        let bytes = ar.read_name(n).unwrap();
        let ch = parse_naf_channels(&bytes).unwrap_or_else(|e| panic!("{n} : {e}"));
        multi += (ch.len() > 1) as usize;
        for a in &ch {
            check_anim(n, a);
        }
        ok += 1;
    }
    eprintln!("naf : {ok} lus, {multi} à plusieurs canaux");
    assert!(ok >= 100);
    let a = parse_naf(&ar.read_name("asm_bwa_run_biped.naf").unwrap()).unwrap();
    assert!(
        (a.duration() - 0.8).abs() < 0.01,
        "course asura : 3841 ticks = 0,8 s"
    );
}

#[test]
fn real_cob_and_assembly() {
    let Some(mut ar) = archive() else { return };
    // Objets composés : races, PNJ, monstre, et un échantillon.
    let mut picked: Vec<String> = names(&ar, ".cob").into_iter().step_by(41).collect();
    for n in [
        "asm.cob",
        "asf.cob",
        "dem.cob",
        "def.cob",
        "gam.cob",
        "gaf.cob",
        "npc_def_common004.cob",
        "beast_desert wildcat.cob",
    ] {
        picked.push(n.into());
    }
    let (mut refs, mut found) = (0, 0);
    for n in &picked {
        let bytes = ar.read_name(n).unwrap();
        let cob = match parse_cob(&bytes) {
            Ok(c) => c,
            Err(e) if known_bad(&e) => continue,
            Err(e) => panic!("{n} : {e}"),
        };
        for (_, f) in cob
            .default_meshes()
            .iter()
            .chain(cob.idle_animations().iter())
        {
            refs += 1;
            found += ar.find(f).is_some() as usize;
        }
    }
    eprintln!("cob : {found}/{refs} fichiers référencés présents");
    assert!(found * 10 >= refs * 9);

    let asm = parse_cob(&ar.read_name("asm.cob").unwrap()).unwrap();
    assert_eq!(asm.parts.len(), 11);
    assert_eq!((asm.parts[0].clan, asm.parts[0].sex), (5, 2));

    // Table des modèles par défaut.
    let table = parse_default_model_res(&ar.read_name("db_defaultmodelres.rdb").unwrap()).unwrap();
    assert!(table.iter().any(|r| r.id == 301 && r.part_type == 16));
    for race in 3..=5 {
        for sex in 1..=2 {
            let c = default_character(race, sex).unwrap();
            for f in c
                .meshes
                .iter()
                .chain(c.idle.iter())
                .chain(std::iter::once(&c.cob))
            {
                assert!(ar.find(f).is_some(), "{f} absent");
            }
            let m = character_meshes(&table, race, sex, &[101, 201, 301, 401, 501]);
            // Ordre des ids demandés : cheveux, visage, corps nu, mains, pieds.
            let expected = [1, 0, 2, 3, 4].map(|i| c.meshes[i].clone());
            assert_eq!(m, expected);
        }
    }
    // PNJ et monstre.
    let npc = entity_cob_name("npc_def_common004").unwrap();
    let cob = parse_cob(&ar.read_name(&npc).unwrap()).unwrap();
    assert!(cob
        .default_meshes()
        .iter()
        .any(|(_, f)| f == "def_body_common001.nx3"));
    assert_eq!(cob.idle_animations()[0].1, "def_una_default01_biped.naf");
}

/// Personnage nu skinné avec son attente : proportions plausibles (et export .obj éventuel).
#[test]
fn real_character_pose() {
    let Some(mut ar) = archive() else { return };
    let out = std::env::var_os("RZ_MODEL_OBJ_DIR").map(std::path::PathBuf::from);
    for (race, sex) in [(5, 2), (4, 1), (3, 2)] {
        let c = default_character(race, sex).unwrap();
        let biped = parse_naf(&ar.read_name(&c.idle[0]).unwrap()).unwrap();
        let hair = parse_naf(&ar.read_name(&c.idle[1]).unwrap()).unwrap();
        let pose = Pose::from_animations(&biped, &[&hair], 0.0);
        let mut store = Vec::new();
        let (mut zmin, mut zmax) = (f32::MAX, f32::MIN);
        for f in &c.meshes {
            let m = parse_nx3(&ar.read_name(f).unwrap()).unwrap();
            let mats = skinning_matrices_for_pose(&m, &pose);
            let missing = m
                .bones
                .iter()
                .filter(|b| pose.get(&b.name).is_none())
                .count();
            for mesh in m.meshes.iter().filter(|x| x.kind == MeshKind::Render) {
                let (p, n) = deform_mesh(mesh, &mats, 0.0);
                for v in &p {
                    zmin = zmin.min(v[2]);
                    zmax = zmax.max(v[2]);
                }
                store.push((
                    format!("{f}:{}", mesh.name),
                    mesh.texture.clone(),
                    p,
                    n,
                    mesh.uvs.clone(),
                    mesh.indices.clone(),
                ));
            }
            assert!(
                missing * 2 <= m.bones.len() + 2,
                "{f} : {missing} os absents de la pose"
            );
        }
        eprintln!("{} : hauteur {zmin:.2} .. {zmax:.2}", c.cob);
        assert!(
            zmin > -1.0 && zmin < 2.0 && zmax > 12.0 && zmax < 25.0,
            "{} : {zmin}..{zmax}",
            c.cob
        );
        if let Some(dir) = &out {
            let parts: Vec<obj::ObjPart> = store
                .iter()
                .map(|(n, t, p, nn, uv, ix)| obj::ObjPart {
                    name: n,
                    texture: t,
                    positions: p,
                    normals: nn,
                    uvs: uv,
                    indices: ix,
                })
                .collect();
            std::fs::write(
                dir.join(format!("{}_idle.obj", race_prefix(race, sex).unwrap())),
                obj::write_obj(&parts),
            )
            .unwrap();
        }
    }
    // Monstre : cob → maillages + attente.
    let cob = parse_cob(
        &ar.read_name(&entity_cob_name("beast_desert wildcat").unwrap())
            .unwrap(),
    )
    .unwrap();
    let idle = parse_naf(&ar.read_name(&cob.idle_animations()[0].1).unwrap()).unwrap();
    let mut store = Vec::new();
    let mut zmax = f32::MIN;
    for (_, f) in cob.default_meshes() {
        let m = parse_nx3(&ar.read_name(&f).unwrap()).unwrap();
        let mats = skinning_matrices(&m, &idle, 0.5);
        for mesh in &m.meshes {
            let (p, _) = deform_mesh(mesh, &mats, 0.0);
            zmax = p.iter().fold(zmax, |z, v| z.max(v[2]));
            store.push((
                p,
                mesh.uvs.clone(),
                mesh.indices.clone(),
                mesh.texture.clone(),
            ));
        }
    }
    eprintln!("chat sauvage : hauteur max {zmax:.2}");
    assert!(zmax > 2.0 && zmax < 30.0);
    if let Some(dir) = &out {
        let parts: Vec<obj::ObjPart> = store
            .iter()
            .map(|(p, uv, ix, t)| obj::ObjPart {
                name: "desertwildcat",
                texture: t,
                positions: p,
                normals: &[],
                uvs: uv,
                indices: ix,
            })
            .collect();
        std::fs::write(dir.join("desertwildcat_idle.obj"), obj::write_obj(&parts)).unwrap();
    }
    // Arme : sommets ramenés dans le repère de l'os de la main (liaison inverse de l'os).
    let m = parse_nx3(&ar.read_name("1handaxe_ice.nx3").unwrap()).unwrap();
    let mesh = &m.meshes[0];
    let skin = mesh.skin.as_ref().unwrap();
    let local: Vec<[f32; 3]> = mesh
        .positions
        .iter()
        .zip(skin)
        .map(|(p, w)| math::transform_point(*p, &m.bones[w[0].0 as usize].inverse_bind))
        .collect();
    let len = local
        .iter()
        .map(|v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt())
        .fold(0.0f32, f32::max);
    eprintln!("1handaxe_ice : rayon {len:.2} dans le repère de la main");
    assert!(len > 2.0 && len < 15.0);
    if let Some(dir) = &out {
        let p = obj::ObjPart {
            name: "1handaxe_ice",
            texture: &mesh.texture,
            positions: &local,
            normals: &[],
            uvs: &mesh.uvs,
            indices: &mesh.indices,
        };
        std::fs::write(dir.join("1handaxe_ice.obj"), obj::write_obj(&[p])).unwrap();
    }
}

#[test]
fn winding_is_clockwise() {
    let Some(mut ar) = archive() else { return };
    let m = parse_nx3(&ar.read_name("asm_body_common001.nx3").unwrap()).unwrap();
    let (mut cw, mut ccw) = (0, 0);
    for mesh in &m.meshes {
        for t in mesh.indices.chunks_exact(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| mesh.positions[i as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let vn: Vec<f32> = (0..3)
                .map(|k| t.iter().map(|&i| mesh.normals[i as usize][k]).sum())
                .collect();
            if n[0] * vn[0] + n[1] * vn[1] + n[2] * vn[2] < 0.0 {
                cw += 1;
            } else {
                ccw += 1;
            }
        }
    }
    assert!(cw > ccw * 20, "horaire {cw}, antihoraire {ccw}");
}

/// Balayage complet (lent) : `cargo test -p rz-model --release -- --ignored --nocapture`.
#[test]
#[ignore]
fn all_files() {
    let Some(mut ar) = archive() else { return };
    for ext in [".nx3", ".naf", ".cob"] {
        let (mut ok, mut known, mut other) = (0, 0, Vec::new());
        for n in names(&ar, ext) {
            let b = ar.read_name(&n).unwrap();
            let r = match ext {
                ".nx3" => parse_nx3(&b).map(|m| check_model(&n, &m)),
                ".naf" => parse_naf_channels(&b).map(|v| v.iter().for_each(|a| check_anim(&n, a))),
                _ => parse_cob(&b).map(|_| ()),
            };
            match r {
                Ok(()) => ok += 1,
                Err(e) if known_bad(&e) => known += 1,
                Err(e) => other.push(format!("{n} : {e}")),
            }
        }
        eprintln!(
            "{ext} : {ok} lus, {known} inexploitables connus, autres erreurs {:?}",
            other
        );
    }
}
