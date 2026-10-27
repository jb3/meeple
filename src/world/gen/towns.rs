use super::*;

mod roads;
mod lots;

/// Axis-aligned road segment (inclusive voxel coordinates).
#[derive(Clone, Copy, Debug)]
pub struct Seg {
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
}

#[derive(Clone, Debug)]
pub struct Town {
    pub id: u32,
    pub rx: i32,
    pub rz: i32,
    pub cx: i32,
    pub cz: i32,
    pub level: i32,
    pub radius: i32,
    pub city: bool,
    pub name: String,
    pub rail: Option<RailLine>,
    /// Local building style, from where the town sits.
    pub style: Style,
}

/// Regional building styles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    /// Timber-framed and plastered, roofs of every colour.
    Lowland,
    /// Stone cottages under slate in the hills.
    Highland,
    /// Whitewashed walls and blue roofs by the water.
    Coastal,
    /// Log cabins in the cold north.
    Northern,
    /// Whitewash, ochre and terracotta roofs in the warm south.
    Mediterranean,
    /// Plank houses under green roofs in the jungle.
    Tropical,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LotKind {
    House,
    Apartment,
    Tavern,
    Bakery,
    Smithy,
    Shop,
    Church,
    Farm,
    Garden,
    School,
    Library,
    /// A country house (not part of a town).
    Manor,
}

#[derive(Clone, Debug)]
pub struct Lot {
    pub kind: LotKind,
    /// Inclusive lot bounds (8x8 voxels).
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
    /// Direction the front door faces: 0 = -z, 1 = +x, 2 = +z, 3 = -x.
    pub face: u8,
    pub door: (i32, i32),
    /// Walkable point just outside the door.
    pub front: (f32, f32),
    /// Point on the street centre line in front of the door.
    pub street: (f32, f32),
    pub floors: i32,
    pub wall: u8,
    pub roof: u8,
    pub seed: u32,
}

impl Seg {
    pub fn new(a: (i32, i32), b: (i32, i32)) -> Self {
        Seg { x0: a.0.min(b.0), z0: a.1.min(b.1), x1: a.0.max(b.0), z1: a.1.max(b.1) }
    }

    /// Chebyshev distance from a voxel to the segment.
    pub fn dist(&self, x: i32, z: i32) -> i32 {
        let dx = (self.x0 - x).max(x - self.x1).max(0);
        let dz = (self.z0 - z).max(z - self.z1).max(0);
        dx.max(dz)
    }

    pub fn hits_rect(&self, x0: i32, z0: i32, x1: i32, z1: i32, pad: i32) -> bool {
        self.x1 + pad >= x0 && self.x0 - pad <= x1 && self.z1 + pad >= z0 && self.z0 - pad <= z1
    }
}

impl Town {
    pub fn center(&self) -> (f32, f32) {
        (self.cx as f32 + 0.5, self.cz as f32 + 0.5)
    }

    /// Half-extent of the central plaza square.
    pub fn plaza_half(&self) -> i32 {
        STREET - 2
    }

    /// Station platform centre along the rail line, if the town has one.
    /// Station platform centre along the rail line: half a block past the
    /// centre street, so the platform sits between street lines rather than
    /// across one.
    /// The ground the station and its platform cover, if the town has one
    /// (it can stand well away from the town centre, out by the line).
    pub fn station_box(&self) -> Option<(i32, i32, i32, i32)> {
        let (l, s) = (self.rail?, self.station_s()?);
        let (ax, az) = l.point(s - 6.0, -1.0);
        let (bx, bz) = l.point(s + 6.0, 5.0);
        Some((ax.min(bx) as i32 - 2, az.min(bz) as i32 - 2, ax.max(bx) as i32 + 2, az.max(bz) as i32 + 2))
    }

    pub fn station_s(&self) -> Option<f32> {
        let half = (STREET / 2 + 1) as f32;
        self.rail.map(|l| l.along(self.cx as f32, self.cz as f32).floor() + half)
    }
}

impl Lot {
    pub fn footprint(&self) -> (i32, i32, i32, i32) {
        match self.kind {
            LotKind::Apartment | LotKind::Tavern | LotKind::Church | LotKind::Library | LotKind::Manor => {
                (self.x0, self.z0, self.x1, self.z1)
            }
            // set back an extra block on the forge's side, leaving a strip of
            // open yard beyond the roof's overhang
            LotKind::Smithy => match self.forge_side() {
                1 => (self.x0 + 1, self.z0 + 1, self.x1 - 2, self.z1 - 1),
                _ => (self.x0 + 1, self.z0 + 1, self.x1 - 1, self.z1 - 2),
            },
            _ => (self.x0 + 1, self.z0 + 1, self.x1 - 1, self.z1 - 1),
        }
    }

    /// Unit step from the building towards the street.
    pub fn outward(&self) -> (i32, i32) {
        [(0, -1), (1, 0), (0, 1), (-1, 0)][self.face as usize % 4]
    }

    /// Which side of a smithy its forge stands on: the front if that faces
    /// the default camera (east or south), otherwise round the corner on a
    /// side that does, so the glow isn't hidden behind the building.
    pub fn forge_side(&self) -> u8 {
        match self.face {
            1 | 2 => self.face,
            0 => 1,
            _ => 2,
        }
    }

    /// Where a smithy's forge stands, in the open strip of yard on its side.
    pub fn forge(&self) -> (i32, i32) {
        match self.face {
            0 => (self.x1, self.z0 + 2),
            3 => (self.x0 + 2, self.z1),
            _ => {
                let (fx, fz) = (self.front.0.floor() as i32, self.front.1.floor() as i32);
                let (ox, oz) = self.outward();
                let (sx, sz) = if self.face % 2 == 0 { (1, 0) } else { (0, 1) };
                (fx + ox - sx * 2, fz + oz - sz * 2)
            }
        }
    }

    pub fn center(&self) -> (f32, f32) {
        ((self.x0 + self.x1) as f32 * 0.5 + 0.5, (self.z0 + self.z1) as f32 * 0.5 + 0.5)
    }

    /// A few houses are new builds, finished some days after the world is
    /// first seen; the rest have always been there.
    pub fn finished_day(&self, first_day: i64) -> Option<i64> {
        let h = hash_u32(self.seed ^ 0xb17d);
        (self.kind == LotKind::House && h % 100 < NEW_BUILD_CHANCE).then(|| first_day + 1 + (h >> 8) as i64 % GROWTH_DAYS)
    }

    /// The day a few houses are left to fall derelict.
    pub fn decay_day(&self) -> Option<i64> {
        let h = hash_u32(self.seed ^ 0xdec4);
        (self.kind == LotKind::House && h % 100 < DECAY_CHANCE).then(|| 40 + (h >> 8) as i64 % 220)
    }

    pub fn stage(&self, first_day: i64, day: i64) -> Stage {
        let built = match self.finished_day(first_day) {
            Some(f) if day >= f => Stage::Done,
            Some(f) if day >= f - BUILD_DAYS + 1 => Stage::Frame((day - (f - BUILD_DAYS)) as f32 / BUILD_DAYS as f32),
            Some(f) if day >= f - BUILD_DAYS - 2 => Stage::Plot,
            Some(_) => Stage::Vacant,
            None => Stage::Done,
        };
        match (built, self.decay_day()) {
            (Stage::Done, Some(d)) if day >= d && self.finished_day(first_day).map_or(true, |f| d > f + 10) => Stage::Ruin,
            (b, _) => b,
        }
    }

    pub fn residents(&self) -> u32 {
        match self.kind {
            LotKind::House => 1 + hash_u32(self.seed ^ 77) % 3,
            LotKind::Apartment => (self.floors as u32) * 2,
            LotKind::Tavern | LotKind::Bakery | LotKind::Shop | LotKind::Smithy | LotKind::School | LotKind::Library => 1,
            _ => 0,
        }
    }
}

impl Gen {
    /// A village green just outside the town, level and open, where fairs,
    /// cricket and skating happen: its centre, if there's room for one.
    pub fn green(&self, t: &Town) -> Option<(i32, i32)> {
        if let Some(g) = self.greens.borrow().get(&t.id) {
            return *g;
        }
        let g = self.find_green(t);
        self.greens.borrow_mut().insert(t.id, g);
        g
    }

    pub(in crate::world::gen) fn find_green(&self, t: &Town) -> Option<(i32, i32)> {
        let d = (t.radius + 15) as f32;
        let start = hash_u32(t.id ^ 0x92ee) as usize;
        let level = t.level as f32;
        for k in 0..8 {
            let a = (start + k) as f32 * std::f32::consts::FRAC_PI_4;
            let (gx, gz) = ((t.cx as f32 + a.cos() * d).round() as i32, (t.cz as f32 + a.sin() * d).round() as i32);
            let rails = self.rails_near(gx - 30, gz - 30, gx + 30, gz + 30);
            let chords = self.chords_near(gx - 30, gz - 30, gx + 30, gz + 30);
            let roads = self.roads_near(gx - 30, gz - 30, gx + 30, gz + 30);
            let canals = self.canals_near(gz - 30, gz + 30);
            let ok = [(0, 0), (11, 0), (-11, 0), (0, 11), (0, -11), (8, 8), (-8, -8), (8, -8), (-8, 8)].iter().all(|&(dx, dz)| {
                let (x, z) = (gx + dx, gz + dz);
                let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
                let h = self.raw_height(fx, fz);
                h >= SEA as f32 + 2.0
                    && (h - level).abs() < 2.6
                    && rails.iter().all(|l| l.perp(fx, fz).abs() > 15.0)
                    && chords.iter().all(|c| c.dist(Vec2::new(fx, fz)) > 15.0)
                    && roads.iter().all(|s| s.dist(x, z) > 5)
                    && canals.iter().all(|&c| (c - z).abs() > 12 || !self.canal_open(fx, c))
            });
            if ok {
                return Some((gx, gz));
            }
        }
        None
    }

    /// Is a point out of the way of every town green near it?
    pub fn green_clear(&self, x: f32, z: f32, pad: f32) -> bool {
        let ri = (Self::GREEN_R + pad) as i32 + 1;
        self.towns_near(x as i32 - ri - 90, z as i32 - ri - 90, x as i32 + ri + 90, z as i32 + ri + 90, 0)
            .iter()
            .filter_map(|t| self.green(t))
            .all(|(gx, gz)| Vec2::new(gx as f32 + 0.5 - x, gz as f32 + 0.5 - z).length() > Self::GREEN_R + pad)
    }

    pub fn town(&self, rx: i32, rz: i32) -> Option<Rc<Town>> {
        if let Some(t) = self.towns.borrow().get(&(rx, rz)) {
            return t.clone();
        }
        let t = self.make_town(rx, rz).map(Rc::new);
        self.towns.borrow_mut().insert((rx, rz), t.clone());
        t
    }

    pub(in crate::world::gen) fn make_town(&self, rx: i32, rz: i32) -> Option<Town> {
        let h = hash2(self.seed ^ 0x7071, rx, rz);
        if unit(h) > 0.64 {
            return None;
        }
        let m = 64;
        let mut cx = rx * REGION + m + (hash_u32(h ^ 1) % (REGION - 2 * m) as u32) as i32;
        let mut cz = rz * REGION + m + (hash_u32(h ^ 2) % (REGION - 2 * m) as u32) as i32;
        // keep the plaza clear: shift the centre away from any railway that
        // would otherwise run through it
        for l in self.rails_near(cx - 30, cz - 30, cx + 30, cz + 30) {
            let p = l.perp(cx as f32 + 0.5, cz as f32 + 0.5);
            if p.abs() < PLAZA_RAIL_CLEAR {
                // move square to the track until the centre is the clearance away
                let target = (PLAZA_RAIL_CLEAR + 1.0) * if p < 0.0 { -1.0 } else { 1.0 };
                let s = l.along(cx as f32, cz as f32);
                let (ax, az) = l.point(s, 0.0);
                let (bx, bz) = l.point(s, 1.0);
                cx += ((bx - ax) * (target - p)).round() as i32;
                cz += ((bz - az) * (target - p)).round() as i32;
            }
        }
        // no junction curves through the plaza either
        let centre = Vec2::new(cx as f32 + 0.5, cz as f32 + 0.5);
        if self.chords_near(cx - 30, cz - 30, cx + 30, cz + 30).iter().any(|c| c.dist(centre) < PLAZA_RAIL_CLEAR) {
            return None;
        }
        let raw = self.raw_height(cx as f32, cz as f32);
        if raw < SEA as f32 + 1.0 || raw > (SEA + 20) as f32 {
            return None;
        }
        let city = unit(hash_u32(h ^ 3)) < 0.25;
        let radius = if city { 38 + (hash_u32(h ^ 4) % 14) as i32 } else { 24 + (hash_u32(h ^ 4) % 12) as i32 };
        let mut wet = 0;
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.0;
            let rr = radius as f32 * 0.8;
            if self.raw_height(cx as f32 + a.cos() * rr, cz as f32 + a.sin() * rr) < SEA as f32 {
                wet += 1;
            }
        }
        if wet > 3 {
            return None;
        }
        if self.canals_near(cz - radius - 10, cz + radius + 10).iter().any(|&c| self.canal_open(cx as f32, c)) {
            return None;
        }
        let rail = self
            .rails_near(cx - radius - 12, cz - radius - 12, cx + radius + 12, cz + radius + 12)
            .into_iter()
            .min_by_key(|l| l.perp(cx as f32, cz as f32).abs() as i32);
        let level = match rail {
            Some(l) => {
                let (px, pz) = l.point(l.along(cx as f32, cz as f32), 0.0);
                self.rail_level(px, pz).floor() as i32
            }
            None => raw.round() as i32,
        }
        .max(SEA + 1);
        // near water if the land falls away to it a little beyond the edge
        let shore = (0..12).any(|k| {
            let a = k as f32 * std::f32::consts::TAU / 12.0;
            let rr = radius as f32 + 18.0;
            self.raw_height(cx as f32 + a.cos() * rr, cz as f32 + a.sin() * rr) < SEA as f32
        });
        let (warmth, _) = self.biome_noise(cx as f32, cz as f32);
        let style = if self.coldness(cx as f32, cz as f32) > 0.35 {
            Style::Northern
        } else if self.biome(cx as f32, cz as f32, raw) == Biome::Jungle {
            Style::Tropical
        } else if level > SEA + 9 {
            Style::Highland
        } else if warmth > 0.22 {
            Style::Mediterranean
        } else if shore || wet > 0 {
            Style::Coastal
        } else {
            Style::Lowland
        };
        Some(Town {
            id: h,
            rx,
            rz,
            cx,
            cz,
            level,
            radius,
            city,
            name: names::town_name(h),
            rail,
            style,
        })
    }

    pub fn towns_near(&self, x0: i32, z0: i32, x1: i32, z1: i32, margin: i32) -> Vec<Rc<Town>> {
        let mut out = Vec::new();
        for rx in div_floor(x0 - margin - 80, REGION)..=div_floor(x1 + margin + 80, REGION) {
            for rz in div_floor(z0 - margin - 80, REGION)..=div_floor(z1 + margin + 80, REGION) {
                if let Some(t) = self.town(rx, rz) {
                    let r = t.radius + margin;
                    let station = t.station_box().is_some_and(|(a, b, c, d)| c >= x0 && a <= x1 && d >= z0 && b <= z1);
                    if (t.cx + r >= x0 && t.cx - r <= x1 && t.cz + r >= z0 && t.cz - r <= z1) || station {
                        out.push(t);
                    }
                }
            }
        }
        out
    }
}
