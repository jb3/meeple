use super::*;

impl TownLife {
    /// Walk along the path, steering round people in the way. `crowd` is
    /// where everyone out and about was at the start of this frame.
    pub(crate) fn step(&mut self, i: usize, dt: f32, crowd: &[Option<Vec2>]) -> bool {
        let play = self.people[i].goal == Goal::Play;
        let speed = self.people[i].speed * if play { 1.4 } else { 1.0 };
        let mut left = speed * dt;
        // barely moved for a couple of seconds despite walking: stuck behind
        // people standing in the way; skip ahead and push through for a bit
        {
            let p = &mut self.people[i];
            if p.pos.distance(p.jam_pos) > 0.4 {
                p.jam_pos = p.pos;
                p.jam_t = 0.0;
            } else {
                p.jam_t += dt;
            }
            p.ghost_t = (p.ghost_t - dt).max(0.0);
        }
        let jammed = self.people[i].jam_t > 2.2;
        if jammed {
            let p = &mut self.people[i];
            p.jam_t = 0.0;
            p.ghost_t = 1.6;
        }
        loop {
            let p = &self.people[i];
            if p.pi >= p.path.len() {
                break;
            }
            let (t, pos) = (p.path[p.pi], p.pos);
            let d = t - pos;
            let dist = d.length();
            // nearly there but getting nowhere (others are standing on the
            // spot): close enough
            // (and, held up for good by people standing in the way, press on
            // towards the one after: don't stand there forever)
            let blocked = (dist < STALL_RADIUS && p.stall_t > STALL_TIME) || (jammed && dist < 6.0);
            if dist <= left || blocked {
                let p = &mut self.people[i];
                if !blocked {
                    p.pos = t;
                    left -= dist;
                }
                p.pi += 1;
                p.stall_best = f32::MAX;
                p.stall_t = 0.0;
                if blocked {
                    break;
                }
                continue;
            }
            let dir = d / dist;
            let nudge = if self.people[i].ghost_t > 0.0 { Vec2::ZERO } else { self.steer(i, pos, dir, crowd) };
            let want = (dir + nudge).normalize_or(dir);
            // turn smoothly; never let the smoothed heading point backwards
            let h = self.people[i].heading;
            let mv = if h.dot(dir) <= 0.0 { want } else { (h + (want - h) * approach(8.0, dt)).normalize_or(want) };
            self.people[i].heading = mv;
            // never into a wall: steer, else head straight on, else slide
            // along it, else wait a moment
            let step = [mv * left, dir * left, Vec2::new(dir.x * left, 0.0), Vec2::new(0.0, dir.y * left)];
            let to = step.iter().map(|s| pos + *s).find(|q| !self.inside_building(*q)).unwrap_or(pos);
            let p = &mut self.people[i];
            p.yaw = lerp_angle(p.yaw, yaw_of(mv), approach(8.0, dt));
            p.pos = to;
            if dist < p.stall_best - 0.05 {
                p.stall_best = dist;
                p.stall_t = 0.0;
            } else {
                p.stall_t += dt;
            }
            break;
        }
        let p = &mut self.people[i];
        p.phase += dt * speed * 4.5;
        p.pi >= p.path.len()
    }

    /// A sideways nudge to slip past whoever is just ahead. People pass on
    /// the side the other isn't on; anyone dead ahead is passed on the right,
    /// so two people walking at each other both step aside and pass.
    pub(crate) fn steer(&self, i: usize, pos: Vec2, dir: Vec2, crowd: &[Option<Vec2>]) -> Vec2 {
        let right = Vec2::new(dir.y, -dir.x);
        let mut s = Vec2::ZERO;
        for (k, o) in crowd.iter().enumerate() {
            let Some(o) = o else { continue };
            if k == i {
                continue;
            }
            let r = *o - pos;
            let ahead = r.dot(dir);
            let side = r.dot(right);
            if ahead <= 0.0 || ahead > LOOK_AHEAD || side.abs() > PASS_WIDTH {
                continue;
            }
            let away = if side > 0.15 { -right } else { right };
            s += away * (1.0 - ahead / LOOK_AHEAD) * (PASS_WIDTH - side.abs()) * 1.8;
        }
        s
    }

    /// Overnight, new houses are finished and families move in, and children
    /// grow up.
    pub(crate) fn next_day(&mut self, env: &Env) {
        let first = env.gen.first_day.get();
        let lots = self.lots.clone();
        for (h, lot) in lots.iter().enumerate() {
            if lot.finished_day(first) != Some(env.day) || self.homes.contains(&h) {
                continue;
            }
            self.homes.push(h);
            for k in 0..lot.residents() {
                let seed = hash3(lot.seed, k as i32, 3, 7);
                let role = if k > 0 && unit(seed) < 0.3 { Role::Child } else { Role::Villager };
                let p = self.make_person(seed, role, h, None);
                self.people.push(p);
            }
        }
        // houses left to fall down: their families move in elsewhere
        let ruined: Vec<usize> = self.homes.iter().copied().filter(|&h| lots[h].stage(first, env.day) == Stage::Ruin).collect();
        if !ruined.is_empty() {
            self.homes.retain(|h| !ruined.contains(h));
            for h in ruined {
                let to = (!self.homes.is_empty()).then(|| self.homes[hash_u32(lots[h].seed) as usize % self.homes.len()]);
                let who: Vec<usize> = (0..self.people.len()).filter(|&i| self.people[i].home == h).collect();
                if let (Some(t), false) = (to, who.is_empty()) {
                    for &i in &who {
                        let p = &mut self.people[i];
                        p.home = t;
                        if p.at == Place::Lot(h) {
                            p.at = Place::Lot(t);
                            p.pos = Vec2::from(lots[t].front);
                        }
                    }
                    let note = format!("The {} family left their old house in {}, now falling to ruin.", crate::names::surname(lots[h].seed), self.town.name);
                    self.happen.log.push((env.day, note));
                    if self.happen.log.len() > 40 {
                        self.happen.log.remove(0);
                    }
                }
            }
        }
        self.family_day(env);
        for i in 0..self.people.len() {
            if self.people[i].role == Role::Child && env.day >= self.people[i].grown_day(first) {
                self.come_of_age(i, env);
            }
        }
        self.married_today.clear();
        self.today = env.day;
    }

    /// Hammering from houses going up, through the working day.
    pub(crate) fn building_work(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        if !(8.0..17.0).contains(&env.hour) || env.rain > 0.5 {
            return;
        }
        let k = (env.time * 0.7) as u32;
        if (env.time * 0.7).fract() > env.dt * 0.7 {
            return;
        }
        for lot in self.lots.iter() {
            if matches!(env.gen.stage(lot), Stage::Frame(_)) && hash_u32(lot.seed ^ k) % 3 == 0 {
                let (x, z) = lot.center();
                let pitch = 1.5 + unit(hash_u32(lot.seed ^ k ^ 9)) * 0.4;
                sounds.push(Sound { name: "clang", pos: Vec3::new(x, self.y + 2.0, z), vol: 0.22, pitch });
            }
        }
    }

    /// Personal space: anyone walking or standing about is nudged away from
    /// people close by and from carts, so they curve round each other
    /// instead of passing through.
    pub(crate) fn keep_apart(&mut self, carts: &[(Vec2, Vec2)], dt: f32) {
        const SPACE: f32 = 0.6;
        const CART_SPACE: f32 = 1.1;
        let n = self.people.len();
        for i in 0..n {
            let a = &self.people[i];
            if !a.visible || !matches!(a.state, State::Walking | State::Idle | State::Singing) || a.ghost_t > 0.0 {
                continue;
            }
            let (mut push, mut around) = (Vec2::ZERO, Vec2::ZERO);
            for (j, b) in self.people.iter().enumerate() {
                if j == i || !b.visible || b.state == State::Inside {
                    continue;
                }
                let d = a.pos - b.pos;
                let dist = d.length();
                if dist < SPACE {
                    // exactly overlapping: split them sideways by index
                    let dir = if dist > 1e-3 { d / dist } else { Vec2::new(if i < j { 1.0 } else { -1.0 }, 0.0) };
                    // someone sitting, working or chatting can't step aside,
                    // so it's all on the one moving
                    if matches!(b.state, State::Walking | State::Idle | State::Singing) {
                        push += dir * (SPACE - dist) * 0.5;
                    } else {
                        around += dir * (SPACE - dist);
                    }
                }
            }
            // a soft, speed-limited ease apart: a stiff shove every frame
            // fights the walking and makes crowds shake; stepping round
            // someone standing still can be brisker, as only one side moves
            let mut delta = (push * approach(5.0, dt)).clamp_length_max(1.2 * dt)
                + (around * approach(8.0, dt)).clamp_length_max(2.6 * dt);
            // steering takes people round a cart; this only eases anyone who
            // ends up right against one (a cart rolling up beside them, or
            // parked in the street) out of it, softly and straight away, so
            // there's nothing to fight the walking and shake
            for &(o, fwd) in carts {
                let d = a.pos - o;
                let dist = d.length();
                if dist < CART_SPACE {
                    let dir = if dist > 1e-3 { d / dist } else { Vec2::new(fwd.y, -fwd.x) };
                    delta += (dir * (CART_SPACE - dist) * approach(4.0, dt)).clamp_length_max(1.0 * dt);
                }
            }
            if delta != Vec2::ZERO {
                // never nudge anyone into a wall
                let to = self.people[i].pos + delta;
                if !self.inside_building(to) {
                    self.people[i].pos = to;
                }
            }
        }
    }

    pub(crate) fn socialise(&mut self, env: &Env) {
        let n = self.people.len();
        for i in 0..n {
            let a = &self.people[i];
            // the bride and groom have somewhere to be
            let busy = |p: &Person| matches!(p.wed, Wed::Bride | Wed::Groom) && p.state == State::Walking;
            if !a.visible || a.chat_cd > 0.0 || !matches!(a.state, State::Walking | State::Idle) || busy(a) {
                continue;
            }
            for j in i + 1..n {
                let b = &self.people[j];
                if !b.visible || b.chat_cd > 0.0 || !matches!(b.state, State::Walking | State::Idle) || busy(b) {
                    continue;
                }
                // close enough to strike up a conversation (two blocks)
                if a.pos.distance_squared(b.pos) > 4.0 {
                    continue;
                }
                let h = hash_u32(a.seed ^ b.seed ^ env.slot);
                let both_cd = 8.0 + unit(h) * 12.0;
                if unit(h >> 3) < 0.55 {
                    let friends = a.friend == Some(j) || b.friend == Some(i);
                    let emote = if friends && unit(h >> 5) < 0.5 {
                        Emote::Heart
                    } else {
                        [Emote::Talk, Emote::Talk, Emote::Laugh, Emote::Note][(h >> 7) as usize % 4]
                    };
                    let dur = 5.0 + unit(h >> 9) * 6.0;
                    for (me, other, first) in [(i, j, true), (j, i, false)] {
                        let p = &mut self.people[me];
                        p.resume = if p.state == State::Walking { State::Walking } else { State::Idle };
                        p.state = State::Chatting(other);
                        p.timer = dur;
                        p.emote = emote;
                        p.chat_start = env.time;
                        p.first_speaker = first;
                        p.talk_slot = -1;
                        if unit(h >> 11) < 0.3 {
                            p.friend = Some(other);
                        }
                    }
                } else {
                    self.people[i].chat_cd = both_cd;
                    self.people[j].chat_cd = both_cd;
                }
                break;
            }
        }
    }
}
