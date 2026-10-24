use super::*;

impl Fauna {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_herds(&mut self, world: &World, dt: f32, view: Vec2, view_r: f32, night: f32, season: f32, on_screen: &dyn Fn(Vec3) -> bool, warmup: bool) {
        let gen = &world.gen;
        let dir = match (season % 4.0) as usize {
            0 => Some(Vec2::new(0.0, -1.0)),
            2 => Some(Vec2::new(0.0, 1.0)),
            _ => None,
        };
        self.herd_t -= dt;
        if let (Some(d), true) = (dir, self.herds.is_empty() && self.herd_t <= 0.0 && night < 0.6) {
            self.herd_t = self.rng.range(40.0, 90.0);
            let side = Vec2::new(-d.y, d.x) * self.rng.range(-view_r * 0.4, view_r * 0.4);
            let start = view - d * (view_r + 30.0) + side;
            let at = start + d * 4.0;
            let ctx = gen.ctx(start.x as i32 - 40, start.y as i32 - 40, start.x as i32 + 40, start.y as i32 + 40);
            let dry = !Self::is_water(gen, &ctx, start);
            let seen = !warmup && on_screen(Vec3::new(start.x, gen.walk_y(&ctx, start.x, start.y), start.y));
            if dry && !seen {
                let n = self.rng.int(7, 13);
                let members = (0..n)
                    .map(|i| {
                        let along = -(i as f32) * 2.2 + self.rng.range(-0.8, 0.8);
                        let across = self.rng.range(-3.5, 3.5);
                        let p = at + d * along + Vec2::new(-d.y, d.x) * across;
                        Animal { pos: p, y: gen.walk_y(&ctx, p.x, p.y), yaw: yaw_of(d), target: p + d * 3.0, timer: 0.0, graze: 0.0, phase: self.rng.range(0.0, 6.0), tint: rgb(150, 100, 60), flee: 0.0 }
                    })
                    .collect();
                self.herds.push(Herd { members, dir: d, travelled: 0.0, ctx, ctx_at: start });
            }
        }
        for h in &mut self.herds {
            let mid = h.members.iter().fold(Vec2::ZERO, |s, a| s + a.pos) / h.members.len().max(1) as f32;
            if mid.distance(h.ctx_at) > 22.0 {
                h.ctx = gen.ctx(mid.x as i32 - 40, mid.y as i32 - 40, mid.x as i32 + 40, mid.y as i32 + 40);
                h.ctx_at = mid;
            }
            // water ahead: bear to one side until clear
            let ahead = mid + h.dir * 6.0;
            if Self::is_water(gen, &h.ctx, ahead) {
                let side = Vec2::new(-h.dir.y, h.dir.x);
                let turn = if Self::is_water(gen, &h.ctx, mid + side * 6.0) { -side } else { side };
                h.dir = (h.dir + turn * 0.6 * dt * 4.0).normalize_or(h.dir);
            } else {
                let back = Vec2::new(0.0, h.dir.y.signum());
                h.dir = (h.dir + (back - h.dir) * dt * 0.5).normalize_or(h.dir);
            }
            let step = h.dir * 1.5 * dt;
            for a in &mut h.members {
                a.phase += dt;
                let next = a.pos + step;
                if !Self::is_water(gen, &h.ctx, next) {
                    a.pos = next;
                }
                a.target = a.pos + h.dir * 3.0;
                a.y += (gen.walk_y(&h.ctx, a.pos.x, a.pos.y) - a.y) * approach(10.0, dt);
                let want = yaw_of(h.dir);
                let mut turn = (want - a.yaw).rem_euclid(std::f32::consts::TAU);
                if turn > std::f32::consts::PI {
                    turn -= std::f32::consts::TAU;
                }
                a.yaw += turn * approach(3.0, dt);
            }
            h.travelled += step.length();
        }
        self.herds.retain(|h| h.travelled < view_r * 2.0 + 140.0 && dir.is_some());
    }
}
