//! Thin helpers over WebGL2.

use std::cell::RefCell;
use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec3};
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlShader, WebGlUniformLocation};

fn shader(gl: &GL, kind: u32, src: &str) -> Result<WebGlShader, String> {
    let s = gl.create_shader(kind).ok_or("create_shader failed")?;
    gl.shader_source(&s, src);
    gl.compile_shader(&s);
    if gl.get_shader_parameter(&s, GL::COMPILE_STATUS).as_bool().unwrap_or(false) {
        Ok(s)
    } else {
        Err(gl.get_shader_info_log(&s).unwrap_or_default())
    }
}

pub struct Program {
    pub p: WebGlProgram,
    gl: GL,
    locs: RefCell<HashMap<&'static str, Option<WebGlUniformLocation>>>,
}

impl Program {
    pub fn new(gl: &GL, vs: &str, fs: &str) -> Result<Program, String> {
        let v = shader(gl, GL::VERTEX_SHADER, vs).map_err(|e| format!("vertex: {e}"))?;
        let f = shader(gl, GL::FRAGMENT_SHADER, fs).map_err(|e| format!("fragment: {e}"))?;
        let p = gl.create_program().ok_or("create_program failed")?;
        gl.attach_shader(&p, &v);
        gl.attach_shader(&p, &f);
        gl.link_program(&p);
        if !gl.get_program_parameter(&p, GL::LINK_STATUS).as_bool().unwrap_or(false) {
            return Err(gl.get_program_info_log(&p).unwrap_or_default());
        }
        Ok(Program { p, gl: gl.clone(), locs: RefCell::new(HashMap::new()) })
    }

    pub fn bind(&self) {
        self.gl.use_program(Some(&self.p));
    }

    fn loc(&self, name: &'static str) -> Option<WebGlUniformLocation> {
        self.locs
            .borrow_mut()
            .entry(name)
            .or_insert_with(|| self.gl.get_uniform_location(&self.p, name))
            .clone()
    }

    pub fn mat4(&self, name: &'static str, m: &Mat4) {
        self.gl.uniform_matrix4fv_with_f32_array(self.loc(name).as_ref(), false, &m.to_cols_array());
    }
    pub fn f(&self, name: &'static str, v: f32) {
        self.gl.uniform1f(self.loc(name).as_ref(), v);
    }
    pub fn i(&self, name: &'static str, v: i32) {
        self.gl.uniform1i(self.loc(name).as_ref(), v);
    }
    pub fn v2(&self, name: &'static str, v: Vec2) {
        self.gl.uniform2f(self.loc(name).as_ref(), v.x, v.y);
    }
    pub fn v3(&self, name: &'static str, v: Vec3) {
        self.gl.uniform3f(self.loc(name).as_ref(), v.x, v.y, v.z);
    }
    pub fn v4(&self, name: &'static str, v: [f32; 4]) {
        self.gl.uniform4f(self.loc(name).as_ref(), v[0], v[1], v[2], v[3]);
    }
    pub fn v4s(&self, name: &'static str, v: &[f32]) {
        if !v.is_empty() {
            self.gl.uniform4fv_with_f32_array(self.loc(name).as_ref(), v);
        }
    }
}

pub fn f32_bytes(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}
