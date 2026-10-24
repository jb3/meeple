use super::*;

impl Fauna {
    /// Narrowboats working the canals, entering from out of view.
    pub(super) fn update_narrowboats(&mut self, gen: &Gen, dt: f32, view: Vec2, view_r: f32, warmup: bool, on_screen: &dyn Fn(Vec3) -> bool) {
        // narrowboats working the canals, entering from out of view
        self.boat_t -= dt;
        if self.boat_t <= 0.0 {
            // try again soon if there's a canal about but nowhere to start yet
            self.boat_t = 5.0;
            let r = view_r as i32 + 40;
            for cz in gen.canals_near(view.y as i32 - r, view.y as i32 + r) {
                if self.narrowboats.iter().any(|b| b.canal == cz) {
                    continue;
                }
                // somewhere along the canal just out of sight (canals are
                // short, so look both ways), or anywhere as the world loads
                let mut spots: Vec<(f32, f32)> = (0..24)
                    .flat_map(|k| {
                        let d = view_r * 0.3 + k as f32 * 8.0;
                        [(view.x - d, 1.0), (view.x + d, -1.0)]
                    })
                    .filter(|&(x, _)| gen.canal_open(x, cz) && gen.canal_open(x + 6.0, cz) && gen.canal_open(x - 6.0, cz))
                    .collect();
                spots.retain(|&(x, _)| warmup || !on_screen(Vec3::new(x, SEA as f32 + 1.0, cz as f32 + 0.5)));
                if let Some(&(x, dir)) = spots.first() {
                    self.boat_t = self.rng.range(20.0, 50.0);
                    let paints = [rgb(40, 90, 60), rgb(150, 40, 40), rgb(40, 60, 120), rgb(110, 40, 90)];
                    self.narrowboats.push(Narrowboat {
                        canal: cz,
                        x,
                        dir,
                        wait: 0.0,
                        hull: *self.rng.pick(&paints),
                        trim: *self.rng.pick(&[rgb(220, 180, 60), rgb(230, 230, 220), rgb(200, 90, 50)]),
                    });
                }
            }
        }
        for b in &mut self.narrowboats {
            if b.wait > 0.0 {
                b.wait -= dt;
                continue;
            }
            // the canal ends where the land rises: turn round and head back
            if !gen.canal_open(b.x + b.dir * 5.0, b.canal) {
                b.dir = -b.dir;
                b.wait = 6.0;
                continue;
            }
            b.x += b.dir * 1.6 * dt;
        }
        self.narrowboats.retain(|b| (b.x - view.x).abs() < view_r + 300.0);
    }
}
