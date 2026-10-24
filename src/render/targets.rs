use super::*;

impl Renderer {
    pub(super) fn make_shadow(gl: &GL, size: i32) -> Result<(WebGlFramebuffer, WebGlTexture), String> {
        let tex = gl.create_texture().ok_or("texture")?;
        gl.bind_texture(GL::TEXTURE_2D, Some(&tex));
        gl.tex_storage_2d(GL::TEXTURE_2D, 1, GL::DEPTH_COMPONENT24, size, size);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_COMPARE_MODE, GL::COMPARE_REF_TO_TEXTURE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_COMPARE_FUNC, GL::LEQUAL as i32);
        let fbo = gl.create_framebuffer().ok_or("fbo")?;
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&fbo));
        gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::TEXTURE_2D, Some(&tex), 0);
        gl.draw_buffers(&js_sys::Array::of1(&wasm_bindgen::JsValue::from(GL::NONE)));
        gl.read_buffer(GL::NONE);
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        Ok((fbo, tex))
    }

    /// Resize the canvas backing store; returns the internal render size.
    pub fn resize(&mut self, css_w: f64, css_h: f64, dpr: f64) -> (i32, i32) {
        self.dpr = dpr.min(2.0);
        let cw = (css_w * self.dpr).round() as u32;
        let ch = (css_h * self.dpr).round() as u32;
        if self.canvas.width() != cw || self.canvas.height() != ch {
            self.canvas.set_width(cw);
            self.canvas.set_height(ch);
        }
        let (w, h) = self.internal_size();
        let stale = match &self.scene {
            Some((_, _, _, sw, sh)) => *sw != w || *sh != h || self.scene_hdr != self.hdr,
            None => true,
        };
        if stale {
            self.make_scene(w, h);
        }
        (w, h)
    }

    pub fn internal_size(&self) -> (i32, i32) {
        let cw = self.canvas.width() as f64;
        let ch = self.canvas.height() as f64;
        if self.eff_pixel() <= 1 {
            (cw as i32, ch as i32)
        } else {
            let s = self.eff_pixel() as f64 * self.dpr;
            ((cw / s).ceil() as i32, (ch / s).ceil() as i32)
        }
    }

    pub(super) fn free_target(gl: &GL, t: Target) {
        gl.delete_framebuffer(Some(&t.fbo));
        gl.delete_texture(Some(&t.tex));
    }

    pub(super) fn free_aux(&mut self) {
        let gl = &self.gl;
        if let Some(c) = self.copy.take() {
            gl.delete_framebuffer(Some(&c.fbo));
            gl.delete_texture(Some(&c.color));
            gl.delete_texture(Some(&c.depth));
        }
        for t in self.bloom.drain(..) {
            Self::free_target(gl, t);
        }
        if let Some(t) = self.taa.take() {
            for t in t.targets {
                Self::free_target(gl, t);
            }
        }
    }

    /// A texture with a framebuffer to draw into it.
    pub(super) fn make_target(&self, w: i32, h: i32, fmt: u32, linear: bool) -> Option<Target> {
        let gl = &self.gl;
        let tex = gl.create_texture()?;
        gl.bind_texture(GL::TEXTURE_2D, Some(&tex));
        gl.tex_storage_2d(GL::TEXTURE_2D, 1, fmt, w.max(1), h.max(1));
        let f = if linear { GL::LINEAR } else { GL::NEAREST } as i32;
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, f);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, f);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        let fbo = gl.create_framebuffer()?;
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&fbo));
        gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&tex), 0);
        let ok = gl.check_framebuffer_status(GL::FRAMEBUFFER) == GL::FRAMEBUFFER_COMPLETE;
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        if !ok {
            gl.delete_framebuffer(Some(&fbo));
            gl.delete_texture(Some(&tex));
            return None;
        }
        Some(Target { fbo, tex, w: w.max(1), h: h.max(1) })
    }

    /// The buffers the optional effects draw into, at the scene's size.
    pub(super) fn make_aux(&mut self, w: i32, h: i32) {
        self.free_aux();
        let fmt = if self.scene_hdr { GL::RGBA16F } else { GL::RGBA8 };
        // water looks at what was drawn before it
        if self.fx.water >= 2 {
            let gl = &self.gl;
            let mk = |fmt: u32| {
                let t = gl.create_texture()?;
                gl.bind_texture(GL::TEXTURE_2D, Some(&t));
                gl.tex_storage_2d(GL::TEXTURE_2D, 1, fmt, w.max(1), h.max(1));
                for (k, v) in [
                    (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
                    (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
                    (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
                    (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
                ] {
                    gl.tex_parameteri(GL::TEXTURE_2D, k, v as i32);
                }
                Some(t)
            };
            if let (Some(color), Some(depth), Some(fbo)) = (mk(fmt), mk(GL::DEPTH_COMPONENT24), gl.create_framebuffer()) {
                gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&fbo));
                gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&color), 0);
                gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::TEXTURE_2D, Some(&depth), 0);
                let ok = gl.check_framebuffer_status(GL::FRAMEBUFFER) == GL::FRAMEBUFFER_COMPLETE;
                gl.bind_framebuffer(GL::FRAMEBUFFER, None);
                if ok {
                    self.copy = Some(SceneCopy { fbo, color, depth });
                } else {
                    gl.delete_framebuffer(Some(&fbo));
                    gl.delete_texture(Some(&color));
                    gl.delete_texture(Some(&depth));
                }
            }
        }
        if self.mip_bloom() {
            let (mut lw, mut lh) = (w, h);
            for _ in 0..4 {
                lw = (lw / 2).max(1);
                lh = (lh / 2).max(1);
                match self.make_target(lw, lh, GL::RGBA16F, true) {
                    Some(t) => self.bloom.push(t),
                    None => {
                        for t in self.bloom.drain(..) {
                            Self::free_target(&self.gl, t);
                        }
                        break;
                    }
                }
            }
        }
        if self.taa_on() {
            if let (Some(a), Some(b)) = (self.make_target(w, h, fmt, true), self.make_target(w, h, fmt, true)) {
                self.taa = Some(Taa { targets: [a, b], next: 0, valid: false, prev_vp: Mat4::IDENTITY });
            }
        }
    }

    pub(super) fn make_scene(&mut self, w: i32, h: i32) {
        self.free_aux();
        let gl = &self.gl;
        if let Some((fbo, tex)) = self.mask_buf.take() {
            gl.delete_framebuffer(Some(&fbo));
            gl.delete_texture(Some(&tex));
        }
        if let Some((fbo, c, d, _, _)) = self.scene.take() {
            gl.delete_framebuffer(Some(&fbo));
            gl.delete_texture(Some(&c));
            gl.delete_texture(Some(&d));
        }
        let tex = |fmt: u32| {
            let t = gl.create_texture().unwrap();
            gl.bind_texture(GL::TEXTURE_2D, Some(&t));
            gl.tex_storage_2d(GL::TEXTURE_2D, 1, fmt, w.max(1), h.max(1));
            gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
            gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
            gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
            gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
            t
        };
        let hdr = self.hdr;
        let color = tex(if hdr { GL::RGBA16F } else { GL::RGBA8 });
        let depth = tex(GL::DEPTH_COMPONENT24);
        let fbo = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&fbo));
        gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&color), 0);
        gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::TEXTURE_2D, Some(&depth), 0);
        let complete = gl.check_framebuffer_status(GL::FRAMEBUFFER) == GL::FRAMEBUFFER_COMPLETE;
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        if hdr && !complete {
            // this GPU won't render to a float target after all
            gl.delete_framebuffer(Some(&fbo));
            gl.delete_texture(Some(&color));
            gl.delete_texture(Some(&depth));
            self.hdr = false;
            self.hdr_ok = false;
            return self.make_scene(w, h);
        }
        self.scene_hdr = hdr;
        // the see-through mask, same size as the scene
        let mtex = tex(GL::R8);
        let mfbo = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&mfbo));
        gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&mtex), 0);
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        self.mask_buf = Some((mfbo, mtex));
        self.scene = Some((fbo, color, depth, w, h));
        self.make_aux(w, h);
    }
}
