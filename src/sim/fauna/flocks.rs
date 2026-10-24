use super::*;

impl Fauna {
    /// Flocks of birds crossing the sky, and the great migrations.
    pub(super) fn update_flocks(&mut self, dt: f32, view: Vec2, view_r: f32, night: f32, season: f32) {
        // birds cross the sky by day; in autumn great skeins head south for
        // the winter, and in spring they come back north
        let migrating = match (season % 4.0) as usize {
            0 => Some(Vec2::new(0.0, -1.0)),
            2 => Some(Vec2::new(0.0, 1.0)),
            _ => None,
        };
        self.flock_t -= dt;
        if self.flock_t <= 0.0 && night < 0.5 && self.flocks.len() < if migrating.is_some() { 3 } else { 2 } {
            self.flock_t = if migrating.is_some() { self.rng.range(12.0, 32.0) } else { self.rng.range(20.0, 50.0) };
            let dir = match migrating {
                Some(d) => {
                    let a = self.rng.range(-0.3, 0.3);
                    Vec2::new(d.x * a.cos() - d.y * a.sin(), d.x * a.sin() + d.y * a.cos())
                }
                None => {
                    let a = self.rng.range(0.0, 6.28);
                    Vec2::new(a.cos(), a.sin())
                }
            };
            let side = Vec2::new(-dir.y, dir.x) * self.rng.range(-view_r * 0.5, view_r * 0.5);
            let start = view - dir * (view_r + 50.0) + side;
            let n = if migrating.is_some() { self.rng.int(10, 22) } else { self.rng.int(4, 10) };
            let birds = (0..n)
                .map(|i| {
                    let row = ((i + 1) / 2) as f32;
                    let s = if i % 2 == 0 { 1.0 } else { -1.0 };
                    Bird { off: Vec2::new(s * row * 1.4, -row * 1.6), phase: self.rng.range(0.0, 6.0) }
                })
                .collect();
            self.flocks.push(Flock {
                pos: Vec3::new(start.x, self.rng.range(42.0, 52.0), start.y),
                dir,
                travelled: 0.0,
                birds,
                dark: self.rng.chance(0.4),
            });
        }
        for f in &mut self.flocks {
            let v = f.dir * 7.0 * dt;
            f.pos += Vec3::new(v.x, 0.0, v.y);
            f.travelled += v.length();
            for b in &mut f.birds {
                b.phase += dt * 9.0;
            }
        }
        self.flocks.retain(|f| f.travelled < view_r * 2.0 + 140.0);
    }
}
