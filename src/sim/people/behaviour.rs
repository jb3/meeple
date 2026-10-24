use super::*;

mod motion;

impl TownLife {
    pub(super) fn think(&mut self, i: usize, env: &Env, sounds: &mut Vec<Sound>) {
        let g = self.want(&self.people[i], env);
        let (state, goal, dest) = {
            let p = &self.people[i];
            (p.state, p.goal, p.dest)
        };
        let j = self.rng.f32();
        if g == goal {
            match (state, dest) {
                (State::Inside, Dest::Lot(l)) => {
                    // occasionally pop out onto the doorstep for some air
                    if j < 0.12 && env.rain < 0.3 && env.night < 0.6 && goal != Goal::Wedding {
                        let front = Vec2::from(self.lots[l].front);
                        let p = &mut self.people[i];
                        p.visible = true;
                        p.pos = front;
                        p.state = State::Idle;
                        p.timer = 6.0 + j * 40.0;
                        p.yaw = self.rng.range(0.0, 6.28);
                    } else {
                        self.people[i].timer = 20.0 + j * 60.0;
                    }
                }
                (State::Idle, Dest::Lot(_)) if goal == Goal::Work && self.people[i].role == Role::Postie => {
                    let d = self.dest_for(&self.people[i], g, env);
                    self.plan(i, d);
                }
                (State::Idle, Dest::Lot(l)) => {
                    let front = Vec2::from(self.lots[l].front);
                    self.set_path(i, vec![front], Dest::Lot(l));
                }
                (State::Working, Dest::Field(l)) => {
                    let fp = self.field_point(l);
                    self.set_path(i, vec![fp], Dest::Field(l));
                }
                (State::Idle, Dest::Plaza) => {
                    // mingle: often wander over to someone else hanging about
                    let me = self.people[i].pos;
                    let company = self
                        .people
                        .iter()
                        .enumerate()
                        .filter(|(k, q)| {
                            *k != i && q.visible && q.at == Place::Plaza && q.state == State::Idle && q.chat_cd <= 0.0
                        })
                        .map(|(_, q)| q.pos)
                        .min_by(|a, b| a.distance(me).partial_cmp(&b.distance(me)).unwrap());
                    let to = match company {
                        Some(q) if j < 0.6 => q + (me - q).normalize_or(Vec2::X) * 1.3,
                        _ => self.plaza_point(),
                    };
                    self.set_path(i, vec![to], Dest::Plaza);
                }
                (State::Working, _) | (State::Sitting, _) => {
                    self.people[i].timer = if self.people[i].bella { 6.0 + j * 6.0 } else { 10.0 + j * 30.0 };
                }
                (State::Dancing | State::Singing | State::Playing, _) => {
                    self.people[i].timer = 20.0 + j * 25.0;
                }
                (State::Idle, Dest::Gather) => self.plan(i, Dest::Gather),
                // on his way to the weights (or just left them): back to the mat
                (State::Idle, Dest::Gym) => self.plan(i, Dest::Gym),
                _ => {
                    self.people[i].timer = 5.0 + j * 10.0;
                }
            }
            return;
        }
        let mut d = self.dest_for(&self.people[i], g, env);
        // one person per bench: try the others, or just hang about the plaza
        if let Dest::Bench(b) = d {
            d = (0..4)
                .map(|k| (b + k) % 4)
                .find(|&k| !self.bench_taken(k, i))
                .map(Dest::Bench)
                .unwrap_or(Dest::Plaza);
        }
        if self.people[i].state == State::Inside {
            if let Place::Lot(l) = self.people[i].at {
                let front = Vec2::from(self.lots[l].front);
                let p = &mut self.people[i];
                p.pos = front;
                sounds.push(Sound { name: "door", pos: p.pos3(), vol: 0.4, pitch: 1.0 });
            }
        }
        self.people[i].goal = g;
        if g == Goal::Deliver {
            self.people[i].carry = Carry::Bread;
        }
        self.plan(i, d);
    }

    pub(super) fn arrive(&mut self, i: usize, sounds: &mut Vec<Sound>) {
        let j = self.rng.f32();
        let dest = self.people[i].dest;
        let bench = if let Dest::Bench(b) = dest { Some(self.bench(b)) } else { None };
        let door_yaw = if let Dest::Lot(l) = dest {
            let (d, f) = (self.lots[l].door, Vec2::from(self.lots[l].front));
            Some(yaw_of(Vec2::new(d.0 as f32 + 0.5, d.1 as f32 + 0.5) - f))
        } else {
            None
        };
        let anvil_yaw = if let Dest::Anvil(l) = dest {
            // he stands a block along +x (doors facing north/south) or +z from the anvil
            Some(if self.lots[l].face % 2 == 0 { -std::f32::consts::FRAC_PI_2 } else { std::f32::consts::PI })
        } else {
            None
        };
        let centre = self.center();
        let spin = self.gathering.as_ref().map_or(1.0, |g| g.spin);
        let today = self.today;
        let p = &mut self.people[i];
        match dest {
            Dest::Gym => {
                p.at = Place::Plaza;
                p.state = State::Working;
                p.yaw = std::f32::consts::FRAC_PI_2;
                // short spells, so he's quick to notice it's bedtime
                p.timer = 6.0 + j * 6.0;
            }
            Dest::Stall(_) => {
                p.at = Place::Plaza;
                let out = (p.pos - centre).normalize_or(Vec2::X);
                if p.goal == Goal::Market {
                    p.state = State::Working;
                    p.yaw = yaw_of(out);
                    p.timer = 30.0 + j * 30.0;
                } else {
                    // browse a while, and go home with a basketful
                    p.state = State::Idle;
                    p.yaw = yaw_of(-out);
                    p.timer = 6.0 + j * 8.0;
                    p.carry = Carry::Basket;
                }
            }
            // hand over the bread, or the post, at the door, and move on
            Dest::Lot(l) if p.goal == Goal::Deliver || (p.goal == Goal::Work && p.role == Role::Postie) => {
                p.state = State::Idle;
                p.at = Place::Lot(l);
                p.timer = if p.goal == Goal::Deliver { 4.0 } else { 2.5 };
                if p.goal == Goal::Deliver {
                    p.carry = Carry::Nothing;
                    p.delivered = today;
                }
                if let Some(y) = door_yaw {
                    p.yaw = y;
                }
                sounds.push(Sound { name: "door", pos: p.pos3(), vol: 0.3, pitch: 1.0 });
            }
            Dest::Gather => {
                p.at = Place::Plaza;
                p.timer = 25.0 + j * 25.0;
                let out = p.pos - centre;
                p.dance_a = out.y.atan2(out.x);
                p.dance_r = out.length();
                p.state = match p.gather_role {
                    GatherRole::Musician => {
                        p.yaw = yaw_of(out);
                        State::Playing
                    }
                    GatherRole::Singer => {
                        p.yaw = yaw_of(-out);
                        State::Singing
                    }
                    _ => {
                        p.yaw = yaw_of(Vec2::new(-out.y, out.x) * spin);
                        State::Dancing
                    }
                };
            }
            Dest::Lot(l) => {
                p.state = State::Inside;
                p.visible = false;
                p.at = Place::Lot(l);
                p.timer = 30.0 + j * 90.0;
                // called to the gathering: only ducking in, not settling
                if p.coming {
                    p.timer = p.timer.min(2.0 + j * 10.0);
                }
                if l == p.home {
                    p.carry = Carry::Nothing;
                }
                if let Some(y) = door_yaw {
                    p.yaw = y;
                }
                sounds.push(Sound { name: "door", pos: p.pos3(), vol: 0.4, pitch: 1.0 });
            }
            Dest::Field(l) => {
                p.state = State::Working;
                p.at = Place::Lot(l);
                p.timer = 4.0 + j * 6.0;
            }
            Dest::Anvil(l) => {
                p.state = State::Working;
                p.at = Place::Lot(l);
                p.timer = 20.0 + j * 30.0;
                p.yaw = anvil_yaw.unwrap_or(0.0);
            }
            Dest::Plaza => {
                p.state = State::Idle;
                p.at = Place::Plaza;
                p.timer = 3.0 + j * 9.0;
            }
            Dest::Bench(_) => {
                p.state = State::Sitting;
                p.at = Place::Plaza;
                p.timer = 30.0 + j * 60.0;
                if let Some((seat, _)) = bench {
                    p.pos = seat;
                }
                p.yaw = std::f32::consts::PI;
            }
        }
    }

    /// `carts` are (position, heading) of carts people step aside from.
    pub fn update(&mut self, env: &Env, carts: &[(Vec2, Vec2)], sounds: &mut Vec<Sound>) {
        let dt = env.dt;
        self.strikes.clear();
        if env.day != self.today {
            self.next_day(env);
        }
        self.update_wedding(env, sounds);
        self.update_gathering(env);
        self.update_happenings(env, sounds);
        self.building_work(env, sounds);
        // the town crier calls the market as it opens
        if self.market_on(env) && env.hour < 9.5 {
            self.crier_t -= dt;
            if self.crier_t <= 0.0 {
                self.crier_t = 40.0;
                let c = self.center();
                sounds.push(Sound { name: "crier", pos: Vec3::new(c.x, self.y, c.y), vol: 0.9, pitch: 0.85 });
            }
        }
        let hour = env.hour as i32;
        if hour != self.last_hour {
            if let Some(c) = self.church {
                if [8, 12, 18].contains(&hour) {
                    let (x, z) = self.lots[c].center();
                    sounds.push(Sound { name: "bell", pos: Vec3::new(x, self.y + 12.0, z), vol: 1.0, pitch: 1.0 });
                }
            }
            self.last_hour = hour;
        }

        let mut crowd: Vec<Option<Vec2>> = self
            .people
            .iter()
            .map(|p| (p.visible && p.state != State::Inside).then_some(p.pos))
            .collect();
        // walkers steer round carts and their horses just as they do round
        // each other, rather than walking into them and being shoved back
        crowd.extend(carts.iter().map(|(p, _)| Some(*p)));
        for i in 0..self.people.len() {
            self.people[i].chat_cd -= dt;
            match self.people[i].state {
                State::Walking => {
                    if self.step(i, dt, &crowd) {
                        self.arrive(i, sounds);
                    }
                }
                State::Chatting(o) => {
                    let (op, t0) = (self.people[o].pos, self.people[i].chat_start);
                    let p = &mut self.people[i];
                    let d = op - p.pos;
                    if d.length() > 0.01 {
                        p.yaw = lerp_angle(p.yaw, yaw_of(d), approach(6.0, dt));
                    }
                    // shuffle back to a comfortable distance to talk
                    let len = d.length();
                    if len < CHAT_DISTANCE {
                        let dir = if len > 1e-3 { d / len } else { Vec2::new(if i < o { 1.0 } else { -1.0 }, 0.0) };
                        let back = ((CHAT_DISTANCE - len) * 0.5 * approach(3.0, dt)).min(0.8 * dt);
                        let to = p.pos - dir * back;
                        if !self.inside_building(to) {
                            self.people[i].pos = to;
                        }
                    }
                    let p = &mut self.people[i];
                    let slot = ((env.time - t0) / 1.7) as i32;
                    let mine = (slot % 2 == 0) == p.first_speaker;
                    if slot != p.talk_slot {
                        p.talk_slot = slot;
                        if mine {
                            // babble in the mood of the speech bubble
                            let name = match p.emote {
                                Emote::Talk => "chat",
                                Emote::Heart => "chat_heart",
                                Emote::Laugh => "chat_laugh",
                                Emote::Note => "chat_note",
                            };
                            // the trade rides along in the hundreds, for the voice's character
                            sounds.push(Sound { name, pos: p.pos3(), vol: 0.9, pitch: p.voice() + 100.0 * p.role_code() });
                        }
                    }
                    p.timer -= dt;
                    if p.timer <= 0.0 {
                        p.state = p.resume;
                        p.timer = 1.0;
                        p.chat_cd = 15.0 + unit(hash_u32(p.seed ^ slot as u32)) * 20.0;
                    }
                }
                _ => {
                    let p = &mut self.people[i];
                    p.timer -= dt;
                    if p.state == State::Working {
                        p.work_t += dt;
                        if p.role == Role::Smith {
                            let before = ((p.work_t - dt) / 1.1) as i32;
                            if (p.work_t / 1.1) as i32 != before {
                                sounds.push(Sound { name: "clang", pos: p.pos3(), vol: 0.7, pitch: 1.0 });
                                if let Place::Lot(l) = p.at {
                                    let a = self.lots[l].front;
                                    self.strikes.push(Vec3::new(a.0, p.y + 0.62, a.1));
                                }
                            }
                        }
                    }
                    if p.timer <= 0.0 {
                        self.think(i, env, sounds);
                    }
                }
            }
            let p = &mut self.people[i];
            let target = if p.visible { 1.0 } else { 0.0 };
            p.fade += (target - p.fade).clamp(-dt * FADE_RATE, dt * FADE_RATE);
            if p.visible {
                let c = Vec2::new(self.town.cx as f32, self.town.cz as f32);
                p.y = if p.pos.distance(c) < self.town.radius as f32 - 1.0 {
                    self.y
                } else {
                    env.gen.walk_y(&self.ctx, p.pos.x, p.pos.y)
                };
            }
        }

        self.play_games(env, sounds);
        self.keep_apart(carts, env.dt);
        self.socialise(env);
        self.update_dogs(env, sounds);
        self.update_cats(env, sounds);
        self.update_chickens(env, sounds);
    }

}
