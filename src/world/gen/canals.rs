use super::*;

impl Gen {
    pub(super) fn canal_z(&self, k: i32) -> i32 {
        k * CANAL_SPACING + 300 + (hash2(self.seed ^ 0xca4a, k, 0) % 100) as i32 - 50
    }

    pub fn canals_near(&self, z0: i32, z1: i32) -> Vec<i32> {
        (div_floor(z0 - 400, CANAL_SPACING)..=div_floor(z1 + 100, CANAL_SPACING))
            .map(|k| self.canal_z(k))
            .filter(|&c| c >= z0 - 8 && c <= z1 + 8)
            .collect()
    }

    /// Is the canal at z = `cz` dug at this x? Canals cut through low land
    /// from one stretch of open water to another: never across the water
    /// itself, and never ending blind in a field.
    pub fn canal_open(&self, x: f32, cz: i32) -> bool {
        let i = (x.floor() as i32).div_euclid(CANAL_STEP);
        let n = CANAL_TILE / CANAL_STEP;
        let tile = i.div_euclid(n);
        let dug = self.canal_dug_tile(cz, tile);
        dug[i.rem_euclid(n) as usize]
    }

    /// The land along a canal at sample `i`: 0 open water, 1 low land, 2 high.
    pub(super) fn canal_land(&self, cz: i32, i: i32) -> u8 {
        let n = CANAL_TILE / CANAL_STEP;
        let tile = i.div_euclid(n);
        let cached = self.canal_land.borrow().get(&(cz, tile)).cloned();
        let land = cached.unwrap_or_else(|| {
            let v: Rc<Vec<u8>> = Rc::new(
                (0..n)
                    .map(|k| {
                        let x = ((tile * n + k) * CANAL_STEP) as f32 + 0.5;
                        let h = self.raw_height(x, cz as f32 + 0.5);
                        if h < SEA as f32 - 0.5 {
                            0
                        } else if h < SEA as f32 + CANAL_MAX_RISE {
                            1
                        } else {
                            2
                        }
                    })
                    .collect(),
            );
            self.canal_land.borrow_mut().insert((cz, tile), v.clone());
            v
        });
        land[i.rem_euclid(n) as usize]
    }

    pub(super) fn canal_dug_tile(&self, cz: i32, tile: i32) -> Rc<Vec<bool>> {
        if let Some(v) = self.canal_dug.borrow().get(&(cz, tile)) {
            return v.clone();
        }
        let n = CANAL_TILE / CANAL_STEP;
        let reach = CANAL_MAX_RUN / CANAL_STEP;
        let mut dug = vec![false; n as usize];
        let mut k = 0;
        while k < n {
            let i = tile * n + k;
            if self.canal_land(cz, i) != 1 {
                k += 1;
                continue;
            }
            // find both ends of this stretch of low land
            let (mut a, mut b) = (i, i);
            while a > i - reach && self.canal_land(cz, a - 1) == 1 {
                a -= 1;
            }
            while b < i + reach && self.canal_land(cz, b + 1) == 1 {
                b += 1;
            }
            let joins = self.canal_land(cz, a - 1) == 0 && self.canal_land(cz, b + 1) == 0;
            let len = (b - a + 1) * CANAL_STEP;
            let ok = joins && (CANAL_MIN_RUN..=CANAL_MAX_RUN).contains(&len);
            let end = (b - tile * n).min(n - 1);
            for j in k..=end {
                dug[j as usize] = ok;
            }
            k = end + 1;
        }
        let v = Rc::new(dug);
        self.canal_dug.borrow_mut().insert((cz, tile), v.clone());
        v
    }

    /// Look outward from a town for the nearest water: the water point, the
    /// last dry land before it, the direction out to it, and whether it's a
    /// big lake or sea (rather than a river).
    pub fn water_near(&self, t: &Town) -> Option<(Vec2, Vec2, Vec2, bool)> {
        let c = Vec2::new(t.cx as f32 + 0.5, t.cz as f32 + 0.5);
        for d in [8.0, 16.0, 24.0, 32.0] {
            for k in 0..24 {
                let a = (k as f32 + unit(t.id) * 3.0) * std::f32::consts::TAU / 24.0;
                let dir = Vec2::new(a.cos(), a.sin());
                let w = c + dir * (t.radius as f32 + d);
                if self.raw_height(w.x, w.y) > SEA as f32 - 0.5 {
                    continue;
                }
                // step back towards town to the shore
                let mut land = w;
                for _ in 0..14 {
                    land -= dir;
                    if self.raw_height(land.x, land.y) > SEA as f32 + 1.2 {
                        let wet = (0..12)
                            .filter(|i| {
                                let b = *i as f32 * std::f32::consts::TAU / 12.0;
                                let q = w + Vec2::new(b.cos(), b.sin()) * 14.0;
                                self.raw_height(q.x, q.y) < SEA as f32
                            })
                            .count();
                        return Some((w, land, dir, wet >= 8));
                    }
                }
            }
        }
        None
    }
}
