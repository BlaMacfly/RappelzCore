//! Diagnostic du terrain : `cargo run --example terrain_probe -- <Data> <x> <y>`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut chain = rz_mpq::MpqChain::open_client_data(&a[1], "frFR").unwrap();
    let (x, y): (f32, f32) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let list = chain.list().unwrap();
    let find = |n: &str| list.iter().find(|f| f.to_ascii_lowercase().ends_with(&n.to_ascii_lowercase())).cloned();
    let (mx, my) = ((x / 16128.0) as u32, (y / 16128.0) as u32);
    let nfm_name = find(&format!("m{mx:03}_{my:03}.nfm")).expect("nfm");
    let tex = find("terraintextureinfo.cfg").and_then(|n| chain.read_file(&n).ok()).map(|b| rz_terrain::parse_texture_info(&b));
    let props = find("terrainpropinfo.cfg").and_then(|n| chain.read_file(&n).ok()).map(|b| rz_terrain::parse_prop_info(&b));
    println!("cfg textures: {:?}, props: {:?}", tex.as_ref().map(|t| t.len()), props.as_ref().map(|p| p.len()));
    let mut map = rz_terrain::parse_nfm(&chain.read_file(&nfm_name).unwrap()).unwrap();
    map.resolve_names(tex.as_ref(), props.as_ref());
    let (lx, ly) = (x - mx as f32 * 16128.0, y - my as f32 * 16128.0);
    println!("hauteur en ({lx}, {ly}) = {}", map.height_at(lx, ly));
    for l in map.texture_layers.iter().take(12) {
        println!("couche {} : {:?} detail {:?} existe={}", l.texture_id, l.texture, l.detail, find(&l.texture).is_some());
    }
    for (dx, dy) in [(0.0, 0.0), (100.0, 0.0), (0.0, 100.0), (300.0, 300.0), (-200.0, 50.0)] {
        println!("({}, {}) textures {:?} poids {:?}", lx + dx, ly + dy, map.textures_at(lx + dx, ly + dy), map.layer_weights_at(lx + dx, ly + dy));
    }
    let near: Vec<_> = map.props.iter().filter(|p| (p.position[0] - lx).abs() < 1500.0 && (p.position[1] - ly).abs() < 1500.0).collect();
    println!("{} décors au total, {} à moins de 1500", map.props.len(), near.len());
    for p in near.iter().take(10) {
        let exists = find(&p.model).is_some();
        let parsed = find(&p.model).and_then(|n| chain.read_file(&n).ok()).and_then(|b| rz_model::parse_cob(&b).ok());
        let meshes: Vec<String> = parsed.map(|c| c.parts.iter().flat_map(|pt| pt.meshes.clone()).collect()).unwrap_or_default();
        println!("  {:?} pos {:?} rot {:?} scale {:?} existe={exists} maillages {:?}", p.model, p.position, p.rotation, p.scale, meshes);
    }
    dump_cob(&mut chain, &list, "de_vil001_building005.cob");
    dump_cob(&mut chain, &list, "npc_dem_common002.cob");
    dump_nx3(&mut chain, &list, "de_vil001_building005.nx3");
    dump_nx3(&mut chain, &list, "def_body_under001.nx3");
    println!("lumière {:?}", map.properties.primary_light);
    println!("couleur sommet centre {:?}", map.vertex_color((lx / 42.0) as i64, (ly / 42.0) as i64));
}

#[allow(dead_code)]
fn dump_cob(chain: &mut rz_mpq::MpqChain, list: &[String], name: &str) {
    let n = list.iter().find(|f| f.to_ascii_lowercase().ends_with(name)).unwrap();
    let c = rz_model::parse_cob(&chain.read_file(n).unwrap()).unwrap();
    println!("{name}: {:#?}", c.parts.iter().map(|p| (&p.meshes, &p.mesh_slots, &p.lods, &p.animations, p.render_type)).collect::<Vec<_>>());
}

#[allow(dead_code)]
pub fn dump_nx3(chain: &mut rz_mpq::MpqChain, list: &[String], name: &str) {
    let n = list.iter().find(|f| f.to_ascii_lowercase().ends_with(name)).unwrap();
    let m = rz_model::parse_nx3(&chain.read_file(n).unwrap()).unwrap();
    println!("{name}: {} maillages, mtl_id {:?}", m.meshes.len(), m.materials.iter().map(|v| v.iter().map(|x| (x.mtl_id, x.channel_id, x.name.clone())).take(4).collect::<Vec<_>>()).collect::<Vec<_>>());
    for mesh in m.meshes.iter().take(6) {
        let (umin, umax) = mesh.uvs.iter().fold((f32::MAX, f32::MIN), |a, uv| (a.0.min(uv[1]), a.1.max(uv[1])));
        let base = mesh.texture.to_ascii_lowercase();
        let base = base.rsplit(['\\', '/']).next().unwrap_or(&base).to_string();
        let stem = base.rsplit_once('.').map(|(s, _)| s.to_string()).unwrap_or(base.clone());
        let found = list.iter().find(|f| f.to_ascii_lowercase().ends_with(&format!("\\{stem}.dds"))).cloned();
        let decoded = found.as_ref().and_then(|f| chain.read_file(f).ok()).map(|b| rz_texture::decode("x.dds", &b).map(|t| (t.width, t.height, t.pixels[..8].to_vec())).map_err(|e| e.to_string()));
        println!("  {:?} kind {:?} mat {} tex_idx {} block {} tex {:?} uv.v [{umin}, {umax}] → {:?} {:?}", mesh.name, mesh.kind, mesh.material_id, mesh.texture_index, mesh.block, mesh.texture, found, decoded.map(|r| r.is_ok()));
    }
}
