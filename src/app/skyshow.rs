use super::*;

/// Rainbows, northern lights and shooting stars.
#[derive(Default)]
pub(super) struct SkyShow {
    pub(super) rainbow: f32,
    pub(super) aurora: f32,
    /// Northern lights held at this strength regardless (`?aurora`).
    pub(super) force_aurora: Option<f32>,
    pub(super) meteor: [f32; 4],
    pub(super) meteor_t: f32,
    pub(super) next_meteor: f32,
    pub(super) rng_state: u32,
}

/// The light's view of a square `half` units to each side of `center`; `res`
/// is the shadow map's size, so the view moves in whole texels. (`half` is
/// quantised by the caller so zooming doesn't change the texel size every frame.)
pub(super) fn light_matrix(dir: Vec3, center: Vec3, half: f32, res: f32) -> Mat4 {
    let up = if dir.y.abs() > 0.98 { Vec3::Z } else { Vec3::Y };
    let view = Mat4::look_at_rh(dir * 200.0, Vec3::ZERO, up);
    let c = view.transform_point3(center);
    let texel = 2.0 * half / res;
    let cx = (c.x / texel).floor() * texel;
    let cy = (c.y / texel).floor() * texel;
    let proj = Mat4::orthographic_rh_gl(cx - half, cx + half, cy - half, cy + half, -c.z - 250.0, -c.z + 250.0);
    proj * view
}

impl SkyShow {
    pub(super) fn rand(&mut self) -> f32 {
        self.rng_state = hash_u32(self.rng_state.wrapping_add(0x9e37_79b9));
        unit(self.rng_state)
    }

    pub(super) fn update(&mut self, dt: f32, w: &Weather, night: f32, cold: f32) {
        // a rainbow when the sun breaks through as the rain clears
        let bow = w.precip < 0.15 && w.wet > 0.3 && !w.snowing && night < 0.25;
        self.rainbow += (if bow { 1.0 } else { 0.0 } - self.rainbow) * approach(0.15, dt);
        // northern lights on clear, cold nights
        let lights = night > 0.7 && w.cloud < 0.6 && cold > 0.12;
        let strength = match self.force_aurora {
            Some(f) => f,
            None if lights => (0.45 + cold * 1.2).min(1.0),
            None => 0.0,
        };
        // (held lights are there at once; weather-made ones fade in over a few seconds)
        self.aurora = if self.force_aurora.is_some() { strength } else { self.aurora + (strength - self.aurora) * approach(0.1, dt) };
        // the odd shooting star on a clear night
        if self.meteor_t > 0.0 {
            self.meteor_t += dt / 0.9;
            if self.meteor_t >= 1.0 {
                self.meteor_t = 0.0;
            }
        } else if night > 0.7 && w.cloud < 0.4 {
            self.next_meteor -= dt;
            if self.next_meteor <= 0.0 {
                self.next_meteor = 15.0 + self.rand() * 45.0;
                let x = 0.15 + self.rand() * 0.7;
                let y = 0.75 + self.rand() * 0.2;
                let dx = (self.rand() - 0.5) * 0.7;
                self.meteor = [x, y, dx, -0.25 - self.rand() * 0.15];
                self.meteor_t = 0.001;
            }
        }
    }
}

/// One northern-lights curtain: where its middle is, which way it runs, how long, and its look.
pub(super) struct Curtain {
    pub centre: Vec2,
    pub dir: Vec2,
    pub len: f32,
    pub phase: f32,
    pub lift: f32,
}

/// How big the squares are that curtains are hung over.
const AURORA_CELL: i32 = 300;

/// The curtains in one square of the world's lattice (the same every time, for a
/// given world): most squares have one, some two, a few none.
pub(super) fn curtains_in(seed: u32, ci: i32, cj: i32) -> Vec<Curtain> {
    let h = hash3(0xa04a ^ seed, ci, 0, cj);
    let n = if unit(h) < 0.2 { 0 } else if unit(h >> 5) < 0.3 { 2 } else { 1 };
    (0..n)
        .map(|k| {
            let hk = hash_u32(h ^ (k as u32 * 0x9e37_79b9));
            let ang = unit(hk >> 16) * std::f32::consts::PI;
            Curtain {
                centre: Vec2::new((ci * AURORA_CELL) as f32 + unit(hk) * AURORA_CELL as f32, (cj * AURORA_CELL) as f32 + unit(hk >> 8) * AURORA_CELL as f32),
                dir: Vec2::new(ang.cos(), ang.sin()),
                len: 160.0 + unit(hk >> 3) * 140.0,
                phase: unit(hk >> 11) * 20.0,
                lift: 14.0 + unit(hk >> 21) * 14.0,
            }
        })
        .collect()
}

/// The curtains near the view, flattened for the aurora shader (see `AURORA_VS`):
/// eight floats each, up to eight of them, the nearest first. Each hangs over
/// fixed ground, so it stays put as the camera moves. Its base is above the
/// highest ground along it, so it never runs into a hillside.
pub(super) fn curtains_near(gen: &crate::world::gen::Gen, view: Vec2, view_r: f32) -> Vec<f32> {
    let reach = view_r + 90.0;
    let cell = AURORA_CELL as f32;
    let (c0x, c0z) = (((view.x - reach) / cell).floor() as i32 - 1, ((view.y - reach) / cell).floor() as i32 - 1);
    let (c1x, c1z) = (((view.x + reach) / cell).floor() as i32 + 1, ((view.y + reach) / cell).floor() as i32 + 1);
    let mut found: Vec<(f32, [f32; 8])> = Vec::new();
    for ci in c0x..=c1x {
        for cj in c0z..=c1z {
            for c in curtains_in(gen.seed, ci, cj) {
                let near = c.centre.distance(view);
                if near > c.len * 0.5 + 40.0 + reach {
                    continue;
                }
                let top = (-2..=2)
                    .map(|k| {
                        let p = c.centre + c.dir * (k as f32 * c.len * 0.25);
                        gen.raw_height(p.x, p.y)
                    })
                    .fold(crate::world::gen::SEA as f32, f32::max);
                found.push((near, [c.centre.x, c.centre.y, c.dir.x, c.dir.y, c.len, c.phase, top + 14.0 + c.lift, 0.0]));
            }
        }
    }
    found.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    found.into_iter().take(8).flat_map(|(_, v)| v).collect()
}

#[cfg(test)]
mod aurora_tests {
    use super::*;

    /// Curtains are fixed to the world: the same ones every time for a world,
    /// different ones in different worlds, and plenty of them.
    #[test]
    fn curtains_belong_to_the_world() {
        let a: Vec<_> = (-3..3).flat_map(|i| (-3..3).flat_map(move |j| curtains_in(1234, i, j))).map(|c| (c.centre.x, c.centre.y)).collect();
        let b: Vec<_> = (-3..3).flat_map(|i| (-3..3).flat_map(move |j| curtains_in(1234, i, j))).map(|c| (c.centre.x, c.centre.y)).collect();
        let other: Vec<_> = (-3..3).flat_map(|i| (-3..3).flat_map(move |j| curtains_in(99, i, j))).map(|c| (c.centre.x, c.centre.y)).collect();
        assert_eq!(a, b);
        assert_ne!(a, other);
        assert!(a.len() >= 20, "only {} curtains in 36 squares", a.len());
        println!("first curtains of 1234: {:?}", &a[..4]);
    }
}
