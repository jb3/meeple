use super::*;

/// A program that's allowed to fail to build: the feature just stays off.
pub(super) fn optional(gl: &GL, name: &str, vs: &str, fs: &str) -> Option<Program> {
    match Program::new(gl, vs, fs) {
        Ok(p) => Some(p),
        Err(e) => {
            log(&format!("{name} shader failed, effect off: {e}"));
            None
        }
    }
}

/// `#define`s for the shaders: what the tier switches on is compiled in, what
/// it doesn't costs nothing.
pub(super) fn defines(fx: &Fx, mip_bloom: bool) -> String {
    let b = |v: bool| v as u8;
    format!(
        "#define FX_CASCADE {}\n#define FX_CLOUDS {}\n#define FX_MATERIALS {}\n#define FX_WATER {}\n#define FX_SSAO {}\n#define FX_SSAO_TAPS {}\n#define FX_RAYS {}\n#define FX_RAYS_STEPS {}\n#define FX_DOF {}\n#define FX_MIPBLOOM {}\n#define FX_GRADE {}\n#define FX_SKY {}\n#define FX_CAB {}\n#define FX_FOGSCATTER {}\n",
        b(fx.cascades),
        b(fx.clouds),
        b(fx.materials),
        fx.water,
        fx.ssao,
        if fx.ssao >= 2 { 14 } else { 8 },
        fx.rays,
        if fx.rays >= 2 { 14 } else { 8 },
        b(fx.dof),
        b(mip_bloom),
        b(fx.grade),
        fx.sky,
        b(fx.cab),
        b(fx.fog_scatter),
    )
}

/// Puts `defs` on the line after `#version`.
pub(super) fn inject(src: &str, defs: &str) -> String {
    match src.split_once('\n') {
        Some((v, rest)) => format!("{v}\n{defs}\n{rest}"),
        None => src.to_string(),
    }
}

pub(super) fn lit_fs(body: &str, defs: &str) -> String {
    format!("#version 300 es\n{defs}\n{}\n{body}", COMMON.replace("BAYER", BAYER))
}

/// The programs whose source depends on the effect set: chunks, water,
/// instanced boxes and the post pass.
pub(super) fn effect_programs(gl: &GL, defs: &str) -> Result<(Program, Program, Program, Program), String> {
    let chunk = Program::new(gl, &CHUNK_VS.replace("SWAY", SWAY), &lit_fs(&CHUNK_FS.replace("OCCLUDE", OCCLUDE), defs))?;
    let water = Program::new(gl, WATER_VS, &lit_fs(WATER_FS, defs))?;
    let inst = Program::new(gl, INST_VS, &lit_fs(INST_FS, defs))?;
    let post = Program::new(gl, POST_VS, &inject(POST_FS, defs))?;
    Ok((chunk, water, inst, post))
}

impl Renderer {
    pub(super) fn rebuild_programs(&mut self) {
        let defs = defines(&self.fx, self.mip_bloom());
        if defs == self.defs {
            return;
        }
        match effect_programs(&self.gl, &defs) {
            Ok((chunk, water, inst, post)) => {
                self.chunk = chunk;
                self.water = water;
                self.inst = inst;
                self.post = post;
                self.defs = defs;
            }
            Err(e) => {
                // a driver that can't take the effects still gets the plain picture
                log(&format!("effect shaders failed, using the plain look: {e}"));
                self.fx = Fx::NONE;
                let plain = defines(&Fx::NONE, false);
                if plain != self.defs {
                    if let Ok((chunk, water, inst, post)) = effect_programs(&self.gl, &plain) {
                        self.chunk = chunk;
                        self.water = water;
                        self.inst = inst;
                        self.post = post;
                        self.defs = plain;
                    }
                }
            }
        }
    }

    pub(super) fn make_batch(gl: &GL, cube_vbo: &WebGlBuffer, cube_ibo: &WebGlBuffer) -> Result<InstBatch, String> {
        let vao = gl.create_vertex_array().ok_or("vao")?;
        gl.bind_vertex_array(Some(&vao));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(cube_vbo));
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 24, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 24, 12);
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(cube_ibo));
        let vbo = gl.create_buffer().ok_or("buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
        let stride = (FLOATS * 4) as i32;
        for k in 0..3u32 {
            gl.enable_vertex_attrib_array(2 + k);
            gl.vertex_attrib_pointer_with_i32(2 + k, 4, GL::FLOAT, false, stride, (k * 16) as i32);
            gl.vertex_attrib_divisor(2 + k, 1);
        }
        gl.bind_vertex_array(None);
        Ok(InstBatch { vao, vbo, count: 0 })
    }
}
