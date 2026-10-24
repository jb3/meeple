use super::*;

/// An infinite railway running at some angle across the land, bending
/// gently either side of its straight course. Lines come in a few families,
/// each at its own angle (and every world's set is turned differently), so
/// where they cross they do so at all sorts of angles. `s` is the distance
/// along the line's direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct RailLine {
    /// Direction of the line, in tenths of a degree.
    pub ang: u16,
    /// Where the straight course lies, measured sideways from the origin.
    pub coord: i32,
    /// Seeds the bends, so each line meanders differently.
    pub bends: u32,
}

/// A curved length of track joining two crossing lines: it leaves line `a`
/// at `a_s` (with a set of points there) and joins line `b` at `b_s`,
/// curving round the corner of the crossing between them.
pub struct Chord {
    pub id: u32,
    pub a: RailLine,
    pub a_s: f32,
    pub b: RailLine,
    pub b_s: f32,
    /// Where the lines cross, as a distance along each.
    pub cross_a: f32,
    pub cross_b: f32,
    pub(super) pts: Vec<Vec2>,
    pub(super) cum: Vec<f32>,
    pub(super) lo: Vec2,
    pub(super) hi: Vec2,
}

/// Where two lines cross (they must run in different directions).
pub fn crossing(a: RailLine, b: RailLine) -> Vec2 {
    let meet = |p0: Vec2, d0: Vec2, p1: Vec2, d1: Vec2| {
        let cross = d0.x * d1.y - d0.y * d1.x;
        if cross.abs() < 0.05 {
            return (p0 + p1) * 0.5;
        }
        let q = p1 - p0;
        p0 + d0 * ((q.x * d1.y - q.y * d1.x) / cross)
    };
    let (da, na) = a.frame();
    let (db, nb) = b.frame();
    let mut p = meet(na * a.coord as f32, da, nb * b.coord as f32, db);
    // the bends move each line's course, so home in on where they meet
    for _ in 0..12 {
        let pa = Vec2::from(a.point(a.along(p.x, p.y), 0.0));
        let pb = Vec2::from(b.point(b.along(p.x, p.y), 0.0));
        let ta = Vec2::from(a.tangent(a.along(pa.x, pa.y)));
        let tb = Vec2::from(b.tangent(b.along(pb.x, pb.y)));
        p = meet(pa, ta, pb, tb);
    }
    p
}

/// Does a lot come within a building's width of a railway, or straddle it?
pub(super) fn lot_near_rail(l: &RailLine, x0: i32, z0: i32, x1: i32, z1: i32) -> bool {
    let (x0, z0, x1, z1) = (x0 as f32, z0 as f32, x1 as f32 + 1.0, z1 as f32 + 1.0);
    let mut sides = (false, false);
    for (x, z) in [(x0, z0), (x1, z0), (x0, z1), (x1, z1), ((x0 + x1) * 0.5, (z0 + z1) * 0.5)] {
        let p = l.perp(x, z);
        if p.abs() < 6.5 {
            return true;
        }
        if p < 0.0 {
            sides.0 = true;
        } else {
            sides.1 = true;
        }
    }
    sides.0 && sides.1
}

impl RailLine {
    pub(super) fn waves(&self) -> [(f32, f32, f32); 2] {
        let h = self.bends;
        [
            (44.0, std::f32::consts::TAU / (1000.0 + unit(h) * 400.0), unit(hash_u32(h ^ 1)) * 6.28),
            (14.0, std::f32::consts::TAU / (380.0 + unit(hash_u32(h ^ 2)) * 160.0), unit(hash_u32(h ^ 3)) * 6.28),
        ]
    }

    /// Unit vector along the line's direction, and the one square to it.
    pub(super) fn frame(&self) -> (Vec2, Vec2) {
        let t = self.ang as f32 * 0.1 * std::f32::consts::PI / 180.0;
        let d = Vec2::new(t.cos(), t.sin());
        (d, Vec2::new(-d.y, d.x))
    }

    /// Sideways position of the centre line at `s`.
    pub fn centre(&self, s: f32) -> f32 {
        self.coord as f32 + self.waves().iter().map(|(a, f, p)| a * (s * f + p).sin()).sum::<f32>()
    }

    /// d(centre)/ds.
    pub fn slope(&self, s: f32) -> f32 {
        self.waves().iter().map(|(a, f, p)| a * f * (s * f + p).cos()).sum()
    }

    pub fn along(&self, x: f32, z: f32) -> f32 {
        let (d, _) = self.frame();
        x * d.x + z * d.y
    }

    pub(super) fn across(&self, x: f32, z: f32) -> f32 {
        let (_, n) = self.frame();
        x * n.x + z * n.y
    }

    pub(super) fn world(&self, along: f32, across: f32) -> (f32, f32) {
        let (d, n) = self.frame();
        (along * d.x + across * n.x, along * d.y + across * n.y)
    }

    /// Signed distance from the track centre line (measured square to it).
    pub fn perp(&self, x: f32, z: f32) -> f32 {
        let s = self.along(x, z);
        let k = self.slope(s);
        (self.across(x, z) - self.centre(s)) / (1.0 + k * k).sqrt()
    }

    /// The point `perp` to the side of the centre line at `s`.
    pub fn point(&self, s: f32, perp: f32) -> (f32, f32) {
        let k = self.slope(s);
        let len = (1.0 + k * k).sqrt();
        self.world(s - perp * k / len, self.centre(s) + perp / len)
    }

    /// Unit direction of travel (increasing `s`) in world x/z.
    pub fn tangent(&self, s: f32) -> (f32, f32) {
        let k = self.slope(s);
        let len = (1.0 + k * k).sqrt();
        self.world(1.0 / len, k / len)
    }

    /// The sort order two lines are named in.
    pub fn key(&self) -> (u16, i32) {
        (self.ang, self.coord)
    }
}

impl Chord {
    /// A junction curve at the crossing of `a` and `b`: it leaves `a` before
    /// the crossing, heading `dir_a` along it, and joins `b` after it,
    /// heading whichever way along `b` keeps going the same general way.
    pub(super) fn new(id: u32, a: RailLine, b: RailLine, dir_a: f32) -> Chord {
        let c = crossing(a, b);
        let (sa, sb) = (a.along(c.x, c.y), b.along(c.x, c.y));
        let ta = Vec2::from(a.tangent(sa)) * dir_a;
        let tb = Vec2::from(b.tangent(sb));
        let dir_b = if ta.dot(tb) >= 0.0 { 1.0 } else { -1.0 };
        let a_s = sa - dir_a * JUNCTION_REACH;
        let b_s = sb + dir_b * JUNCTION_REACH;
        let p0 = Vec2::from(a.point(a_s, 0.0));
        let p2 = Vec2::from(b.point(b_s, 0.0));
        // a quadratic curve with its control point at the crossing leaves and
        // joins each line along it
        let pts: Vec<Vec2> = (0..=64)
            .map(|i| {
                let t = i as f32 / 64.0;
                p0 * (1.0 - t) * (1.0 - t) + c * 2.0 * (1.0 - t) * t + p2 * t * t
            })
            .collect();
        let mut cum = vec![0.0];
        for w in pts.windows(2) {
            cum.push(cum.last().unwrap() + w[0].distance(w[1]));
        }
        let lo = pts.iter().fold(Vec2::splat(f32::MAX), |m, p| m.min(*p));
        let hi = pts.iter().fold(Vec2::splat(f32::MIN), |m, p| m.max(*p));
        Chord { id, a, a_s, b, b_s, cross_a: sa, cross_b: sb, pts, cum, lo, hi }
    }

    /// A passing loop: track that leaves `line` at `s0` through a set of
    /// points, runs alongside it `off` to one side, and rejoins at `s0 + len`.
    /// To a train it's a chord from the line back onto itself: the "crossing"
    /// each end faces is the other end.
    pub(super) fn passing_loop(id: u32, line: RailLine, s0: f32, len: f32, off: f32) -> Chord {
        let ease = 48.0;
        let pts: Vec<Vec2> = (0..=(len / 2.0) as usize)
            .map(|i| {
                let u = (i as f32 * 2.0).min(len);
                let w = smoothstep(0.0, ease, u).min(smoothstep(len, len - ease, u));
                Vec2::from(line.point(s0 + u, off * w))
            })
            .collect();
        let mut cum = vec![0.0];
        for w in pts.windows(2) {
            cum.push(cum.last().unwrap() + w[0].distance(w[1]));
        }
        let lo = pts.iter().fold(Vec2::splat(f32::MAX), |m, p| m.min(*p));
        let hi = pts.iter().fold(Vec2::splat(f32::MIN), |m, p| m.max(*p));
        Chord { id, a: line, a_s: s0, b: line, b_s: s0 + len, cross_a: s0 + len, cross_b: s0, pts, cum, lo, hi }
    }

    pub fn length(&self) -> f32 {
        *self.cum.last().unwrap()
    }

    pub(super) fn seg(&self, u: f32) -> (usize, f32) {
        let u = u.clamp(0.0, self.length());
        let i = self.cum.partition_point(|&c| c <= u).clamp(1, self.pts.len() - 1) - 1;
        let len = (self.cum[i + 1] - self.cum[i]).max(1e-5);
        (i, (u - self.cum[i]) / len)
    }

    /// Point `u` along the chord from its `a` end.
    pub fn at(&self, u: f32) -> Vec2 {
        let (i, t) = self.seg(u);
        self.pts[i].lerp(self.pts[i + 1], t)
    }

    /// Unit direction of travel from the `a` end towards the `b` end at `u`.
    pub fn tangent(&self, u: f32) -> Vec2 {
        let (i, _) = self.seg(u);
        (self.pts[i + 1] - self.pts[i]).normalize_or_zero()
    }

    pub fn near(&self, x0: f32, z0: f32, x1: f32, z1: f32, pad: f32) -> bool {
        self.hi.x + pad >= x0 && self.lo.x - pad <= x1 && self.hi.y + pad >= z0 && self.lo.y - pad <= z1
    }

    /// Distance from a point to the chord's centre line.
    pub fn dist(&self, p: Vec2) -> f32 {
        if !self.near(p.x, p.y, p.x, p.y, 12.0) {
            return f32::MAX;
        }
        self.pts
            .windows(2)
            .map(|w| {
                let d = w[1] - w[0];
                let t = ((p - w[0]).dot(d) / d.length_squared().max(1e-6)).clamp(0.0, 1.0);
                p.distance(w[0] + d * t)
            })
            .fold(f32::MAX, f32::min)
    }

    /// Points along the chord, `step` apart, for building track.
    pub fn samples(&self, step: f32) -> impl Iterator<Item = f32> + '_ {
        let n = (self.length() / step).ceil() as usize;
        (0..=n).map(move |i| (i as f32 * step).min(self.length()))
    }
}

impl Gen {
    /// Direction of each family of lines, in tenths of a degree: evenly
    /// fanned, the whole fan turned differently in every world.
    pub(super) fn rail_angle(&self, fam: u32) -> u16 {
        let turn = hash_u32(self.seed ^ 0xa17) % 360;
        ((turn + fam * 360) % 1800) as u16
    }

    /// The `k`th line of family `fam`.
    pub(super) fn rail_line(&self, fam: u32, k: i32) -> RailLine {
        let jitter = (hash2(self.seed ^ 0x2a11, k, fam as i32) % (2 * RAIL_JITTER as u32)) as i32 - RAIL_JITTER;
        RailLine {
            ang: self.rail_angle(fam),
            coord: k * RAIL_SPACING + jitter,
            bends: hash2(self.seed ^ 0xbe4d, k, fam as i32 + 1),
        }
    }

    pub fn rails_near(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<RailLine> {
        let mut out = Vec::new();
        let corners = [Vec2::new(x0 as f32, z0 as f32), Vec2::new(x1 as f32, z0 as f32), Vec2::new(x0 as f32, z1 as f32), Vec2::new(x1 as f32, z1 as f32)];
        for fam in 0..RAIL_FAMILIES {
            let probe = RailLine { ang: self.rail_angle(fam), coord: 0, bends: 0 };
            let (lo, hi) = corners.iter().fold((f32::MAX, f32::MIN), |(lo, hi), c| {
                let v = probe.across(c.x, c.y);
                (lo.min(v), hi.max(v))
            });
            let (reach, pad) = (RAIL_JITTER as f32 + RAIL_BEND_MAX, 8.0 + RAIL_BEND_MAX);
            for k in div_floor((lo - reach) as i32, RAIL_SPACING)..=div_floor((hi + reach) as i32, RAIL_SPACING) + 1 {
                let l = self.rail_line(fam, k);
                if l.coord as f32 >= lo - pad && l.coord as f32 <= hi + pad {
                    out.push(l);
                }
            }
        }
        out
    }

    /// The lie of the land under a railway's centre line, before the line is
    /// graded in: low in a canal's channel, otherwise the natural ground.
    pub(super) fn line_ground(&self, ctx: &Ctx, x: f32, z: f32) -> f32 {
        let g = self.raw_height(x, z);
        let in_canal = ctx.canals.iter().any(|&cz| (z - (cz as f32 + 0.5)).abs() < 3.4 && self.canal_open(x, cz));
        if in_canal {
            g.min((SEA - 2) as f32)
        } else {
            g
        }
    }

    /// The junction chord (if any) where two lines cross.
    pub fn chord(&self, a: RailLine, b: RailLine) -> Option<Rc<Chord>> {
        let (a, b) = if a.key() <= b.key() { (a, b) } else { (b, a) };
        let key = (a.key(), b.key());
        if let Some(c) = self.chords.borrow().get(&key) {
            return c.clone();
        }
        let h = hash2(self.seed ^ 0xc40d, a.coord ^ (a.ang as i32) << 12, b.coord ^ (b.ang as i32) << 12);
        // no junctions in cuttings, on embankments or in tunnels: they sit
        // on level ground only
        let c = (unit(h) < JUNCTION_CHANCE)
            .then(|| Chord::new(h, a, b, if h & 1 == 0 { 1.0 } else { -1.0 }))
            .filter(|c| {
                (0..=c.length() as i32 / 6).all(|i| {
                    let p = c.at(i as f32 * 6.0);
                    let (lvl, g) = (self.rail_level(p.x, p.y), self.raw_height(p.x, p.y));
                    g > (SEA + 2) as f32 && lvl - g < 3.0 && g - lvl < 4.0
                })
            })
            .map(Rc::new);
        self.chords.borrow_mut().insert(key, c.clone());
        c
    }

    /// Every place two lines cross near a box (the lines named in a fixed
    /// order), and the crossing point.
    pub fn crossings_near(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<(RailLine, RailLine, Vec2)> {
        let lines = self.rails_near(x0, z0, x1, z1);
        let mut out = Vec::new();
        for (i, a) in lines.iter().enumerate() {
            for b in &lines[i + 1..] {
                if a.ang == b.ang {
                    continue;
                }
                let (a, b) = if a.key() <= b.key() { (*a, *b) } else { (*b, *a) };
                let c = crossing(a, b);
                if c.x >= x0 as f32 && c.x <= x1 as f32 && c.y >= z0 as f32 && c.y <= z1 as f32 {
                    out.push((a, b, c));
                }
            }
        }
        out
    }

    pub fn chords_near(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<Rc<Chord>> {
        let mut out = Vec::new();
        let m = (JUNCTION_REACH + 30.0) as i32;
        for (a, b, _) in self.crossings_near(x0 - m, z0 - m, x1 + m, z1 + m) {
            if let Some(c) = self.chord(a, b) {
                if c.near(x0 as f32, z0 as f32, x1 as f32, z1 as f32, 10.0) {
                    out.push(c);
                }
            }
        }
        for l in self.rails_near(x0, z0, x1, z1) {
            let ss = [(x0, z0), (x1, z0), (x0, z1), (x1, z1)].map(|(x, z)| l.along(x as f32, z as f32));
            let lo = ss.iter().cloned().fold(f32::MAX, f32::min) as i32 - 30;
            let hi = ss.iter().cloned().fold(f32::MIN, f32::max) as i32 + 30;
            for n in div_floor(lo - LOOP_LEN as i32 - 20, LOOP_PERIOD)..=div_floor(hi, LOOP_PERIOD) {
                if let Some(c) = self.passing_loop(l, n) {
                    if c.near(x0 as f32, z0 as f32, x1 as f32, z1 as f32, 10.0) {
                        out.push(c);
                    }
                }
            }
        }
        out
    }

    /// Is a town's centre likely within `r` of a point? Cheap and free of
    /// the rest of the planner, so the countryside can steer clear of towns
    /// without towns and track depending on each other.
    pub(super) fn town_hint(&self, x: f32, z: f32, r: f32) -> bool {
        let (r0, r1) = (div_floor((x - r) as i32 - REGION, REGION), div_floor((x + r) as i32, REGION));
        let (q0, q1) = (div_floor((z - r) as i32 - REGION, REGION), div_floor((z + r) as i32, REGION));
        for rx in r0..=r1 {
            for rz in q0..=q1 {
                let h = hash2(self.seed ^ 0x7071, rx, rz);
                if unit(h) > 0.64 {
                    continue;
                }
                let m = 64;
                let cx = (rx * REGION + m + (hash_u32(h ^ 1) % (REGION - 2 * m) as u32) as i32) as f32;
                let cz = (rz * REGION + m + (hash_u32(h ^ 2) % (REGION - 2 * m) as u32) as i32) as f32;
                if Vec2::new(cx - x, cz - z).length() < r {
                    return true;
                }
            }
        }
        false
    }

    /// The passing loop (if any) in the `n`th stretch of a line: only on
    /// open, level, dry ground clear of towns, canals and crossings.
    pub(super) fn passing_loop_at(&self, l: RailLine, n: i32) -> Option<Rc<Chord>> {
        // a few tries at different spots along the stretch
        (0..3).find_map(|k| self.passing_loop_try(l, n, k))
    }

    pub(super) fn passing_loop_try(&self, l: RailLine, n: i32, k: i32) -> Option<Rc<Chord>> {
        let h = hash2(self.seed ^ 0x100b ^ (k as u32).wrapping_mul(0x9e37), n, l.coord ^ (l.ang as i32) << 12);
        if unit(h) > LOOP_CHANCE {
            return None;
        }
        let s0 = (n * LOOP_PERIOD) as f32 + 40.0 + unit(hash_u32(h ^ 5)) * (LOOP_PERIOD as f32 - LOOP_LEN - 80.0);
        let s1 = s0 + LOOP_LEN;
        let off = if hash_u32(h ^ 6) & 1 == 0 { LOOP_OFFSET } else { -LOOP_OFFSET };
        let mid = Vec2::from(l.point((s0 + s1) * 0.5, 0.0));
        for i in 0..=4 {
            let q = Vec2::from(l.point(s0 + LOOP_LEN * i as f32 / 4.0, 0.0));
            if self.town_hint(q.x, q.y, 70.0) {
                return None;
            }
        }
        // nowhere near a crossing, whose junction curves need the room
        let (a, b) = (mid - Vec2::splat(400.0), mid + Vec2::splat(400.0));
        for (ea, eb, c) in self.crossings_near(a.x as i32, a.y as i32, b.x as i32, b.y as i32) {
            let sc = if l == ea || l == eb { l.along(c.x, c.y) } else { continue };
            if sc > s0 - 70.0 && sc < s1 + 70.0 {
                return None;
            }
        }
        // sit on gentle, dry ground: no viaduct, tunnel, cutting or canal
        for i in 0..=8 {
            let s = s0 + LOOP_LEN * i as f32 / 8.0;
            for p in [0.0, off, off * 0.5] {
                let (x, z) = l.point(s, p);
                let lvl = self.rail_level(x, z);
                let g = self.raw_height(x, z);
                if g < (SEA + 2) as f32 || lvl - g > 3.5 || g - lvl > 5.0 {
                    return None;
                }
                if self.canals_near(z as i32 - 10, z as i32 + 10).iter().any(|&c| (c as f32 - z).abs() < 12.0 && self.canal_open(x, c)) {
                    return None;
                }
            }
        }
        Some(Rc::new(Chord::passing_loop(h, l, s0, LOOP_LEN, off)))
    }

    pub(super) fn passing_loop(&self, l: RailLine, n: i32) -> Option<Rc<Chord>> {
        if let Some(c) = self.loops.borrow().get(&(l, n)) {
            return c.clone();
        }
        let c = self.passing_loop_at(l, n);
        self.loops.borrow_mut().insert((l, n), c.clone());
        c
    }

    pub(super) fn rail_node(&self, i: i32, j: i32) -> f32 {
        if let Some(v) = self.rail_nodes.borrow().get(&(i, j)) {
            return *v;
        }
        let (x, z) = ((i * RAIL_NODE) as f32, (j * RAIL_NODE) as f32);
        let mut sum = 0.0;
        for (dx, dz) in [(0.0, 0.0), (40.0, 0.0), (-40.0, 0.0), (0.0, 40.0), (0.0, -40.0)] {
            sum += self.raw_height(x + dx, z + dz);
        }
        let v = (sum / 5.0).max((SEA + 4) as f32).min((SEA + 18) as f32);
        self.rail_nodes.borrow_mut().insert((i, j), v);
        v
    }

    /// Smooth height field the railways follow (same everywhere, so crossings agree).
    pub fn rail_level(&self, x: f32, z: f32) -> f32 {
        let fx = x / RAIL_NODE as f32;
        let fz = z / RAIL_NODE as f32;
        let (i, j) = (fx.floor() as i32, fz.floor() as i32);
        // x - floor(x), not fract(): fract is negative for negative coordinates
        let (tx, tz) = (smoothstep(0.0, 1.0, fx - fx.floor()), smoothstep(0.0, 1.0, fz - fz.floor()));
        let a = self.rail_node(i, j);
        let b = self.rail_node(i + 1, j);
        let c = self.rail_node(i, j + 1);
        let d = self.rail_node(i + 1, j + 1);
        lerp(lerp(a, b, tx), lerp(c, d, tx), tz)
    }

    /// Integer surface height of the track bed at a point along a line.
    pub fn rail_bed(&self, line: RailLine, s: f32) -> i32 {
        let (x, z) = line.point(s, 0.0);
        self.rail_level(x, z).floor() as i32
    }
}
