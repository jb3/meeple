use super::super::*;

impl App {
    pub(in crate::app) fn start_travel(&mut self) {
        let here = Vec2::new(self.cam.target.x, self.cam.target.z);
        // every so often: a long journey, flown high and wide
        let long = self.dir.legs >= 4 && (self.dir.legs >= 8 || self.dir.rng.chance(0.35));
        if long {
            if let Some(c) = self.far_candidate(here) {
                let d = &mut self.dir;
                d.dest = c.pos;
                d.visited.push_back(c.id);
                if d.visited.len() > 10 {
                    d.visited.pop_front();
                }
                d.kinds.push_back(c.kind);
                if d.kinds.len() > 3 {
                    d.kinds.pop_front();
                }
                d.promise = None;
                d.reason = format!("A long journey: {}", c.why.to_lowercase());
                d.await_hour = None;
                d.heading = (d.dest - here).normalize_or_zero();
                d.checked_in = false;
                d.commit_t = 120.0;
                d.mode = Mode::Travel;
                d.legs = 0;
                d.expedition = true;
                d.zoom_target = d.rng.range(55.0, 75.0);
                d.pitch_target = d.rng.range(0.78, 0.92);
                self.hud_follow = format!("Heading for: {}", self.dir.reason);
                self.announce();
                return;
            }
        }
        self.dir.expedition = false;
        self.dir.legs += 1;
        let (hour, day, season) = (self.clock.hour(), self.clock.day(), self.clock.season());
        let sph = (self.clock.day_len / 24.0 / self.clock.speed.max(1.0)) as f32;
        // a story whose chapters have gone by is over; now and then a new one begins
        if let Some(s) = &mut self.dir.story {
            s.catch_up(day, hour);
        }
        if self.dir.story.as_ref().is_some_and(|s| s.finished()) {
            self.dir.story = None;
        }
        if self.dir.story.is_none() && self.dir.legs >= 2 && self.dir.rng.chance(0.3) {
            let mut rng = Rng::new(self.dir.rng.next_u32() as u64);
            self.dir.story = Story::plan(&self.world.gen, here, day, hour, season, &mut rng);
        }
        let cands = self.candidates(here, hour, day, season, sph);
        let d = &mut self.dir;
        let mut best: Option<(f32, usize)> = None;
        for (i, c) in cands.iter().enumerate() {
            let dist = c.pos.distance(here);
            if (dist < 120.0 && c.kind != "story") || d.visited.contains(&c.id) {
                continue;
            }
            // never the same sight twice running: salmon at one weir, then another
            if c.kind == "weir" && d.kinds.back() == Some(&"weir") {
                continue;
            }
            let along = (c.pos - here).normalize_or_zero().dot(d.heading);
            let mut pen = 0.0;
            if d.kinds.back() == Some(&c.kind) {
                pen += 0.7;
            } else if d.kinds.iter().rev().nth(1) == Some(&c.kind) {
                pen += 0.3;
            }
            // prefer the unusual: towns are common, so they're marked down, and
            // two in a row is too many
            let town = matches!(c.kind, "city" | "village");
            let town_run = d.kinds.iter().rev().take(2).filter(|k| matches!(**k, "city" | "village")).count();
            let flavour = if town { -0.5 - 0.5 * town_run as f32 } else { 0.5 };
            let score = c.weight * 1.35 + flavour + along * 0.7 - dist / 420.0 + d.rng.range(0.0, 0.8) - pen;
            if best.map_or(true, |b| score > b.0) {
                best = Some((score, i));
            }
        }
        match best {
            Some((_, i)) => {
                let c = &cands[i];
                d.dest = c.pos;
                d.visited.push_back(c.id);
                if d.visited.len() > 10 {
                    d.visited.pop_front();
                }
                d.kinds.push_back(c.kind);
                if d.kinds.len() > 3 {
                    d.kinds.pop_front();
                }
                d.reason = c.why.clone();
                d.await_hour = c.wait;
                d.promise = c.promise;
            }
            None => {
                let a = d.rng.range(-0.8, 0.8);
                let h = d.heading;
                d.heading = Vec2::new(h.x * a.cos() - h.y * a.sin(), h.x * a.sin() + h.y * a.cos());
                d.dest = here + d.heading * 300.0;
                d.reason = String::new();
                d.await_hour = None;
                d.promise = None;
            }
        }
        d.heading = (d.dest - here).normalize_or_zero();
        d.checked_in = false;
        d.commit_t = 25.0;
        d.mode = Mode::Travel;
        d.zoom_target = d.rng.range(24.0, 34.0);
        d.pitch_target = d.rng.range(0.55, 0.7);
        self.hud_follow = if self.dir.reason.is_empty() { String::new() } else { format!("Heading for: {}", self.dir.reason) };
        if !self.hud_follow.is_empty() {
            self.announce();
        }
    }

    /// On reaching a place: pick how to film it, and wait for the thing
    /// it was chosen for if it hasn't started.
    pub(in crate::app) fn arrive(&mut self) {
        // arriving for a chapter of the story turns the page
        let (dest, day, now_hour) = (self.dir.dest, self.clock.day(), self.clock.hour());
        if let Some(s) = &mut self.dir.story {
            if s.pos.distance(dest) < 30.0 && s.beat().is_some_and(|b| b.day == day && now_hour <= b.to + 0.2) {
                s.next += 1;
            }
            if s.finished() {
                self.dir.story = None;
            }
        }
        let sph = (self.clock.day_len / 24.0 / self.clock.speed.max(1.0)) as f32;
        let hour = self.clock.hour();
        let d = &mut self.dir;
        d.mode = Mode::Linger;
        d.expedition = false;
        d.commit_t = 15.0;
        d.center = d.dest;
        // towns are glanced at; the rarer places are lingered over
        let town = matches!(d.kinds.back(), Some(&"city") | Some(&"village"));
        d.timer = if town { d.rng.range(28.0, 55.0) } else { d.rng.range(55.0, 100.0) };
        if let Some(at) = d.await_hour.take() {
            // be there when it starts, and watch some of it
            let wait = ((at - hour).max(0.0) * sph + 20.0).min(150.0);
            d.timer = d.timer.max(wait + 25.0);
        }
        d.person_cd = d.rng.range(15.0, 35.0);
        d.shot = match d.rng.next_u32() % 5 {
            0 | 1 => Shot::Orbit,
            2 | 3 => Shot::Push,
            _ => Shot::Hold,
        };
        d.linger_total = d.timer;
        match d.shot {
            Shot::Orbit => {
                d.zoom_target = d.rng.range(15.0, 22.0);
                d.pitch_target = d.rng.range(0.55, 0.7);
            }
            Shot::Push => {
                d.zoom_target = 28.0;
                d.pitch_target = d.rng.range(0.5, 0.62);
            }
            Shot::Hold => {
                d.zoom_target = d.rng.range(12.0, 16.0);
                d.pitch_target = d.rng.range(0.42, 0.5);
            }
        }
        // a weir is a small thing: in close, to see the water and the leaping fish
        if d.kinds.back() == Some(&"weir") {
            d.shot = Shot::Hold;
            d.zoom_target = d.rng.range(7.0, 10.0);
            d.pitch_target = d.rng.range(0.42, 0.5);
        }
        if !d.reason.is_empty() {
            self.hud_follow = d.reason.clone();
        }
        self.announce();
    }

    /// The angle to view a haunted ruin's ghost from: it faces +x, so in front
    /// of it (and a little to one side) shows it best, but every direction is
    /// tried by walking a line of sight from the ghost's path out to where the
    /// camera would be through the real blocks, and the clearest wins, with a
    /// preference for the front.
    pub(in crate::app) fn best_ghost_yaw(&self, st: &crate::world::gen::Site) -> f32 {
        use crate::world::chunk;
        let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
        let (c0x, c0z) = World::chunk_of(x - 16.0, z - 16.0);
        let (c1x, c1z) = World::chunk_of(x + 16.0, z + 16.0);
        let mut chunks = Vec::new();
        for cx in c0x..=c1x {
            for cz in c0z..=c1z {
                chunks.push(chunk::generate(&self.world.gen, cx, cz));
            }
        }
        let block = |p: Vec3| -> u8 {
            let (cx, cz) = World::chunk_of(p.x, p.z);
            chunks.iter().find(|c| c.cx == cx && c.cz == cz).map_or(0, |c| c.get_w(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32))
        };
        let front = Vec2::new(1.0, 0.35).normalize();
        let pitch = 0.42f32;
        let gy = st.level as f32 + 1.0 + 1.0;
        let mut best = (f32::MAX, front.x.atan2(front.y));
        for k in 0..32 {
            let yaw = k as f32 / 32.0 * std::f32::consts::TAU;
            let dir = Vec3::new(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos());
            let mut blocked = 0.0;
            for f in [0.1f32, 0.3, 0.5, 0.7, 0.9] {
                let g = Vec3::new(x - 2.5 + f * 5.0, gy, z - 1.2 + (f * 3.0).sin() * 0.5);
                let mut t = 1.0;
                while t < 14.0 {
                    let b = block(g + dir * t);
                    if b != crate::world::blocks::AIR && b != crate::world::blocks::WATER {
                        blocked += 1.0;
                    }
                    t += 0.5;
                }
            }
            let facing = Vec2::new(yaw.sin(), yaw.cos()).dot(front);
            let cost = blocked + (1.0 - facing) * 4.0;
            if cost < best.0 {
                best = (cost, yaw);
            }
        }
        best.1
    }
}
