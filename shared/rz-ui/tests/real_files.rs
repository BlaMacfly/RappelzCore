//! Tests sur les vrais fichiers du client (ignorés si `Rappelz NoLimits/` est absent).

use rz_rappelz::DataArchive;
use rz_ui::{decode_text, layout, parse_nui, parse_spr_bytes, ControlKind, DrawCmd, SpriteSet, UiState};
use std::collections::HashSet;
use std::path::PathBuf;

fn client_dir() -> Option<PathBuf> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Rappelz NoLimits");
    d.join("data.000").exists().then_some(d)
}

fn names_with(ar: &DataArchive, ext: &str) -> Vec<String> {
    let mut v: Vec<String> = ar
        .entries()
        .iter()
        .map(|e| e.name.clone())
        .filter(|n| n.to_ascii_lowercase().ends_with(ext))
        .collect();
    v.sort();
    v
}

#[test]
fn parse_all_and_resolve() {
    let Some(dir) = client_dir() else {
        eprintln!("client absent : test ignoré");
        return;
    };
    let mut ar = DataArchive::open(&dir).unwrap();
    let all: HashSet<String> = ar.entries().iter().map(|e| e.name.to_ascii_lowercase()).collect();

    // Tous les .spr.
    let sprs = names_with(&ar, ".spr");
    assert_eq!(sprs.len(), 17, "{sprs:?}");
    let (mut anim_count, mut binary, mut frames, mut frames_found) = (0, 0, 0, 0);
    for s in &sprs {
        let bytes = ar.read_name(s).unwrap();
        binary += bytes.starts_with(rz_ui::kbin::MAGIC) as usize;
        let a = parse_spr_bytes(&bytes, s).unwrap_or_else(|e| panic!("{s} : {e}"));
        assert!(!a.is_empty(), "{s} vide");
        anim_count += a.len();
        for f in a.iter().flat_map(|a| a.frames.iter()) {
            frames += 1;
            frames_found += rz_ui::image_candidates(&f.image).iter().any(|c| all.contains(&c.to_ascii_lowercase())) as usize;
        }
    }
    eprintln!(
        ".spr : {} fichiers ({binary} binaires), {anim_count} animations, images présentes {frames_found}/{frames}",
        sprs.len()
    );

    // Tous les .nui.
    let nuis = names_with(&ar, ".nui");
    assert_eq!(nuis.len(), 485);
    let mut docs = Vec::new();
    for n in &nuis {
        let text = decode_text(&ar.read_name(n).unwrap());
        let d = parse_nui(&text).unwrap_or_else(|e| panic!("{n} : {e}"));
        assert!(!d.controls.is_empty(), "{n} vide");
        let unknown: Vec<_> = d.controls.iter().flat_map(|c| c.unknown_bits.iter()).collect();
        assert!(unknown.is_empty(), "{n} : bits inconnus {unknown:?}");
        docs.push((n.clone(), d));
    }
    let others: HashSet<&str> = docs
        .iter()
        .flat_map(|(_, d)| d.controls.iter())
        .filter(|c| c.kind == ControlKind::Other)
        .map(|c| c.type_name.as_str())
        .collect();
    assert!(others.is_empty(), "types non répertoriés : {others:?}");

    // Résolution des animations avec le jeu ui_frame.spr (+ préchargés).
    let mut sprites = SpriteSet::load_ui_set(|f| ar.read_name(f).ok(), "ui_frame.spr");
    let mut lookup = |name: &str| all.contains(&name.to_ascii_lowercase()).then(|| (name.to_string(), 1, 1));
    let (mut t_win, mut ok_win, mut t_old, mut ok_old) = (0, 0, 0, 0);
    let (mut exact, mut by_ext, mut miss_img) = (0, 0, 0);
    let mut drawable_win = 0;
    for (n, d) in &docs {
        let mut st = rz_ui::ResolveStats::default();
        for a in d.animation_refs() {
            let ok = sprites.resolve_animation(&a, &mut lookup, &mut st);
            if n.starts_with("window_") {
                drawable_win += ok as usize;
            }
        }
        let ok = st.animations - st.missing_animations.len();
        if n.starts_with("window_") {
            t_win += st.animations;
            ok_win += ok;
        } else {
            t_old += st.animations;
            ok_old += ok;
        }
        exact += st.exact;
        by_ext += st.by_extension;
        miss_img += st.missing_images.len();
        if n.starts_with("window_") && !(st.missing_animations.is_empty() && st.missing_images.is_empty()) {
            eprintln!("  {n} : animations inconnues {:?}, images absentes {:?}", st.missing_animations, st.missing_images);
        }
    }
    let pct = |a: usize, b: usize| 100.0 * a as f64 / b.max(1) as f64;
    eprintln!(
        "animations référencées (par fichier) : window_* {ok_win}/{t_win} ({:.1} %), anciens .nui {ok_old}/{t_old} ({:.1} %)",
        pct(ok_win, t_win),
        pct(ok_old, t_old)
    );
    eprintln!("  window_* dessinables (toutes images présentes) : {drawable_win} ({:.1} %)", pct(drawable_win, t_win));
    eprintln!("images : {exact} nom exact, {by_ext} autre extension, {miss_img} absentes");
    assert!(pct(ok_win, t_win) > 98.0);

    // Mise en page de la fenêtre de connexion.
    let d = &docs.iter().find(|(n, _)| n == "window_lobby_login.nui").unwrap().1;
    let cmds = layout(d, &sprites, (0, 0), &UiState::default());
    assert!(cmds.iter().filter(|c| matches!(c, DrawCmd::Image(_))).count() >= 27);
    assert!(d.find("accountedit").is_some() && d.find("passwordedit").is_some() && d.find("idok").is_some());
}
