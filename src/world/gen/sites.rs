use super::*;

/// Things built out in the countryside rather than in towns.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SiteKind {
    Windmill,
    Manor,
    Ruin,
    Quarry,
    Watermill,
    Lighthouse,
    /// Megaliths in a ring, with a stone to catch the solstice dawn.
    StoneCircle,
    /// A giant figure cut into a chalk hillside.
    ChalkFigure,
    /// A hot spring with a geyser, in the cold high country.
    Geyser,
    /// A desert pool among palms, where a camel caravan stops.
    Oasis,
    /// Chalets at the foot of a ski lift.
    SkiResort,
    /// A mine in the hills: a winding headframe over the shaft, a spoil heap
    /// and a tunnel mouth where little trucks run in and out.
    Mine,
    /// A town's jetty, running out over the water from (x, z) along `face`.
    Harbour,
    /// Cloister, church and a bell that rings the hours, on gentle hills.
    Monastery,
    /// A wayside shrine with a lamp that burns all night.
    Shrine,
    /// A ring bank and palisade round a keep, on high ground.
    HillFort,
    /// A travellers' camp: tents, a caravan and a fire.
    Camp,
    /// Rows of apple trees, picked in autumn.
    Orchard,
    /// Hives among flowers, with a beekeeper and a honey stall.
    Apiary,
}

/// A countryside site: where it is, the ground level it's built at, and
/// how far round it the land is levelled (or carved, for a quarry).
#[derive(Clone, Debug)]
pub struct Site {
    pub kind: SiteKind,
    pub x: i32,
    pub z: i32,
    pub level: i32,
    pub r: f32,
    /// Which way it faces (0 = -z, 1 = +x, 2 = +z, 3 = -x).
    pub face: u8,
    pub seed: u32,
    /// A ski resort's slope: how much the run rises per cell going uphill.
    pub grade: f32,
}

impl Site {
    /// A point in the frame of a ski resort: `u` up the slope from its
    /// centre, `v` across it (the same frame the premade run is laid in).
    pub fn ski_uv(&self, x: f32, z: f32) -> (f32, f32) {
        let (ox, oz) = [(0.0f32, -1.0f32), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)][self.face as usize % 4];
        let (dx, dz) = (x - (self.x as f32 + 0.5), z - (self.z as f32 + 0.5));
        (dx * ox + dz * oz, -dx * oz + dz * ox)
    }

    /// The world position of a point in that frame.
    pub fn ski_at(&self, u: f32, v: f32) -> Vec2 {
        let (ox, oz) = [(0.0f32, -1.0f32), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)][self.face as usize % 4];
        Vec2::new(self.x as f32 + 0.5 + ox * u - oz * v, self.z as f32 + 0.5 + oz * u + ox * v)
    }

    /// The height the premade run is laid at, `u` along it (before it's
    /// blended into the land round the edges).
    pub fn ski_height(&self, u: f32) -> f32 {
        self.level as f32 + 0.5 + self.grade * u.clamp(-SKI_HALF_LEN, SKI_HALF_LEN)
    }

    /// Where you stand on the run at (u, v), inside its corridor: the top of
    /// the block the terrain lays there.
    pub fn ski_surface(&self, u: f32) -> f32 {
        self.ski_height(u).floor() + 1.0
    }

    /// How far outside the premade run's corridor a point is (0 inside it).
    pub fn ski_off(&self, x: f32, z: f32) -> f32 {
        let (u, v) = self.ski_uv(x, z);
        let (eu, ev) = ((u.abs() - SKI_HALF_LEN).max(0.0), (v.abs() - SKI_HALF_WIDTH).max(0.0));
        (eu * eu + ev * ev).sqrt()
    }

    pub fn centre(&self) -> Vec2 {
        Vec2::new(self.x as f32 + 0.5, self.z as f32 + 0.5)
    }

    pub fn near(&self, x0: i32, z0: i32, x1: i32, z1: i32, pad: f32) -> bool {
        let reach = (self.r + pad) as i32 + 1;
        self.x + reach >= x0 && self.x - reach <= x1 && self.z + reach >= z0 && self.z - reach <= z1
    }
}

impl Gen {
    /// A spot on a river bank in this region for a watermill: the mill's
    /// centre (a little way back from the water) and the way to the water.
    pub(super) fn mill_spot(&self, rx: i32, rz: i32) -> Option<(Vec2, Vec2)> {
        const STEP: i32 = 6;
        let n = REGION / STEP;
        let h = hash2(self.seed ^ 0x3111, rx, rz);
        (0..n * n).map(|k| (k + (h % (n * n) as u32) as i32) % (n * n)).find_map(|k| {
            let w = Vec2::new((rx * REGION + (k % n) * STEP) as f32 + 0.5, (rz * REGION + (k / n) * STEP) as f32 + 0.5);
            if self.river(w.x, w.y) > RIVER_WIDTH * 0.6 || self.raw_height(w.x, w.y) > SEA as f32 - 0.5 {
                return None;
            }
            // walk out to the nearest bank
            (0..8).find_map(|d| {
                let a = d as f32 * std::f32::consts::FRAC_PI_4;
                let dir = Vec2::new(a.cos(), a.sin());
                let land = (1..14).map(|i| w + dir * i as f32).find(|p| self.raw_height(p.x, p.y) > SEA as f32 + 1.2)?;
                // the last of the water before the bank: the wheel goes there,
                // and the mill stands back on dry land behind it
                let edge = (0..14).map(|i| w + dir * i as f32).take_while(|p| self.raw_height(p.x, p.y) < SEA as f32 - 0.3).last()?;
                let spot = edge + dir * 4.6;
                if self.raw_height(spot.x, spot.y) < SEA as f32 + 0.8 || spot.distance(land) > 6.0 {
                    return None;
                }
                // kept off the region's edges, so neighbouring regions' mills
                // can't meet
                let (lx, lz) = (spot.x - (rx * REGION) as f32, spot.y - (rz * REGION) as f32);
                let inside = [lx, lz].iter().all(|v| (12.0..REGION as f32 - 12.0).contains(v));
                let flat = (self.raw_height(spot.x, spot.y) - self.raw_height(land.x, land.y)).abs() < 4.0;
                let clear = self.site_clear(spot.x as i32, spot.y as i32, 4.0);
                (inside && flat && clear).then_some((spot, -dir))
            })
        })
    }

    /// Is this a clear spot for a site of radius `r`: dry, off the roads and
    /// railways, and away from towns?
    /// Room for one person to stand: no town buildings, road, rail or canal
    /// right on the spot.
    pub fn standing_clear(&self, x: i32, z: i32) -> bool {
        let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
        let ri = 6;
        self.towns_near(x - ri, z - ri, x + ri, z + ri, 8).is_empty()
            && self.roads_near(x - ri, z - ri, x + ri, z + ri).iter().all(|s| s.dist(x, z) as f32 > 4.0)
            && self.rails_near(x - ri, z - ri, x + ri, z + ri).iter().all(|l| l.perp(fx, fz).abs() > 5.0)
            && self.canals_near(z - ri, z + ri).iter().all(|&c| (c - z).abs() as f32 > 4.0 || !self.canal_open(fx, c))
    }

    /// As `site_clear`, but without keeping away from towns (for sites that
    /// belong beside a village), though still clear of the roads and rails.
    pub(super) fn site_clear_near_town(&self, x: i32, z: i32, r: f32) -> bool {
        let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
        let ri = r as i32 + 12;
        self.raw_height(fx, fz) > SEA as f32 + 1.5
            && self.towns_near(x - ri, z - ri, x + ri, z + ri, 0).iter().all(|t| Vec2::new((t.cx - x) as f32, (t.cz - z) as f32).length() > t.radius as f32 + r + 4.0)
            && self.green_clear(fx, fz, r)
            && self.roads_near(x - ri, z - ri, x + ri, z + ri).iter().all(|s| s.dist(x, z) as f32 > r + 3.0)
            && self.rails_near(x - ri, z - ri, x + ri, z + ri).iter().all(|l| l.perp(fx, fz).abs() > r + 10.0)
            && self.canals_near(z - ri, z + ri).iter().all(|&c| (c - z).abs() as f32 > r + 8.0 || !self.canal_open(fx, c))
    }

    pub(super) fn site_clear(&self, x: i32, z: i32, r: f32) -> bool {
        let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
        let ri = r as i32 + 12;
        self.raw_height(fx, fz) > SEA as f32 + 1.5
            && self.towns_near(x - ri, z - ri, x + ri, z + ri, 8).is_empty()
            && self.green_clear(fx, fz, r)
            && self.roads_near(x - ri, z - ri, x + ri, z + ri).iter().all(|s| s.dist(x, z) as f32 > r + 6.0)
            && self.rails_near(x - ri, z - ri, x + ri, z + ri).iter().all(|l| l.perp(fx, fz).abs() > r + 10.0)
            && self.chords_near(x - ri, z - ri, x + ri, z + ri).iter().all(|c| c.dist(Vec2::new(fx, fz)) > r + 10.0)
            && self.canals_near(z - ri, z + ri).iter().all(|&c| (c - z).abs() as f32 > r + 8.0 || !self.canal_open(fx, c))
    }

    /// The countryside sites in a region: a windmill by its village, and
    /// perhaps a manor, ruin or quarry out in the wilds.
    pub fn sites(&self, rx: i32, rz: i32) -> Rc<Vec<Rc<Site>>> {
        // no two sites share ground (a jetty runs out over the water, so it
        // only needs a little room)
        let apart = |out: &Vec<Rc<Site>>, x: i32, z: i32, r: f32| {
            out.iter().all(|o| {
                let room = if o.kind == SiteKind::Harbour { 6.0 } else { o.r };
                Vec2::new((o.x - x) as f32, (o.z - z) as f32).length() > room + r + 6.0
            })
        };
        if let Some(v) = self.sites.borrow().get(&(rx, rz)) {
            return v.clone();
        }
        let mut out = Vec::new();
        let h = hash2(self.seed ^ 0x517e, rx, rz);
        let at = |salt: u32| {
            let k = hash_u32(h ^ salt);
            (rx * REGION + 40 + (k % (REGION as u32 - 80)) as i32, rz * REGION + 40 + ((k >> 12) % (REGION as u32 - 80)) as i32)
        };
        let mk = |kind, x: i32, z: i32, r: f32, salt: u32| {
            let level = self.raw_height(x as f32 + 0.5, z as f32 + 0.5).round() as i32;
            Rc::new(Site { kind, x, z, level: level.max(SEA + 1), r, face: (hash_u32(h ^ salt) % 4) as u8, seed: hash_u32(h ^ salt ^ 0x99), grade: 0.0 })
        };
        // by the water: a watermill on a river, or a harbour and lighthouse
        // on a lake or the sea
        if let Some(t) = self.town(rx, rz) {
            if let Some((_, land, dir, big)) = self.water_near(&t) {
                let face = if dir.x.abs() > dir.y.abs() {
                    if dir.x > 0.0 { 1 } else { 3 }
                } else if dir.y > 0.0 {
                    2
                } else {
                    0
                };
                let snap = |p: Vec2| (p.x.floor() as i32, p.y.floor() as i32);
                let clear = |p: Vec2| {
                    let (x, z) = snap(p);
                    self.roads_near(x - 8, z - 8, x + 8, z + 8).iter().all(|s| s.dist(x, z) > 4)
                        && self.rails_near(x - 12, z - 12, x + 12, z + 12).iter().all(|l| l.perp(p.x, p.y).abs() > 9.0)
                };
                let level = |p: Vec2| self.raw_height(p.x, p.y).round().max((SEA + 1) as f32) as i32;
                let centre = Vec2::new(t.cx as f32 + 0.5, t.cz as f32 + 0.5);
                let outside = |p: Vec2, room: f32| centre.distance(p) > t.radius as f32 + room;
                if big {
                    // the jetty starts at the town's edge at the earliest
                    let edge = centre + dir * (t.radius as f32 + 3.0);
                    let start = if outside(land, 3.0) { land } else { edge };
                    if clear(start) {
                        let (x, z) = snap(start);
                        out.push(Rc::new(Site { kind: SiteKind::Harbour, x, z, level: SEA + 1, r: 0.0, face, seed: hash_u32(h ^ 21), grade: 0.0 }));
                    }
                    // a lighthouse along the shore, well clear of the houses
                    let along = Vec2::new(-dir.y, dir.x) * if h % 2 == 0 { 1.0 } else { -1.0 };
                    let spot = [18.0, 26.0, 34.0, -18.0, -26.0, -34.0].iter().map(|d| land + along * *d).find(|p| {
                        outside(*p, 10.0) && self.raw_height(p.x, p.y) > SEA as f32 + 1.2 && clear(*p)
                    });
                    if let Some(spot) = spot {
                        let (x, z) = snap(spot);
                        out.push(Rc::new(Site { kind: SiteKind::Lighthouse, x, z, level: level(spot), r: 3.0, face, seed: hash_u32(h ^ 22), grade: 0.0 }));
                    }
                } else {
                    let spot = land - dir * 3.0;
                    if clear(spot) && outside(spot, 10.0) {
                        let (x, z) = snap(spot);
                        out.push(Rc::new(Site { kind: SiteKind::Watermill, x, z, level: level(spot), r: 3.0, face, seed: hash_u32(h ^ 23), grade: 0.0 }));
                    }
                }
            }
        }
        // watermills on rivers out in the countryside
        if unit(hash_u32(h ^ 31)) < MILL_CHANCE {
            if let Some((p, to_water)) = self.mill_spot(rx, rz) {
                let face = if to_water.x.abs() > to_water.y.abs() {
                    if to_water.x > 0.0 { 1 } else { 3 }
                } else if to_water.y > 0.0 {
                    2
                } else {
                    0
                };
                let (x, z) = (p.x.floor() as i32, p.y.floor() as i32);
                if apart(&out, x, z, 3.0) {
                    let level = self.raw_height(p.x, p.y).round().max((SEA + 1) as f32) as i32;
                    out.push(Rc::new(Site { kind: SiteKind::Watermill, x, z, level, r: 3.0, face, seed: hash_u32(h ^ 33), grade: 0.0 }));
                }
            }
        }
        // the windmill goes in after the waterside buildings, clear of them
        if let Some(t) = self.town(rx, rz) {
            // a windmill in the fields just outside a village
            if !t.city && unit(hash_u32(h ^ 1)) < 0.65 {
                for k in 0..4 {
                    let a = unit(hash_u32(h ^ (k + 10))) * std::f32::consts::TAU;
                    let d = t.radius as f32 + 12.0;
                    let (x, z) = ((t.cx as f32 + a.cos() * d) as i32, (t.cz as f32 + a.sin() * d) as i32);
                    let clear = self.raw_height(x as f32, z as f32) > SEA as f32 + 1.5
                        && apart(&out, x, z, 4.0)
                        // and out of the way of the village green (the fair, the cricket)
                        && self.green_clear(x as f32 + 0.5, z as f32 + 0.5, 8.0)
                        && self.roads_near(x - 10, z - 10, x + 10, z + 10).iter().all(|s| s.dist(x, z) > 7)
                        && self.rails_near(x - 14, z - 14, x + 14, z + 14).iter().all(|l| l.perp(x as f32, z as f32).abs() > 12.0);
                    if clear {
                        out.push(mk(SiteKind::Windmill, x, z, 4.0, 3));
                        break;
                    }
                }
            }
        }
        let roll = unit(hash_u32(h ^ 2));
        let (x, z) = at(4);
        let raw = self.raw_height(x as f32, z as f32);
        // quarries on high, stony ground; manors on gentle hills; ruins anywhere
        let (kind, r) = if roll < 0.35 && raw > SEA as f32 + 11.0 {
            (Some(SiteKind::Quarry), 10.0)
        } else if roll < 0.14 && (SEA as f32 + 6.0..SEA as f32 + 20.0).contains(&raw) {
            (Some(SiteKind::Manor), 11.0)
        } else if roll < 0.3 {
            (Some(SiteKind::Ruin), 6.0)
        } else {
            (None, 0.0)
        };
        if let Some(kind) = kind {
            if self.site_clear(x, z, r) && apart(&out, x, z, r) {
                out.push(mk(kind, x, z, r, 5));
            }
        }
        // rarer wonders, by the lie of the land
        {
            let wr = unit(hash_u32(h ^ 0x30f1));
            let (x, z) = at(7);
            let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
            let raw = self.raw_height(fx, fz);
            let wonder = match self.biome(fx, fz, raw) {
                Biome::Desert if wr < 0.6 && raw > SEA as f32 + 2.0 => Some((SiteKind::Oasis, 11.0)),
                Biome::Tundra | Biome::Mountain if wr < 0.3 && raw > SEA as f32 + 6.0 => Some((SiteKind::Geyser, 9.0)),
                Biome::Plains | Biome::Forest if wr < 0.1 && raw > SEA as f32 + 9.0 => Some((SiteKind::ChalkFigure, 18.0)),
                Biome::Plains | Biome::Forest if wr > 0.1 && wr < 0.26 && raw > SEA as f32 + 3.0 => Some((SiteKind::StoneCircle, 9.0)),
                _ if wr > 0.68 && wr < 0.9 && raw > SEA as f32 + 10.0 => Some((SiteKind::Mine, 12.0)),
                _ => None,
            };
            if let Some((kind, r)) = wonder {
                if self.site_clear(x, z, r) && apart(&out, x, z, r) {
                    out.push(mk(kind, x, z, r, 7));
                }
            }
        }
        // a ski resort: a premade run laid on a mountainside that suits it
        if unit(hash_u32(h ^ 0x5c11)) < 0.7 {
            for k in 0..40u32 {
                let hk = hash_u32(h ^ 0x5c12 ^ k.wrapping_mul(0x9e37));
                let (x, z) = (rx * REGION + 50 + (hk % (REGION as u32 - 100)) as i32, rz * REGION + 50 + ((hk >> 12) % (REGION as u32 - 100)) as i32);
                if let Some(st) = self.ski_site(x, z, hash_u32(hk ^ 0x77)) {
                    if apart(&out, x, z, st.r) {
                        out.push(st);
                        break;
                    }
                }
            }
        }
        // quiet places of the temperate lands: monasteries on gentle hills, forts
        // on the heights, shrines and travellers' camps
        {
            let qr = unit(hash_u32(h ^ 0x9a17));
            let (x, z) = at(9);
            let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
            let raw = self.raw_height(fx, fz);
            let temperate = matches!(self.biome(fx, fz, raw), Biome::Plains | Biome::Forest);
            let sea = SEA as f32;
            let pick = if temperate && qr < 0.12 && (sea + 4.0..sea + 20.0).contains(&raw) {
                Some((SiteKind::Monastery, 11.0))
            } else if qr < 0.24 && raw > sea + 12.0 && !matches!(self.biome(fx, fz, raw), Biome::Desert) {
                Some((SiteKind::HillFort, 13.0))
            } else if temperate && qr < 0.4 && raw > sea + 2.0 {
                Some((SiteKind::Shrine, 3.0))
            } else if temperate && qr < 0.52 && raw > sea + 2.0 {
                Some((SiteKind::Camp, 6.0))
            } else {
                None
            };
            if let Some((kind, r)) = pick {
                if self.site_clear(x, z, r) && apart(&out, x, z, r) {
                    out.push(mk(kind, x, z, r, 9));
                }
            }
        }
        // an orchard or an apiary in the fields just outside a village
        if let Some(t) = self.town(rx, rz) {
            if !t.city && unit(hash_u32(h ^ 41)) < 0.7 {
                let kind = if unit(hash_u32(h ^ 42)) < 0.55 { (SiteKind::Orchard, 10.0) } else { (SiteKind::Apiary, 6.0) };
                for k in 0..6 {
                    let a = unit(hash_u32(h ^ (k + 50))) * std::f32::consts::TAU;
                    let d = t.radius as f32 + 18.0 + kind.1;
                    let (x, z) = ((t.cx as f32 + a.cos() * d) as i32, (t.cz as f32 + a.sin() * d) as i32);
                    let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
                    let raw = self.raw_height(fx, fz);
                    let temperate = matches!(self.biome(fx, fz, raw), Biome::Plains | Biome::Forest);
                    if temperate && raw > SEA as f32 + 2.0 && self.site_clear_near_town(x, z, kind.1) && apart(&out, x, z, kind.1) {
                        out.push(mk(kind.0, x, z, kind.1, 11));
                        break;
                    }
                }
            }
        }
        let v = Rc::new(out);
        self.sites.borrow_mut().insert((rx, rz), v.clone());
        v
    }

    pub fn sites_near(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<Rc<Site>> {
        let mut out = Vec::new();
        for rx in div_floor(x0 - 60, REGION)..=div_floor(x1 + 60, REGION) {
            for rz in div_floor(z0 - 60, REGION)..=div_floor(z1 + 60, REGION) {
                for s in self.sites(rx, rz).iter() {
                    if s.near(x0, z0, x1, z1, 8.0) {
                        out.push(s.clone());
                    }
                }
            }
        }
        out
    }

    /// A ski resort with its centre here, if the land has a run in it: the
    /// mountainside rises evenly along one of the four directions (so the
    /// premade run sits on it without great cuts), the ground to either side
    /// is level, and nothing (town, road, railway, canal, water) is within reach
    /// of the whole corridor. Picks the best-fitting direction.
    pub(super) fn ski_site(&self, x: i32, z: i32, seed: u32) -> Option<Rc<Site>> {
        let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
        let raw = self.raw_height(fx, fz);
        let sea = SEA as f32;
        if !matches!(self.biome(fx, fz, raw), Biome::Tundra | Biome::Mountain) || raw < sea + 14.0 || self.coldness(fx, fz) < 0.15 {
            return None;
        }
        let mut best: Option<(f32, u8, f32)> = None;
        for face in 0..4u8 {
            let probe = Site { kind: SiteKind::SkiResort, x, z, level: raw.round() as i32, r: 0.0, face, seed, grade: 0.0 };
            let h = |u: f32, v: f32| {
                let p = probe.ski_at(u, v);
                self.raw_height(p.x, p.y)
            };
            let (lo, hi) = (h(-SKI_HALF_LEN, 0.0), h(SKI_HALF_LEN, 0.0));
            let rise = hi - lo;
            // a run worth skiing: rising 14 to 32 over its length
            if !(12.0..=34.0).contains(&rise) {
                continue;
            }
            let grade = rise / (2.0 * SKI_HALF_LEN);
            // how far the land strays from the plane the run will be laid at,
            // along it and to either side
            let mut miss = 0.0f32;
            let mut ok = true;
            for k in -4..=4 {
                let u = k as f32 * SKI_HALF_LEN / 4.0;
                let plane = raw + grade * u;
                for v in [-SKI_HALF_WIDTH, 0.0, SKI_HALF_WIDTH] {
                    let d = (h(u, v) - plane).abs();
                    miss = miss.max(d);
                    ok &= h(u, v) > sea + 4.0;
                }
            }
            if ok && miss < 8.0 && best.map_or(true, |b| miss < b.0) {
                best = Some((miss, face, grade));
            }
        }
        let (_, face, grade) = best?;
        let st = Site { kind: SiteKind::SkiResort, x, z, level: raw.round() as i32, r: SKI_HALF_LEN + SKI_HALF_WIDTH * 0.7 + 6.0, face, seed, grade };
        // nothing in the way anywhere along the corridor
        let clear = (-5..=5).all(|k| {
            let p = st.ski_at(k as f32 * SKI_HALF_LEN / 5.0, 0.0);
            self.site_clear(p.x.floor() as i32, p.y.floor() as i32, SKI_HALF_WIDTH + 4.0)
        });
        clear.then(|| Rc::new(st))
    }
}
