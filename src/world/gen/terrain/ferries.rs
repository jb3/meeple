//! Ferries over water too wide to bridge, bridge decks and where a walker's feet are.

use super::super::*;

impl Gen {
    /// The ferry crossings on the road between two towns: each is where the
    /// road, followed the whole way, enters water and leaves it again, as the
    /// first and last points in the water. Only short, nearly straight
    /// stretches are ferried (`FERRY_SPAN` to `FERRY_MAX` cells across, the
    /// road never straying more than `FERRY_STRAY` from the line between its
    /// landings); a road that wanders along a river, or runs a long way over
    /// water, keeps its bridge. Decided once for the whole road, so a crossing
    /// is wholly a ferry or wholly a bridge.
    pub(in crate::world::gen) fn link_ferries(&self, a: &Town, b: &Town) -> Rc<Vec<(Vec2, Vec2)>> {
        let key = (a.id, b.id);
        if let Some(r) = self.ferry_runs.borrow().get(&key) {
            return r.clone();
        }
        let path: Vec<Vec2> = Self::road_path(a, b).iter().map(|&(x, z)| Vec2::new(x as f32 + 0.5, z as f32 + 0.5)).collect();
        let (mut x0, mut z0, mut x1, mut z1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for p in &path {
            x0 = x0.min(p.x as i32);
            z0 = z0.min(p.y as i32);
            x1 = x1.max(p.x as i32);
            z1 = z1.max(p.y as i32);
        }
        let ctx = self.ctx(x0 - 4, z0 - 4, x1 + 4, z1 + 4);
        // walk the road in half-cell steps, noting the stretches under water
        let mut runs: Vec<Vec<Vec2>> = Vec::new();
        let mut cur: Vec<Vec2> = Vec::new();
        for w in path.windows(2) {
            let n = (w[0].distance(w[1]) * 2.0).ceil().max(1.0) as usize;
            for k in 0..=n {
                let p = w[0].lerp(w[1], k as f32 / n as f32);
                let col = self.column(&ctx, p.x.floor() as i32, p.y.floor() as i32);
                if col.h < SEA && col.town.is_none() {
                    cur.push(p);
                } else if !cur.is_empty() {
                    runs.push(std::mem::take(&mut cur));
                }
            }
        }
        if !cur.is_empty() {
            runs.push(cur);
        }
        let out: Vec<(Vec2, Vec2)> = runs
            .into_iter()
            .filter_map(|r| {
                let (p0, p1) = (*r.first()?, *r.last()?);
                let len = p0.distance(p1);
                if !(FERRY_SPAN as f32..=FERRY_MAX as f32).contains(&len) {
                    return None;
                }
                let dir = (p1 - p0) / len;
                let stray = r.iter().map(|p| (*p - p0).perp_dot(dir).abs()).fold(0.0, f32::max);
                (stray <= FERRY_STRAY).then_some((p0, p1))
            })
            .collect();
        let out = Rc::new(out);
        self.ferry_runs.borrow_mut().insert(key, out.clone());
        out
    }

    /// The ferry crossings (landing to landing) with any part near the rectangle.
    pub fn ferry_runs_near(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<(Vec2, Vec2)> {
        let mut out = Vec::new();
        for rx in div_floor(x0, REGION) - 2..=div_floor(x1, REGION) + 1 {
            for rz in div_floor(z0, REGION) - 2..=div_floor(z1, REGION) + 1 {
                let Some(t) = self.town(rx, rz) else { continue };
                for u in self.links(&t) {
                    for &(p, q) in self.link_ferries(&t, &u).iter() {
                        let (lo, hi) = (p.min(q) - Vec2::splat(3.0), p.max(q) + Vec2::splat(3.0));
                        if hi.x >= x0 as f32 && lo.x <= x1 as f32 && hi.y >= z0 as f32 && lo.y <= z1 as f32 {
                            out.push((p, q));
                        }
                    }
                }
            }
        }
        out
    }

    /// Where this road-over-water cell sits on a ferry crossing: None if it's
    /// a bridge cell (or not road over water), else how far it is from the
    /// nearer landing, in cells.
    pub fn ferry_at(&self, ctx: &Ctx, x: i32, z: i32) -> Option<f32> {
        let key = (x, z);
        if let Some(&f) = self.ferry_cells.borrow().get(&key) {
            return f;
        }
        let col = self.column(ctx, x, z);
        let out = if col.road && col.h < SEA && col.town.is_none() {
            let c = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
            self.ferry_runs_near(x - 1, z - 1, x + 1, z + 1).iter().find_map(|&(p, q)| {
                let (d, len) = (q - p, p.distance(q));
                let u = d / len;
                let along = (c - p).dot(u);
                let off = (c - p).perp_dot(u).abs();
                // the road is three cells wide; the landings are cells at the ends
                (off <= FERRY_ROAD_HALF && (-1.5..=len + 1.5).contains(&along)).then(|| along.min(len - along).max(0.0))
            })
        } else {
            None
        };
        self.ferry_cells.borrow_mut().insert(key, out);
        out
    }

    pub fn is_ferry(&self, ctx: &Ctx, x: i32, z: i32) -> bool {
        self.ferry_at(ctx, x, z).is_some()
    }

    /// The part of a road's path (cell centres, in order) a cart or walker
    /// can use without taking a ferry: the stretch before the first ferry
    /// cell, or, if `from_end`, the one after the last. The whole path if
    /// there's no ferry on it; None if what's left is under a few cells.
    pub fn dry_part(&self, pts: &[Vec2], from_end: bool) -> Option<Vec<Vec2>> {
        let (mut x0, mut z0, mut x1, mut z1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for q in pts {
            x0 = x0.min(q.x as i32);
            z0 = z0.min(q.y as i32);
            x1 = x1.max(q.x as i32);
            z1 = z1.max(q.y as i32);
        }
        let ctx = self.ctx(x0 - 4, z0 - 4, x1 + 4, z1 + 4);
        let wet: Vec<usize> = (0..pts.len()).filter(|&i| self.is_ferry(&ctx, pts[i].x.floor() as i32, pts[i].y.floor() as i32)).collect();
        let (Some(&first), Some(&last)) = (wet.first(), wet.last()) else { return Some(pts.to_vec()) };
        // stop a couple of cells short of the water, at the quay
        let part = if from_end { pts[(last + 3).min(pts.len())..].to_vec() } else { pts[..first.saturating_sub(2)].to_vec() };
        (part.len() >= 3).then_some(part)
    }

    /// A road bridge over water is a humpback: the deck climbs from the banks
    /// to three blocks over the water, so boats go under. Returns the deck's
    /// block level and how many cells this one is from the nearer bank (1 =
    /// the first cell over water). Shared by the chunk builder and `walk_y`
    /// so what's drawn is what's walked on.
    pub fn bridge_deck(&self, ctx: &Ctx, x: i32, z: i32) -> (i32, i32) {
        if let Some(&b) = self.bridges.borrow().get(&(x, z)) {
            return b;
        }
        let (a, b) = self.banks(ctx, x, z);
        let d = a.min(b);
        // a short crossing still gets a full hump
        let span = a + b - 1;
        let rise = (d + (span < 5) as i32).min(3);
        let out = (SEA + rise, d);
        self.bridges.borrow_mut().insert((x, z), out);
        out
    }

    /// Height a walker stands on (top of the solid surface, or bridge deck).
    pub fn walk_y(&self, ctx: &Ctx, x: f32, z: f32) -> f32 {
        let c = self.column(ctx, x.floor() as i32, z.floor() as i32);
        let mut y = c.h as f32 + 1.0;
        if c.road && c.h < SEA && c.town.is_none() && !self.is_ferry(ctx, x.floor() as i32, z.floor() as i32) {
            y = (self.bridge_deck(ctx, x.floor() as i32, z.floor() as i32).0 + 1) as f32;
        }
        if c.h < SEA {
            y = y.max(SEA as f32 + 0.85);
        }
        y
    }
}
