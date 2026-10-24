use super::*;

/// A low weir across a river: a stone sill from bank to bank with the
/// water spilling over it.
#[derive(Clone, Copy, Debug)]
pub struct Weir {
    /// The middle of the sill, in the river's course.
    pub c: Vec2,
    /// Unit vector across the river.
    pub cross: Vec2,
    /// Unit vector downstream.
    pub flow: Vec2,
    /// How far the water reaches from `c` either way across the river
    /// (along `-cross` and `+cross`) before the banks.
    pub a: f32,
    pub b: f32,
}

/// Weirs keep at least this far from the edges of their region, so
/// neighbouring regions' weirs can't meet.
const WEIR_MARGIN: f32 = 20.0;

impl Gen {
    /// The weirs on the rivers of a region.
    pub fn weirs(&self, rx: i32, rz: i32) -> Rc<Vec<Weir>> {
        if let Some(v) = self.weirs.borrow().get(&(rx, rz)) {
            return v.clone();
        }
        const STEP: i32 = 6;
        let n = REGION / STEP;
        let h = hash2(self.seed ^ 0x3e17, rx, rz);
        let mills: Vec<Vec2> = self.sites(rx, rz).iter().filter(|s| s.kind == SiteKind::Watermill).map(|s| s.centre()).collect();
        let mut out: Vec<Weir> = Vec::new();
        let want = 1 + (unit(hash_u32(h ^ 5)) < 0.4) as usize;
        if unit(hash_u32(h ^ 7)) < 0.85 {
            for k in 0..n * n {
                let k = (k + (h % (n * n) as u32) as i32) % (n * n);
                let p = Vec2::new((rx * REGION + (k % n) * STEP) as f32 + 0.5, (rz * REGION + (k / n) * STEP) as f32 + 0.5);
                if out.iter().any(|w| w.c.distance(p) < 60.0) || mills.iter().any(|m| m.distance(p) < 24.0) {
                    continue;
                }
                if let Some(w) = self.weir_at(p, rx, rz, hash_u32(h ^ k as u32)) {
                    out.push(w);
                    if out.len() >= want {
                        break;
                    }
                }
            }
        }
        let out = Rc::new(out);
        self.weirs.borrow_mut().insert((rx, rz), out.clone());
        out
    }

    fn weir_at(&self, p: Vec2, rx: i32, rz: i32, h: u32) -> Option<Weir> {
        let (lx, lz) = (p.x - (rx * REGION) as f32, p.y - (rz * REGION) as f32);
        if ![lx, lz].iter().all(|v| (WEIR_MARGIN..REGION as f32 - WEIR_MARGIN).contains(v)) {
            return None;
        }
        // in the deep middle of a river's course
        if self.river(p.x, p.y) > RIVER_WIDTH * 0.35 || self.raw_height(p.x, p.y) > SEA as f32 - 2.0 {
            return None;
        }
        let grad = Vec2::new(self.river(p.x + 1.0, p.y) - self.river(p.x - 1.0, p.y), self.river(p.x, p.y + 1.0) - self.river(p.x, p.y - 1.0));
        if grad.length() < 1e-6 {
            return None;
        }
        let cross = grad.normalize();
        let wet = |q: Vec2| self.raw_height(q.x, q.y) < SEA as f32 - 0.5;
        let reach = |dir: Vec2| (1..=64).map(|i| i as f32 * 0.5).find(|&t| !wet(p + dir * t));
        let (a, b) = (reach(-cross)?, reach(cross)?);
        // a river rather than a lake, and straight enough to span
        if !(6.0..=34.0).contains(&(a + b)) {
            return None;
        }
        let flow = Vec2::new(-cross.y, cross.x) * if h % 2 == 0 { 1.0 } else { -1.0 };
        if [-12.0, -6.0, 6.0, 12.0].iter().any(|s| !wet(p + flow * *s)) {
            return None;
        }
        // clear of towns, roads and railways (and the bridges on them)
        let (x, z) = (p.x as i32, p.y as i32);
        let ri = 28;
        let clear = self.towns_near(x - ri, z - ri, x + ri, z + ri, 8).is_empty()
            && self.roads_near(x - ri, z - ri, x + ri, z + ri).iter().all(|s| s.dist(x, z) > 14)
            && self.rails_near(x - ri, z - ri, x + ri, z + ri).iter().all(|l| l.perp(p.x, p.y).abs() > 18.0)
            && self.canals_near(z - ri, z + ri).iter().all(|&c| (c - z).abs() > 16);
        clear.then_some(Weir { c: p, cross, flow, a, b })
    }

    /// Weirs near a box of the world.
    pub fn weirs_near(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<Weir> {
        let reach = 40;
        let mut out = Vec::new();
        for rx in div_floor(x0 - reach, REGION)..=div_floor(x1 + reach, REGION) {
            for rz in div_floor(z0 - reach, REGION)..=div_floor(z1 + reach, REGION) {
                for w in self.weirs(rx, rz).iter() {
                    let r = w.a.max(w.b) + 4.0;
                    if w.c.x + r >= x0 as f32 && w.c.x - r <= x1 as f32 && w.c.y + r >= z0 as f32 && w.c.y - r <= z1 as f32 {
                        out.push(*w);
                    }
                }
            }
        }
        out
    }
}
