use super::*;

impl App {
    pub(super) fn toast(&mut self, text: &str) {
        self.set_text("toast", text);
        self.remove_class("toast", "hidden");
        self.add_class("toast", "on");
        self.toast_t = 4.0;
    }

    pub(super) fn manual(&mut self) {
        self.dir.manual_t = 25.0;
        self.dir.mode = Mode::Travel;
        self.dir.follow_train = None;
        self.hud_follow.clear();
    }

    pub(super) fn toggle_class(&self, id: &str, class: &str) {
        if let Some(e) = self.doc.get_element_by_id(id) {
            let _ = e.class_list().toggle(class);
        }
    }

    pub(super) fn add_class(&self, id: &str, class: &str) {
        if let Some(e) = self.doc.get_element_by_id(id) {
            let _ = e.class_list().add_1(class);
        }
    }

    pub(super) fn remove_class(&self, id: &str, class: &str) {
        if let Some(e) = self.doc.get_element_by_id(id) {
            let _ = e.class_list().remove_1(class);
        }
    }

    pub(super) fn set_html(&self, id: &str, html: &str) {
        if let Some(e) = self.doc.get_element_by_id(id) {
            if e.inner_html() != html {
                e.set_inner_html(html);
            }
        }
    }

    /// Copy a link to this exact view: seed, place, hour and season.
    pub(super) fn share(&mut self) {
        let Some(w) = web_sys::window() else { return };
        let loc = w.location();
        let base = format!("{}{}", loc.origin().unwrap_or_default(), loc.pathname().unwrap_or_default());
        let season = self.clock.season_name().to_lowercase();
        let url = format!(
            "{base}?seed={}&x={:.0}&z={:.0}&hour={:.1}&season={season}",
            self.seed, self.cam.target.x, self.cam.target.z, self.clock.hour()
        );
        let _ = w.navigator().clipboard().write_text(&url);
        if let Ok(h) = w.history() {
            let _ = h.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
        self.set_text("toast", "Link to this spot copied");
        self.remove_class("toast", "hidden");
        self.add_class("toast", "on");
        self.toast_t = 3.0;
    }

    /// Now and then the autopilot holds still on a lovely view and captions
    /// it like a postcard: sunsets, rainbows, northern lights, fresh snow, a
    /// festival.
    pub(super) fn postcards(&mut self, dt: f32, hour: f32, light: &crate::sim::clock::Light) {
        let was = self.postcard_t > 0.0;
        self.postcard_t -= dt;
        self.postcard_cd -= dt;
        if was && self.postcard_t <= 0.0 {
            self.remove_class("postcard", "on");
            self.remove_class("hud", "away");
        }
        if self.postcard_cd > 0.0 || !self.dir.auto || self.dir.manual_t > 0.0 || self.card.is_some() || !self.settled || self.locked() {
            return;
        }
        let here = Vec2::new(self.cam.target.x, self.cam.target.z);
        let (place, in_town) = self.place_name(here);
        let w = &self.weather;
        let clear = w.cloud < 0.45 && w.precip < 0.05;
        let of = if in_town { "" } else { "the " };
        let caption = if self.sky.rainbow > 0.5 {
            Some(format!("A rainbow over {of}{place}"))
        } else if self.sky.aurora > 0.5 && light.night > 0.6 {
            Some(format!("Northern lights above {of}{place}"))
        } else if clear && (18.2..19.4).contains(&hour) {
            Some(format!("Sunset over {of}{place}"))
        } else if clear && (5.8..6.8).contains(&hour) {
            Some(format!("Dawn breaks over {of}{place}"))
        } else if w.snowing && w.precip > 0.3 && in_town {
            Some(format!("Snow falling on {place}"))
        } else if self.dir.mode == Mode::Gathering && self.cam.zoom < GATHER_CLOSE + 1.0 {
            let t = self.towns.get(&self.dir.follow_town);
            t.and_then(|t| t.gathering.as_ref().map(|g| format!("{} in {}", g.festival.name(), t.town.name)))
        } else if light.night > 0.8 && clear && self.clock.moon_full() > 0.95 {
            Some(format!("A full moon over {of}{place}"))
        } else {
            None
        };
        let Some(caption) = caption else { return };
        self.postcard_t = POSTCARD_HOLD;
        self.postcard_cd = POSTCARD_GAP;
        self.set_text("postcard-title", &caption);
        let sub = format!("Day {} of {}", self.clock.day() % 12 + 1, self.clock.season_name());
        self.set_text("postcard-sub", &sub);
        self.add_class("postcard", "on");
        self.add_class("hud", "away");
        self.hud_t = 0.0;
    }

    /// Refresh the card, newsletter and map, a few times a second.
    pub(super) fn overlays(&mut self, dt: f32) {
        if self.toast_t > 0.0 {
            self.toast_t -= dt;
            if self.toast_t <= 0.0 {
                self.remove_class("toast", "on");
            }
        }
        if self.badge_t > 0.0 {
            self.badge_t -= dt;
            if self.badge_t <= 0.0 {
                self.remove_class("quality", "on");
            }
        }
        if self.overlay_t > 0.0 {
            return;
        }
        self.overlay_t = 0.4;
        if let Some((id, i)) = self.card {
            match self.towns.get(&id).filter(|t| i < t.people.len()) {
                Some(t) => {
                    let (name, lines) = t.card(i);
                    self.set_text("card-name", &name);
                    let html: String = lines.iter().map(|l| format!("<div>{}</div>", escape(l))).collect();
                    self.set_html("card-body", &html);
                }
                None => self.let_go(),
            }
        }
        if self.news_on {
            if let Some((town, lines)) = &self.news {
                let items: String = lines.iter().map(|l| format!("<li>{}</li>", escape(l))).collect();
                let html = format!(
                    "<h2>Today in {}</h2><div class=\"date\">Day {} · {} · The {} Gazette</div><ul>{items}</ul><div class=\"dim\">J to close</div>",
                    escape(town),
                    self.clock.day() % 12 + 1,
                    self.clock.season_name(),
                    escape(town)
                );
                self.set_html("news", &html);
                self.remove_class("news", "hidden");
            }
        }
        if self.chron_on {
            let era = skies::era(self.seed, self.world.gen.first_day.get(), self.clock.day());
            let when = |d: i64| format!("Day {} of {}", d.rem_euclid(12) + 1, crate::sim::clock::SEASONS[(d.rem_euclid(12) / 3) as usize]);
            let mut seen = HashSet::new();
            let items: String = self
                .chronicle
                .iter()
                .rev()
                .filter(|(d, t)| seen.insert((*d, t.clone())))
                .take(14)
                .map(|(d, t)| format!("<li><b>{}</b> {}</li>", escape(&when(*d)), escape(t)))
                .collect();
            let empty = if items.is_empty() { "<li>Nothing of note yet. Watch a while.</li>" } else { "" };
            let html = format!(
                "<h2>The Chronicle</h2><div class=\"date\">Year {} · {}</div><ul>{items}{empty}</ul><div class=\"dim\">C to close</div>",
                self.clock.day().div_euclid(12) + 1,
                escape(skies::ERA_NAMES[era as usize])
            );
            self.set_html("chronicle", &html);
            self.remove_class("chronicle", "hidden");
        }
        if self.map_on {
            self.map_t -= 0.4;
            if self.map_t <= 0.0 {
                self.map_t = 2.0;
                self.draw_map();
            }
        }
    }

    /// "Where am I?": a little map of the land around the camera, with
    /// towns, roads and railways, and which way we're looking.
    pub(super) fn draw_map(&self) {
        use wasm_bindgen::{Clamped, JsCast};
        const N: usize = 160;
        const SCALE: f32 = 7.5;
        let Some(canvas) = self.doc.get_element_by_id("map-canvas") else { return };
        let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() else { return };
        let Ok(Some(ctx)) = canvas.get_context("2d") else { return };
        let Ok(ctx) = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>() else { return };
        let gen = &self.world.gen;
        let here = Vec2::new(self.cam.target.x, self.cam.target.z);
        let origin = here - Vec2::splat(N as f32 * SCALE * 0.5);
        let mut px = vec![0u8; N * N * 4];
        for j in 0..N {
            for i in 0..N {
                let (x, z) = (origin.x + i as f32 * SCALE, origin.y + j as f32 * SCALE);
                let h = gen.raw_height(x, z);
                let c: [f32; 3] = if h < SEA as f32 {
                    let deep = ((SEA as f32 - h) / 12.0).min(1.0);
                    [70.0 - deep * 30.0, 130.0 - deep * 45.0, 185.0 - deep * 40.0]
                } else if h > 38.0 {
                    [235.0, 238.0, 240.0]
                } else {
                    let base = match gen.biome(x, z, h) {
                        Biome::Plains => [120.0, 175.0, 85.0],
                        Biome::Forest => [70.0, 130.0, 70.0],
                        Biome::Desert => [222.0, 202.0, 145.0],
                        Biome::Tundra => [215.0, 222.0, 228.0],
                        Biome::Mountain => [145.0, 140.0, 132.0],
                        Biome::Jungle => [40.0, 112.0, 52.0],
                    };
                    let lift = 0.82 + ((h - SEA as f32) / 30.0).clamp(0.0, 1.0) * 0.3;
                    [base[0] * lift, base[1] * lift, base[2] * lift]
                };
                let k = (j * N + i) * 4;
                px[k] = c[0].min(255.0) as u8;
                px[k + 1] = c[1].min(255.0) as u8;
                px[k + 2] = c[2].min(255.0) as u8;
                px[k + 3] = 255;
            }
        }
        if let Ok(img) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&px), N as u32, N as u32) {
            let _ = ctx.put_image_data(&img, 0.0, 0.0);
        }
        let to_px = |x: f32, z: f32| (((x - origin.x) / SCALE) as f64, ((z - origin.y) / SCALE) as f64);
        let span = (N as f32 * SCALE * 0.5) as i32;
        let (x0, z0, x1, z1) = (here.x as i32 - span, here.y as i32 - span, here.x as i32 + span, here.y as i32 + span);
        ctx.set_line_width(1.0);
        ctx.set_stroke_style_str("rgba(150, 110, 70, 0.9)");
        for r in gen.roads_near(x0, z0, x1, z1) {
            ctx.begin_path();
            // a dot of a slanting road gets a tick; a straight run is drawn end to end
            let tick = if r.x0 == r.x1 && r.z0 == r.z1 { 1.0 } else { 0.0 };
            let (a, b) = (to_px(r.x0 as f32, r.z0 as f32), to_px(r.x1 as f32 + tick, r.z1 as f32 + tick));
            ctx.move_to(a.0, a.1);
            ctx.line_to(b.0, b.1);
            ctx.stroke();
        }
        ctx.set_stroke_style_str("rgba(45, 35, 35, 0.95)");
        for l in gen.rails_near(x0, z0, x1, z1) {
            ctx.begin_path();
            let s0 = l.along(x0 as f32, z0 as f32).min(l.along(x1 as f32, z1 as f32)) - 20.0;
            let s1 = l.along(x0 as f32, z0 as f32).max(l.along(x1 as f32, z1 as f32)) + 20.0;
            let mut s = s0;
            let mut first = true;
            while s <= s1 {
                let (x, z) = l.point(s, 0.0);
                let p = to_px(x, z);
                if first {
                    ctx.move_to(p.0, p.1);
                    first = false;
                } else {
                    ctx.line_to(p.0, p.1);
                }
                s += 12.0;
            }
            ctx.stroke();
        }
        ctx.set_font("8px monospace");
        ctx.set_text_align("center");
        for t in gen.towns_near(x0, z0, x1, z1, 0) {
            let p = to_px(t.cx as f32, t.cz as f32);
            let r = if t.city { 3.5 } else { 2.5 };
            ctx.set_fill_style_str("#b5523b");
            ctx.fill_rect(p.0 - r, p.1 - r, r * 2.0, r * 2.0);
            ctx.set_fill_style_str("#1b1d2a");
            let _ = ctx.fill_text(&t.name, p.0 + 0.5, p.1 - 5.5);
            ctx.set_fill_style_str("#fdf6e3");
            let _ = ctx.fill_text(&t.name, p.0, p.1 - 6.0);
        }
        // you are here, looking this way
        let c = N as f64 * 0.5;
        let f = Vec2::new(-self.cam.yaw.sin(), -self.cam.yaw.cos());
        let side = Vec2::new(-f.y, f.x);
        let tip = (f * 7.0, f * -3.0 + side * 4.0, f * -3.0 - side * 4.0);
        ctx.set_fill_style_str("#ffd98a");
        ctx.set_stroke_style_str("#1b1d2a");
        ctx.begin_path();
        ctx.move_to(c + tip.0.x as f64, c + tip.0.y as f64);
        ctx.line_to(c + tip.1.x as f64, c + tip.1.y as f64);
        ctx.line_to(c + tip.2.x as f64, c + tip.2.y as f64);
        ctx.close_path();
        ctx.fill();
        ctx.stroke();
        let (place, _) = self.place_name(here);
        self.set_text("map-label", &format!("{place} · {:.0}, {:.0}", here.x, here.y));
    }

    /// Where the camera is: the town it's in, or the lie of the land.
    pub(super) fn place_name(&self, here: Vec2) -> (String, bool) {
        let gen = &self.world.gen;
        let town = gen
            .towns_near(here.x as i32 - 60, here.y as i32 - 60, here.x as i32 + 60, here.y as i32 + 60, 0)
            .into_iter()
            .find(|t| Vec2::new(t.cx as f32, t.cz as f32).distance(here) < t.radius as f32 + 30.0);
        match town {
            Some(t) => (t.name.to_string(), true),
            None => {
                let h = gen.raw_height(here.x, here.y);
                let name = if h < SEA as f32 {
                    "Open water"
                } else {
                    match gen.biome(here.x, here.y, h) {
                        Biome::Plains => "Meadows",
                        Biome::Forest => "Woodland",
                        Biome::Desert => "Dunes",
                        Biome::Tundra => "Frozen moors",
                        Biome::Mountain => "Highlands",
                        Biome::Jungle => "Jungle",
                    }
                };
                (name.to_string(), false)
            }
        }
    }

    pub(super) fn update_hud(&mut self, hour: f32, _light: &crate::sim::clock::Light) {
        let show = !self.hud_hidden && (self.input.moved_t < 4.0 || self.hud_t > 0.0);
        if let Some(e) = self.doc.get_element_by_id("hud") {
            let has = e.class_list().contains("visible");
            if show != has {
                let _ = if show { e.class_list().add_1("visible") } else { e.class_list().remove_1("visible") };
            }
        }
        if (self.time * 2.0) as i32 == self.last_hour {
            return;
        }
        self.last_hour = (self.time * 2.0) as i32;
        let here = Vec2::new(self.cam.target.x, self.cam.target.z);
        let gen = &self.world.gen;
        let town = gen
            .towns_near(here.x as i32 - 60, here.y as i32 - 60, here.x as i32 + 60, here.y as i32 + 60, 0)
            .into_iter()
            .find(|t| Vec2::new(t.cx as f32, t.cz as f32).distance(here) < t.radius as f32 + 30.0);
        let place = match town {
            Some(t) => format!("{} · {}", t.name, if t.city { "town" } else { "village" }),
            None => {
                let h = gen.raw_height(here.x, here.y);
                if h < SEA as f32 {
                    String::from("Open water")
                } else {
                    match gen.biome(here.x, here.y, h) {
                        Biome::Plains => "Meadows",
                        Biome::Forest => "Woodland",
                        Biome::Desert => "Dunes",
                        Biome::Tundra => "Frozen moors",
                        Biome::Mountain => "Highlands",
                        Biome::Jungle => "Jungle",
                    }
                    .to_string()
                }
            }
        };
        let mins = ((hour.fract()) * 60.0) as i32;
        let speed = if self.clock.speed > 1.0 { format!(" · {}×", self.clock.speed) } else { String::new() };
        let time = format!(
            "Day {} · {} · {:02}:{:02}{}",
            self.clock.day() % 12 + 1,
            self.clock.season_name(),
            hour as i32,
            mins,
            speed
        );
        // the touch menu shows what each option is set to
        if let Some(m) = self.doc.get_element_by_id("menu-panel") {
            let on = |b: bool| if b { "on" } else { "off" };
            let _ = m.set_attribute("data-auto", on(self.dir.auto));
            let _ = m.set_attribute("data-speed", &format!("{}×", self.clock.speed as i32));
            let _ = m.set_attribute("data-weather", self.weather.label());
            let _ = m.set_attribute("data-pixel", &self.r.pixel.to_string());
            let _ = m.set_attribute("data-hud", if self.hud_hidden { "hidden" } else { "shown" });
            let _ = m.set_attribute("data-news", on(self.news_on));
            let _ = m.set_attribute("data-map", on(self.map_on));
            let _ = m.set_attribute("data-chronicle", on(self.chron_on));
            let _ = m.set_attribute("data-photo", on(self.photo));
            let _ = m.set_attribute("data-record", on(is_recording().unwrap_or(false)));
        }
        let auto = if !self.dir.auto { " · autopilot off" } else { "" };
        self.set_text("hud-town", &place);
        self.set_text("hud-time", &time);
        self.set_text("hud-weather", &format!("{}{}", self.weather.label(), auto));
        let follow = self.hud_follow.clone();
        self.set_text("hud-follow", &follow);
    }

    pub(super) fn set_text(&self, id: &str, s: &str) {
        if let Some(e) = self.doc.get_element_by_id(id) {
            if e.text_content().as_deref() != Some(s) {
                e.set_text_content(Some(s));
            }
        }
    }
}
