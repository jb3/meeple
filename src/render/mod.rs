pub mod boxes;
pub mod gl;
pub mod grade;
pub mod shaders;

use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec3};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlBuffer, WebGlFramebuffer, WebGlTexture,
    WebGlVertexArrayObject,
};

mod targets;
mod passes;
mod programs;
use self::programs::*;
mod meshes;

use boxes::{Boxes, FLOATS};
use gl::{f32_bytes, Program};
use shaders::*;

use crate::quality::Fx;
use crate::world::mesh::MeshData;

struct Mesh {
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
    ibo: WebGlBuffer,
    count: i32,
}

struct ChunkMeshes {
    solid: Option<Mesh>,
    water: Option<Mesh>,
}

struct InstBatch {
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
    count: i32,
}

/// Everything the shaders need for one frame.
pub struct Frame {
    /// How far into a postcard moment (0..1), for its shallower depth of field.
    pub postcard: f32,
    /// Where the rainbow's arc stands: a ground point and its plane's horizontal normal.
    pub bow_a: Vec3,
    pub bow_n: Vec2,
    /// Outer radius of the arc.
    pub bow_r: f32,
    /// The people the camera's watching, as (x, y, z, radius) circles, and how
    /// far to fade (0..1)
    /// the buildings between it and the eye (unit vector towards the eye).
    pub foci: Vec<f32>,
    pub clear: f32,
    pub eye: Vec3,
    pub vp: Mat4,
    pub inv_vp: Mat4,
    /// Sub-texel camera offset to undo in the upscale, in uv units.
    pub shift: Vec2,
    pub light_vp: Mat4,
    pub sun_dir: Vec3,
    pub sun_col: Vec3,
    pub amb_sky: Vec3,
    pub amb_ground: Vec3,
    pub sky: Vec3,
    pub time: f32,
    pub wind: Vec2,
    pub cloud: f32,
    pub night: f32,
    pub lamp: f32,
    pub lights: Vec<f32>,
    pub grass_tint: Vec3,
    pub leaf_tint: Vec3,
    pub autumn: f32,
    pub bare: f32,
    pub blossom: f32,
    pub snow: f32,
    pub wet: f32,
    pub crop: f32,
    pub ripe: f32,
    pub win_lit: f32,
    pub ice: f32,
    pub fog: f32,
    pub flash: f32,
    pub rain: f32,
    /// After-rain rainbow strength.
    pub rainbow: f32,
    /// Northern lights strength (cold, clear nights).
    pub aurora: f32,
    /// The curtains hung in the world (see `AURORA_VS`), eight floats each.
    pub curtains: Vec<f32>,
    /// Shooting star: start and travel in screen uv, and progress 0..1.
    pub meteor: [f32; 4],
    pub meteor_t: f32,
    /// Tilt-shift blur strength, for photographs.
    pub tilt: f32,
    /// How far the water stands above its usual level: tides and floods.
    pub tide: f32,
    /// Snow on the high ground through the colder months.
    pub peak_snow: f32,
    /// 0..4: spring, summer, autumn, winter.
    pub season: f32,
    /// The tighter shadow map round the camera (see `Fx::cascades`).
    pub light_vp2: Mat4,
    /// Golden-hour warmth, night, winter and overcast, each 0..1, for the grade.
    pub grade: grade::Params,
    /// The camera's right and up vectors, and the world units one step of
    /// depth spans.
    pub cam_right: Vec3,
    pub cam_up: Vec3,
    pub depth_range: f32,
    /// Half the visible height in world units, and what the camera looks at.
    pub zoom: f32,
    pub focus: Vec3,
    /// Where in its cycle the moon is, 0..1.
    pub moon: f32,
    /// Weather drawn on the GPU: the box round the camera, the ground height
    /// there, rain (or snow) strength, whether it's snow, whether leaves are
    /// falling, and how bright the day is.
    pub part_center: Vec2,
    pub part_ext: f32,
    pub ground_y: f32,
    pub snowing: bool,
    pub leaves: bool,
    pub part_br: f32,
}

/// Where an effect's offscreen buffers are.
struct Target {
    fbo: WebGlFramebuffer,
    tex: WebGlTexture,
    w: i32,
    h: i32,
}

/// A copy of the scene before the water goes on, for the water to look into.
struct SceneCopy {
    fbo: WebGlFramebuffer,
    color: WebGlTexture,
    depth: WebGlTexture,
}

/// Last frame's result, and the one before, for temporal anti-aliasing.
struct Taa {
    targets: [Target; 2],
    /// Which one is written next.
    next: usize,
    valid: bool,
    prev_vp: Mat4,
}

/// Low-discrepancy sub-pixel offsets for temporal anti-aliasing.
fn halton(mut i: u32, base: u32) -> f32 {
    let (mut f, mut r) = (1.0, 0.0);
    while i > 0 {
        f /= base as f32;
        r += f * (i % base) as f32;
        i /= base;
    }
    r
}

pub struct Renderer {
    pub gl: GL,
    pub canvas: HtmlCanvasElement,
    chunk: Program,
    water: Program,
    inst: Program,
    shadow: Program,
    shadow_inst: Program,
    post: Program,
    mask: Program,
    /// Where building and tree fragments between the eye and the watched
    /// people would be: drawn only while the see-through effect is on.
    mask_buf: Option<(WebGlFramebuffer, WebGlTexture)>,
    meshes: HashMap<(i32, i32), ChunkMeshes>,
    solid_batch: InstBatch,
    fx_batch: InstBatch,
    shadow_fbo: WebGlFramebuffer,
    shadow_tex: WebGlTexture,
    scene: Option<(WebGlFramebuffer, WebGlTexture, WebGlTexture, i32, i32)>,
    /// Whether the scene buffer is floating point (the browser can render to
    /// one, and the quality tier asks for it).
    hdr_ok: bool,
    hdr: bool,
    scene_hdr: bool,
    post_vao: WebGlVertexArrayObject,
    pub pixel: u32,
    /// Chunkiness added by the adaptive quality tier.
    pub boost: u32,
    pub dpr: f64,
    shadow_size: i32,
    shadow_taps: f32,
    /// The extra effects the quality tier allows, and the defines the shader
    /// programs were last compiled with.
    fx: Fx,
    defs: String,
    shadow2: Option<(WebGlFramebuffer, WebGlTexture)>,
    bloom_down: Option<Program>,
    bloom_up: Option<Program>,
    taa_prog: Option<Program>,
    part: Option<Program>,
    aurora_prog: Option<Program>,
    copy: Option<SceneCopy>,
    bloom: Vec<Target>,
    taa: Option<Taa>,
    lut: Option<WebGlTexture>,
    lut_key: [u8; 4],
    frame_no: u32,
}

fn log(msg: &str) {
    web_sys::console::log_1(&format!("meeple: {msg}").into());
}

impl Renderer {
    pub fn new(gl: GL, canvas: HtmlCanvasElement) -> Result<Renderer, String> {
        let defs = defines(&Fx::NONE, false);
        let (chunk, water, inst, post) = effect_programs(&gl, &defs)?;
        let shadow = Program::new(&gl, &SHADOW_VS.replace("SWAY", SWAY), &SHADOW_FS.replace("OCCLUDE", OCCLUDE))?;
        let shadow_inst = Program::new(&gl, SHADOW_INST_VS, &SHADOW_INST_FS.replace("BAYER", BAYER))?;
        let bloom_down = optional(&gl, "bloom", POST_VS, BLOOM_DOWN_FS);
        let bloom_up = optional(&gl, "bloom", POST_VS, BLOOM_UP_FS);
        let taa_prog = optional(&gl, "taa", POST_VS, TAA_FS);
        let part = optional(&gl, "particles", PART_VS, PART_FS);
        let aurora_prog = optional(&gl, "aurora", AURORA_VS, AURORA_FS);
        let mask = Program::new(&gl, &CHUNK_VS.replace("SWAY", SWAY), &MASK_FS.replace("OCCLUDE", OCCLUDE))?;

        // unit cube with normals, used by every instanced box
        let mut verts: Vec<f32> = Vec::new();
        let mut idx: Vec<u16> = Vec::new();
        let faces: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
            ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
            ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
            ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
            ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            ([0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
        ];
        for (n, u, v) in faces {
            let base = (verts.len() / 6) as u16;
            for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                for k in 0..3 {
                    verts.push(n[k] + u[k] * su + v[k] * sv);
                }
                verts.extend_from_slice(&n);
            }
            idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let cube_vbo = gl.create_buffer().ok_or("buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&cube_vbo));
        gl.buffer_data_with_u8_array(GL::ARRAY_BUFFER, f32_bytes(&verts), GL::STATIC_DRAW);
        let cube_ibo = gl.create_buffer().ok_or("buffer")?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&cube_ibo));
        let ib: &[u8] = unsafe { std::slice::from_raw_parts(idx.as_ptr() as *const u8, idx.len() * 2) };
        gl.buffer_data_with_u8_array(GL::ELEMENT_ARRAY_BUFFER, ib, GL::STATIC_DRAW);

        let solid_batch = Self::make_batch(&gl, &cube_vbo, &cube_ibo)?;
        let fx_batch = Self::make_batch(&gl, &cube_vbo, &cube_ibo)?;

        let (shadow_fbo, shadow_tex) = Self::make_shadow(&gl, 2048)?;

        // full-screen and particle quads are drawn from gl_VertexID with no
        // vertex data; give the VAO a dummy array on attribute 0 anyway, since
        // drawing with it disabled makes desktop GL drivers emulate it, slowly
        let post_vao = gl.create_vertex_array().ok_or("vao")?;
        gl.bind_vertex_array(Some(&post_vao));
        let dummy = gl.create_buffer().ok_or("buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&dummy));
        gl.buffer_data_with_i32(GL::ARRAY_BUFFER, 16, GL::STATIC_DRAW);
        gl.vertex_attrib_pointer_with_i32(0, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);
        gl.bind_vertex_array(None);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);

        let hdr_ok = gl.get_extension("EXT_color_buffer_float").ok().flatten().is_some()
            || gl.get_extension("EXT_color_buffer_half_float").ok().flatten().is_some();
        Ok(Renderer {
            gl,
            canvas,
            chunk,
            water,
            inst,
            shadow,
            shadow_inst,
            post,
            mask,
            mask_buf: None,
            meshes: HashMap::new(),
            solid_batch,
            fx_batch,
            shadow_fbo,
            shadow_tex,
            scene: None,
            hdr_ok,
            hdr: false,
            scene_hdr: false,
            post_vao,
            pixel: 3,
            boost: 0,
            dpr: 1.0,
            shadow_size: 2048,
            shadow_taps: 4.0,
            fx: Fx::NONE,
            defs,
            shadow2: None,
            bloom_down,
            bloom_up,
            taa_prog,
            part,
            aurora_prog,
            copy: None,
            bloom: Vec::new(),
            taa: None,
            lut: None,
            lut_key: [255; 4],
            frame_no: 0,
        })
    }

    /// Apply a quality tier's shadow settings (size 0 turns shadows off).
    pub fn set_shadows(&mut self, size: i32, taps: f32) {
        self.shadow_taps = taps;
        if size > 0 && size != self.shadow_size {
            if let Ok((fbo, tex)) = Self::make_shadow(&self.gl, size) {
                self.gl.delete_framebuffer(Some(&self.shadow_fbo));
                self.gl.delete_texture(Some(&self.shadow_tex));
                self.shadow_fbo = fbo;
                self.shadow_tex = tex;
                self.shadow_size = size;
            }
            // the close-up map follows the size of the main one
            if let Some((fbo, tex)) = self.shadow2.take() {
                self.gl.delete_framebuffer(Some(&fbo));
                self.gl.delete_texture(Some(&tex));
            }
        }
    }

    /// Turn the floating-point scene buffer (and the bloom and tone mapping
    /// that go with it) on or off; takes effect at the next `refit`.
    pub fn set_hdr(&mut self, on: bool) {
        self.hdr = on && self.hdr_ok;
    }

    /// Choose which extra effects to draw; takes effect at the next `refit`.
    pub fn set_fx(&mut self, fx: Fx) {
        self.fx = fx;
    }

    /// Whether rain, snow and falling leaves are drawn here on the GPU, so the
    /// simulation needn't make boxes for them.
    pub fn gpu_particles(&self) -> bool {
        self.fx.particles > 0 && self.part.is_some()
    }

    /// Whether the floating-point bloom chain is in use.
    fn mip_bloom(&self) -> bool {
        self.hdr && self.bloom_down.is_some() && self.bloom_up.is_some()
    }

    /// Whether temporal anti-aliasing is on: only at full resolution, where
    /// there are sub-pixel edges for it to smooth; chunky pixels would just blur.
    fn taa_on(&self) -> bool {
        self.fx.taa && self.eff_pixel() <= 1 && self.taa_prog.is_some()
    }

    /// The pixel scale actually in use: the chosen one plus the quality boost.
    pub fn eff_pixel(&self) -> u32 {
        (self.pixel + self.boost).min(8)
    }

    /// Re-derive the internal render size after `pixel`, `boost` or the effect
    /// set changed, recompiling the shaders if what they're built from changed.
    pub fn refit(&mut self) {
        self.rebuild_programs();
        let (w, h) = self.internal_size();
        self.make_scene(w, h);
    }

}
