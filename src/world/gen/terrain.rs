use super::*;

mod ferries;
use crate::noise4::*;
use wide::f32x4;

/// Everything near a chunk that columns need to know about.
pub struct Ctx {
    pub towns: Vec<Rc<Town>>,
    pub roads: Vec<Seg>,
    pub rails: Vec<RailLine>,
    pub chords: Vec<Rc<Chord>>,
    pub sites: Vec<Rc<Site>>,
    /// z of each canal near here.
    pub canals: Vec<i32>,
}

/// Terrain noise worked out ahead for a rectangle of columns.
pub struct NoiseBlock {
    w: i32,
    raw: Vec<f32>,
    tn: Vec<f32>,
    mn: Vec<f32>,
}

impl NoiseBlock {
    /// (height, warmth, wetness) at offset (dx, dz) from the block's corner.
    pub fn at(&self, dx: i32, dz: i32) -> (f32, f32, f32) {
        let i = (dz * self.w + dx) as usize;
        (self.raw[i], self.tn[i], self.mn[i])
    }
}

pub struct Column {
    pub h: i32,
    pub biome: Biome,
    pub town: Option<usize>,
    /// Rail line and signed offset from its centre when on the track bed.
    pub rail: Option<(RailLine, f32)>,
    /// On the bed of a junction chord.
    pub chord: bool,
    /// Part of a countryside site (levelled ground, or a quarry's pit).
    pub site: bool,
    pub quarry: bool,
    /// Offset from a canal's centre line when in its channel or on a towpath.
    pub canal: Option<f32>,
    /// Railway carried over low ground on a viaduct (deck level), or through
    /// high ground in a tunnel (track bed level); `along` is the distance
    /// along the line, for spacing piers.
    pub viaduct: Option<i32>,
    pub tunnel: Option<i32>,
    pub along: f32,
    pub road: bool,
    pub street: bool,
    pub plaza: bool,
}

impl Column {
    pub fn built(&self) -> bool {
        self.town.is_some()
            || self.rail.is_some()
            || self.chord
            || self.site
            || self.canal.is_some()
            || self.viaduct.is_some()
            || self.tunnel.is_some()
            || self.road
    }
}

impl Gen {
    pub fn raw_height(&self, x: f32, z: f32) -> f32 {
        let s = self.seed;
        let c = fbm(s, x / 700.0, z / 700.0, 4) + 0.38;
        let base = SEA as f32 + 1.0 + c * 16.0;
        let hills = fbm(s.wrapping_add(11), x / 150.0, z / 150.0, 4) * 8.0;
        let detail = fbm(s.wrapping_add(23), x / 38.0, z / 38.0, 2) * 1.4;
        let mmask = smoothstep(0.08, 0.42, fbm(s.wrapping_add(31), x / 950.0, z / 950.0, 3));
        let ridge = ridged(s.wrapping_add(41), x / 240.0, z / 240.0, 4);
        let mount = ridge * ridge * 36.0 * mmask * smoothstep(-0.1, 0.4, c);
        let mut h = base + hills + detail + mount;

        // gentle terracing gives the landscape a hand-made, stepped look
        let tq = h / 3.0;
        let terr = (tq.floor() + smoothstep(0.55, 1.0, tq.fract())) * 3.0;
        h = lerp(h, terr, 0.55);

        // rivers carve meandering valleys down to below sea level
        let r = self.river(x, z);
        let rw = RIVER_WIDTH;
        if r < rw * 2.5 {
            let t = smoothstep(rw * 2.5, rw * 0.7, r) * (1.0 - mmask * 0.8);
            h = lerp(h, SEA as f32 - 2.5, t);
        }
        h.clamp(2.0, MAX_H as f32)
    }

    /// `raw_height` at four points, the same arithmetic on `f32x4`.
    pub fn raw_height4(&self, x: f32x4, z: f32x4) -> f32x4 {
        let s = self.seed;
        let k = f32x4::splat;
        let c = fbm4(s, x / k(700.0), z / k(700.0), 4) + k(0.38);
        let base = k(SEA as f32 + 1.0) + c * k(16.0);
        let hills = fbm4(s.wrapping_add(11), x / k(150.0), z / k(150.0), 4) * k(8.0);
        let detail = fbm4(s.wrapping_add(23), x / k(38.0), z / k(38.0), 2) * k(1.4);
        let mmask = smoothstep4(0.08, 0.42, fbm4(s.wrapping_add(31), x / k(950.0), z / k(950.0), 3));
        let ridge = ridged4(s.wrapping_add(41), x / k(240.0), z / k(240.0), 4);
        let mount = ridge * ridge * k(36.0) * mmask * smoothstep4(-0.1, 0.4, c);
        let mut h = base + hills + detail + mount;
        let tq = h / k(3.0);
        let fl = tq.floor();
        let terr = (fl + smoothstep4(0.55, 1.0, tq - fl)) * k(3.0);
        h = h + (terr - h) * k(0.55);
        let r = fbm4(s.wrapping_add(53), x / k(560.0), z / k(560.0), 3).abs();
        let rw = RIVER_WIDTH;
        let t = smoothstep4(rw * 2.5, rw * 0.7, r) * (k(1.0) - mmask * k(0.8));
        h = h + (k(SEA as f32 - 2.5) - h) * t;
        h.max(k(2.0)).min(k(MAX_H as f32))
    }

    /// Terrain noise for a block of columns starting at (x0, z0), `w` wide and
    /// `d` deep, four at a time: heights and the two biome fields, row by row.
    pub fn noise_block(&self, x0: i32, z0: i32, w: i32, d: i32) -> NoiseBlock {
        let n = (w * d) as usize;
        let mut b = NoiseBlock { w, raw: vec![0.0; n], tn: vec![0.0; n], mn: vec![0.0; n] };
        let s = self.seed;
        for dz in 0..d {
            let z = f32x4::splat((z0 + dz) as f32 + 0.5);
            let mut dx = 0;
            while dx < w {
                let lanes = [0.0, 1.0, 2.0, 3.0].map(|k| ((x0 + (dx + k as i32).min(w - 1)) as f32) + 0.5);
                let x = f32x4::from(lanes);
                let raw = self.raw_height4(x, z).to_array();
                let k = f32x4::splat;
                let tn = fbm4(s.wrapping_add(61), x / k(1100.0), z / k(1100.0), 3).to_array();
                let mn = fbm4(s.wrapping_add(71), x / k(800.0), z / k(800.0), 3).to_array();
                for l in 0..4 {
                    if dx + l < w {
                        let i = (dz * w + dx + l) as usize;
                        b.raw[i] = raw[l as usize];
                        b.tn[i] = tn[l as usize];
                        b.mn[i] = mn[l as usize];
                    }
                }
                dx += 4;
            }
        }
        b
    }

    /// Near zero along a river's course.
    pub(in crate::world::gen) fn river(&self, x: f32, z: f32) -> f32 {
        fbm(self.seed.wrapping_add(53), x / 560.0, z / 560.0, 3).abs()
    }

    pub fn biome(&self, x: f32, z: f32, h: f32) -> Biome {
        if h > (SEA + 20) as f32 {
            return Biome::Mountain;
        }
        let (tn, mn) = self.biome_noise(x, z);
        Self::biome_from(h, tn, mn)
    }

    /// The two slow noise fields the biome is picked from: warmth and wetness.
    pub fn biome_noise(&self, x: f32, z: f32) -> (f32, f32) {
        let s = self.seed;
        (fbm(s.wrapping_add(61), x / 1100.0, z / 1100.0, 3), fbm(s.wrapping_add(71), x / 800.0, z / 800.0, 3))
    }

    pub fn biome_from(h: f32, tn: f32, mn: f32) -> Biome {
        if h > (SEA + 20) as f32 {
            return Biome::Mountain;
        }
        let t = tn - (h - SEA as f32) / 90.0;
        let m = mn;
        if t > 0.17 && m < 0.05 {
            Biome::Desert
        } else if t > 0.1 && m > 0.1 {
            Biome::Jungle
        } else if t < -0.2 {
            Biome::Tundra
        } else if m > 0.1 {
            Biome::Forest
        } else {
            Biome::Plains
        }
    }

    /// How cold the climate is at a location, 0 (warm) to 1 (frozen).
    pub fn coldness(&self, x: f32, z: f32) -> f32 {
        let t = fbm(self.seed.wrapping_add(61), x / 1100.0, z / 1100.0, 3);
        smoothstep(-0.1, -0.35, t)
    }

    pub fn ctx(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Ctx {
        Ctx {
            towns: self.towns_near(x0, z0, x1, z1, 24 + 34),
            roads: self.roads_near(x0, z0, x1, z1),
            rails: self.rails_near(x0, z0, x1, z1),
            chords: self.chords_near(x0, z0, x1, z1),
            sites: self.sites_near(x0, z0, x1, z1),
            canals: self.canals_near(z0, z1),
        }
    }

    pub fn column(&self, ctx: &Ctx, x: i32, z: i32) -> Column {
        let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
        let raw = self.raw_height(fx, fz);
        let (tn, mn) = self.biome_noise(fx, fz);
        self.column_with(ctx, x, z, raw, tn, mn)
    }

    /// A column whose terrain noise has already been worked out (see `noise_block`).
    pub fn column_with(&self, ctx: &Ctx, x: i32, z: i32, raw: f32, tn: f32, mn: f32) -> Column {
        let fx = x as f32 + 0.5;
        let fz = z as f32 + 0.5;
        let mut h = raw;
        let mut town = None;
        for (i, t) in ctx.towns.iter().enumerate() {
            let (tx, tz) = t.center();
            let d = ((fx - tx).powi(2) + (fz - tz).powi(2)).sqrt();
            let r = t.radius as f32;
            if d < r + 22.0 {
                let w = smoothstep(r + 22.0, r + 3.0, d);
                h = lerp(h, t.level as f32 + 0.5, w);
                if d < r + 3.0 {
                    town = Some(i);
                }
            }
        }
        let townish = h != raw;
        // canals: a level channel with a towpath each side
        let mut canal = None;
        for &cz in &ctx.canals {
            let p = fz - (cz as f32 + 0.5);
            if p.abs() < 7.0 && self.canal_open(fx, cz) {
                if p.abs() < 1.6 {
                    h = (SEA - 2) as f32 + 0.5;
                    canal = Some(p);
                } else if p.abs() < 3.4 {
                    h = (SEA + 1) as f32 + 0.5;
                    canal = Some(p);
                } else {
                    h = lerp(h, (SEA + 1) as f32 + 0.5, smoothstep(7.0, 3.4, p.abs()));
                }
            }
        }
        // railways are graded into the land, but cross low ground on a
        // viaduct and pass through high ground in a tunnel. That's judged by
        // the ground along the line's centre, so the whole track width agrees.
        let mut rail = None;
        let (mut viaduct, mut tunnel, mut along) = (None, None, 0.0);
        // where lines run close together (crossing at a shallow angle), the
        // nearer one decides whether this ground is a viaduct or a tunnel
        let nearest = ctx.rails.iter().map(|l| (l, l.perp(fx, fz).abs())).filter(|(_, p)| *p < 9.0).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|(l, _)| *l);
        // (a line over water is always on a viaduct, however little it rises)
        let owner = nearest.map(|n| {
            let lvl = self.rail_level(fx, fz).floor() + 0.5;
            let (cx_, cz_) = n.point(n.along(fx, fz), 0.0);
            let ground = self.line_ground(ctx, cx_, cz_);
            (lvl - ground, ground < SEA as f32)
        });
        for l in &ctx.rails {
            let p = l.perp(fx, fz);
            if p.abs() < 9.0 {
                // the rail height field at this very spot, so crossing lines agree
                let lvl = self.rail_level(fx, fz).floor() + 0.5;
                let s = l.along(fx, fz);
                let (rise, wet) = owner.unwrap_or((0.0, false));
                if !townish && (rise > VIADUCT_RISE || wet) {
                    if p.abs() < 2.4 {
                        viaduct = Some(lvl.floor() as i32);
                        along = s;
                        if p.abs() < 2.0 {
                            rail = Some((*l, p));
                        }
                    }
                    continue;
                }
                if !townish && -rise > TUNNEL_DEPTH {
                    if p.abs() < 2.6 {
                        tunnel = Some(lvl.floor() as i32);
                        along = s;
                        if p.abs() < 2.0 {
                            rail = Some((*l, p));
                        }
                    }
                    continue;
                }
                h = lerp(h, lvl, smoothstep(9.0, 2.6, p.abs()));
                if p.abs() < 2.0 {
                    rail = Some((*l, p));
                }
            }
        }
        // junction chords are graded the same way
        let mut chord = false;
        for c in &ctx.chords {
            let d = c.dist(Vec2::new(fx, fz));
            if d < 9.0 {
                let lvl = self.rail_level(fx, fz).floor() + 0.5;
                h = lerp(h, lvl, smoothstep(9.0, 2.6, d));
                chord |= d < 2.0;
            }
        }
        // countryside sites: level the ground under buildings, step a quarry down
        let (mut site, mut quarry) = (false, false);
        for st in &ctx.sites {
            let d = Vec2::new(fx, fz).distance(st.centre());
            match st.kind {
                SiteKind::Quarry => {
                    if d < st.r {
                        let depth = ((st.r - d) / 2.0).floor().min(5.0);
                        h = h.min(st.level as f32 + 0.5 - depth);
                        site = true;
                        quarry = true;
                    }
                }
                SiteKind::Ruin | SiteKind::ChalkFigure => site |= d < st.r,
                SiteKind::Harbour => {}
                // a ski resort's premade run: laid at its own even grade down the
                // whole corridor, and blended into the land round about
                SiteKind::SkiResort => {
                    let off = st.ski_off(fx, fz);
                    if off < SKI_BLEND {
                        let (u, _) = st.ski_uv(fx, fz);
                        // (exactly the plan inside the corridor, so what's built on it stands on it)
                        h = if off <= 0.0 { st.ski_height(u) } else { lerp(h, st.ski_height(u), smoothstep(SKI_BLEND, 0.0, off)) };
                        site |= off < 2.0;
                    }
                }
                // a pool sunk into level ground, reeds and rim round it
                SiteKind::Oasis | SiteKind::Geyser if d < st.r + 6.0 => {
                    h = lerp(h, st.level as f32 + 0.5, smoothstep(st.r + 6.0, st.r, d));
                    site |= d < st.r;
                    if d < 4.4 {
                        h = (st.level - 2) as f32 + 0.5;
                    }
                }
                SiteKind::Watermill if d < st.r + 10.0 => {
                    // level ground for the mill, but on the river side a
                    // water-filled race for the wheel, open to the river
                    let (ox, oz) = [(0.0, -1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)][st.face as usize % 4];
                    let rel = Vec2::new(fx, fz) - st.centre();
                    let along = rel.x * ox + rel.y * oz;
                    let lat = (rel.x * oz - rel.y * ox).abs();
                    if along > 2.5 && lat < 2.4 {
                        h = h.min((SEA - 2) as f32 + 0.5);
                        site = true;
                    } else if along > 2.5 {
                        // never build the bank up towards the water
                        h = h.min(lerp(h, st.level as f32 + 0.5, smoothstep(st.r + 6.0, st.r, d)));
                    } else if d < st.r + 6.0 {
                        h = lerp(h, st.level as f32 + 0.5, smoothstep(st.r + 6.0, st.r, d));
                        site |= d < st.r;
                    }
                }
                _ => {
                    if d < st.r + 6.0 {
                        h = lerp(h, st.level as f32 + 0.5, smoothstep(st.r + 6.0, st.r, d));
                        site |= d < st.r;
                    }
                }
            }
        }
        // a village green is levelled flat (blending into the land round it),
        // so the cricket pitch and the skating rink sit on level ground
        if town.is_none() {
            for t in &ctx.towns {
                if let Some((gx, gz)) = self.green(t) {
                    let c = Vec2::new(gx as f32 + 0.5, gz as f32 + 0.5);
                    let d = Vec2::new(fx, fz).distance(c);
                    if d < Self::GREEN_R + 8.0 {
                        let lvl = self.raw_height(c.x, c.y).floor() + 0.5;
                        h = lerp(h, lvl, smoothstep(Self::GREEN_R + 8.0, Self::GREEN_R + 1.0, d));
                    }
                }
            }
        }
        let road = ctx.roads.iter().any(|s| s.dist(x, z) <= 1);
        let (mut street, mut plaza) = (false, false);
        if let Some(ti) = town {
            let t = &ctx.towns[ti];
            let (dx, dz) = (x - t.cx, z - t.cz);
            let lx = dx.rem_euclid(STREET);
            let lz = dz.rem_euclid(STREET);
            let on = |l: i32| l == 0 || l == 1 || l == STREET - 1;
            let d2 = dx * dx + dz * dz;
            street = (on(lx) || on(lz)) && d2 < (t.radius - 1) * (t.radius - 1);
            let ph = t.plaza_half();
            plaza = dx.abs() <= ph && dz.abs() <= ph;
        }
        let hi = (h.floor() as i32).clamp(1, MAX_H);
        Column { h: hi, biome: Self::biome_from(raw, tn, mn), town, rail, chord, site, quarry, canal, viaduct, tunnel, along, road, street, plaza }
    }

    /// How many cells along the road from here to land either way (the first
    /// cell over water is 1 from the bank), looking at most `BANK_CAP` cells.
    pub fn banks(&self, ctx: &Ctx, x: i32, z: i32) -> (i32, i32) {
        let along_z = ctx.roads.iter().find(|s| s.dist(x, z) <= 1).map_or(true, |s| s.x0 == s.x1);
        let bank = |sgn: i32| {
            (1..=BANK_CAP)
                .find(|&k| {
                    let (bx, bz) = if along_z { (x, z + sgn * k) } else { (x + sgn * k, z) };
                    self.column(ctx, bx, bz).h >= SEA
                })
                .unwrap_or(BANK_CAP + 1)
        };
        (bank(-1), bank(1))
    }
}
