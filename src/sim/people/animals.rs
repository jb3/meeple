use super::*;

impl TownLife {
    pub(super) fn update_dogs(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        let dt = env.dt;
        for d in &mut self.dogs {
            let o = &self.people[d.owner];
            let fwd = Vec2::new(o.yaw.sin(), o.yaw.cos());
            let target = if o.visible {
                o.pos - fwd * 1.0 + Vec2::new(fwd.y, -fwd.x) * 0.6
            } else {
                if d.pos.distance(d.target) < 0.2 {
                    let h = hash_u32((env.time * 0.2) as u32 ^ d.owner as u32);
                    if h % 60 == 0 {
                        d.target = d.home + Vec2::new(unit(h) - 0.5, unit(h >> 8) - 0.5) * 3.0;
                    }
                }
                d.target
            };
            let diff = target - d.pos;
            let dist = diff.length();
            d.moving = dist > 0.3;
            if d.moving {
                let sp = (dist * 1.5).clamp(0.6, 3.0);
                d.pos += diff / dist * (sp * dt).min(dist);
                d.yaw = lerp_angle(d.yaw, yaw_of(diff), approach(8.0, dt));
                d.phase += dt * sp * 6.0;
            }
            d.bark_t -= dt;
            if d.bark_t <= 0.0 {
                d.bark_t = 12.0 + unit(hash_u32(d.owner as u32 ^ env.slot)) * 40.0;
                sounds.push(Sound { name: "bark", pos: Vec3::new(d.pos.x, self.y, d.pos.y), vol: 0.6, pitch: 1.0 });
            }
        }
    }

    pub(super) fn update_cats(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        let dt = env.dt;
        let sheltering = env.rain > 0.3;
        let night = env.night > 0.5;
        for (n, c) in self.cats.iter_mut().enumerate() {
            c.timer -= dt;
            match c.mode {
                CatMode::Walk => {
                    let diff = c.target - c.pos;
                    let dist = diff.length();
                    if dist < 0.1 {
                        // arrived: sit a while, or (by day, in the dry) curl up and sleep
                        let h = hash_u32((env.time * 10.0) as u32 ^ n as u32 ^ 0xca7);
                        c.mode = if !night && !sheltering && h % 100 < 55 { CatMode::Sleep } else { CatMode::Sit };
                        c.timer = if c.mode == CatMode::Sleep { 25.0 + unit(h >> 8) * 50.0 } else { 4.0 + unit(h >> 8) * 12.0 };
                    } else {
                        let sp = if sheltering { 1.6 } else { 0.9 };
                        c.pos += diff / dist * (sp * dt).min(dist);
                        c.yaw = lerp_angle(c.yaw, yaw_of(diff), approach(8.0, dt));
                        c.phase += dt * sp * 8.0;
                    }
                }
                CatMode::Sit | CatMode::Sleep => {
                    if c.timer <= 0.0 || (sheltering && c.pos.distance(c.home) > 1.5) {
                        let h = hash_u32((env.time * 10.0) as u32 ^ n as u32 ^ 0xca8);
                        // rain sends a cat home; otherwise a short hop along the street
                        let mut goal = if sheltering { Some(c.home) } else { None };
                        if goal.is_none() {
                            for k in 0..8u32 {
                                let hk = hash_u32(h ^ k.wrapping_mul(0x9e37_79b1));
                                let a = unit(hk) * 6.28;
                                let q = c.pos + Vec2::new(a.cos(), a.sin()) * (1.5 + unit(hk >> 8) * 2.5);
                                if q.distance(c.home) > if night { 12.0 } else { 8.0 } {
                                    continue;
                                }
                                let col = env.gen.column(&self.ctx, q.x.floor() as i32, q.y.floor() as i32);
                                if col.street || col.plaza {
                                    goal = Some(q);
                                    break;
                                }
                            }
                        }
                        match goal {
                            Some(g) => {
                                c.target = g;
                                c.mode = CatMode::Walk;
                            }
                            None => c.timer = 5.0,
                        }
                    }
                }
            }
            c.meow_t -= dt;
            if c.meow_t <= 0.0 {
                c.meow_t = 25.0 + unit(hash_u32(n as u32 ^ env.slot)) * 60.0;
                if c.mode != CatMode::Sleep {
                    sounds.push(Sound { name: "meow", pos: Vec3::new(c.pos.x, self.y, c.pos.y), vol: 0.45, pitch: 1.0 });
                }
            }
        }
    }

    pub(super) fn update_chickens(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        let dt = env.dt;
        for c in &mut self.chickens {
            c.timer -= dt;
            c.peck = (c.peck - dt * 3.0).max(0.0);
            if c.timer <= 0.0 {
                let l = &self.lots[c.lot];
                let h = hash_u32((env.time * 10.0) as u32 ^ c.lot as u32 ^ (c.pos.x * 7.0) as u32);
                if h % 3 == 0 {
                    c.peck = 1.0;
                } else {
                    c.target = Vec2::new(
                        l.x0 as f32 + 1.5 + unit(h) * (l.x1 - l.x0 - 2) as f32,
                        l.z0 as f32 + 1.5 + unit(h >> 8) * (l.z1 - l.z0 - 2) as f32,
                    );
                }
                c.timer = 1.0 + unit(h >> 16) * 3.0;
            }
            let d = c.target - c.pos;
            if d.length() > 0.05 {
                c.pos += d.normalize() * (0.7 * dt).min(d.length());
                c.yaw = lerp_angle(c.yaw, yaw_of(d), approach(8.0, dt));
            }
            c.cluck_t -= dt;
            if c.cluck_t <= 0.0 {
                c.cluck_t = 25.0 + unit(hash_u32(c.lot as u32 ^ (env.time as u32))) * 40.0;
                sounds.push(Sound { name: "cluck", pos: Vec3::new(c.pos.x, self.y, c.pos.y), vol: 0.5, pitch: 1.0 });
            }
        }
    }
}
