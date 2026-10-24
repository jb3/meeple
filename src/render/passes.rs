use super::*;

impl Renderer {
    pub(super) fn upload_batch(gl: &GL, batch: &mut InstBatch, b: &Boxes) {
        batch.count = b.len() as i32;
        if batch.count > 0 {
            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&batch.vbo));
            gl.buffer_data_with_u8_array(GL::ARRAY_BUFFER, f32_bytes(&b.data), GL::STREAM_DRAW);
        }
    }

    pub(super) fn draw_batch(gl: &GL, batch: &InstBatch) {
        if batch.count > 0 {
            gl.bind_vertex_array(Some(&batch.vao));
            gl.draw_elements_instanced_with_i32(GL::TRIANGLES, 36, GL::UNSIGNED_SHORT, 0, batch.count);
        }
    }

    pub(super) fn lighting(p: &Program, f: &Frame, vp: &Mat4, shadow_size: i32, shadow_taps: f32, cascade: Option<i32>) {
        p.mat4("u_vp", vp);
        p.mat4("u_lightVP", &f.light_vp);
        p.v2("u_shadowTexel", Vec2::splat(1.0 / shadow_size as f32));
        p.f("u_shadowTaps", shadow_taps);
        p.v3("u_sunDir", f.sun_dir);
        p.v3("u_sunCol", f.sun_col);
        p.v3("u_ambSky", f.amb_sky);
        p.v3("u_ambGround", f.amb_ground);
        p.f("u_time", f.time);
        p.f("u_cloud", f.cloud);
        p.f("u_night", f.night);
        p.f("u_lamp", f.lamp);
        p.v2("u_wind", f.wind);
        p.v4s("u_lights", &f.lights);
        p.i("u_nlights", (f.lights.len() / 4) as i32);
        p.i("u_shadow", 0);
        if let Some(size) = cascade {
            p.i("u_shadow2", 1);
            p.mat4("u_lightVP2", &f.light_vp2);
            p.v2("u_shadowTexel2", Vec2::splat(1.0 / size as f32));
        }
    }

    /// Rebuild the grading table when the hour or season has moved it enough.
    pub(super) fn update_lut(&mut self, p: grade::Params) {
        let key = grade::quantise(p);
        if self.lut.is_some() && key == self.lut_key {
            return;
        }
        let gl = &self.gl;
        let n = grade::SIZE as i32;
        if self.lut.is_none() {
            let Some(t) = gl.create_texture() else { return };
            gl.bind_texture(GL::TEXTURE_3D, Some(&t));
            gl.tex_storage_3d(GL::TEXTURE_3D, 1, GL::RGBA8, n, n, n);
            for (k, v) in [
                (GL::TEXTURE_MIN_FILTER, GL::LINEAR),
                (GL::TEXTURE_MAG_FILTER, GL::LINEAR),
                (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
                (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
                (GL::TEXTURE_WRAP_R, GL::CLAMP_TO_EDGE),
            ] {
                gl.tex_parameteri(GL::TEXTURE_3D, k, v as i32);
            }
            self.lut = Some(t);
        }
        // the table is built from the rounded parameters, so it's the same one every time
        let rounded = key.map(|v| v as f32 / 24.0);
        let data = grade::build(rounded);
        gl.bind_texture(GL::TEXTURE_3D, self.lut.as_ref());
        let _ = gl.tex_sub_image_3d_with_opt_u8_array(GL::TEXTURE_3D, 0, 0, 0, 0, n, n, n, GL::RGBA, GL::UNSIGNED_BYTE, Some(&data));
        self.lut_key = key;
    }

    /// One shadow pass from the light's point of view.
    pub(super) fn shadow_pass(&self, gl: &GL, fbo: &WebGlFramebuffer, size: i32, vp: &Mat4, f: &Frame, solid: &Boxes) {
        let _ = solid;
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(fbo));
        gl.viewport(0, 0, size, size);
        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);
        gl.depth_mask(true);
        gl.disable(GL::CULL_FACE);
        gl.disable(GL::BLEND);
        gl.clear(GL::DEPTH_BUFFER_BIT);
        gl.enable(GL::POLYGON_OFFSET_FILL);
        gl.polygon_offset(1.5, 3.0);
        self.shadow.bind();
        self.shadow.mat4("u_vp", vp);
        self.shadow.f("u_bare", f.bare);
        self.shadow.f("u_season", f.season);
        self.shadow.f("u_time", f.time);
        self.shadow.v2("u_wind", f.wind);
        self.shadow.v4s("u_focusPts", &f.foci);
        self.shadow.i("u_nFocus", (f.foci.len() / 4) as i32);
        self.shadow.v3("u_eye", f.eye);
        self.shadow.f("u_clear", f.clear);
        for m in self.meshes.values() {
            if let Some(s) = &m.solid {
                gl.bind_vertex_array(Some(&s.vao));
                gl.draw_elements_with_i32(GL::TRIANGLES, s.count, GL::UNSIGNED_INT, 0);
            }
        }
        self.shadow_inst.bind();
        self.shadow_inst.mat4("u_vp", vp);
        Self::draw_batch(gl, &self.solid_batch);
        gl.disable(GL::POLYGON_OFFSET_FILL);
    }

    /// Bright-pass, downsample and blur the scene into the bloom chain.
    pub(super) fn bloom_pass(&self, gl: &GL, src: &WebGlTexture, w: i32, h: i32) {
        let (Some(down), Some(up)) = (&self.bloom_down, &self.bloom_up) else { return };
        gl.bind_vertex_array(Some(&self.post_vao));
        gl.disable(GL::BLEND);
        down.bind();
        down.i("u_src", 0);
        gl.active_texture(GL::TEXTURE0);
        for (i, t) in self.bloom.iter().enumerate() {
            let (tex, sw, sh) = if i == 0 { (src, w, h) } else { (&self.bloom[i - 1].tex, self.bloom[i - 1].w, self.bloom[i - 1].h) };
            gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&t.fbo));
            gl.viewport(0, 0, t.w, t.h);
            gl.bind_texture(GL::TEXTURE_2D, Some(tex));
            down.v2("u_texel", Vec2::new(1.0 / sw as f32, 1.0 / sh as f32));
            down.f("u_first", if i == 0 { 1.0 } else { 0.0 });
            gl.draw_arrays(GL::TRIANGLES, 0, 3);
        }
        gl.enable(GL::BLEND);
        gl.blend_func(GL::ONE, GL::ONE);
        up.bind();
        up.i("u_src", 0);
        up.f("u_w", 0.7);
        for i in (0..self.bloom.len().saturating_sub(1)).rev() {
            let (dst, from) = (&self.bloom[i], &self.bloom[i + 1]);
            gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&dst.fbo));
            gl.viewport(0, 0, dst.w, dst.h);
            gl.bind_texture(GL::TEXTURE_2D, Some(&from.tex));
            up.v2("u_texel", Vec2::new(1.0 / from.w as f32, 1.0 / from.h as f32));
            gl.draw_arrays(GL::TRIANGLES, 0, 3);
        }
        gl.disable(GL::BLEND);
    }

    pub fn render(&mut self, f: &Frame, solid: &Boxes, fx: &Boxes) {
        let gl = self.gl.clone();
        Self::upload_batch(&gl, &mut self.solid_batch, solid);
        Self::upload_batch(&gl, &mut self.fx_batch, fx);
        let (scene_fbo, color, depth, w, h) = match self.scene.as_ref() {
            Some((a, b, c, d, e)) => (a.clone(), b.clone(), c.clone(), *d, *e),
            None => return,
        };
        self.frame_no = self.frame_no.wrapping_add(1);

        // a sub-pixel wobble of the whole picture, for temporal anti-aliasing
        let taa_ready = self.taa_on() && self.taa.is_some();
        let (vp, inv_vp) = if taa_ready {
            let k = self.frame_no % 8 + 1;
            let j = Mat4::from_translation(Vec3::new((halton(k, 2) - 0.5) * 2.0 / w as f32, (halton(k, 3) - 0.5) * 2.0 / h as f32, 0.0));
            let v = j * f.vp;
            (v, v.inverse())
        } else {
            (f.vp, f.inv_vp)
        };

        // 1. shadow maps from the sun (or moon): the whole view, and a tighter
        // one round the camera when the tier has the budget
        let (ss, st) = (self.shadow_size, self.shadow_taps);
        let cascade = self.fx.cascades && st > 0.0;
        if cascade && self.shadow2.is_none() {
            self.shadow2 = Self::make_shadow(&gl, ss).ok();
        }
        let cascade = cascade && self.shadow2.is_some();
        if st > 0.0 {
            self.shadow_pass(&gl, &self.shadow_fbo, ss, &f.light_vp, f, solid);
            if let (true, Some((fbo2, _))) = (cascade, self.shadow2.as_ref()) {
                self.shadow_pass(&gl, fbo2, ss, &f.light_vp2, f, solid);
            }
        }
        let casc = if cascade { Some(ss) } else { None };

        // 2. scene at internal (possibly chunky) resolution
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&scene_fbo));
        gl.viewport(0, 0, w, h);
        let sky = f.sky;
        gl.clear_color(sky.x, sky.y, sky.z, 1.0);
        gl.enable(GL::DEPTH_TEST);
        gl.depth_mask(true);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        gl.active_texture(GL::TEXTURE1);
        if let Some((_, t2)) = self.shadow2.as_ref().filter(|_| cascade) {
            gl.bind_texture(GL::TEXTURE_2D, Some(t2));
        }
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.shadow_tex));

        let c = &self.chunk;
        c.bind();
        Self::lighting(c, f, &vp, ss, st, casc);
        c.f("u_crop", f.crop);
        c.v3("u_grassTint", f.grass_tint);
        c.v3("u_leafTint", f.leaf_tint);
        c.f("u_autumn", f.autumn);
        c.f("u_bare", f.bare);
        c.v4s("u_focusPts", &f.foci);
        c.i("u_nFocus", (f.foci.len() / 4) as i32);
        c.v3("u_eye", f.eye);
        c.f("u_clear", f.clear);
        c.f("u_blossom", f.blossom);
        c.f("u_snow", f.snow);
        c.f("u_wet", f.wet);
        c.f("u_ripe", f.ripe);
        c.f("u_winLit", f.win_lit);
        c.v3("u_sky", f.sky);
        c.f("u_rain", f.rain);
        c.f("u_peakSnow", f.peak_snow);
        c.f("u_season", f.season);
        for m in self.meshes.values() {
            if let Some(s) = &m.solid {
                gl.bind_vertex_array(Some(&s.vao));
                gl.draw_elements_with_i32(GL::TRIANGLES, s.count, GL::UNSIGNED_INT, 0);
            }
        }

        self.inst.bind();
        Self::lighting(&self.inst, f, &vp, ss, st, casc);
        Self::draw_batch(&gl, &self.solid_batch);
        Self::draw_batch(&gl, &self.fx_batch);

        // the water looks at the scene as it stands: copy it aside first
        let looking = self.fx.water >= 2 && self.copy.is_some();
        if let (true, Some(cp)) = (looking, self.copy.as_ref()) {
            gl.bind_framebuffer(GL::READ_FRAMEBUFFER, Some(&scene_fbo));
            gl.bind_framebuffer(GL::DRAW_FRAMEBUFFER, Some(&cp.fbo));
            gl.blit_framebuffer(0, 0, w, h, 0, 0, w, h, GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT, GL::NEAREST);
            gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&scene_fbo));
            gl.active_texture(GL::TEXTURE2);
            gl.bind_texture(GL::TEXTURE_2D, Some(&cp.color));
            gl.active_texture(GL::TEXTURE3);
            gl.bind_texture(GL::TEXTURE_2D, Some(&cp.depth));
            gl.active_texture(GL::TEXTURE0);
        }

        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        gl.depth_mask(false);
        self.water.bind();
        Self::lighting(&self.water, f, &vp, ss, st, casc);
        self.water.f("u_ice", f.ice);
        self.water.f("u_tide", f.tide);
        self.water.v3("u_eye", f.eye);
        self.water.v3("u_sky", f.sky);
        if looking {
            self.water.i("u_sceneColor", 2);
            self.water.i("u_sceneDepth", 3);
            self.water.v2("u_invSize", Vec2::new(1.0 / w as f32, 1.0 / h as f32));
            self.water.f("u_depthRange", f.depth_range);
        }
        for m in self.meshes.values() {
            if let Some(s) = &m.water {
                gl.bind_vertex_array(Some(&s.vao));
                gl.draw_elements_with_i32(GL::TRIANGLES, s.count, GL::UNSIGNED_INT, 0);
            }
        }

        // rain, snow and falling leaves, worked out on the GPU
        if let (true, Some(pp)) = (self.fx.particles > 0, self.part.as_ref()) {
            let area = (2.0 * f.part_ext).powi(2);
            let cap = self.fx.particles as f32;
            let kinds: [(i32, f32); 3] = [
                (0, if f.rain > 0.02 && !f.snowing { area * 0.08 * f.rain } else { 0.0 }),
                (1, if f.rain > 0.02 && f.snowing { area * 0.06 * f.rain } else { 0.0 }),
                (2, if f.leaves { area * 0.012 } else { 0.0 }),
            ];
            if kinds.iter().any(|k| k.1 >= 1.0) {
                gl.disable(GL::CULL_FACE);
                pp.bind();
                pp.mat4("u_vp", &vp);
                pp.f("u_time", f.time);
                pp.v2("u_center", f.part_center);
                pp.f("u_ext", f.part_ext);
                pp.f("u_ground", f.ground_y);
                pp.v2("u_wind", f.wind);
                pp.f("u_texel", 2.0 * f.zoom / h as f32);
                pp.v3("u_right", f.cam_right);
                pp.v3("u_up", f.cam_up);
                pp.f("u_br", f.part_br);
                pp.f("u_rain", f.rain);
                gl.bind_vertex_array(Some(&self.post_vao));
                for (kind, n) in kinds {
                    if n >= 1.0 {
                        pp.i("u_kind", kind);
                        gl.draw_arrays_instanced(GL::TRIANGLE_STRIP, 0, 4, n.min(cap) as i32);
                    }
                }
                gl.enable(GL::CULL_FACE);
            }
        }
        // northern lights: ribbons hung in the world, added to the picture and hidden
        // by anything in front of them
        if let (true, Some(ap)) = (f.aurora > 0.02 && !f.curtains.is_empty() && self.fx.sky > 0, self.aurora_prog.as_ref()) {
            let n = (f.curtains.len() / 8).min(8);
            gl.disable(GL::CULL_FACE);
            gl.blend_func(GL::ONE, GL::ONE);
            ap.bind();
            ap.mat4("u_vp", &vp);
            ap.f("u_time", f.time);
            ap.f("u_strength", f.aurora);
            ap.i("u_n", n as i32);
            let mut data: Vec<f32> = f.curtains[..n * 8].to_vec();
            data.resize(64, 0.0);
            ap.v4s("u_c", &data);
            gl.bind_vertex_array(Some(&self.post_vao));
            gl.draw_arrays_instanced(GL::TRIANGLE_STRIP, 0, 4, (n * 120) as i32);
            gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
            gl.enable(GL::CULL_FACE);
        }
        gl.depth_mask(true);
        gl.disable(GL::BLEND);

        // the see-through outline: mark where the removed buildings would be
        let masked = f.clear > 0.01;
        if let (true, Some((mfbo, _))) = (masked, self.mask_buf.as_ref()) {
            gl.bind_framebuffer(GL::FRAMEBUFFER, Some(mfbo));
            gl.viewport(0, 0, w, h);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(GL::COLOR_BUFFER_BIT);
            gl.disable(GL::DEPTH_TEST);
            gl.enable(GL::CULL_FACE);
            let m = &self.mask;
            m.bind();
            m.mat4("u_vp", &vp);
            m.f("u_time", f.time);
            m.v2("u_wind", f.wind);
            m.f("u_crop", f.crop);
            m.f("u_bare", f.bare);
            m.v4s("u_focusPts", &f.foci);
            m.i("u_nFocus", (f.foci.len() / 4) as i32);
            m.v3("u_eye", f.eye);
            m.f("u_clear", f.clear);
            for mesh in self.meshes.values() {
                if let Some(s) = &mesh.solid {
                    gl.bind_vertex_array(Some(&s.vao));
                    gl.draw_elements_with_i32(GL::TRIANGLES, s.count, GL::UNSIGNED_INT, 0);
                }
            }
        }

        // temporal anti-aliasing: blend with last frame's picture
        gl.disable(GL::DEPTH_TEST);
        gl.disable(GL::CULL_FACE);
        gl.disable(GL::BLEND);
        let mut src = color.clone();
        if let (true, Some(tp)) = (taa_ready, self.taa_prog.as_ref()) {
            if let Some(t) = self.taa.as_mut() {
                let (wi, hi) = (t.next, 1 - t.next);
                gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&t.targets[wi].fbo));
                gl.viewport(0, 0, w, h);
                tp.bind();
                tp.i("u_cur", 0);
                tp.i("u_hist", 1);
                tp.i("u_depth", 2);
                gl.active_texture(GL::TEXTURE0);
                gl.bind_texture(GL::TEXTURE_2D, Some(&color));
                gl.active_texture(GL::TEXTURE1);
                gl.bind_texture(GL::TEXTURE_2D, Some(&t.targets[hi].tex));
                gl.active_texture(GL::TEXTURE2);
                gl.bind_texture(GL::TEXTURE_2D, Some(&depth));
                tp.v2("u_texel", Vec2::new(1.0 / w as f32, 1.0 / h as f32));
                // reproject with the matrices as they are, jitter and all left out, so a still
                // view looks up exactly the same texel of the last frame every time
                tp.mat4("u_invVP", &f.inv_vp);
                tp.mat4("u_prevVP", &t.prev_vp);
                tp.f("u_first", if t.valid { 0.0 } else { 1.0 });
                gl.bind_vertex_array(Some(&self.post_vao));
                gl.draw_arrays(GL::TRIANGLES, 0, 3);
                src = t.targets[wi].tex.clone();
                t.next = hi;
                t.valid = true;
                t.prev_vp = f.vp;
                gl.active_texture(GL::TEXTURE0);
            }
        }

        // bloom: a chain of ever blurrier, ever smaller copies of what's bright
        let mip = self.mip_bloom() && self.bloom.len() == 4 && self.scene_hdr;
        if mip {
            self.bloom_pass(&gl, &src, w, h);
        }
        if self.fx.grade {
            self.update_lut(f.grade);
        }

        // 3. upscale + post-process to the canvas
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        gl.viewport(0, 0, self.canvas.width() as i32, self.canvas.height() as i32);
        let p = &self.post;
        p.bind();
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&src));
        gl.active_texture(GL::TEXTURE1);
        gl.bind_texture(GL::TEXTURE_2D, Some(&depth));
        // every sampler gets its own unit, whether or not it's used this frame
        p.i("u_color", 0);
        p.i("u_depth", 1);
        p.i("u_mask", 2);
        p.i("u_bloom", 3);
        p.i("u_bloomWide", 4);
        p.i("u_lut", 5);
        p.i("u_shadowP", 6);
        if let (true, Some((_, mtex))) = (masked, self.mask_buf.as_ref()) {
            gl.active_texture(GL::TEXTURE2);
            gl.bind_texture(GL::TEXTURE_2D, Some(mtex));
            p.f("u_maskOn", 1.0);
        } else {
            p.f("u_maskOn", 0.0);
        }
        if mip {
            gl.active_texture(GL::TEXTURE3);
            gl.bind_texture(GL::TEXTURE_2D, Some(&self.bloom[0].tex));
            gl.active_texture(GL::TEXTURE4);
            gl.bind_texture(GL::TEXTURE_2D, Some(&self.bloom[2].tex));
        }
        if self.fx.grade {
            gl.active_texture(GL::TEXTURE5);
            gl.bind_texture(GL::TEXTURE_3D, self.lut.as_ref());
        }
        if self.fx.rays > 0 && st > 0.0 {
            gl.active_texture(GL::TEXTURE6);
            gl.bind_texture(GL::TEXTURE_2D, Some(&self.shadow_tex));
            p.mat4("u_lightVP", &f.light_vp);
            p.f("u_rayLen", 26.0);
        }
        gl.active_texture(GL::TEXTURE0);
        let texel_world = 2.0 * f.zoom / h as f32;
        if self.fx.dof {
            let scale = self.eff_pixel() as f64 * self.dpr;
            // far out, every terrace is a texel or two across and the blur would just smudge them
            let near = 1.0 - ((f.zoom - 60.0) / 80.0).clamp(0.0, 1.0);
            p.f("u_dofR", (6.0 / scale).clamp(1.0, 3.5) as f32 * near * (1.0 + 1.1 * f.postcard));
            p.f("u_focusZ", vp.project_point3(f.focus).z * 0.5 + 0.5);
            p.f("u_dofSpan", f.zoom * 1.4 / (1.0 + 0.9 * f.postcard));
        }
        if self.fx.ssao > 0 {
            p.f("u_aoRad", (1.3 / texel_world).clamp(2.0, 10.0));
            // and for the same reason there's nothing for occlusion to say when zoomed far out
            p.f("u_aoK", 0.6 * (1.0 - ((texel_world - 0.25) / 0.45).clamp(0.0, 1.0)));
            p.f("u_aoMin", 0.4 + texel_world * 0.8);
        }
        p.v2("u_texel", Vec2::new(1.0 / w as f32, 1.0 / h as f32));
        p.mat4("u_invVP", &inv_vp);
        p.v2("u_shift", f.shift);
        p.v2("u_wind", f.wind);
        p.f("u_night", f.night);
        p.f("u_hdr", if self.scene_hdr { 1.0 } else { 0.0 });
        p.f("u_rainbow", f.rainbow);
        p.v3("u_bowA", f.bow_a);
        p.v2("u_bowN", f.bow_n);
        p.f("u_bowR", f.bow_r);
        p.f("u_aurora", f.aurora);
        p.v4("u_meteor", f.meteor);
        p.f("u_meteorT", f.meteor_t);
        p.f("u_tilt", f.tilt);
        p.v3("u_fogCol", f.sky);
        p.f("u_fog", f.fog);
        p.f("u_flash", f.flash);
        p.f("u_outline", if self.eff_pixel() > 1 { 0.32 } else { 0.18 });
        p.f("u_rain", f.rain);
        p.f("u_time", f.time);
        p.v3("u_eye", f.eye);
        p.v3("u_camR", f.cam_right);
        p.v3("u_camU", f.cam_up);
        p.v3("u_sunDir", f.sun_dir);
        p.v3("u_sunCol", f.sun_col);
        p.f("u_cloud", f.cloud);
        p.f("u_moon", f.moon);
        p.f("u_depthRange", f.depth_range);
        gl.bind_vertex_array(Some(&self.post_vao));
        gl.draw_arrays(GL::TRIANGLES, 0, 3);
        gl.active_texture(GL::TEXTURE0);
    }
}
