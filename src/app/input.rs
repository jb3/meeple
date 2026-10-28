use super::*;

impl App {
    pub fn resize(&mut self, w: f64, h: f64, dpr: f64) {
        self.css = (w, h);
        self.r.resize(w, h, dpr);
        self.cam.aspect = (w / h.max(1.0)) as f32;
    }

    pub fn key(&mut self, key: &str) {
        self.input.moved_t = 0.0;
        match key {
            " " => {
                self.dir.auto = !self.dir.auto;
                if self.dir.auto {
                    self.dir.manual_t = 0.0;
                    self.start_travel();
                }
            }
            "p" | "P" => {
                self.r.pixel = match self.r.pixel {
                    1 => 2,
                    2 => 3,
                    3 => 4,
                    _ => 1,
                };
                let (w, h) = self.css;
                self.r.resize(w, h, self.r.dpr);
            }
            "t" | "T" => {
                self.clock.speed = match self.clock.speed as i32 {
                    1 => 6.0,
                    6 => 30.0,
                    30 => 120.0,
                    _ => 1.0,
                };
                self.hud_t = 4.0;
            }
            "r" | "R" => self.weather.cycle(),
            "h" | "H" => self.hud_hidden = !self.hud_hidden,
            "?" | "/" => self.toggle_class("help", "hidden"),
            "Escape" => {
                self.add_class("help", "hidden");
                self.news_on = false;
                self.add_class("news", "hidden");
                self.chron_on = false;
                self.add_class("chronicle", "hidden");
                if self.photo {
                    self.toggle_photo();
                }
                self.let_go();
            }
            "j" | "J" => {
                self.news_on = !self.news_on;
                self.overlay_t = 0.0;
                if !self.news_on {
                    self.add_class("news", "hidden");
                }
            }
            "z" | "Z" => {
                self.map_on = !self.map_on;
                self.map_t = 0.0;
                self.toggle_class("map", "hidden");
            }
            "l" | "L" => self.share(),
            "n" | "N" => {
                if let Some(w) = web_sys::window() {
                    let s = hash_u32(self.seed ^ (js_sys::Date::now() as u32)) % 1_000_000;
                    let _ = w.location().set_search(&format!("?seed={s}"));
                }
            }
            "c" | "C" => {
                self.chron_on = !self.chron_on;
                self.overlay_t = 0.0;
                if !self.chron_on {
                    self.add_class("chronicle", "hidden");
                }
            }
            "o" | "O" => self.toggle_photo(),
            "Enter" if self.photo => self.snap = true,
            "x" | "X" => match toggle_recording() {
                Ok(true) => self.toast("Recording: X stops and saves the video"),
                Ok(false) => self.toast("Video saved, or this browser can't record"),
                Err(_) => {}
            },
            "k" | "K" => self.look_preset(),
            "v" | "V" => {
                let d = js_sys::Date::new_0();
                let ymd = d.get_utc_full_year() * 10000 + (d.get_utc_month() + 1) * 100 + d.get_utc_date();
                self.go_seed(hash_u32(ymd) % 1_000_000);
            }
            "g" | "G" => {
                let at = GALLERY.iter().position(|s| *s == self.seed).map_or(0, |i| i + 1);
                self.go_seed(GALLERY[at % GALLERY.len()]);
            }
            "y" | "Y" => self.toggle_timelapse(),
            "i" | "I" => {
                self.peek_on = !self.peek_on;
                self.toast(if self.peek_on { "Zoom in on a house to look inside: I to stop" } else { "Roofs stay on: I to look inside again" });
            }
            "q" | "Q" | "e" | "E" => {
                self.input.rotate += if matches!(key, "q" | "Q") { -0.35 } else { 0.35 };
                // turned by hand while on a train: the director leaves it there
                if self.dir.mode == Mode::FollowTrain {
                    self.dir.yaw_locked = true;
                }
            }
            "+" | "=" => self.input.wheel -= 120.0,
            "-" | "_" => self.input.wheel += 120.0,
            _ => {}
        }
    }

    pub(super) fn go_seed(&mut self, seed: u32) {
        if let Some(w) = web_sys::window() {
            let _ = w.location().set_search(&format!("?seed={seed}"));
        }
    }

    /// Photograph mode: hide the interface, stop the clock, blur the top
    /// and bottom of the view, and hold the camera where it is.
    pub(super) fn toggle_photo(&mut self) {
        self.photo = !self.photo;
        if self.photo {
            self.photo_prev = (self.hud_hidden, self.clock.speed);
            self.hud_hidden = true;
            self.clock.speed = 0.0;
            self.dir.manual_t = 1.0e9;
            self.toast("Photograph mode: K changes the light, Enter saves a picture, O leaves");
        } else {
            self.hud_hidden = self.photo_prev.0;
            self.clock.speed = self.photo_prev.1.max(1.0);
            self.dir.manual_t = 0.0;
            if self.dir.auto {
                self.start_travel();
            }
        }
    }

    /// Light for a picture: golden hour, a misty dawn, snowfall, a moonlit night.
    pub(super) fn look_preset(&mut self) {
        let (rise, set) = self.clock.sun_times();
        let day = self.clock.days.floor();
        let n = ((self.clock.hour() * 10.0) as i64 + (self.time as i64)) % 4;
        let (hour, weather): (f32, Option<crate::sim::weather::Kind>) = match n {
            0 => (set - 1.1, Some(crate::sim::weather::Kind::Clear)),
            1 => (rise + 0.3, Some(crate::sim::weather::Kind::Fog)),
            2 => (12.0, Some(crate::sim::weather::Kind::Snow)),
            _ => (22.0, Some(crate::sim::weather::Kind::Clear)),
        };
        self.clock.days = day + hour as f64 / 24.0;
        if let Some(k) = weather {
            self.weather.set(k);
        }
    }

    pub(super) fn apply_input(&mut self, dt: f32) {
        let mut pan = std::mem::take(&mut self.input.pan);
        let held = |k: &[&str], keys: &HashSet<String>| k.iter().any(|x| keys.contains(*x));
        let speed = self.cam.zoom * 1.4 * dt;
        let keys = &self.input.keys;
        let (right, fwd) = self.cam.ground_axes();
        let mut kv = Vec2::ZERO;
        if held(&["w", "W", "ArrowUp"], keys) {
            kv += fwd;
        }
        if held(&["s", "S", "ArrowDown"], keys) {
            kv -= fwd;
        }
        if held(&["d", "D", "ArrowRight"], keys) {
            kv += right;
        }
        if held(&["a", "A", "ArrowLeft"], keys) {
            kv -= right;
        }
        if kv != Vec2::ZERO {
            pan += kv.normalize() * speed;
        }
        if pan != Vec2::ZERO {
            self.manual();
            self.cam.target.x += pan.x;
            self.cam.target.z += pan.y;
            self.dir.vel = Vec2::ZERO;
        }
        if self.input.wheel != 0.0 {
            self.cam.zoom = (self.cam.zoom * (self.input.wheel * 0.0012).exp()).clamp(8.0, MAX_ZOOM);
            self.dir.zoom_target = self.cam.zoom / TIGHTER;
            self.input.wheel = 0.0;
        }
        if self.input.rotate != 0.0 {
            let step = self.input.rotate * approach(6.0, dt);
            self.cam.yaw += step;
            self.input.rotate -= step;
            if self.input.rotate.abs() < 1e-4 {
                self.input.rotate = 0.0;
            }
        }
    }

    /// Turn the view by `d` radians (two fingers twisting on a touch screen):
    /// positive is clockwise on screen. The autopilot keeps the new heading.
    pub fn twist(&mut self, d: f32) {
        self.cam.yaw += d;
        self.dir.yaw_goal = self.cam.yaw;
        self.input.moved_t = 0.0;
        if self.dir.mode == Mode::FollowTrain {
            self.dir.yaw_locked = true;
        }
    }

    /// Mouse drag in CSS pixels.
    pub fn drag(&mut self, dx: f64, dy: f64) {
        let (right, fwd) = self.cam.ground_axes();
        let k = 2.0 * self.cam.zoom / self.css.1 as f32;
        let p = -right * dx as f32 * k + fwd * dy as f32 * k / self.cam.pitch.sin();
        self.input.pan += p;
        self.input.moved_t = 0.0;
    }

    /// Pick someone worth following nearby: leaning towards people doing
    /// something (making music, working a forge, minding a baby), and
    /// checking back on people the camera has followed before.
    pub(super) fn pick_person(&mut self, here: Vec2) {
        let mut cands: Vec<(u32, usize, f32, bool)> = Vec::new();
        for (id, t) in &self.towns {
            for (i, p) in t.people.iter().enumerate() {
                if !p.visible || p.pos.distance(here) > 32.0 {
                    continue;
                }
                let mut w = t.spotlight(i);
                if w <= 0.0 {
                    continue;
                }
                let fave = self.dir.faves.contains(&(*id, t.person_seed(i)));
                if fave {
                    w += 2.2;
                }
                cands.push((*id, i, w, fave));
            }
        }
        let total: f32 = cands.iter().map(|c| c.2).sum();
        if total <= 0.0 {
            return;
        }
        let mut roll = self.dir.rng.range(0.0, total);
        let mut pick = cands[0];
        for c in &cands {
            roll -= c.2;
            if roll <= 0.0 {
                pick = *c;
                break;
            }
        }
        let (id, i, _, fave) = pick;
        let t = &self.towns[&id];
        let seed = t.person_seed(i);
        let text = if fave { format!("Checking back in on {}", t.describe(i)) } else { format!("Following {}", t.describe(i)) };
        let d = &mut self.dir;
        d.mode = Mode::FollowPerson;
        d.follow_town = id;
        d.follow_idx = i;
        d.timer = d.rng.range(20.0, 35.0);
        d.zoom_target = 13.0;
        if !d.faves.contains(&(id, seed)) {
            d.faves.push_back((id, seed));
            if d.faves.len() > 6 {
                d.faves.pop_front();
            }
        }
        self.hud_follow = text;
        self.announce();
    }

    /// A click on a villager shows their card and follows them; a click
    /// anywhere else lets them go.
    pub fn click(&mut self, x: f64, y: f64) {
        let (vp, _) = self.cam.view_proj(None);
        let (w, h) = self.css;
        let mut best: Option<(u32, usize, f64)> = None;
        for (id, t) in &self.towns {
            for (i, p) in t.people.iter().enumerate() {
                if !p.visible {
                    continue;
                }
                let c = vp * (p.pos3() + Vec3::Y * 0.8).extend(1.0);
                if c.w <= 0.0 {
                    continue;
                }
                let sx = (c.x / c.w * 0.5 + 0.5) as f64 * w;
                let sy = (0.5 - c.y / c.w * 0.5) as f64 * h;
                let d = ((sx - x).powi(2) + (sy - y).powi(2)).sqrt();
                if d < 26.0 && best.map_or(true, |b| d < b.2) {
                    best = Some((*id, i, d));
                }
            }
        }
        // a train: any carriage near the click
        let train = if best.is_some() {
            None
        } else {
            let gen = &self.world.gen;
            let mut hit: Option<(u32, f64)> = None;
            for t in &self.traffic.trains {
                for p in t.car_points(gen) {
                    let c = vp * (p + Vec3::Y * 0.5).extend(1.0);
                    if c.w <= 0.0 {
                        continue;
                    }
                    let sx = (c.x / c.w * 0.5 + 0.5) as f64 * w;
                    let sy = (0.5 - c.y / c.w * 0.5) as f64 * h;
                    let d = ((sx - x).powi(2) + (sy - y).powi(2)).sqrt();
                    if d < 30.0 && hit.map_or(true, |b| d < b.1) {
                        hit = Some((t.id, d));
                    }
                }
            }
            hit.map(|h| h.0)
        };
        if let Some(id) = train {
            self.let_go();
            self.overlay_t = 0.0;
            let d = &mut self.dir;
            d.auto = true;
            d.manual_t = 0.0;
            d.mode = Mode::FollowTrain;
            d.follow_train = Some(id);
            d.follow_pinned = true;
            d.timer = f32::MAX;
            d.zoom_target = 18.0;
            self.hud_follow = String::from("Following the train (click elsewhere or Esc to let go)");
            self.announce();
            return;
        }
        match best {
            Some((id, i, _)) => {
                self.dir.follow_pinned = false;
                self.card = Some((id, i));
                self.overlay_t = 0.0;
                let d = &mut self.dir;
                d.auto = true;
                d.manual_t = 0.0;
                d.mode = Mode::FollowPerson;
                d.follow_town = id;
                d.follow_idx = i;
                d.timer = f32::MAX;
                d.zoom_target = 11.0;
                self.hud_follow = format!("Following {}", self.towns[&id].describe(i));
                self.remove_class("card", "hidden");
                self.announce();
            }
            None => self.let_go(),
        }
    }

    pub(super) fn let_go(&mut self) {
        if self.dir.follow_pinned {
            // a train followed by choice: back to wandering from where it is
            self.dir.follow_pinned = false;
            self.dir.follow_train = None;
            self.dir.mode = Mode::Linger;
            self.dir.center = Vec2::new(self.cam.target.x, self.cam.target.z);
            self.dir.timer = 30.0;
            self.hud_follow.clear();
        }
        if self.card.take().is_some() {
            self.add_class("card", "hidden");
            if self.dir.mode == Mode::FollowPerson {
                self.dir.mode = Mode::Linger;
                self.dir.center = Vec2::new(self.cam.target.x, self.cam.target.z);
                self.dir.timer = 30.0;
                self.hud_follow.clear();
            }
        }
    }
}
