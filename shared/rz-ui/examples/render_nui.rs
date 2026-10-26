//! Rend une ou plusieurs fenêtres `.nui` du client Rappelz dans un PNG (vérification visuelle
//! des règles de dessin de `rz-ui`).
//!
//! ```text
//! cargo run -p rz-ui --release --example render_nui -- "Rappelz NoLimits" sortie.png \
//!     [--canvas 1024x768] [--bg 30,30,40 | --bg-image ui_loading01.tga] [--state hover]
//!     [--hover id] [--press id] [--disable id] [--check id] [--hide id,préfixe*,…]
//!     [--gauge id=0.5] [--set id=texte] [--codepage windows-1252] [--list] [--zoom 2]
//!     window_lobby_login.nui[@x,y] [autre.nui@x,y …]
//! ```
//! Sans `@x,y`, la fenêtre est centrée. Le texte est rendu avec `C:\Windows\Fonts\tahoma.ttf`
//! (police `FONT_DEFAULT` de `localeinfo.ini`) ; les `<$id>` viennent de `db_string`.

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use rz_rappelz::DataArchive;
use rz_ui::{layout, parse_nui, ControlState, DrawCmd, HAlign, NuiDocument, SpriteSet, TextRun, UiState, VAlign};
use std::collections::HashMap;

type Canvas = image::RgbaImage;

struct Textures {
    ar: DataArchive,
    cache: HashMap<String, Option<rz_texture::Rgba>>,
}

impl Textures {
    fn get(&mut self, name: &str) -> Option<&rz_texture::Rgba> {
        let key = name.to_ascii_lowercase();
        if !self.cache.contains_key(&key) {
            let t = self.ar.read_name(name).ok().and_then(|b| rz_texture::decode(name, &b).ok());
            self.cache.insert(key.clone(), t);
        }
        self.cache.get(&key).and_then(|t| t.as_ref())
    }
}

fn blend(c: &mut Canvas, x: i32, y: i32, rgba: [u8; 4]) {
    if x < 0 || y < 0 || x >= c.width() as i32 || y >= c.height() as i32 || rgba[3] == 0 {
        return;
    }
    let p = c.get_pixel_mut(x as u32, y as u32);
    let a = rgba[3] as u32;
    for i in 0..3 {
        p.0[i] = ((rgba[i] as u32 * a + p.0[i] as u32 * (255 - a)) / 255) as u8;
    }
    p.0[3] = (a + p.0[3] as u32 * (255 - a) / 255).min(255) as u8;
}

fn draw_image(c: &mut Canvas, t: &rz_texture::Rgba, src: rz_ui::IRect, dst: rz_ui::IRect) {
    for dy in 0..dst.h {
        let sy = src.y + (dy as i64 * src.h as i64 / dst.h as i64) as i32;
        for dx in 0..dst.w {
            let sx = src.x + (dx as i64 * src.w as i64 / dst.w as i64) as i32;
            if sx < 0 || sy < 0 || sx >= t.width as i32 || sy >= t.height as i32 {
                continue;
            }
            let o = ((sy as u32 * t.width + sx as u32) * 4) as usize;
            let px = [t.pixels[o], t.pixels[o + 1], t.pixels[o + 2], t.pixels[o + 3]];
            blend(c, dst.x + dx, dst.y + dy, px);
        }
    }
}

struct Fonts {
    regular: FontVec,
    bold: FontVec,
}

fn load_font(names: &[&str]) -> FontVec {
    for n in names {
        if let Ok(b) = std::fs::read(format!("C:\\Windows\\Fonts\\{n}")) {
            if let Ok(f) = FontVec::try_from_vec(b) {
                return f;
            }
        }
    }
    panic!("aucune police trouvée parmi {names:?}");
}

/// Taille `<size:N>` → pixels : on suppose des points à 96 dpi [D].
fn px(size: u32) -> f32 {
    size as f32 * 4.0 / 3.0
}

fn run_width(f: &FontVec, r: &TextRun) -> f32 {
    let s = f.as_scaled(PxScale::from(px(r.style.size)));
    r.text.chars().map(|ch| s.h_advance(f.glyph_id(ch))).sum()
}

/// Découpe en lignes : sauts explicites, puis retour à la ligne automatique (au dernier
/// espace, sinon au caractère) si le texte dépasse la largeur et n'est pas SINGLE_LINE.
fn wrap(fonts: &Fonts, runs: &[TextRun], width: i32, single_line: bool) -> Vec<Vec<TextRun>> {
    let mut lines: Vec<Vec<(char, rz_ui::TextStyle)>> = vec![Vec::new()];
    let mut w = 0.0f32;
    for r in runs {
        let f = if r.style.bold { &fonts.bold } else { &fonts.regular };
        let sf = f.as_scaled(PxScale::from(px(r.style.size)));
        for ch in r.text.chars() {
            if ch == '\n' {
                if single_line {
                    break;
                }
                lines.push(Vec::new());
                w = 0.0;
                continue;
            }
            let adv = sf.h_advance(f.glyph_id(ch));
            if !single_line && w + adv > width as f32 && !lines.last().unwrap().is_empty() {
                let cur = lines.last_mut().unwrap();
                let cut = cur.iter().rposition(|(c, _)| *c == ' ').map(|p| p + 1).unwrap_or(cur.len());
                let rest: Vec<_> = cur.split_off(cut);
                w = rest.iter().map(|(c, s)| {
                    let f = if s.bold { &fonts.bold } else { &fonts.regular };
                    f.as_scaled(PxScale::from(px(s.size))).h_advance(f.glyph_id(*c))
                }).sum();
                lines.push(rest);
            }
            lines.last_mut().unwrap().push((ch, r.style.clone()));
            w += adv;
        }
    }
    lines
        .into_iter()
        .map(|l| {
            let mut out: Vec<TextRun> = Vec::new();
            for (ch, st) in l {
                match out.last_mut() {
                    Some(r) if r.style == st => r.text.push(ch),
                    _ => out.push(TextRun { text: ch.to_string(), style: st }),
                }
            }
            out
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn draw_text(c: &mut Canvas, fonts: &Fonts, runs: &[TextRun], rect: rz_ui::IRect, ha: HAlign, va: VAlign, single_line: bool) {
    let lines = wrap(fonts, runs, rect.w, single_line);
    // Le client rend le texte dans une texture de la taille du rect : on coupe au rect.
    let blend = |c: &mut Canvas, x: i32, y: i32, rgba: [u8; 4]| {
        if x >= rect.x - 1 && x <= rect.x + rect.w && y >= rect.y - 1 && y <= rect.y + rect.h {
            blend(c, x, y, rgba)
        }
    };
    let line_h = |l: &Vec<TextRun>| l.iter().map(|r| px(r.style.size)).fold(px(10), f32::max).ceil();
    let total_h: f32 = lines.iter().map(line_h).sum();
    let mut y = match va {
        VAlign::Top => rect.y as f32,
        VAlign::Center => rect.y as f32 + (rect.h as f32 - total_h) / 2.0,
        VAlign::Bottom => (rect.y + rect.h) as f32 - total_h,
    };
    for l in &lines {
        let lh = line_h(l);
        let font_of = |r: &TextRun| if r.style.bold { &fonts.bold } else { &fonts.regular };
        let w: f32 = l.iter().map(|r| run_width(font_of(r), r)).sum();
        let mut x = match ha {
            HAlign::Left => rect.x as f32,
            HAlign::Center => rect.x as f32 + (rect.w as f32 - w) / 2.0,
            HAlign::Right => (rect.x + rect.w) as f32 - w,
        };
        for r in l {
            let f = font_of(r);
            let scale = PxScale::from(px(r.style.size));
            let sf = f.as_scaled(scale);
            let base = y + sf.ascent() + (lh - sf.height()) / 2.0 + r.style.offset_y as f32;
            let col = r.style.color;
            let rgb = [(col >> 16) as u8, (col >> 8) as u8, col as u8];
            let halo = r.style.glow || r.style.outline || r.style.shadow;
            let oc = r.style.outline_color.unwrap_or(0);
            let orgb = [(oc >> 16) as u8, (oc >> 8) as u8, oc as u8];
            for ch in r.text.chars() {
                let gid = f.glyph_id(ch);
                let g = gid.with_scale_and_position(scale, ab_glyph::point(x, base));
                x += sf.h_advance(gid);
                let Some(o) = f.outline_glyph(g) else { continue };
                let b = o.px_bounds();
                let mut cov = Vec::new();
                o.draw(|gx, gy, v| cov.push((b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32, v)));
                if halo {
                    let offs: &[(i32, i32)] = if r.style.shadow && !r.style.glow && !r.style.outline {
                        &[(1, 1)]
                    } else {
                        &[(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1), (-1, 1), (1, -1)]
                    };
                    for &(gx, gy, v) in &cov {
                        for (ox, oy) in offs {
                            blend(c, gx + ox, gy + oy, [orgb[0], orgb[1], orgb[2], (v * 160.0) as u8]);
                        }
                    }
                }
                for &(gx, gy, v) in &cov {
                    blend(c, gx, gy, [rgb[0], rgb[1], rgb[2], (v.min(1.0) * 255.0) as u8]);
                }
            }
        }
        y += lh;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage : render_nui <dossier client> <sortie.png> [options] fenêtre.nui[@x,y] …");
        std::process::exit(2);
    }
    let client = &args[0];
    let out_path = &args[1];
    let mut canvas_size: Option<(u32, u32)> = None;
    let mut bg = [24u8, 24, 32, 255];
    let mut bg_image: Option<String> = None;
    let mut st = UiState::default();
    let mut hide: Vec<String> = Vec::new();
    let mut codepage = "windows-1252".to_string();
    let mut list = false;
    let mut zoom = 1u32;
    let mut windows: Vec<(String, Option<(i32, i32)>)> = Vec::new();
    let mut i = 2;
    while i < args.len() {
        let a = args[i].as_str();
        let mut val = || {
            i += 1;
            args.get(i).cloned().unwrap_or_default()
        };
        match a {
            "--canvas" => {
                let v = val();
                let (w, h) = v.split_once('x').expect("--canvas LxH");
                canvas_size = Some((w.parse().unwrap(), h.parse().unwrap()));
            }
            "--bg" => {
                let v: Vec<u8> = val().split(',').map(|x| x.parse().unwrap()).collect();
                bg = [v[0], v[1], v[2], 255];
            }
            "--bg-image" => bg_image = Some(val()),
            "--state" => {
                st.default_state = match val().as_str() {
                    "hover" => ControlState::Hover,
                    "pressed" => ControlState::Pressed,
                    "disabled" => ControlState::Disabled,
                    _ => ControlState::Normal,
                }
            }
            "--hover" => {
                st.set_state(&val(), ControlState::Hover);
            }
            "--press" => {
                st.set_state(&val(), ControlState::Pressed);
            }
            "--disable" => {
                st.set_state(&val(), ControlState::Disabled);
            }
            "--check" => {
                for id in val().split(',') {
                    st.check(id);
                }
            }
            "--hide" => hide.extend(val().split(',').map(|s| s.to_ascii_lowercase())),
            "--gauge" => {
                let v = val();
                let (id, x) = v.split_once('=').unwrap();
                st.set_gauge(id, x.parse().unwrap());
            }
            "--set" => {
                let v = val();
                let (id, x) = v.split_once('=').unwrap();
                st.set_caption(id, x);
            }
            "--codepage" => codepage = val(),
            "--list" => list = true,
            "--zoom" => zoom = val().parse().unwrap(),
            _ => {
                let (f, pos) = match a.split_once('@') {
                    Some((f, p)) => {
                        let (x, y) = p.split_once(',').unwrap();
                        (f.to_string(), Some((x.parse().unwrap(), y.parse().unwrap())))
                    }
                    None => (a.to_string(), None),
                };
                windows.push((f, pos));
            }
        }
        i += 1;
    }

    let mut ar = DataArchive::open(client).expect("archive");
    let all: std::collections::HashSet<String> = ar.entries().iter().map(|e| e.name.to_ascii_lowercase()).collect();
    // Le client lit d'abord Resource\ (fichiers à jour), puis l'archive.
    let sname = format!("db_string({codepage}).rdb");
    let strings: HashMap<u32, String> = match std::fs::read(std::path::Path::new(client).join("Resource").join(&sname))
        .or_else(|_| ar.read_name(&sname).map_err(|e| std::io::Error::other(e.to_string())))
        .map_err(|e| e.to_string())
        .and_then(|b| rz_rappelz::rdb::parse_strings(&b, &rz_rappelz::rdb::Decoder::new(&codepage)).map_err(|e| e.to_string()))
    {
        Ok(v) => v.into_iter().map(|s| (s.code as u32, s.value)).collect(),
        Err(e) => {
            eprintln!("db_string illisible : {e}");
            HashMap::new()
        }
    };
    eprintln!("{} chaînes db_string", strings.len());

    let mut docs: Vec<(NuiDocument, Option<(i32, i32)>)> = Vec::new();
    for (f, pos) in &windows {
        let bytes = ar.read_name(f).unwrap_or_else(|_| panic!("{f} absent de l'archive"));
        let doc = parse_nui(&rz_ui::decode_text(&bytes)).unwrap_or_else(|e| panic!("{f} : {e}"));
        docs.push((doc, *pos));
    }
    // Contrôles masqués (id exact ou préfixe*).
    for (doc, _) in &docs {
        for c in &doc.controls {
            let id = c.id.to_ascii_lowercase();
            if hide.iter().any(|h| h.strip_suffix('*').map_or(*h == id, |p| id.starts_with(p))) {
                st.hide(&id);
            }
        }
    }

    let spr_name = docs
        .iter()
        .flat_map(|(d, _)| d.controls.iter())
        .find_map(|c| c.spr.clone())
        .unwrap_or_else(|| "ui_frame.spr".into());
    let mut sprites = SpriteSet::load_ui_set(|f| ar.read_name(f).ok(), &spr_name);
    let mut tex = Textures { ar, cache: HashMap::new() };
    for (doc, _) in &docs {
        let stats = sprites.resolve_document(doc, |name| {
            if !all.contains(&name.to_ascii_lowercase()) {
                return None;
            }
            let t = tex.get(name)?;
            Some((name.to_string(), t.width, t.height))
        });
        eprintln!(
            "animations : {} demandées, inconnues {:?}, images absentes {:?}",
            stats.animations, stats.missing_animations, stats.missing_images
        );
    }

    let (cw, ch) = canvas_size.unwrap_or_else(|| {
        let (w, h) = docs.iter().map(|(d, _)| d.size()).fold((0, 0), |a, b| (a.0.max(b.0), a.1.max(b.1)));
        (w as u32 + 40, h as u32 + 40)
    });
    let mut canvas = Canvas::from_pixel(cw, ch, image::Rgba(bg));
    if let Some(name) = &bg_image {
        if let Some(t) = tex.get(name).cloned() {
            draw_image(&mut canvas, &t, rz_ui::IRect::new(0, 0, t.width as i32, t.height as i32), rz_ui::IRect::new(0, 0, cw as i32, ch as i32));
        }
    }
    let fonts = Fonts { regular: load_font(&["tahoma.ttf", "arial.ttf"]), bold: load_font(&["tahomabd.ttf", "arialbd.ttf"]) };
    for (doc, pos) in &docs {
        let (w, h) = doc.size();
        let origin = pos.unwrap_or(((cw as i32 - w) / 2, (ch as i32 - h) / 2));
        let cmds = layout(doc, &sprites, origin, &st);
        for cmd in &cmds {
            match cmd {
                DrawCmd::Image(c) => {
                    if list {
                        println!("img {:<32} {:<48} src {:?} dst {:?}", c.control, c.image, c.src, c.dst);
                    }
                    if let Some(t) = tex.get(&c.image) {
                        let t = t.clone();
                        draw_image(&mut canvas, &t, c.src, c.dst);
                    }
                }
                DrawCmd::Text(t) => {
                    let runs = t.caption.resolve(|id| strings.get(&id).cloned());
                    if list {
                        let s: String = runs.iter().map(|r| r.text.as_str()).collect();
                        println!("txt {:<32} {:?} {:?}", t.control, s, t.rect);
                    }
                    draw_text(&mut canvas, &fonts, &runs, t.rect, t.caption.halign, t.caption.valign, t.single_line);
                }
            }
        }
    }
    if zoom > 1 {
        canvas = image::imageops::resize(&canvas, cw * zoom, ch * zoom, image::imageops::FilterType::Nearest);
    }
    canvas.save(out_path).expect("écriture PNG");
    eprintln!("écrit {out_path} ({cw}x{ch})");
}
