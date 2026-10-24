use super::*;

impl TownLife {
    /// Is the gathering on now, at its proper hours?
    pub(super) fn gather_open(&self, env: &Env) -> bool {
        match &self.gathering {
            Some(g) if g.festival == Festival::Wedding => env.hour < WEDDING_END && self.wedding.is_some(),
            Some(_) => GATHER_HOURS.contains(&env.hour),
            None => false,
        }
    }

    /// How much of the party has reached the gathering (0..1).
    pub fn gathered(&self) -> f32 {
        self.gathering.as_ref().map_or(0.0, |g| (g.joined as f32 / g.expected.max(1) as f32).min(1.0))
    }

    /// Each fine evening there's a chance of a gathering in the plaza; it
    /// breaks up at the end of the evening or if the weather turns.
    pub(super) fn update_gathering(&mut self, env: &Env) {
        let evening = GATHER_HOURS.contains(&env.hour);
        let promised = self.promised(Occasion::Gathering, env.day);
        let fine = self.fine_for(Occasion::Gathering, env);
        // the camera's on its way: don't start until it gets here
        if evening && env.day != self.gather_day && self.gathering.is_none() && !self.held(Occasion::Gathering, env) {
            self.gather_day = env.day;
            let roll = unit(hash3(self.town.id, env.day as i32, 7, 77));
            if fine && (roll < GATHER_CHANCE || promised) {
                // decide now who's coming (most folk; the innkeeper stays on,
                // and children only early on) so turnout is known up front
                let day = env.day as u32;
                let late = env.hour > 20.5;
                let coming: Vec<bool> = self
                    .people
                    .iter()
                    .map(|p| {
                        p.role != Role::Barkeep
                            && !(p.role == Role::Child && late)
                            && unit(hash_u32(p.seed ^ day.wrapping_mul(0x9e37_79b9) ^ 0x6a7)) < if promised { 0.95 } else { 0.8 }
                    })
                    .collect();
                let expected = coming.iter().filter(|c| **c).count();
                // not worth calling with only a handful
                if expected >= if promised { 3 } else { MIN_TURNOUT } {
                    let spin = if hash_u32(self.town.id ^ day) % 2 == 0 { 1.0 } else { -1.0 };
                    let festival = Festival::for_season(env.season);
                    self.gathering = Some(Gathering { musician: None, joined: 0, expected, spin, festival });
                    // word gets round: those coming set off soon
                    for (p, c) in self.people.iter_mut().zip(coming) {
                        p.coming = c;
                        if c && !matches!(p.state, State::Walking | State::Chatting(_)) {
                            p.timer = p.timer.min(unit(hash_u32(p.seed ^ 0x9a7)) * 20.0);
                        }
                    }
                }
            }
        }
        let wedding = self.gathering.as_ref().is_some_and(|g| g.festival == Festival::Wedding);
        let open = if wedding { env.hour < WEDDING_END } else { evening };
        if self.gathering.is_some() && (!open || (!fine && !wedding)) {
            self.gathering = None;
            for p in &mut self.people {
                p.coming = false;
                if matches!(p.state, State::Dancing | State::Singing | State::Playing) {
                    p.timer = p.timer.min(unit(hash_u32(p.seed ^ env.slot)) * 4.0);
                }
            }
        }
        let c = self.center();
        let spin = self.gathering.as_ref().map_or(1.0, |g| g.spin);
        for p in &mut self.people {
            match p.state {
                State::Dancing if self.stalls => {
                    // on the spot: hop and twirl
                    p.phase += env.dt * 6.0;
                    p.yaw += env.dt * 2.2 * spin;
                }
                State::Dancing => {
                    // circle dance round the fountain, easing onto the ring
                    p.dance_a += env.dt * 0.16 * spin;
                    let ring = if matches!(p.wed, Wed::Bride | Wed::Groom) { COUPLE_RING } else { DANCE_RING };
                    p.dance_r += (ring - p.dance_r) * approach(1.5, env.dt);
                    let out = Vec2::new(p.dance_a.cos(), p.dance_a.sin());
                    p.pos = c + out * p.dance_r;
                    p.yaw = yaw_of(Vec2::new(-out.y, out.x) * spin);
                    p.phase += env.dt * 5.0;
                }
                State::Singing | State::Playing => p.phase += env.dt * 3.0,
                _ => {}
            }
        }
        if !self.stalls {
            self.space_dancers(env.dt);
        }
    }

    /// Children playing in the plaza play proper games: tag, catch, or
    /// skipping, a different one each half hour.
    pub(super) fn play_games(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        let players: Vec<usize> = (0..self.people.len())
            .filter(|&i| {
                let p = &self.people[i];
                p.visible
                    && p.role == Role::Child
                    && p.goal == Goal::Play
                    // those called to the gathering leave the game
                    && !(p.coming && self.gather_open(env))
                    && p.at == Place::Plaza
                    && matches!(p.state, State::Idle | State::Walking)
            })
            .collect();
        if players.len() < 2 {
            self.game = None;
            return;
        }
        let h = hash_u32(self.town.id ^ env.slot.wrapping_mul(0x51ed));
        if self.game.as_ref().map_or(true, |g| g.slot != env.slot) {
            let kind = match h % 3 {
                0 => GameKind::Tag,
                1 => GameKind::Ball,
                _ if players.len() >= 3 => GameKind::Skipping,
                _ => GameKind::Ball,
            };
            self.game = Some(Game { kind, slot: env.slot, it: players[0], ball: (players[0], players[1], 0.0), rope: 0.0, retarget: 0.0 });
        }
        let Some(mut g) = self.game.take() else { return };
        let c = self.center();
        let dt = env.dt;
        match g.kind {
            GameKind::Tag => {
                if !players.contains(&g.it) {
                    g.it = players[0];
                }
                let it = self.people[g.it].pos;
                let near = players
                    .iter()
                    .copied()
                    .filter(|&k| k != g.it)
                    .min_by(|a, b| self.people[*a].pos.distance(it).partial_cmp(&self.people[*b].pos.distance(it)).unwrap())
                    .unwrap();
                g.retarget -= dt;
                if g.retarget <= 0.0 {
                    g.retarget = 0.4;
                    let to = self.people[near].pos;
                    self.set_path(g.it, vec![to], Dest::Plaza);
                }
                if self.people[near].pos.distance(it) < 0.75 {
                    // tagged! the new "it" counts to one before chasing
                    sounds.push(Sound { name: "chat_laugh", pos: self.people[near].pos3(), vol: 0.7, pitch: self.people[near].voice() });
                    g.it = near;
                    g.retarget = 1.2;
                }
                for &k in &players {
                    let p = &self.people[k];
                    if k != g.it && p.state == State::Idle && p.pos.distance(it) < 3.5 {
                        let mut to = p.pos + (p.pos - it).normalize_or(Vec2::X) * 4.5;
                        if to.distance(c) > 7.5 {
                            let d = (to - c).normalize_or(Vec2::X);
                            to = c + Vec2::new(d.x * 0.7 - d.y * 0.7, d.x * 0.7 + d.y * 0.7) * 7.0;
                        }
                        self.set_path(k, vec![to], Dest::Plaza);
                    }
                }
            }
            GameKind::Ball | GameKind::Skipping => {
                let a0 = unit(h >> 4) * std::f32::consts::TAU;
                let n = players.len();
                let spot = |k: usize| -> Vec2 {
                    if g.kind == GameKind::Ball {
                        let a = a0 + k as f32 * std::f32::consts::TAU / n as f32;
                        c + Vec2::new(a.cos(), a.sin()) * 5.0
                    } else {
                        let mid = c + Vec2::new(a0.cos(), a0.sin()) * 5.6;
                        let side = Vec2::new(-a0.sin(), a0.cos());
                        match k {
                            0 => mid + side * 1.4,
                            1 => mid - side * 1.4,
                            k => mid + Vec2::new(a0.cos(), a0.sin()) * ((k - 2) as f32 * 0.7),
                        }
                    }
                };
                let mut ready = true;
                for (k, &i) in players.iter().enumerate() {
                    let to = spot(k);
                    let p = &self.people[i];
                    if p.pos.distance(to) > 0.4 {
                        ready = false;
                        if p.state == State::Idle {
                            self.set_path(i, vec![to], Dest::Plaza);
                        }
                    } else {
                        // stay in the game
                        let p = &mut self.people[i];
                        p.timer = p.timer.max(1.0);
                    }
                }
                if g.kind == GameKind::Ball {
                    if !players.contains(&g.ball.0) || !players.contains(&g.ball.1) || g.ball.0 == g.ball.1 {
                        g.ball = (players[0], players[1], 0.0);
                    }
                    if ready {
                        g.ball.2 += dt / 1.1;
                        if g.ball.2 >= 1.0 {
                            let next = players[(hash_u32(h ^ (env.time * 3.0) as u32) as usize) % n];
                            let next = if next == g.ball.1 { players[(players.iter().position(|&q| q == next).unwrap() + 1) % n] } else { next };
                            g.ball = (g.ball.1, next, 0.0);
                        }
                    }
                    let holder = self.people[g.ball.0].pos;
                    for &i in &players {
                        let p = &mut self.people[i];
                        if p.state == State::Idle && p.pos.distance(holder) > 0.1 {
                            p.yaw = yaw_of(holder - p.pos);
                        }
                    }
                } else if ready {
                    g.rope += dt * 5.0;
                    let mid = (self.people[players[0]].pos + self.people[players[1]].pos) * 0.5;
                    for &i in &players[2..] {
                        // jump as the rope comes round under your feet
                        let p = &mut self.people[i];
                        p.y += g.rope.cos().max(0.0).powf(4.0) * 0.35;
                        p.yaw = yaw_of(mid - c);
                    }
                    for &i in &players[..2] {
                        let p = &mut self.people[i];
                        p.yaw = yaw_of(mid - p.pos);
                    }
                }
            }
        }
        self.game = Some(g);
    }

    /// Spread the circle dancers evenly: neighbours closer than an arm's
    /// length round the ring drift apart.
    pub(super) fn space_dancers(&mut self, dt: f32) {
        let mut ring: Vec<(f32, usize)> = self
            .people
            .iter()
            .enumerate()
            .filter(|(_, p)| p.state == State::Dancing && !matches!(p.wed, Wed::Bride | Wed::Groom))
            .map(|(i, p)| (p.dance_a.rem_euclid(std::f32::consts::TAU), i))
            .collect();
        if ring.len() < 2 {
            return;
        }
        ring.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let gap_min = (DANCE_SPACING / DANCE_RING).min(std::f32::consts::TAU / ring.len() as f32);
        let k = approach(1.5, dt);
        for w in 0..ring.len() {
            let (a, ia) = ring[w];
            let (b, ib) = ring[(w + 1) % ring.len()];
            let gap = (b - a).rem_euclid(std::f32::consts::TAU);
            if gap < gap_min {
                let nudge = (gap_min - gap) * 0.5 * k;
                self.people[ia].dance_a -= nudge;
                self.people[ib].dance_a += nudge;
            }
        }
    }

    /// Plaza centre, gathering size (dancers and musician) and number singing,
    /// for the soundscape; None if nobody's gathered yet.
    pub fn festival(&self) -> Option<(Vec3, f32, f32)> {
        self.gathering.as_ref()?;
        let (mut band, mut choir) = (0.0, 0.0);
        for p in self.people.iter().filter(|p| p.visible) {
            match p.state {
                State::Dancing | State::Playing => band += 1.0,
                State::Singing => choir += 1.0,
                _ => {}
            }
        }
        if band + choir == 0.0 {
            return None;
        }
        let c = self.center();
        Some((Vec3::new(c.x, self.y, c.y), band, choir))
    }
}
