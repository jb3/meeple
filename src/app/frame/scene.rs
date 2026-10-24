use super::super::*;

/// What `frame` has worked out by the time it assembles the renderer's view
/// of the scene.
pub(super) struct FrameInputs<'a> {
    pub season: f32,
    pub hour: f32,
    pub tide: f32,
    /// Particle brightness (daylight dimmed by cloud).
    pub brightness: f32,
    pub lamps: &'a [[f32; 3]],
    pub forge: &'a [[f32; 3]],
    pub glows: &'a [[f32; 4]],
}

impl App {
    /// Everything the renderer needs to know about the moment: weather-tinted
    /// light, the nearest point lights, season colours, the see-through foci
    /// (people being watched, a house being peeked into), the rainbow and the
    /// camera matrices.
    pub(super) fn build_frame(&mut self, dt: f32, here: Vec2, view_r: f32, light: &crate::sim::clock::Light, inp: FrameInputs) -> Frame {
        let FrameInputs { season, hour, tide, .. } = inp;
        let gen = &self.world.gen;
        let w = &self.weather;
        // lighting, tinted by weather
        let lum = |v: Vec3| Vec3::splat(v.dot(Vec3::new(0.3, 0.59, 0.11)));
        let gloom = w.cloud * 0.5 + w.storm * 0.25 + w.precip * 0.1;
        let ecl = Skies::eclipse(self.seed, self.clock.days);
        let sun_col = light.color * (1.0 - gloom).max(0.25) * (1.0 - ecl * 0.93);
        let amb_sky = light.amb_sky.lerp(lum(light.amb_sky), w.cloud * 0.4) * (1.0 - w.storm * 0.25) * (1.0 - ecl * 0.55);
        let amb_ground = light.amb_ground.lerp(lum(light.amb_ground), w.cloud * 0.4) * (1.0 - w.storm * 0.25) * (1.0 - ecl * 0.55);
        let sky = light.sky.lerp(lum(light.sky) * Vec3::new(0.95, 0.98, 1.05), w.cloud * 0.55) * (1.0 - w.storm * 0.3) * (1.0 - ecl * 0.8);
        let mist = (1.0 - smoothstep(7.0, 9.5, hour)) * smoothstep(4.0, 6.0, hour) * 0.35;
        let fog = (w.fog * 0.85 + w.precip * 0.22 + mist).min(0.95);

        // street lamps, plus forges glowing a little dimmer and flickering
        let flicker = 1.1 + (self.time * 9.0).sin() * 0.12 + (self.time * 23.0).sin() * 0.08;
        let mut lights: Vec<(f32, [f32; 4])> = inp.lamps
            .iter()
            .map(|l| [l[0], l[1], l[2], 1.7])
            .chain(inp.forge.iter().map(|l| [l[0], l[1] + 0.6, l[2], flicker]))
            .chain(inp.glows.iter().copied())
            .map(|l| (Vec2::new(l[0], l[2]).distance_squared(here), l))
            .collect();
        lights.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let lights: Vec<f32> = lights.iter().take(16).flat_map(|(_, l)| *l).collect();

        let win_lit: f32 = if (17.0..23.0).contains(&hour) {
            0.65
        } else if (5.0..8.0).contains(&hour) {
            0.35
        } else {
            0.12
        };
        // a storm darkens the day enough to light the windows, and so does an eclipse
        let win_lit = win_lit.max(w.storm * 0.6).max(ecl * 0.7);
        let crop = if season < 1.0 {
            0.12 + season * 0.5
        } else if season < 2.3 {
            (0.62 + (season - 1.0) * 0.5).min(1.0)
        } else if season < 2.5 {
            lerp(1.0, 0.06, (season - 2.3) / 0.2)
        } else {
            0.06
        };
        let ripe = smoothstep(1.3, 1.9, season) * (1.0 - smoothstep(2.4, 2.5, season));

        let (iw, ih) = self.r.internal_size();
        let (vp, shift) = self.cam.view_proj(if self.r.eff_pixel() > 1 { Some(ih as f32) } else { None });
        // while the autopilot is watching particular people, fade whatever
        // stands between the camera and them
        // (not for a house fire: the burning house is the thing to see)
        let fire = self.towns.get(&self.dir.follow_town).is_some_and(|t| t.happening(crate::sim::people::happenings::Kind::Fire));
        let watching = self.dir.auto
            && !fire
            && self.dir.manual_t <= 0.0
            && match self.dir.mode {
                // only while they're actually out in the open to be seen
                Mode::FollowPerson => self.towns.get(&self.dir.follow_town).is_some_and(|t| t.outdoors(self.dir.follow_idx)),
                Mode::WatchChat => self.towns.get(&self.dir.follow_town).is_some_and(|t| t.outdoors(self.dir.follow_idx) && t.outdoors(self.dir.chat_b)),
                Mode::Event | Mode::Gathering => self.locked() || self.dir.wrap_t > 0.0,
                Mode::Linger => self.dir.wed_wait || self.dir.ghost_focus.is_some(),
                _ => false,
            };
        let mut foci = if watching { self.see_through_foci() } else { Vec::new() };
        // close in on a house with someone home and its roof lifts off, to look in on them
        let aim = Vec2::new(self.cam.target.x, self.cam.target.z);
        let mut peek_at = None;
        // a postcard is of the view, not of whoever's inside it: nothing is
        // cleared away unless the postcard is of the people (a festival)
        let postcard = self.postcard_t > 0.0;
        let postcard_clear = postcard && self.dir.mode != Mode::Gathering;
        if self.peek_on && self.cam.zoom < 13.5 && self.settled && !self.photo && !postcard {
            peek_at = self.towns.values().find_map(|t| t.lit_house_near(aim).map(|l| (t.town.id, l)));
        }
        for t in self.towns.values_mut() {
            t.set_peek(match peek_at {
                Some((id, l)) if id == t.town.id => Some(l),
                _ => None,
            });
        }
        if let Some((id, l)) = peek_at {
            foci.extend(self.towns[&id].peek_foci(l));
        }
        if postcard_clear {
            foci.clear();
        }
        self.see_through += (if foci.is_empty() { 0.0 } else { 1.0 } - self.see_through) * approach(1.2, dt);
        // a rainbow stands in the world, off in the distance beyond where the
        // camera was looking when it appeared, rather than being painted on the
        // screen; the autopilot keeps its heading while it lasts
        if self.sky.rainbow > 0.02 {
            if self.bow.is_none() {
                let back = self.cam.back();
                let n = Vec2::new(back.x, back.z).normalize_or_zero();
                // sized to fit the view as it is now
                let r = (self.cam.zoom * 0.8).clamp(7.0, 24.0);
                let at = Vec2::new(self.cam.target.x, self.cam.target.z) - n * r * 0.55;
                let y = self.world.gen.raw_height(at.x, at.y).max(SEA as f32);
                self.bow = Some((Vec3::new(at.x, y, at.y), n, r));
            }
            self.dir.yaw_t = self.dir.yaw_t.max(5.0);
            self.dir.yaw_goal = self.cam.yaw;
        } else if self.sky.rainbow < 0.01 {
            self.bow = None;
        }
        // the people being watched: a circle round each, or one round a whole
        // wedding party
        let (bow_a, bow_n, bow_r) = self.bow.unwrap_or((Vec3::ZERO, Vec2::Y, 10.0));
        let grass_tint = seasonal(
            season,
            [
                    Vec3::new(1.02, 1.12, 0.9),
                    Vec3::new(0.97, 1.0, 0.86),
                    Vec3::new(1.12, 0.92, 0.6),
                    Vec3::new(0.82, 0.86, 0.8),
                ],
        );
        let fx = self.quality.fx();
        // the ground bounces its colour up into the shade: green under summer
        // trees, a cool white over snow
        let amb_ground = if fx.bounce {
            let g = grass_tint.lerp(Vec3::new(0.93, 0.95, 1.0), w.ground_snow.clamp(0.0, 1.0));
            amb_ground * (g / g.dot(Vec3::new(0.3, 0.59, 0.11)).max(0.1)).lerp(Vec3::ONE, 0.55)
        } else {
            amb_ground
        };
        let (_, cam_right, cam_up) = self.cam.basis();
        let half = ((view_r * 0.85 + 12.0) / 16.0).ceil().min(20.0) * 16.0;
        let half2 = ((view_r * 0.3 + 6.0) / 8.0).ceil().min(8.0) * 8.0;
        let frame = Frame {
            bow_a,
            bow_n,
            bow_r,
            // the photographic look of a postcard: a shallower, softer focus, eased in and out
            postcard: if self.postcard_t > 0.0 { ((POSTCARD_HOLD - self.postcard_t) / 1.5).clamp(0.0, 1.0).min((self.postcard_t / 1.5).clamp(0.0, 1.0)) } else { 0.0 },
            clear: if self.see_through < 0.01 || foci.is_empty() { 0.0 } else { self.see_through },
            foci,
            eye: self.cam.back(),
            vp,
            inv_vp: vp.inverse(),
            shift: Vec2::new(shift.x / iw as f32, shift.y / ih as f32),
            light_vp: light_matrix(light.dir, self.cam.target, half, 2048.0),
            light_vp2: light_matrix(light.dir, self.cam.target, half2, self.quality.shadow_size().max(1) as f32),
            grade: [
                (1.0 - smoothstep(0.1, 0.5, light.dir.y)) * (1.0 - light.night),
                light.night,
                smoothstep(2.7, 3.2, season) * (1.0 - smoothstep(3.8, 4.0, season)),
                (w.cloud * 0.6 + w.precip * 0.5 + w.storm * 0.4).min(1.0),
            ],
            cam_right,
            cam_up,
            depth_range: self.cam.depth_range(),
            zoom: self.cam.zoom,
            focus: self.cam.target,
            moon: (self.clock.days / 29.5).fract() as f32,
            part_center: Vec2::new(self.cam.target.x, self.cam.target.z),
            part_ext: view_r * 1.05,
            ground_y: gen.raw_height(self.cam.target.x, self.cam.target.z).max(SEA as f32 + 1.0),
            snowing: w.snowing,
            leaves: (season as u32) % 4 == 2 && w.precip < 0.6,
            part_br: inp.brightness,
            sun_dir: light.dir,
            sun_col,
            amb_sky,
            amb_ground,
            sky,
            time: self.time,
            wind: w.wind,
            cloud: w.cloud,
            night: light.night,
            lamp: smoothstep(0.35, 0.8, light.night) + w.storm * 0.3 + ecl * 0.9,
            lights,
            grass_tint,
            leaf_tint: seasonal(
                season,
                [
                    Vec3::new(1.08, 1.14, 0.9),
                    Vec3::new(1.0, 1.0, 0.95),
                    Vec3::new(1.0, 0.95, 0.85),
                    Vec3::new(0.78, 0.78, 0.72),
                ],
            ),
            autumn: smoothstep(2.0, 2.35, season) * (1.0 - smoothstep(2.85, 3.1, season)),
            bare: smoothstep(2.85, 3.1, season) * (1.0 - smoothstep(3.75, 3.98, season)) * 0.85,
            blossom: smoothstep(0.0, 0.15, season) * (1.0 - smoothstep(0.5, 0.8, season)) * 0.22,
            snow: w.ground_snow,
            wet: w.wet,
            crop,
            ripe,
            win_lit,
            ice: if (2.95..3.9).contains(&season) { w.ground_snow * 0.85 } else { 0.0 },
            fog,
            flash: w.flash,
            rain: w.precip,
            rainbow: self.sky.rainbow,
            aurora: self.sky.aurora,
            curtains: if self.sky.aurora > 0.02 { super::skyshow::curtains_near(gen, here, view_r) } else { Vec::new() },
            meteor: self.sky.meteor,
            meteor_t: self.sky.meteor_t,
            tilt: if self.photo { 0.55 } else { 0.0 },
            tide,
            // snow on the heights from late autumn to early spring
            peak_snow: smoothstep(2.55, 2.95, season).max(1.0 - smoothstep(0.1, 0.5, season)) * 0.9,
            season,
        };
        frame
    }
}
