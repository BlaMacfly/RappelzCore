//! Interprétation alpha cutout expérimentale, explicitement choisie pour les rigs hors ligne.
//! Le rôle du canal alpha dans le client d'origine reste inconnu. Ce mode ne modifie ni les
//! textures ni les UV et ne remplace pas le rendu historique par défaut.

use macroquad::miniquad::{Backend, Comparison, CullFace, PipelineParams};
use macroquad::prelude::*;

pub fn parse_character_cutout_args(args: &[String]) -> anyhow::Result<bool> {
    anyhow::ensure!(
        !args.iter().any(|a| a.starts_with("-character-cutout") && a != "-character-cutout"),
        "-character-cutout is a standalone boolean flag"
    );
    let count = args.iter().filter(|a| a.as_str() == "-character-cutout").count();
    anyhow::ensure!(count <= 1, "-character-cutout must be supplied only once");
    if count == 0 {
        return Ok(false);
    }
    let flag_index = args.iter().position(|a| a == "-character-cutout").unwrap();
    anyhow::ensure!(
        args.get(flag_index + 1).is_none_or(|next| next.starts_with('-')),
        "-character-cutout does not accept a value"
    );
    anyhow::ensure!(args.iter().any(|a| a == "-demo"), "-character-cutout requires -demo");
    anyhow::ensure!(
        !args.iter().any(|a| a == "-login" || a.starts_with("-login=") || a == "-enter" || a.starts_with("-enter=")),
        "-character-cutout cannot be combined with login or world entry"
    );
    anyhow::ensure!(
        !args.iter().any(|a| a.starts_with("-validate-") && a != "-validate-terrain-assets"),
        "-character-cutout cannot be combined with online validation scenarios"
    );
    Ok(true)
}

pub(super) fn load_character_cutout() -> anyhow::Result<Material> {
    let backend = unsafe { get_internal_gl().quad_context.info().backend };
    anyhow::ensure!(matches!(backend, Backend::OpenGl), "experimental character cutout requires OpenGL");
    load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Nothing,
                color_blend: None,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .map_err(|e| anyhow::anyhow!("cannot load experimental character cutout: {e}"))
}

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

const FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    lowp vec4 sampled = color * texture2D(Texture, uv);
    if (sampled.a < 0.5) discard;
    gl_FragColor = vec4(sampled.rgb, 1.0);
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn normal_and_connected_invocations_keep_legacy_materials() {
        for invocation in [vec![], args(&["-demo"]), args(&["-login", "user", "-enter"])] {
            assert!(!parse_character_cutout_args(&invocation).unwrap());
        }
    }

    #[test]
    fn experimental_cutout_requires_one_standalone_flag_and_offline_demo() {
        assert!(parse_character_cutout_args(&args(&["-demo", "-character-cutout"])).unwrap());
        assert!(parse_character_cutout_args(&args(&["-character-cutout"])).is_err());
        assert!(parse_character_cutout_args(&args(&["-demo", "-character-cutout", "-character-cutout"])).is_err());
        for value in ["-character-cutout=true", "-character-cutout=false"] {
            assert!(parse_character_cutout_args(&args(&["-demo", value])).is_err());
        }
        for value in ["false", "original"] {
            assert!(parse_character_cutout_args(&args(&["-demo", "-character-cutout", value])).is_err());
        }
        assert!(parse_character_cutout_args(&args(&["-demo", "-character-cutout-unknown"])).is_err());
    }

    #[test]
    fn demo_flag_cannot_bypass_login_entry_or_online_validation_guards() {
        for online in ["-login", "-login=user", "-enter", "-enter=1", "-validate-skills", "-validate-terrain", "-validate-soap-shutdown", "-validate-unknown-future"] {
            assert!(parse_character_cutout_args(&args(&["-demo", "-character-cutout", online])).is_err(), "{online}");
        }
        assert!(parse_character_cutout_args(&args(&["-demo", "-character-cutout", "-validate-terrain-assets", "new-output"])).unwrap());
    }
}
