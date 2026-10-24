use super::*;

impl TownLife {
    pub(super) fn ground(&self, env: &Env, p: Vec2) -> f32 {
        if p.distance(self.center()) < self.town.radius as f32 - 1.0 {
            self.y
        } else {
            env.gen.walk_y(&self.ctx, p.x, p.y)
        }
    }

    /// The village green's centre, if the town has one.
    pub(super) fn green_at(&mut self, env: &Env) -> Option<Vec2> {
        let town = self.town.clone();
        *self.happen.green.get_or_insert_with(|| env.gen.green(&town).map(|(x, z)| Vec2::new(x as f32 + 0.5, z as f32 + 0.5)))
    }

    pub(super) fn lot_front(&self, l: usize) -> Vec2 {
        Vec2::from(self.lots[l].front)
    }

    /// Joins two spots by the street grid.
    pub(super) fn street_path(&mut self, a: Vec2, b: Vec2) -> Vec<Vec2> {
        let flip = self.rng.chance(0.5);
        let c = self.center();
        let mut pts = self.route(a, b, flip);
        // a street crossing right at the fountain: step round it, never to it
        for q in &mut pts {
            let d = *q - c;
            if d.x.abs().max(d.y.abs()) < FOUNTAIN_DETOUR {
                *q = if d.x.abs() >= d.y.abs() { c + Vec2::new(FOUNTAIN_DETOUR * if d.x < 0.0 { -1.0 } else { 1.0 }, d.y) } else { c + Vec2::new(d.x, FOUNTAIN_DETOUR * if d.y < 0.0 { -1.0 } else { 1.0 }) };
            }
        }
        // never through the fountain
        let mut out = vec![a];
        out.extend(avoid_square(a, &pts, self.center(), FOUNTAIN_CLEAR, FOUNTAIN_DETOUR));
        out
    }

    pub(super) fn house_lot(&mut self) -> Option<usize> {
        if self.homes.is_empty() {
            return None;
        }
        let k = self.rng.next_u32() as usize % self.homes.len();
        Some(self.homes[k])
    }

    /// Where the fair lets off its fireworks, in the last hour of its evening.
    pub fn finale(&self, env: &Env) -> Option<Vec3> {
        if !self.fair_on(env) || !(21.0..22.0).contains(&env.hour) || env.rain > 0.3 {
            return None;
        }
        let g = self.happen.green.flatten()?;
        Some(Vec3::new(g.x, self.y, g.y))
    }

    pub fn fair_on(&self, env: &Env) -> bool {
        let off = (hash_u32(self.town.id ^ 0xfa1) % FAIR_EVERY as u32) as i64;
        let winter = (2.95..4.0).contains(&env.season);
        !winter && (env.day + off).rem_euclid(FAIR_EVERY) < FAIR_DAYS && (10.0..23.5).contains(&env.hour)
    }

    /// Make something happen right now (for tests).
    #[cfg(test)]
    pub fn kindle(&mut self, kind: Kind) {
        self.happen.forced = Some(kind);
    }

    pub(super) fn start(&mut self, env: &Env, kind: Kind, lot: usize, path: Vec<Vec2>, dur: f32, note: String) {
        let seed = hash_u32(self.town.id ^ env.day as u32 ^ (kind.slot() as u32 * 0x9e37));
        self.happen.active.push(Active { kind, t0: env.time, dur, lot, path, seed });
        self.happen.log.push((env.day, note));
        if self.happen.log.len() > 40 {
            self.happen.log.remove(0);
        }
    }

    /// The routes of everything going on, for checking they stay on the streets.
    #[cfg(test)]
    pub fn event_paths(&self) -> Vec<(Kind, Vec<Vec2>)> {
        self.happen.active.iter().map(|a| (a.kind, a.path.clone())).collect()
    }

    /// Where the people in a christening or closing-time are right now.
    pub fn party_points(&self, kind: Kind) -> Vec<Vec3> {
        self.happen.party.borrow().iter().filter(|(k, _)| *k == kind).map(|(_, p)| *p).collect()
    }

    pub fn happening(&self, kind: Kind) -> bool {
        self.happen.active.iter().any(|a| a.kind == kind)
    }

    /// What's going on that's worth a look: for the camera director.
    pub fn interest(&self, env: &Env) -> Option<Interest> {
        let name = &self.town.name;
        let mut best: Option<Interest> = None;
        let mut offer = |weight: f32, key: &'static str, caption: String, at: Vec2, zoom: f32, secs: f32, lift: f32| {
            if best.as_ref().map_or(true, |b| weight > b.weight) {
                best = Some(Interest { weight, key, caption, at, zoom, secs, lift });
            }
        };
        for a in &self.happen.active {
            let t = env.time - a.t0;
            let lead = |speed: f32| walk(&a.path, t * speed).0;
            match a.kind {
                Kind::Fire => {
                    let (c, _, _, _) = self.roof_of(a.lot);
                    offer(3.2, "fire", format!("A house fire in {name}: the brigade is out"), c, 15.0, 55.0, 6.0)
                }
                // these three are worth getting right up close and staying with
                Kind::Christening => {
                    offer(3.5, "christening", format!("A christening in {name}"), lead(1.0), 9.5, a.dur - t + 2.0, 0.5)
                }
                Kind::Closing => {
                    let tau = (t - 3.0).max(0.0);
                    let at = walk(&a.path, (0.8 * tau + 0.38 * (1.7 * tau).sin()).max(0.0)).0;
                    offer(3.4, "closing", format!("Closing time at the inn in {name}"), at, 8.5, a.dur - t + 2.0, 0.5)
                }
                Kind::Chase => offer(1.3, "chase", format!("A dog on the loose in {name}"), self.center(), 18.0, 20.0, 0.0),
                _ => {}
            }
        }
        if self.fair_on(env) {
            if let Some(g) = self.happen.green.flatten() {
                offer(1.8, "fair", format!("The fair is on at {name}"), g, 22.0, 60.0, 2.0);
            }
        } else if self.cricket_on(env) {
            if let Some(g) = self.happen.green.flatten() {
                offer(1.1, "cricket", format!("Cricket on the green at {name}"), g, 20.0, 40.0, 0.0);
            }
        } else if self.snowball_fight_on(env) {
            if let Some(g) = self.happen.green.flatten() {
                offer(1.3, "snowballs", format!("A snowball fight on the green at {name}"), g, 17.0, 40.0, 0.5);
            }
        }
        best
    }

    pub fn set_fire_risk(&mut self, risk: f32) {
        self.happen.risk = risk;
    }

    /// Lightning strikes a house here: a fire starts at the next opportunity.
    pub fn lightning(&mut self) {
        if !self.homes.is_empty() && !self.happening(Kind::Fire) {
            self.happen.forced = Some(Kind::Fire);
            self.happen.struck = true;
        }
    }

    pub fn update_happenings(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        let day = env.day;
        let h = env.hour;
        self.green_at(env);
        let mut forced = self.happen.forced.take();
        // a christening the camera's coming for happens as soon as it arrives,
        // whatever the dice, the weather and the usual hour
        let cs = Kind::Christening.slot();
        let expected_here = self.visit.is_some_and(|v| v.occasion == Occasion::Christening && v.day == day && v.arrived);
        let window = if expected_here { 0.0..13.0 } else { 10.0..13.0 };
        if forced.is_none() && self.church.is_some() && self.promised(Occasion::Christening, day) && window.contains(&h)
            && self.happen.rolled[cs] != day && !self.held(Occasion::Christening, env) && !self.happening(Kind::Christening)
        {
            forced = Some(Kind::Christening);
            self.happen.rolled[cs] = day;
        }
        let roll = |me: &mut TownLife, kind: Kind, window: std::ops::Range<f32>, hourly: bool, chance: f32| -> bool {
            let s = kind.slot();
            if forced == Some(kind) {
                return true;
            }
            if !window.contains(&h) {
                return false;
            }
            let hr = env.hour as i32;
            if hourly {
                if me.happen.rolled[s] == day && me.happen.rolled_hour[s] == hr {
                    return false;
                }
                me.happen.rolled_hour[s] = hr;
            } else if me.happen.rolled[s] == day {
                return false;
            }
            me.happen.rolled[s] = day;
            unit(hash3(me.town.id, day as i32, hr * hourly as i32, 0x4a9 + s as i32)) < chance
        };
        let busy = |me: &TownLife, k: Kind| me.happening(k);

        if !self.homes.is_empty() && !busy(self, Kind::Fire) && !self.held(Occasion::Fire, env) && roll(self, Kind::Fire, 13.0..19.0, false, FIRE_CHANCE * self.happen.risk.max(1.0)) {
            if let Some(lot) = self.house_lot() {
                let c = self.center();
                let edge = {
                    let dir = (Vec2::from(self.lots[lot].center()) - c).normalize_or_zero();
                    let dir = if dir.x.abs() > dir.y.abs() { Vec2::new(dir.x.signum(), 0.0) } else { Vec2::new(0.0, dir.y.signum()) };
                    c + dir * (self.town.radius as f32 - 3.0)
                };
                let front = self.lot_front(lot);
                let path = self.street_path(edge, front);
                let n = if std::mem::take(&mut self.happen.struck) {
                    format!("Lightning struck a house in {}; the brigade came with the pump.", self.town.name)
                } else {
                    format!("A house in {} caught fire; the brigade came with the pump.", self.town.name)
                };
                self.start(env, Kind::Fire, lot, path, 110.0, n);
                self.happen.soot.push((lot, day));
            }
        }
        if let Some(ch) = self.church {
            if !busy(self, Kind::Christening) && (self.fine_for(Occasion::Christening, env) || forced == Some(Kind::Christening)) && !self.held(Occasion::Christening, env) && roll(self, Kind::Christening, 10.0..13.0, false, CHRISTENING_CHANCE) {
                if let Some(home) = self.house_lot() {
                    let (a, b) = (self.lot_front(home), self.lot_front(ch));
                    let path = self.street_path(a, b);
                    let dur = path_len(&path) / 1.0 + 26.0;
                    let n = format!("A baby was christened at the church in {}.", self.town.name);
                    self.start(env, Kind::Christening, ch, path, dur, n);
                }
            }
        }
        if self.homes.len() >= 3 && !busy(self, Kind::Chase) && env.nice && roll(self, Kind::Chase, 10.0..17.0, true, CHASE_CHANCE) {
            let c = self.center();
            self.start(env, Kind::Chase, 0, vec![c], 30.0, String::new());
            self.happen.log.pop();
        }
        if self.homes.len() >= 4 && !busy(self, Kind::Doctor) && roll(self, Kind::Doctor, 9.0..17.0, true, DOCTOR_CHANCE) {
            if let (Some(a), Some(b)) = (self.house_lot(), self.house_lot()) {
                if a != b {
                    let path = self.street_path(self.lot_front(a), self.lot_front(b));
                    let dur = path_len(&path) / 1.25 + 8.0;
                    self.start(env, Kind::Doctor, b, path, dur, String::new());
                    self.happen.log.pop();
                }
            }
        }
        if let Some(t) = self.tavern {
            if !busy(self, Kind::Closing) && !self.homes.is_empty() && roll(self, Kind::Closing, 23.0..23.9, false, CLOSING_CHANCE) {
                if let Some(home) = self.house_lot() {
                    let path = self.street_path(self.lot_front(t), self.lot_front(home));
                    let dur = path_len(&path) / 0.8 + 6.0;
                    self.start(env, Kind::Closing, t, path, dur, String::new());
                    self.happen.log.pop();
                    let (x, z) = self.lots[t].center();
                    sounds.push(Sound { name: "bell", pos: Vec3::new(x, self.y + 3.0, z), vol: 0.7, pitch: 1.5 });
                }
            }
        }

        // running events: sounds, then retire the finished
        let time = env.time;
        let mut sfx: Vec<Sound> = Vec::new();
        let mut bell_t = self.happen.bell_t - env.dt;
        for a in &self.happen.active {
            let t = time - a.t0;
            let c = self.center();
            match a.kind {
                Kind::Fire if t < 28.0 && bell_t <= 0.0 => {
                    bell_t = 1.3;
                    if let Some(ch) = self.church {
                        let (x, z) = self.lots[ch].center();
                        sfx.push(Sound { name: "bell", pos: Vec3::new(x, self.y + 12.0, z), vol: 0.9, pitch: 1.35 });
                    }
                }
                Kind::Christening if t > 0.5 && bell_t <= 0.0 && t > a.dur * 0.45 && t < a.dur * 0.7 => {
                    bell_t = 1.6;
                    if let Some(ch) = self.church {
                        let (x, z) = self.lots[ch].center();
                        sfx.push(Sound { name: "bell", pos: Vec3::new(x, self.y + 12.0, z), vol: 0.9, pitch: 1.0 + 0.12 * (a.seed % 3) as f32 });
                    }
                }
                Kind::Chase if t.fract() < env.dt && (t as i32) % 2 == 0 => {
                    sfx.push(Sound { name: "bark", pos: Vec3::new(c.x, self.y, c.y), vol: 0.8, pitch: 1.0 });
                }
                Kind::Fire if (t - 96.0).abs() < env.dt => {
                    let (x, z) = self.lots[a.lot].center();
                    sfx.push(Sound { name: "hiss", pos: Vec3::new(x, self.y + 2.0, z), vol: 0.8, pitch: 1.0 });
                }
                _ => {}
            }
        }
        self.happen.bell_t = if bell_t <= 0.0 && !self.happen.active.is_empty() { bell_t.max(-1.0) } else { bell_t };
        sounds.extend(sfx);
        self.happen.active.retain(|a| time - a.t0 < a.dur);
        let day_now = env.day;
        self.happen.soot.retain(|s| day_now - s.1 < 2);

        // the fair's calliope, the cricketers' cheers, the fiddler
        self.ambient_sounds(env, sounds);
    }

    pub(super) fn ambient_sounds(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        let Some(g) = self.happen.green.flatten() else { return };
        let dt = env.dt;
        let at = |p: Vec2, y: f32| Vec3::new(p.x, y + 1.5, p.y);
        if self.fair_on(env) && env.rain < 0.5 {
            self.happen.music_t -= dt;
            if self.happen.music_t <= 0.0 {
                // a jaunty calliope tune
                const TUNE: [f32; 16] = [1.0, 1.26, 1.5, 2.0, 1.5, 1.26, 1.5, 1.26, 1.12, 1.41, 1.68, 2.24, 1.68, 1.41, 1.5, 1.0];
                self.happen.music_t = 0.28;
                let n = self.happen.note;
                self.happen.note = (n + 1) % TUNE.len();
                sounds.push(Sound { name: "calliope", pos: at(g, self.y), vol: 0.5, pitch: TUNE[n] });
                if n % 4 == 0 {
                    sounds.push(Sound { name: "calliope", pos: at(g, self.y), vol: 0.35, pitch: TUNE[n] * 0.5 });
                }
            }
            // and now and then the strongman rings his bell
            let k = (env.time / 7.0) as i32;
            if (env.time / 7.0).fract() * 7.0 < dt && hash_u32(k as u32 ^ self.town.id) % 3 == 0 {
                sounds.push(Sound { name: "clang", pos: at(g, self.y), vol: 0.9, pitch: 1.0 });
                sounds.push(Sound { name: "cheer", pos: at(g, self.y), vol: 0.5, pitch: 1.0 });
            }
        } else if self.cricket_on(env) {
            self.happen.cheer_t -= dt;
            if self.happen.cheer_t <= 0.0 {
                self.happen.cheer_t = 8.0;
                if unit(hash_u32((env.time / 8.0) as u32 ^ self.town.id)) < 0.5 {
                    sounds.push(Sound { name: "clang", pos: at(g, self.y), vol: 0.5, pitch: 2.4 });
                    sounds.push(Sound { name: "cheer", pos: at(g, self.y), vol: 0.45, pitch: 1.0 });
                }
            }
        }
        if let Some(t) = self.tavern {
            if (19.2..23.0).contains(&env.hour) && env.rain < 0.5 {
                self.happen.music_t -= dt;
                if self.happen.music_t <= 0.0 {
                    const JIG: [f32; 12] = [1.0, 1.12, 1.26, 1.5, 1.26, 1.12, 1.0, 0.84, 1.0, 1.26, 1.5, 1.68];
                    self.happen.music_t = 0.24;
                    let n = self.happen.note;
                    self.happen.note = (n + 1) % JIG.len();
                    let (x, z) = self.lots[t].center();
                    if n % 12 != 11 {
                        sounds.push(Sound { name: "fiddle", pos: Vec3::new(x, self.y + 1.0, z), vol: 0.28, pitch: JIG[n] });
                    }
                }
            }
        }
    }

    /// Snowballs fly on the green on winter afternoons.
    pub fn snowball_fight_on(&self, env: &Env) -> bool {
        (2.95..3.95).contains(&env.season) && (12.0..16.5).contains(&env.hour) && env.rain < 0.4 && !self.fair_on(env) && self.happen.green.flatten().is_some()
    }

    pub(super) fn cricket_on(&self, env: &Env) -> bool {
        (1.0..2.0).contains(&env.season) && (13.0..18.0).contains(&env.hour) && env.nice && !self.fair_on(env) && self.happen.green.flatten().is_some()
    }

    pub fn glows(&self, env: &Env) -> Vec<[f32; 4]> {
        let mut out = Vec::new();
        for a in &self.happen.active {
            if a.kind == Kind::Fire {
                let t = env.time - a.t0;
                let k = fire_level(t);
                if k > 0.05 {
                    let (c, _, _, apex) = self.roof_of(a.lot);
                    let flick = 1.0 + (env.time * 13.0).sin() * 0.12 + (env.time * 29.0).sin() * 0.08;
                    out.push([c.x, apex + 1.0, c.y, 3.6 * k * flick]);
                }
            }
        }
        if self.fair_on(env) && env.night > 0.3 {
            if let Some(g) = self.happen.green.flatten() {
                for (dx, dz) in [(0.0, 0.0), (-7.0, 4.0), (7.0, 4.0)] {
                    out.push([g.x + dx, self.y + 3.0, g.y + dz, 2.0]);
                }
            }
        }
        out
    }

    /// Smoke columns (position, size) rising from fires and sooty ruins.
    pub fn smokes(&self, env: &Env) -> Vec<(Vec3, f32)> {
        let mut out = Vec::new();
        for a in &self.happen.active {
            if a.kind == Kind::Fire {
                let k = fire_level(env.time - a.t0);
                if k > 0.05 {
                    let (c, _, _, apex) = self.roof_of(a.lot);
                    out.push((Vec3::new(c.x, apex + 3.2, c.y), 0.12 + 0.2 * k));
                }
            }
        }
        out
    }
}
