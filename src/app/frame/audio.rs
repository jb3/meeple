use super::super::*;

impl App {
    /// Turn the frame's sound events and the state of the world around the
    /// camera into the parameter block and event string `meepleAudio.update`
    /// takes (see `web/audio.js`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn drive_audio(&mut self, here: Vec2, hour: f32, night: f32, season: f32, sounds: &[Sound], thunder: Vec<f32>, terrain: (f32, f32, f32)) {
        let gen = &self.world.gen;
        let w = &self.weather;
        let (water, nature, nchunks) = terrain;
        // the church clock strikes the hours, and one soft stroke on the half
        let mut chime: Option<String> = None;
        let slot = (hour * 2.0).floor() as i32;
        if slot != self.chime_slot {
            let first = self.chime_slot == i32::MIN;
            self.chime_slot = slot;
            if !first {
                let (_, right0, _) = self.cam.basis();
                let near = self
                    .towns
                    .values()
                    .filter_map(|t| t.church_front().map(|(c, _, _)| c))
                    .map(|c| (c.distance(here), c))
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                if let Some((d, c)) = near {
                    let reach = 220.0 + self.cam.zoom * 2.0;
                    if d < reach {
                        let v = (1.0 - d / reach).sqrt() * 0.9;
                        let pn = (((c.x - self.cam.target.x) * right0.x + (c.y - self.cam.target.z) * right0.z) / (self.cam.zoom * self.cam.aspect)).clamp(-1.0, 1.0) * 0.85;
                        let code = if slot % 2 == 0 { (((hour.floor() as i32) + 11) % 12 + 1) as f32 } else { 13.0 };
                        chime = Some(format!("chime:{v:.2}:{pn:.2}:{code};"));
                    }
                }
            }
        }
        let att = |p: Vec3| {
            let d = Vec2::new(p.x, p.z).distance(here);
            (1.0 - d / (self.cam.zoom * 1.6)).clamp(0.0, 1.0) * (22.0 / self.cam.zoom).clamp(0.4, 1.2)
        };
        // each sound is placed left-right by where it is across the screen
        let (_, right, _) = self.cam.basis();
        let half_w = self.cam.zoom * self.cam.aspect;
        let pan = |p: Vec3| ((p - self.cam.target).dot(right) / half_w).clamp(-1.0, 1.0) * 0.85;
        let mut events = String::new();
        for s in sounds {
            // big, sharp sounds carry: horns and bells, and the signals that
            // stand well out from the crossings they guard
            let v = s.vol
                * match s.name {
                    "horn" | "bell" => att(s.pos).max(0.0).sqrt(),
                    "semaphore" => {
                        let d = Vec2::new(s.pos.x, s.pos.z).distance(here);
                        (1.0 - d / (self.cam.zoom * 4.0 + 30.0)).clamp(0.0, 1.0).sqrt()
                    }
                    _ => att(s.pos),
                };
            if v > 0.02 {
                events.push_str(&format!("{}:{:.2}:{:.2}:{:.2};", s.name, v, pan(s.pos), s.pitch));
            }
        }
        for v in thunder {
            events.push_str(&format!("thunder:{v:.2};"));
        }
        if let Some(c) = chime {
            events.push_str(&c);
        }
        let crowd: usize = self.towns.values().map(|t| t.visible_count_near(here, self.cam.zoom * 1.2)).sum();
        let (mut train, mut tspeed) = (0.0f32, 0.0f32);
        for t in &self.traffic.trains {
            let p = t.front(gen);
            let a = att(p);
            if a > train {
                train = a;
                tspeed = t.speed / 9.0;
            }
        }
        let n = nchunks.max(1.0);
        // the nearest gathering's band and choir, faded by distance
        let mut festival = (0.0f32, 0.0f32);
        for t in self.towns.values() {
            // a wedding has its own music; the band plays on without a tune
            if t.couple().is_some() {
                continue;
            }
            if let Some((c, band, choir)) = t.festival() {
                let a = att(c).sqrt();
                festival.0 = festival.0.max(a * (band / 5.0).min(1.0));
                festival.1 = festival.1.max(a * (choir / 4.0).min(1.0));
            }
        }
        // each town has its own key and tune
        let place = self
            .towns
            .values()
            .min_by(|a, b| {
                let d = |t: &TownLife| Vec2::new(t.town.cx as f32, t.town.cz as f32).distance(here);
                d(a).partial_cmp(&d(b)).unwrap()
            })
            .map_or(0.5, |t| unit(hash_u32(t.town.id)));
        for b in self.fx.booms.clone() {
            events.push_str(&format!("boom:{:.2}:{:.2}:1;", att(b).max(0.35), pan(b)));
        }
        // windmill sails and waterwheels near enough to hear
        let (mut mill, mut mill_pan, mut wheel) = (0.0f32, 0.0f32, 0.0f32);
        for (_, f) in self.world.features_near(here.x, here.y, 90.0) {
            for &(hub, _) in &f.sails {
                let p = Vec3::from(hub);
                let a = (1.0 - Vec2::new(p.x, p.z).distance(here) / (self.cam.zoom * 1.4 + 30.0)).clamp(0.0, 1.0);
                if a > mill {
                    mill = a;
                    mill_pan = pan(p);
                }
            }
            for &(hub, _) in &f.wheels {
                let p = Vec3::from(hub);
                wheel = wheel.max((1.0 - Vec2::new(p.x, p.z).distance(here) / (self.cam.zoom * 1.2 + 24.0)).clamp(0.0, 1.0));
            }
        }
        // what kind of country this is, for the scale the music plays
        let biome = {
            let raw = gen.raw_height(here.x, here.y);
            match gen.biome(here.x, here.y, raw) {
                Biome::Plains => 0.0,
                Biome::Desert => 1.0,
                Biome::Tundra | Biome::Mountain => 2.0,
                Biome::Forest | Biome::Jungle => 3.0,
            }
        };
        // a wedding, christening, closing time or fire close by gets its own music
        let (mut cue, mut cue_level) = (0.0f32, 0.0f32);
        for t in self.towns.values() {
            let c = Vec2::new(t.town.cx as f32, t.town.cz as f32);
            let near = (1.0 - (c.distance(here) - t.town.radius as f32).max(0.0) / (self.cam.zoom * 1.2 + 40.0)).clamp(0.0, 1.0);
            for (code, on) in [
                (1.0, t.happening(crate::sim::people::happenings::Kind::Fire)),
                (2.0, t.couple().is_some() && !t.wedding_party().is_empty()),
                (3.0, t.happening(crate::sim::people::happenings::Kind::Christening)),
                (4.0, t.happening(crate::sim::people::happenings::Kind::Closing)),
            ] {
                if on && near > cue_level {
                    cue = code;
                    cue_level = near;
                }
            }
        }
        // the inn of the nearest town, children at play, and how enclosed the streets are
        let (mut tavern, mut children, mut enclosed, mut church) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for t in self.towns.values() {
            let c = Vec2::new(t.town.cx as f32, t.town.cz as f32);
            if let Some(f) = t.tavern_front() {
                tavern = tavern.max((1.0 - f.distance(here) / (self.cam.zoom * 1.3 + 30.0)).clamp(0.0, 1.0));
            }
            let kids = t.children_near(here, self.cam.zoom * 1.3 + 12.0);
            children = children.max((kids as f32 / 4.0).min(1.0));
            let r = t.town.radius as f32;
            enclosed = enclosed.max((1.0 - (c.distance(here) - r * 0.6).max(0.0) / (r * 0.6)).clamp(0.0, 1.0));
            if let Some((cc, _, _)) = t.church_front() {
                church = church.max((1.0 - cc.distance(here) / 30.0).clamp(0.0, 1.0));
            }
        }
        let params = [
            hour,
            if w.snowing { 0.0 } else { w.precip },
            (w.wind.length() * 0.8 + w.storm * 0.3).min(1.0),
            if w.snowing { w.precip } else { 0.0 },
            crowd as f32 / 14.0,
            (water / n * 2.5).min(1.0),
            (nature / n * 1.4).min(1.0),
            train,
            tspeed,
            night,
            season,
            w.storm,
            festival.0,
            festival.1,
            place,
            self.cam.zoom,
            mill,
            self.sail_angle,
            mill_pan,
            wheel,
            biome,
            w.fog,
            cue,
            cue_level,
            tavern,
            children,
            enclosed,
            church,
            self.quality.tier as f32,
            (self.seed & 0xff_ffff) as f32,
        ];
        let _ = audio_update(&params, &events);
    }
}
