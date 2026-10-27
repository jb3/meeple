use super::*;

mod audio;
mod climate;
mod scene;
mod stream;
use self::scene::FrameInputs;

impl App {
    pub fn frame(&mut self, dt: f32, now: &dyn Fn() -> f64) {
        if let Some(p) = &mut self.prof {
            p.start(now());
        }
        if self.quality.observe(dt) {
            self.apply_quality();
        }
        let dt = dt.clamp(0.0, 0.1);
        self.time += dt;
        self.clock.tick(dt as f64);
        self.new_day();
        self.persist(dt);
        self.input.moved_t += dt;
        self.hud_t -= dt;

        self.apply_input(dt);
        self.direct(dt);

        let here = Vec2::new(self.cam.target.x, self.cam.target.z);
        let gy = self.world.gen.raw_height(here.x, here.y).max(SEA as f32 + 1.0);
        // aim at the middle of a lighthouse, not its foot, so the lantern isn't cut off
        let lift = if matches!(self.dir.mode, Mode::Travel | Mode::Linger) && self.dir.kinds.back() == Some(&"lighthouse") && (self.dir.dest - here).length() < 40.0 { LIGHTHOUSE_LIFT } else { 0.0 };
        self.cam.target.y += (gy + lift - self.cam.target.y) * approach(0.5, dt);

        // stream the world
        self.stream_world(here, now);

        if let Some(p) = &mut self.prof {
            p.mark("camera+stream", now());
        }
        // the world is streamed for the whole view, but life (people, animals,
        // carts) is only simulated and drawn out to this far, however far out
        // the camera is
        let view_r = self.cam.view_radius().min(300.0);

        // weather & light
        let season = self.clock.season();
        let cold = self.world.gen.coldness(here.x, here.y);
        let thunder = self.update_climate(dt, here, season);
        let light = self.clock.light();
        let gen = &self.world.gen;
        let w = &self.weather;
        let hour = self.clock.hour();
        let slot = (self.clock.days * 48.0) as u32;
        let env = Env {
            gen,
            dt,
            time: self.time,
            hour,
            slot,
            rain: if w.snowing { w.precip * 0.6 } else { w.precip },
            night: light.night,
            day: self.clock.day(),
            // fine enough for a gathering: dry, calm and clear (a winter
            // bonfire is fine, a snowstorm isn't)
            nice: !w.snowing && w.precip < 0.05 && w.storm < 0.1 && w.fog < 0.5,
            wind: w.wind,
            season,
        };

        // towns come alive near the camera
        let r = view_r as i32 + 90;
        let nearby = gen.towns_near(here.x as i32 - r, here.y as i32 - r, here.x as i32 + r, here.y as i32 + r, 0);
        let ids: HashSet<u32> = nearby.iter().map(|t| t.id).collect();
        self.stations.sync(gen, &nearby);
        for t in nearby {
            if !self.towns.contains_key(&t.id) {
                let life = TownLife::new(gen, t.clone(), &env);
                self.towns.insert(t.id, life);
            }
        }
        let follow = self.dir.follow_town;
        let following = matches!(self.dir.mode, Mode::FollowPerson | Mode::Gathering | Mode::Event);
        self.towns.retain(|id, _| ids.contains(id) || (following && *id == follow));

        let mut sounds: Vec<Sound> = Vec::new();
        let carts = self.traffic.cart_obstacles();
        // the autopilot tells a town what it's coming for, so the occasion
        // happens and waits for the camera; the promise lapses if the camera
        // is called away or the time passes
        let aim = Vec2::new(self.cam.target.x, self.cam.target.z);
        let d = &self.dir;
        let keep = d.promise.filter(|p| {
            p.day == env.day && env.hour < p.to && d.auto && d.manual_t <= 0.0 && (matches!(d.mode, Mode::Travel | Mode::Linger) || d.follow_town == p.town)
        });
        self.dir.promise = keep;
        for t in self.towns.values_mut() {
            let want = keep.filter(|p| p.town == t.town.id).map(|p| (p.occasion, p.day, p.until));
            t.expect_visitor(want, Vec2::new(t.town.cx as f32, t.town.cz as f32).distance(aim) < 70.0);
        }
        for t in self.towns.values_mut() {
            t.update(&env, &carts, &mut sounds);
        }
        if let Some(p) = &mut self.prof {
            p.mark("towns", now());
        }
        self.interests.clear();
        for t in self.towns.values() {
            if let Some(i) = t.interest(&env) {
                self.interests.push((t.town.id, i));
            }
        }
        // the fair ends its last evening with fireworks
        self.firework_t -= dt;
        if self.firework_t <= 0.0 {
            self.firework_t = 2.5;
            if let Some(g) = self.towns.values().find_map(|t| t.finale(&env)) {
                let o = Vec3::new(self.dir.rng.range(-6.0, 6.0), 0.0, self.dir.rng.range(-6.0, 6.0));
                self.fx.firework(g + o, 0.0);
            }
        }
        // note the valley's goings-on for the chronicle
        let today = self.clock.day();
        for t in self.towns.values() {
            let seen = self.log_seen.entry(t.town.id).or_insert(0);
            if *seen > t.happen.log.len() {
                *seen = 0;
            }
            for (d, text) in t.happen.log.iter().skip(*seen) {
                self.chronicle.push((*d, text.clone()));
            }
            *seen = t.happen.log.len();
            if t.fair_on(&env) && self.noted.insert((t.town.id, today)) {
                self.chronicle.push((today, format!("The travelling fair came to {}.", t.town.name)));
            }
            if let Some((a, b)) = t.couple() {
                if self.noted.insert((t.town.id ^ 0x5eed, today)) {
                    self.chronicle.push((today, format!("{a} and {b} were married in {}.", t.town.name)));
                }
            }
        }
        let ecl = skies::Skies::eclipse(self.seed, self.clock.days);
        if ecl > 0.5 && self.noted.insert((0xec11, today)) {
            self.chronicle.push((today, "The sun was blotted out at noon: a total eclipse.".into()));
        }
        if self.drought > 0.6 && self.noted.insert((0xd209, today / 6)) {
            self.chronicle.push((today, "A long dry spell: the rivers ran low and thatch was tinder.".into()));
        }
        if self.flood > 0.5 && self.noted.insert((0xf100, today)) {
            self.chronicle.push((today, "After days of rain the rivers rose in flood.".into()));
        }
        let era_now = skies::era(self.seed, gen.first_day.get(), today);
        if self.last_era != Some(era_now) {
            if self.last_era.is_some() {
                self.chronicle.push((today, format!("A new age dawns: {}.", skies::ERA_NAMES[era_now as usize])));
            }
            self.last_era = Some(era_now);
        }
        if self.chronicle.len() > 120 {
            self.chronicle.remove(0);
        }
        let walkers: Vec<Vec2> = self.towns.values().flat_map(|t| t.walkers()).collect();
        self.overlay_t -= dt;
        if self.news_on && self.overlay_t <= 0.0 {
            // the nearest town's newsletter, with its station's comings and goings
            let near = self.towns.values().min_by(|a, b| {
                let d = |t: &TownLife| Vec2::new(t.town.cx as f32, t.town.cz as f32).distance(here);
                d(a).partial_cmp(&d(b)).unwrap()
            });
            self.news = near.map(|t| {
                let mut lines = t.news(&env);
                if let Some(st) = self.stations.by_town.get(&t.town.id) {
                    if st.boarded + st.alighted > 0 {
                        lines.insert(lines.len() - 1, format!("At the station: {} set off by train, {} arrived.", st.boarded, st.alighted));
                    } else {
                        lines.insert(lines.len() - 1, "Trains stop at the station out by the line.".into());
                    }
                }
                lines.push(self.fauna.wildlife_report(Vec2::new(t.town.cx as f32, t.town.cz as f32)));
                lines.push(format!("Weather: {}.", self.weather.label().to_lowercase()));
                (t.town.name.to_string(), lines)
            });
        }
        let mut emit = Emit { smoke: Vec::new() };
        // carts and travellers keep clear of each other as well as of villagers
        let mut on_roads = walkers.clone();
        on_roads.extend(self.travellers.points());
        self.traffic.update(gen, dt, here, view_r, &on_roads, &mut sounds, &mut emit);
        let mut cart_pts = walkers.clone();
        for (c, _) in self.traffic.cart_obstacles() {
            cart_pts.push(c);
        }
        self.travellers.update(gen, dt, here, view_r, hour, &cart_pts, &mut sounds);
        let standing: Vec<_> = self.traffic.trains.iter().filter_map(|t| t.at_station()).collect();
        self.stations.update(dt, &standing, hour, &mut sounds);
        // animals only come into being out of sight (except as the world first loads)
        let (cam_vp, _) = self.cam.view_proj(None);
        let on_screen = |p: Vec3| {
            let c = cam_vp * p.extend(1.0);
            c.x.abs() < c.w * 1.1 && c.y.abs() < c.w * 1.1
        };
        let warmup = !self.settled;
        // shy animals bolt from villagers, carts and trains
        let mut threats = walkers.clone();
        threats.extend(self.traffic.cart_obstacles().iter().map(|c| c.0));
        threats.extend(self.traffic.trains.iter().map(|t| {
            let f = t.front(gen);
            Vec2::new(f.x, f.z)
        }));
        if let Some(p) = &mut self.prof {
            p.mark("traffic+misc", now());
        }
        self.fauna.set_day(self.clock.days);
        self.fauna.update(&self.world, dt, here, view_r, light.night, hour, smoothstep(2.85, 3.1, season) * (1.0 - smoothstep(3.75, 3.98, season)) * 0.85, season, &threats, &on_screen, warmup, &mut sounds);
        let era = skies::era(self.seed, gen.first_day.get(), self.clock.day());
        self.traffic.era = era;
        self.skies.update(dt, here, view_r, era, w.storm < 0.2 && w.precip < 0.3 && w.fog < 0.6, hour, (2.95..4.0).contains(&season), &mut sounds);
        self.wonders.update(dt, &self.world, here, view_r, w.fog, hour, light.night, &mut sounds);

        // particles
        let fxenv = FxEnv {
            time: self.time,
            dt,
            wind: w.wind,
            rain: w.precip,
            snowing: w.snowing,
            night: light.night,
            brightness: (1.0 - light.night * 0.7) * (1.0 - w.cloud * 0.2),
            season,
            hour,
            texel: 2.0 * self.cam.zoom / self.r.internal_size().1.max(1) as f32,
            gpu_ambient: self.r.gpu_particles(),
            steam: if w.precip < 0.05 && !w.snowing { ((w.wet - 0.1) * 2.5).clamp(0.0, 1.0) * (1.0 - light.night) } else { 0.0 },
        };
        let (mut chim, mut fount, mut forge, mut lamps) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut water, mut nature, mut nchunks) = (0.0f32, 0.0f32, 0.0f32);
        for (k, f) in self.world.features_near(here.x, here.y, view_r) {
            chim.extend_from_slice(&f.chimneys);
            fount.extend_from_slice(&f.fountains);
            forge.extend_from_slice(&f.forges);
            lamps.extend_from_slice(&f.lamps);
            let kc = Vec2::new((k.0 * CS + CS / 2) as f32, (k.1 * CS + CS / 2) as f32);
            if kc.distance(here) < self.cam.zoom * 1.3 + 16.0 {
                water += f.water;
                nature += f.nature;
                nchunks += 1.0;
            }
        }
        // a winter bonfire sparks, smokes and lights up the square like a forge
        forge.extend(self.towns.values().filter_map(|t| t.bonfire()).map(|p| p.to_array()));
        self.fx.emitters(&fxenv, &chim, &fount, &forge);
        for t in self.towns.values() {
            for p in &t.strikes {
                self.fx.strike(*p);
            }
        }
        for (p, s) in emit.smoke {
            self.fx.smoke(p, s);
        }
        for (p, sz) in Wonders::steam(&self.world, here, view_r) {
            if (self.time * 3.0 + p.x).fract() < dt * 3.0 {
                self.fx.smoke(p, sz);
            }
        }
        // fires smoke, and glow on the buildings round them
        let mut glows: Vec<[f32; 4]> = Vec::new();
        let puff = (self.time * 9.0).fract() < dt * 9.0;
        for t in self.towns.values() {
            glows.extend(t.glows(&env));
            if puff {
                for (p, s) in t.smokes(&env) {
                    self.fx.smoke(p, s);
                }
            }
        }
        self.fx.update(&fxenv);
        if let Some(p) = &mut self.prof {
            p.mark("fauna+fx", now());
        }

        // draw lists
        self.solid.clear();
        self.fxb.clear();
        for t in self.towns.values() {
            t.draw(&mut self.solid, &mut self.fxb, &env, self.cam.yaw, here, view_r);
        }
        self.traffic.draw(gen, &mut self.solid, here, view_r, light.night);
        self.travellers.draw(&mut self.solid, here, view_r, self.time);
        crate::sim::ferry::draw(&self.world, &mut self.solid, here, view_r, self.time, light.night);
        self.stations.draw(&mut self.solid, here, view_r, self.time);
        let tide = (self.clock.days as f32 * std::f32::consts::TAU * 1.93).sin() * 0.3 + self.flood * 0.55 - self.drought * 0.4;
        self.fauna.draw(&mut self.solid, &mut self.fxb, here, view_r, self.time, tide);
        self.wonders.draw(&mut self.solid, &mut self.fxb, gen, here, view_r, self.time, self.clock.day(), self.clock.sun_times().0, hour, season, light.night, w.fog);
        self.skies.draw(&mut self.solid, self.time, light.night, &|p| gen.raw_height(p.x, p.y).max(SEA as f32));
        if era >= 1 {
            skies::draw_poles(&mut self.solid, gen, here, view_r, self.time);
        }
        // the sails' turning rate follows the wind smoothly, and the angle is
        // added up frame by frame (a rate times the clock would whirl when the
        // wind changed); real sails manage a turn every few seconds at most
        let sail_target = (0.3 + w.wind.length() * 0.9).min(1.3);
        self.sail_rate += (sail_target - self.sail_rate) * approach(0.2, dt);
        self.sail_angle = (self.sail_angle + self.sail_rate * dt).rem_euclid(std::f32::consts::TAU);
        crate::sim::props::draw(&mut self.solid, &mut self.fxb, &self.world, here, view_r, self.time, self.sail_angle, light.night, tide);
        self.fx.draw(&mut self.fxb, &fxenv, gen, self.cam.target, view_r);

        self.sky.update(dt, &self.weather, light.night, cold);

        let frame = self.build_frame(dt, here, view_r, &light, FrameInputs { season, hour, tide, brightness: fxenv.brightness, lamps: &lamps, forge: &forge, glows: &glows });
        if let Some(p) = &mut self.prof {
            p.mark("draw lists+frame", now());
        }
        self.r.render(&frame, &self.solid, &self.fxb);
        if let Some(p) = &mut self.prof {
            p.mark("render (cpu side)", now());
        }
        if self.snap {
            self.snap = false;
            let _ = save_picture();
            self.set_text("toast", "Picture saved");
            self.remove_class("toast", "hidden");
            self.add_class("toast", "on");
            self.toast_t = 3.0;
        }

        // sound
        self.drive_audio(here, hour, light.night, season, &sounds, thunder, (water, nature, nchunks));

        self.watch_gatherings(here, view_r);
        self.watch_events(here);
        self.update_hud(hour, &light);
        self.postcards(dt, hour, &light);
        self.overlays(dt);
        if let Some(p) = &mut self.prof {
            p.mark("audio+hud", now());
            p.report(now());
        }
    }
}
