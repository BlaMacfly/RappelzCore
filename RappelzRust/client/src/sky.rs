//! Opt-in offline sky-gradient experiment; this does not reproduce the original sky shader.
//!
//! Call `draw` after selecting the world Camera3D and before terrain, while the normal
//! material is active. The pass does not change the camera, viewport, scissor or depth
//! enable state, and restores the normal material/empty texture binding afterwards.
//! Each Sky owns one top-level texture and one material; it does not use Assets' GPU cache.
use anyhow::{ensure, Context, Result};
use macroquad::material::{
    gl_use_default_material, gl_use_material, load_material, Material, MaterialParams,
};
use macroquad::miniquad::{
    Backend, Comparison, CullFace, PipelineParams, ShaderSource, TextureWrap, UniformDesc,
    UniformType,
};
use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::*;
use serde_json::{json, Value};

use crate::assets::Assets;

pub const SOURCE_ENTRY: &str = "Textures\\sky_zone001_sky01.dds";
pub const SOLID_FALLBACK: Color = Color::from_rgba(150, 185, 220, 255);
pub const MAX_SIDE: u32 = 2048;
pub const MAX_ENCODED_BYTES: usize = 24 * 1024 * 1024;
pub const MAX_TOP_RGBA_BYTES: usize = 2048 * 2048 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyMode {
    Gradient,
}

/// Only explicit offline demos may select this renderer. Data/patch precedence chooses
/// the original or edited source; the flag neither changes nor persists WTF settings.
pub fn parse_args(args: &[String]) -> Result<Option<SkyMode>> {
    const FLAG: &str = "-sky-gradient";
    ensure!(
        !args.iter().any(|arg| arg.starts_with(FLAG) && arg != FLAG),
        "use the boolean -sky-gradient flag without a value"
    );
    let Some(index) = args.iter().position(|arg| arg == FLAG) else {
        return Ok(None);
    };
    ensure!(
        args.iter().filter(|arg| arg.as_str() == FLAG).count() == 1,
        "choose sky gradient only once"
    );
    ensure!(
        args.get(index + 1).is_none_or(|arg| arg.starts_with('-')),
        "-sky-gradient does not take an original/edited argument; choose the data root"
    );
    ensure!(
        args.iter().any(|arg| arg == "-demo"),
        "-sky-gradient requires -demo"
    );
    ensure!(
        !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "-login" | "-enter")
                || arg.starts_with("-login=")
                || arg.starts_with("-enter=")),
        "sky gradient demo cannot log in or enter a network character"
    );
    ensure!(
        !args
            .iter()
            .any(|arg| arg.starts_with("-validate-") && arg != "-validate-terrain-assets"),
        "sky gradient demo cannot run an online validation scenario"
    );
    Ok(Some(SkyMode::Gradient))
}

/// Projection-only ray basis, independent of camera translation and of GPU state.
/// The fallback aspect must match Camera3D's screen/render-target aspect when it has
/// no explicit aspect. This matches macroquad's perspective and orthographic cameras.
#[derive(Debug, Clone, Copy)]
pub struct SkyProjection {
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    scale: Vec2,
    perspective: bool,
}

impl SkyProjection {
    pub fn from_camera(camera: &Camera3D, fallback_aspect: f32) -> Result<Self> {
        ensure!(
            camera.position.is_finite() && camera.target.is_finite() && camera.up.is_finite(),
            "non-finite sky camera vectors"
        );
        let direction = camera.target - camera.position;
        ensure!(
            direction.length_squared().is_finite() && direction.length_squared() > 1e-12,
            "sky camera direction is degenerate"
        );
        let forward = direction.normalize();
        let right = forward.cross(camera.up);
        ensure!(
            right.length_squared().is_finite() && right.length_squared() > 1e-12,
            "sky camera up is parallel or degenerate"
        );
        let right = right.normalize();
        let up = right.cross(forward).normalize();
        let aspect = camera.aspect.unwrap_or(fallback_aspect);
        ensure!(
            aspect.is_finite() && aspect > 0.,
            "invalid sky camera aspect"
        );
        ensure!(
            camera.z_near.is_finite()
                && camera.z_far.is_finite()
                && camera.z_near > 0.
                && camera.z_far > camera.z_near,
            "invalid sky camera clip planes"
        );
        ensure!(
            camera.fovy.is_finite() && camera.fovy > 0.,
            "invalid sky camera field of view"
        );
        let (scale, perspective, projection) = match camera.projection {
            Projection::Perspective => {
                ensure!(
                    camera.fovy < std::f32::consts::PI,
                    "sky perspective field of view must be less than pi"
                );
                let vertical = (camera.fovy * 0.5).tan();
                (
                    vec2(vertical * aspect, vertical),
                    true,
                    Mat4::perspective_rh_gl(camera.fovy, aspect, camera.z_near, camera.z_far),
                )
            }
            Projection::Orthographics => {
                let top = camera.fovy * 0.5;
                let right = top * aspect;
                (
                    Vec2::ZERO,
                    false,
                    Mat4::orthographic_rh_gl(-right, right, -top, top, camera.z_near, camera.z_far),
                )
            }
        };
        let matrix = projection * Mat4::look_at_rh(camera.position, camera.target, camera.up);
        ensure!(
            scale.is_finite() && matrix.to_cols_array().iter().all(|value| value.is_finite()),
            "sky camera projection matrix is not finite"
        );
        Ok(Self {
            forward,
            right,
            up,
            scale,
            perspective,
        })
    }

    pub fn direction_at(&self, ndc: Vec2) -> Option<Vec3> {
        if !ndc.is_finite() {
            return None;
        }
        let ray =
            self.forward + self.right * (ndc.x * self.scale.x) + self.up * (ndc.y * self.scale.y);
        if !ray.is_finite() || !ray.length_squared().is_finite() || ray.length_squared() <= 1e-12 {
            return None;
        }
        Some(ray.normalize())
    }

    /// Source row 0 is zenith; row 1 is the horizon. Downward rays keep the horizon
    /// color until terrain occludes them. Only the source center column is sampled.
    pub fn gradient_v(&self, ndc: Vec2) -> Option<f32> {
        self.direction_at(ndc)
            .map(|direction| 1. - direction.y.clamp(0., 1.))
    }

    fn snapshot(&self) -> Value {
        json!({
            "projection": if self.perspective { "perspective" } else { "orthographic" },
            "direction": self.forward.to_array(),
            "right": self.right.to_array(), "up": self.up.to_array(), "ray_scale": self.scale.to_array(),
            "gradient_v_center": self.gradient_v(Vec2::ZERO),
            "gradient_v_top": self.gradient_v(vec2(0., 1.)),
            "gradient_v_bottom": self.gradient_v(vec2(0., -1.)),
        })
    }
}

fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("preflight checked DDS header"),
    )
}

fn prepare_gradient(bytes: &[u8]) -> Result<rz_texture::Rgba> {
    ensure!(
        bytes.len() <= MAX_ENCODED_BYTES,
        "sky DDS exceeds the 24 MiB encoded limit"
    );
    ensure!(
        bytes.len() >= 128 && &bytes[..4] == b"DDS ",
        "sky source must be a DDS with a complete header"
    );
    ensure!(
        word(bytes, 4) == 124 && word(bytes, 76) == 32,
        "invalid sky DDS header size"
    );
    let (width, height) = (word(bytes, 16), word(bytes, 12));
    ensure!(
        (1..=MAX_SIDE).contains(&width) && (1..=MAX_SIDE).contains(&height),
        "sky DDS dimensions must be 1..=2048 per side"
    );
    let expected = width as usize * height as usize * 4;
    ensure!(
        expected <= MAX_TOP_RGBA_BYTES,
        "sky decoded image exceeds 16 MiB"
    );
    let image = rz_texture::decode(SOURCE_ENTRY, bytes).context("decode sky DDS")?;
    ensure!(
        image.width == width && image.height == height && image.pixels.len() == expected,
        "sky decoded dimensions or payload disagree with the header"
    );
    ensure!(
        image.pixels.chunks_exact(4).all(|pixel| pixel[3] == 255),
        "sky gradient must be opaque; cloud and face masks are separate assets"
    );
    Ok(image)
}

fn pipeline_params() -> PipelineParams {
    PipelineParams {
        depth_test: Comparison::Always,
        depth_write: false,
        cull_face: CullFace::Nothing,
        color_blend: None,
        alpha_blend: None,
        ..Default::default()
    }
}

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
varying highp vec2 sky_ndc;
void main() {
    gl_Position = vec4(position.xy, 0.0, 1.0);
    sky_ndc = position.xy;
}"#;

const FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;
varying highp vec2 sky_ndc;
uniform sampler2D Texture;
uniform vec3 SkyForward;
uniform vec3 SkyRight;
uniform vec3 SkyUp;
uniform vec2 SkyScale;
void main() {
    vec3 direction = normalize(SkyForward + SkyRight * sky_ndc.x * SkyScale.x + SkyUp * sky_ndc.y * SkyScale.y);
    float v = 1.0 - clamp(direction.y, 0.0, 1.0);
    gl_FragColor = vec4(texture2D(Texture, vec2(0.5, v)).rgb, 1.0);
}"#;

struct Renderer {
    material: Material,
    quad: Mesh,
}

pub struct Sky {
    mode: SkyMode,
    renderer: Option<Renderer>,
    source_dimensions: Option<[u32; 2]>,
    encoded_bytes: usize,
    decoded_top_bytes: usize,
    load_error: Option<String>,
    camera_error: Option<String>,
    last_camera: Option<SkyProjection>,
    last_used: bool,
    draw_count: u64,
    gradient_draw_count: u64,
    fallback_draw_count: u64,
}

impl Sky {
    fn empty(mode: SkyMode) -> Self {
        Self {
            mode,
            renderer: None,
            source_dimensions: None,
            encoded_bytes: 0,
            decoded_top_bytes: 0,
            load_error: None,
            camera_error: None,
            last_camera: None,
            last_used: false,
            draw_count: 0,
            gradient_draw_count: 0,
            fallback_draw_count: 0,
        }
    }

    /// Requires an active graphics context. The caller creates Sky only for an explicitly
    /// selected offline demo. Failed sources/backends/shaders leave a solid-color fallback.
    pub fn new(mode: SkyMode, assets: &mut Assets) -> Self {
        let mut sky = Self::empty(mode);
        let result = sky.load(assets);
        if let Err(error) = result {
            sky.load_error = Some(format!("{error:#}"));
            tracing::warn!("sky gradient fallback: {error:#}");
        }
        sky
    }

    fn load(&mut self, assets: &mut Assets) -> Result<()> {
        ensure!(
            unsafe { get_internal_gl() }.quad_context.info().backend == Backend::OpenGl,
            "sky gradient prototype requires the OpenGL backend"
        );
        // Assets.read currently allocates the archive result before this module can inspect
        // its length. Limits below bound accepted encoded bytes, decoding and GPU payload;
        // they are not a bound on MPQ decompression allocation or whole-process RSS.
        let bytes = assets
            .read(SOURCE_ENTRY)
            .context("sky gradient DDS is absent")?;
        self.encoded_bytes = bytes.len();
        let image = prepare_gradient(&bytes)?;
        self.source_dimensions = Some([image.width, image.height]);
        self.decoded_top_bytes = image.pixels.len();
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params: pipeline_params(),
                uniforms: vec![
                    UniformDesc::new("SkyForward", UniformType::Float3),
                    UniformDesc::new("SkyRight", UniformType::Float3),
                    UniformDesc::new("SkyUp", UniformType::Float3),
                    UniformDesc::new("SkyScale", UniformType::Float2),
                ],
                ..Default::default()
            },
        )
        .context("compile sky gradient material")?;
        let texture = Texture2D::from_rgba8(image.width as u16, image.height as u16, &image.pixels);
        texture.set_filter(FilterMode::Linear);
        unsafe { get_internal_gl() }.quad_context.texture_set_wrap(
            texture.raw_miniquad_id(),
            TextureWrap::Clamp,
            TextureWrap::Clamp,
        );
        let quad = Mesh {
            vertices: vec![
                Vertex::new(-1., -1., 0., 0., 0., WHITE),
                Vertex::new(1., -1., 0., 0., 0., WHITE),
                Vertex::new(1., 1., 0., 0., 0., WHITE),
                Vertex::new(-1., 1., 0., 0., 0., WHITE),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            texture: Some(texture),
        };
        self.renderer = Some(Renderer { material, quad });
        // image.pixels and encoded bytes leave scope here; only one managed GPU image remains.
        Ok(())
    }

    /// Draw the backdrop before terrain while the world's default material is active.
    /// Camera orientation affects the rays; position and near/far clipping do not move the sky.
    pub fn draw(&mut self, camera: &Camera3D) {
        self.draw_count = self.draw_count.saturating_add(1);
        self.last_used = false;
        self.last_camera = None;
        self.camera_error = None;
        let (width, height) = camera.render_target.as_ref().map_or_else(
            || (screen_width(), screen_height()),
            |target| (target.texture.width(), target.texture.height()),
        );
        let projection = match SkyProjection::from_camera(camera, width / height) {
            Ok(projection) => projection,
            Err(error) => {
                self.camera_error = Some(format!("{error:#}"));
                self.fallback_draw_count = self.fallback_draw_count.saturating_add(1);
                clear_background(SOLID_FALLBACK);
                return;
            }
        };
        self.last_camera = Some(projection);
        let Some(renderer) = &self.renderer else {
            self.fallback_draw_count = self.fallback_draw_count.saturating_add(1);
            clear_background(SOLID_FALLBACK);
            return;
        };
        renderer
            .material
            .set_uniform("SkyForward", projection.forward);
        renderer.material.set_uniform("SkyRight", projection.right);
        renderer.material.set_uniform("SkyUp", projection.up);
        renderer.material.set_uniform("SkyScale", projection.scale);
        gl_use_material(&renderer.material);
        draw_mesh(&renderer.quad);
        gl_use_default_material();
        // Subsequent terrain and UI draws bind their own textures as usual.
        unsafe { get_internal_gl() }.quad_gl.texture(None);
        self.last_used = true;
        self.gradient_draw_count = self.gradient_draw_count.saturating_add(1);
    }

    pub fn snapshot(&self) -> Value {
        let fallback = self.camera_error.as_ref().or(self.load_error.as_ref());
        let [width, height] = self.source_dimensions.unwrap_or([0, 0]);
        let texture_payload = if self.renderer.is_some() {
            self.decoded_top_bytes
        } else {
            0
        };
        json!({
            "enabled": true, "mode": match self.mode { SkyMode::Gradient => "gradient" },
            "source": SOURCE_ENTRY, "loaded": self.renderer.is_some(), "used": self.last_used,
            "fallback": fallback.is_some(), "fallback_reason": fallback,
            "error": fallback, "opaque": self.source_dimensions.is_some(),
            "solid_fallback_rgba": [150, 185, 220, 255],
            "status": if fallback.is_some() { "fallback" } else if self.renderer.is_some() { "ready" } else { "unavailable" },
            "drawn": self.gradient_draw_count > 0, "width": width, "height": height,
            "rgba_bytes": self.decoded_top_bytes, "texture_payload_bytes": texture_payload,
            "source_size": self.source_dimensions, "encoded_bytes": self.encoded_bytes,
            "decoded_top_rgba_bytes": self.decoded_top_bytes,
            "gpu_texture_payload_bytes": if self.renderer.is_some() { self.decoded_top_bytes } else { 0 },
            "owned_gpu_textures": usize::from(self.renderer.is_some()),
            "retained_source_pixel_bytes": 0, "runtime_mip_levels": usize::from(self.renderer.is_some()),
            "limits": { "max_side": MAX_SIDE, "max_encoded_bytes_after_archive_read": MAX_ENCODED_BYTES,
                "max_top_rgba_bytes": MAX_TOP_RGBA_BYTES, "mpq_initial_allocation_bounded": false,
                "whole_process_memory_bound": false },
            "draw_count": self.draw_count, "gradient_draw_count": self.gradient_draw_count,
            "fallback_draw_count": self.fallback_draw_count,
            "camera": self.last_camera.map(|camera| camera.snapshot()),
            "sampling": "center-column, world-ray vertical direction; horizon clamp for downward rays",
            "depth_write": false, "depth_compare": "Always", "default_material_restored": true,
            "original_sky_shader_compatibility": false, "six_face_cubemap": false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn camera() -> Camera3D {
        Camera3D {
            position: Vec3::ZERO,
            target: vec3(0., 0., -1.),
            up: Vec3::Y,
            fovy: std::f32::consts::FRAC_PI_2,
            aspect: Some(16. / 9.),
            z_near: 0.1,
            z_far: 1000.,
            ..Default::default()
        }
    }

    fn close(a: f32, b: f32) {
        assert!((a - b).abs() < 2e-5, "{a} differs from {b}");
    }

    #[test]
    fn explicit_demo_only_and_default_off() {
        assert_eq!(
            parse_args(&args(&["client", "-login", "account:secret"])).unwrap(),
            None
        );
        assert_eq!(parse_args(&args(&["client", "-demo"])).unwrap(), None);
        assert_eq!(
            parse_args(&args(&["client", "-demo", "-sky-gradient"])).unwrap(),
            Some(SkyMode::Gradient)
        );
        assert!(parse_args(&args(&["client", "-sky-gradient"])).is_err());
        for suffix in [
            vec!["-login", "account:secret"],
            vec!["-enter", "player"],
            vec!["-validate-combat", "plan"],
            vec!["-validate-terrain-network", "plan"],
            vec!["-sky-gradient"],
            vec!["original"],
        ] {
            let mut value = args(&["client", "-demo", "-sky-gradient"]);
            value.extend(args(&suffix));
            assert!(parse_args(&value).is_err(), "{value:?}");
        }
        assert!(parse_args(&args(&["client", "-demo", "-sky-gradient=edited"])).is_err());
        assert!(parse_args(&args(&[
            "client",
            "-demo",
            "-sky-gradient",
            "-validate-terrain-assets",
            "out"
        ]))
        .is_ok());
    }

    #[test]
    fn perspective_rays_agree_with_inverse_camera_matrix() {
        let mut camera = camera();
        camera.position = vec3(13., 7., -19.);
        camera.target = camera.position + vec3(0.4, 0.3, -1.);
        let rays = SkyProjection::from_camera(&camera, 1.).unwrap();
        let inverse = (Mat4::perspective_rh_gl(
            camera.fovy,
            camera.aspect.unwrap(),
            camera.z_near,
            camera.z_far,
        ) * Mat4::look_at_rh(camera.position, camera.target, camera.up))
        .inverse();
        for ndc in [vec2(-0.8, -0.7), Vec2::ZERO, vec2(0.9, 0.8)] {
            let far = inverse * vec4(ndc.x, ndc.y, 1., 1.);
            let direction = (far.truncate() / far.w - camera.position).normalize();
            assert!(direction.dot(rays.direction_at(ndc).unwrap()) > 0.99999);
        }
    }

    #[test]
    fn translation_and_yaw_do_not_move_the_vertical_gradient() {
        let mut base = camera();
        base.target.y = 0.3;
        let reference = SkyProjection::from_camera(&base, 1.).unwrap();
        for angle in [0., 0.7, 2.3] {
            for shift in [Vec3::ZERO, vec3(8300., 50., 7160.)] {
                let rotation = Mat4::from_rotation_y(angle);
                let mut changed = camera();
                changed.position = shift;
                changed.target = shift + rotation.transform_vector3(base.target);
                changed.up = rotation.transform_vector3(base.up);
                let actual = SkyProjection::from_camera(&changed, 1.).unwrap();
                for ndc in [vec2(-1., -1.), Vec2::ZERO, vec2(1., 1.)] {
                    assert!(
                        (reference.gradient_v(ndc).unwrap() - actual.gradient_v(ndc).unwrap())
                            .abs()
                            < 4e-4
                    );
                }
            }
        }
    }

    #[test]
    fn pitch_keeps_world_horizon_at_the_correct_screen_height() {
        let mut camera = camera();
        let pitch = 0.35f32;
        camera.target = vec3(0., pitch.sin(), -pitch.cos());
        let rays = SkyProjection::from_camera(&camera, 1.).unwrap();
        let horizon_clip = Mat4::perspective_rh_gl(
            camera.fovy,
            camera.aspect.unwrap(),
            camera.z_near,
            camera.z_far,
        ) * Mat4::look_at_rh(camera.position, camera.target, camera.up)
            * vec4(0., 0., -10., 1.);
        let ndc = horizon_clip.truncate() / horizon_clip.w;
        assert!(ndc.y.abs() > 0.1);
        close(rays.gradient_v(vec2(ndc.x, ndc.y)).unwrap(), 1.);
        close(rays.gradient_v(Vec2::ZERO).unwrap(), 1. - pitch.sin());
        assert!(rays.gradient_v(vec2(0., 1.)).unwrap() < rays.gradient_v(Vec2::ZERO).unwrap());
    }

    #[test]
    fn orthographic_rays_are_parallel_and_invalid_cameras_are_rejected() {
        let mut ortho = camera();
        ortho.projection = Projection::Orthographics;
        ortho.fovy = 800.;
        let rays = SkyProjection::from_camera(&ortho, 1.).unwrap();
        close(
            rays.gradient_v(vec2(-1., -1.)).unwrap(),
            rays.gradient_v(vec2(1., 1.)).unwrap(),
        );
        for case in 0..11 {
            let mut value = camera();
            match case {
                0 => value.target = value.position,
                1 => value.up = Vec3::ZERO,
                2 => value.up = value.target,
                3 => value.position.x = f32::NAN,
                4 => value.fovy = f32::INFINITY,
                5 => value.fovy = 0.,
                6 => value.fovy = std::f32::consts::PI,
                7 => value.z_near = 0.,
                8 => value.z_far = value.z_near,
                9 => value.aspect = Some(0.),
                _ => value.aspect = Some(f32::NAN),
            }
            assert!(
                SkyProjection::from_camera(&value, 1.).is_err(),
                "invalid case {case}"
            );
        }
        assert!(rays.direction_at(vec2(f32::NAN, 0.)).is_none());
    }

    fn rgba_dds(width: u32, height: u32, alpha: u8) -> Vec<u8> {
        let mut bytes = vec![0; 128];
        bytes[..4].copy_from_slice(b"DDS ");
        for (offset, value) in [
            (4, 124u32),
            (12, height),
            (16, width),
            (76, 32),
            (80, 0x41),
            (88, 32),
            (92, 0xff),
            (96, 0xff00),
            (100, 0xff0000),
            (104, 0xff000000),
        ] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        for _ in 0..width * height {
            bytes.extend([40, 90, 197, alpha]);
        }
        bytes
    }

    #[test]
    fn source_validation_bounds_decode_and_rejects_cloud_or_face_alpha() {
        let bytes = rgba_dds(4, 2, 255);
        let image = prepare_gradient(&bytes).unwrap();
        assert_eq!((image.width, image.height, image.pixels.len()), (4, 2, 32));
        assert!(prepare_gradient(&rgba_dds(4, 2, 153))
            .unwrap_err()
            .to_string()
            .contains("opaque"));
        assert!(prepare_gradient(b"not a DDS").is_err());
        assert!(prepare_gradient(&bytes[..128]).is_err());
        let mut header = bytes[..128].to_vec();
        header[16..20].copy_from_slice(&2049u32.to_le_bytes());
        assert!(prepare_gradient(&header)
            .unwrap_err()
            .to_string()
            .contains("2048"));
        let too_large = vec![0; MAX_ENCODED_BYTES + 1];
        assert!(prepare_gradient(&too_large)
            .unwrap_err()
            .to_string()
            .contains("24 MiB"));
    }

    #[test]
    fn fallback_snapshot_and_backdrop_depth_policy_are_explicit() {
        let mut sky = Sky::empty(SkyMode::Gradient);
        sky.load_error = Some("missing source".into());
        let snapshot = sky.snapshot();
        assert_eq!(snapshot["loaded"], false);
        assert_eq!(snapshot["fallback"], true);
        assert_eq!(snapshot["owned_gpu_textures"], 0);
        assert_eq!(snapshot["gpu_texture_payload_bytes"], 0);
        assert_eq!(snapshot["used"], false);
        assert!(snapshot["camera"].is_null());
        let pipeline = pipeline_params();
        assert_eq!(pipeline.depth_test, Comparison::Always);
        assert!(!pipeline.depth_write);
        assert_eq!(pipeline.cull_face, CullFace::Nothing);
        assert!(pipeline.color_blend.is_none());
    }
}
