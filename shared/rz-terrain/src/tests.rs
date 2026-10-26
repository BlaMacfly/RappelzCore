//! Tests sur les vraies données du client (ignorés si `Rappelz NoLimits/` est absent).

use std::collections::HashMap;
use std::path::PathBuf;

use rz_rappelz::DataArchive;

use crate::*;

fn client_dir() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Rappelz NoLimits");
    dir.join("data.000").is_file().then_some(dir)
}

struct Fixture {
    archive: DataArchive,
    world: SeamlessWorld,
    /// Cartes de `TerrainSeamlessWorld.cfg` présentes dans l'archive, par (x, y).
    maps: HashMap<(u32, u32), NfmMap>,
}

fn name(x: u32, y: u32, ext: &str) -> String {
    format!("m{x:03}_{y:03}.{ext}")
}

fn fixture() -> Option<Fixture> {
    let mut archive = DataArchive::open(client_dir()?).unwrap();
    let world = parse_seamless_world(&archive.read_name("terrainseamlessworld.cfg").unwrap()).unwrap();
    let mut maps = HashMap::new();
    for m in &world.maps {
        let n = name(m.x, m.y, "nfm");
        if archive.find(&n).is_none() {
            continue;
        }
        let map = parse_nfm(&archive.read_name(&n).unwrap()).unwrap_or_else(|e| panic!("{n} : {e}"));
        maps.insert((m.x, m.y), map);
    }
    Some(Fixture { archive, world, maps })
}

#[test]
fn real_world_maps_parse_completely() {
    let Some(mut fx) = fixture() else { return };
    assert_eq!(fx.world.map_extent(), 16128.0);
    assert!(fx.maps.len() >= 100, "{} cartes", fx.maps.len());
    let textures = parse_texture_info(&fx.archive.read_name("terraintextureinfo.cfg").unwrap());
    let props = parse_prop_info(&fx.archive.read_name("terrainpropinfo.cfg").unwrap());
    let (mut n_props, mut named, mut n_tex, mut tex_named, mut n_water) = (0, 0, 0, 0, 0);
    for (&(x, y), map) in fx.maps.iter_mut() {
        let n = name(x, y, "nfm");
        // Sections contiguës, toutes lues jusqu'au dernier octet (sinon `warnings`).
        assert!(map.warnings.is_empty(), "{n} : {:?}", map.warnings);
        // M002_002 (listée dans le .cfg) est encore en version 16.
        assert!(map.header.version == 22 || map.header.version == 16, "{n}");
        assert_eq!((map.size, map.tile_length), (384, 42.0), "{n}");
        assert_eq!(map.heights.len(), 384 * 384);
        assert!(map.heights.iter().all(|h| h.is_finite() && h.abs() < 20000.0), "{n}");
        map.resolve_names(Some(&textures), Some(&props));
        for p in &map.props {
            assert!(p.position[0] >= -300.0 && p.position[0] <= 16500.0, "{n} {p:?}");
            assert!(p.position[1] >= -300.0 && p.position[1] <= 16500.0, "{n} {p:?}");
            assert!(p.scale.iter().all(|s| s.is_finite()) && p.rotation.iter().all(|r| r.is_finite()));
        }
        n_props += map.props.len();
        named += map.props.iter().filter(|p| !p.model.is_empty()).count();
        n_tex += map.texture_layers.len();
        tex_named += map.texture_layers.iter().filter(|t| !t.texture.is_empty()).count();
        n_water += map.water.len();
    }
    assert!(n_props > 50_000, "{n_props} objets");
    assert!(named * 100 >= n_props * 95, "{named}/{n_props} objets nommés");
    assert!(tex_named * 100 >= n_tex * 80, "{tex_named}/{n_tex} textures nommées");
    eprintln!("{} cartes, {n_props} objets ({named} nommés), {tex_named}/{n_tex} textures nommées, {n_water} zones d'eau", fx.maps.len());
    // La section eau du .nfm est vide (4 octets) dans toutes les cartes de ce client : l'eau est dans les .nfw.
    assert_eq!(n_water, 0);
}

#[test]
fn every_nfm_in_archive_parses() {
    let Some(dir) = client_dir() else { return };
    let mut archive = DataArchive::open(dir).unwrap();
    let names: Vec<String> = archive
        .entries()
        .iter()
        .map(|e| e.name.to_ascii_lowercase())
        .filter(|n| n.starts_with('m') && n.ends_with(".nfm") && n.len() == 12)
        .collect();
    assert!(names.len() >= 100);
    let mut v16 = 0;
    for n in &names {
        let map = parse_nfm(&archive.read_name(n).unwrap()).unwrap_or_else(|e| panic!("{n} : {e}"));
        v16 += (map.header.version == 16) as usize;
    }
    assert!(v16 > 0, "aucune carte v16");
}

#[test]
fn sidecar_files_parse_exactly() {
    let Some(mut fx) = fixture() else { return };
    let keys: Vec<(u32, u32)> = fx.maps.keys().copied().collect();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut water = 0;
    for (x, y) in keys {
        for ext in ["nfa", "nfw", "nfe", "nfp", "nfc", "nfl", "qpf", "nfs"] {
            let n = name(x, y, ext);
            if fx.archive.find(&n).is_none() {
                continue;
            }
            let b = fx.archive.read_name(&n).unwrap();
            let r = match ext {
                "nfa" => parse_nfa(&b).map(|_| ()),
                "nfw" => parse_nfw(&b).map(|w| water += w.len()),
                "nfe" => parse_nfe(&b).map(|_| ()),
                "nfp" => parse_nfp(&b).map(|_| ()),
                "nfc" => parse_nfc(&b).map(|_| ()),
                "nfl" => parse_nfl(&b).map(|_| ()),
                "qpf" => parse_qpf(&b).map(|_| ()),
                _ => parse_nfs(&b).map(|_| ()),
            };
            r.unwrap_or_else(|e| panic!("{n} : {e}"));
            *counts.entry(ext).or_default() += 1;
        }
    }
    assert!(water > 50, "{water} zones d'eau");
    for ext in ["nfa", "nfw", "nfc", "qpf", "nfs"] {
        assert!(counts.get(ext).copied().unwrap_or(0) >= 10, "{ext} : {counts:?}");
    }
}

#[test]
fn relief_is_continuous_between_neighbours() {
    let Some(fx) = fixture() else { return };
    // Beaucoup de cartes voisines dans la grille ne se touchent pas en jeu (donjons, îles) :
    // on ne juge que les paires dont le bord opposé diffère nettement (> 100 en moyenne).
    let (mut informative, mut continuous) = (0, 0);
    let mut world = TerrainWorld::new();
    for (&(x, y), a) in &fx.maps {
        for (dx, dy) in [(1, 0), (0, 1)] {
            let Some(b) = fx.maps.get(&(x + dx, y + dy)) else { continue };
            let (mut matched, mut wrong) = (0f64, 0f64);
            for k in 0..384i64 {
                let (ea, eb, opp) = if dx == 1 {
                    (a.vertex_height(383, k), b.vertex_height(0, k), a.vertex_height(0, k))
                } else {
                    (a.vertex_height(k, 383), b.vertex_height(k, 0), a.vertex_height(k, 0))
                };
                matched += (ea - eb).abs() as f64 / 384.0;
                wrong += (opp - eb).abs() as f64 / 384.0;
            }
            if wrong > 100.0 {
                informative += 1;
                continuous += (matched * 3.0 < wrong) as usize;
            }
        }
    }
    // Mesuré : 70 paires continues sur 91 (ex. m009_004/m010_004 : 24 contre 697).
    assert!(informative >= 50, "{informative} paires");
    assert!(continuous * 100 >= informative * 70, "{continuous}/{informative} paires continues");

    // Raccord exact par TerrainWorld : de part et d'autre de x = 16128 entre m009_004 et m010_004.
    for &(x, y) in &[(9u32, 4u32), (10, 4), (9, 5)] {
        if let Some(m) = fx.maps.get(&(x, y)) {
            world.insert(x, y, m.clone());
        }
    }
    if world.maps.len() == 3 {
        for k in 0..50 {
            let wy = 4.0 * 16128.0 + 300.0 * k as f32 + 7.0;
            let l = world.height_at(10.0 * 16128.0 - 0.01, wy).unwrap();
            let r = world.height_at(10.0 * 16128.0 + 0.01, wy).unwrap();
            assert!((l - r).abs() < 0.5, "x=16128·10 y={wy} : {l} / {r}");
            let wx = 9.0 * 16128.0 + 300.0 * k as f32 + 7.0;
            let d = world.height_at(wx, 5.0 * 16128.0 - 0.01).unwrap();
            let u = world.height_at(wx, 5.0 * 16128.0 + 0.01).unwrap();
            assert!((d - u).abs() < 0.5, "x={wx} y=16128·5 : {d} / {u}");
        }
    }
}

#[test]
fn npcs_stand_on_walkable_ground() {
    let Some(fx) = fixture() else { return };
    let dir = client_dir().unwrap().join("Resource/db_npcresource.rdb");
    let Ok(data) = std::fs::read(&dir) else { return };
    let dec = rz_rappelz::rdb::Decoder::new("windows-1252");
    let npcs = rz_rappelz::rdb::parse_table("npcresource", &data, &dec).unwrap();
    let mut world = TerrainWorld::new();
    let (mut total, mut blocked, mut in_poly, mut flipped_in_poly) = (0, 0, 0, 0);
    for row in 0..npcs.len() {
        let (x, y) = (npcs.int(row, "x") as f32, npcs.int(row, "y") as f32);
        let (mx, my) = ((x / 16128.0) as u32, (y / 16128.0) as u32);
        let Some(map) = fx.maps.get(&(mx, my)) else { continue };
        let (lx, ly) = (x - mx as f32 * 16128.0, y - my as f32 * 16128.0);
        total += 1;
        blocked += map.blocked_at(lx, ly) as usize;
        in_poly += map.attr_polygons.iter().any(|p| p.contains(lx, ly, 42.0)) as usize;
        // Contrôle : axe Y retourné.
        flipped_in_poly += map.attr_polygons.iter().any(|p| p.contains(lx, 16128.0 - ly, 42.0)) as usize;
        if world.maps.is_empty() {
            world.insert(mx, my, map.clone());
        }
        if let Some(h) = world.height_at(x, y) {
            assert!(h.is_finite());
        }
    }
    assert!(total > 1000, "{total} PNJ");
    // Mesuré : 11 % sur des sous-cases bloquées, 4 % dans un polygone .nfa (qui couvrent ~70 % des cartes).
    assert!(blocked * 100 < total * 20, "{blocked}/{total} PNJ sur une case bloquée");
    assert!(in_poly * 100 < total * 10, "{in_poly}/{total} PNJ dans un polygone bloquant");
    assert!(flipped_in_poly > in_poly * 5, "axe Y retourné : {flipped_in_poly} contre {in_poly}");
}

#[test]
fn prop_scripts_reference_nfm_props() {
    let Some(mut fx) = fixture() else { return };
    let (mut total, mut ok) = (0, 0);
    let keys: Vec<(u32, u32)> = fx.maps.keys().copied().collect();
    for (x, y) in keys {
        let n = name(x, y, "nfs");
        let Ok(b) = fx.archive.read_name(&n) else { continue };
        let nfs = parse_nfs(&b).unwrap();
        let map = &fx.maps[&(x, y)];
        for s in &nfs.prop_scripts {
            total += 1;
            let found = map.props.iter().any(|p| {
                p.global_id(x, y, fx.world.map_size.0) == s.prop_id
                    && p.prop_num == s.prop_num
                    && (p.position[0] - s.position[0]).abs() < 0.5
                    && (p.position[1] - s.position[1]).abs() < 0.5
            });
            ok += found as usize;
        }
    }
    assert!(total >= 10, "{total} scripts d'objets");
    assert_eq!(ok, total, "scripts d'objets sans objet correspondant");
}
