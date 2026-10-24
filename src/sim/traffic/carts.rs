use super::*;

/// Where a cart should appear on `pts` so nobody sees it pop in: the last
/// point before the path first comes within `reach` of `view`, with the
/// index of the segment it lies on. None if the path starts within reach or
/// never comes near.
pub(crate) fn entry_point(pts: &[Vec2], view: Vec2, reach: f32) -> Option<(usize, Vec2)> {
    if pts[0].distance(view) < reach {
        return None;
    }
    let mut prev = (0, pts[0]);
    for s in 0..pts.len() - 1 {
        let (a, b) = (pts[s], pts[s + 1]);
        let n = (a.distance(b) / 2.0).ceil().max(1.0) as usize;
        for k in 1..=n {
            let p = a.lerp(b, k as f32 / n as f32);
            if p.distance(view) < reach {
                return Some(prev);
            }
            prev = (s, p);
        }
    }
    None
}

impl Traffic {
    pub(super) fn spawn_cart(&mut self, gen: &Gen, view: Vec2, view_r: f32) {
        let r = view_r as i32 + 60;
        let towns = gen.towns_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r, 0);
        if towns.is_empty() {
            return;
        }
        let t = self.rng.pick(&towns).clone();
        let links = gen.links(&t);
        if links.is_empty() {
            return;
        }
        let u = self.rng.pick(&links).clone();
        let p = Gen::road_path(&t, &u);
        let mut pts: Vec<Vec2> = p.iter().map(|&(x, z)| Vec2::new(x as f32 + 0.5, z as f32 + 0.5)).collect();
        if self.rng.chance(0.5) {
            pts.reverse();
        }
        // no bridge crosses water too wide for one: carts keep to their own
        // side and wait at the quay for the ferry
        let mut pts = match gen.dry_part(&pts, false) {
            Some(whole) if whole.len() == pts.len() => whole,
            _ => {
                let from_end = self.rng.chance(0.5);
                let Some(dry) = gen.dry_part(&pts, from_end) else { return };
                dry
            }
        };
        // roads run between town centres, which is where the fountains are:
        // start and finish the journey at the plaza's edge instead
        let edge = (STREET - 1) as f32;
        let n = pts.len();
        for (end, next) in [(0, 1), (n - 1, n - 2)] {
            let d = pts[next] - pts[end];
            pts[end] += d.normalize_or_zero() * d.length().min(edge);
        }
        // keep to the right-hand side of the road
        let keep_right = |pts: &[Vec2]| -> Vec<Vec2> {
            (0..pts.len())
                .map(|i| {
                    let d = if i + 1 < pts.len() { pts[i + 1] - pts[i] } else { pts[i] - pts[i - 1] };
                    pts[i] + Vec2::new(-d.y, d.x).normalize_or_zero() * -0.7
                })
                .collect()
        };
        // appear just out of sight, from whichever end of the road is off-screen
        let reach = view_r + 12.0;
        let mut off = keep_right(&pts);
        let mut entry = entry_point(&off, view, reach);
        if entry.is_none() {
            pts.reverse();
            off = keep_right(&pts);
            entry = entry_point(&off, view, reach);
        }
        let Some((seg, start)) = entry else { return };
        let (mut x0, mut z0, mut x1, mut z1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for q in &off {
            x0 = x0.min(q.x as i32);
            z0 = z0.min(q.y as i32);
            x1 = x1.max(q.x as i32);
            z1 = z1.max(q.y as i32);
        }
        let ctx = gen.ctx(x0 - 4, z0 - 4, x1 + 4, z1 + 4);
        let y = gen.walk_y(&ctx, start.x, start.y);
        let carts = [rgb(150, 105, 65), rgb(120, 80, 50), rgb(170, 60, 50), rgb(70, 100, 140)];
        let horses = [rgb(120, 80, 50), rgb(70, 50, 40), rgb(220, 210, 200), rgb(160, 120, 80)];
        self.carts.push(Cart {
            pos: start,
            yaw: yaw_of(off[seg + 1] - start),
            pts: off,
            seg,
            y,
            phase: 0.0,
            ctx,
            body: *self.rng.pick(&carts),
            horse: *self.rng.pick(&horses),
            shirt: *self.rng.pick(&[rgb(200, 70, 60), rgb(70, 110, 170), rgb(90, 150, 90), rgb(220, 180, 70)]),
            // loaded in a good year, mostly empty in a lean one
            cargo: if self.plenty < 0.55 { 2 } else if self.plenty > 1.15 { self.rng.int(0, 1) as u8 } else { self.rng.int(0, 3) as u8 },
            clop_t: 0.0,
            wait: 6.0,
            returned: false,
        });
    }

    pub fn set_plenty(&mut self, plenty: f32) {
        self.plenty = plenty;
    }

    /// Points people should keep clear of (each cart and its horse), with
    /// the cart's heading so they can step aside sideways.
    pub fn cart_obstacles(&self) -> Vec<(Vec2, Vec2)> {
        self.carts
            .iter()
            .flat_map(|c| {
                let fwd = Vec2::new(c.yaw.sin(), c.yaw.cos());
                [(c.pos, fwd), (c.pos + fwd * 1.3, fwd), (c.pos + fwd * 2.6, fwd)]
            })
            .collect()
    }
}
