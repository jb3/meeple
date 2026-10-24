use super::*;

pub struct Train {
    pub id: u32,
    pub(super) front: Front,
    pub speed: f32,
    pub(super) cars: Vec<Car>,
    pub(super) freight: bool,
    pub(super) stations: Vec<f32>,
    pub(super) served: f32,
    pub(super) stop_t: f32,
    pub(super) approach_horn: bool,
    pub(super) smoke_t: f32,
    /// How long it has sat at a signal at danger.
    pub(super) wait_t: f32,
    /// Where the front of the train has been, oldest first. The cars follow
    /// this trail, so they go round junctions the way the locomotive did.
    pub(super) trail: VecDeque<Vec2>,
}

/// Stations ahead on a line, as distances along it.
pub(super) fn stations_for(gen: &Gen, l: RailLine, s0: f32, dir: f32) -> Vec<f32> {
    let (a, b) = (s0.min(s0 + dir * 2000.0), s0.max(s0 + dir * 2000.0));
    let (x0, z0) = l.point(a, -80.0);
    let (x1, z1) = l.point(b, 80.0);
    gen.towns_near(x0.min(x1) as i32, z0.min(z1) as i32, x0.max(x1) as i32, z0.max(z1) as i32, 0)
        .iter()
        .filter(|t| t.rail == Some(l))
        .filter_map(|t| t.station_s())
        .collect()
}

/// Where the signal for one approach to a crossing stands: just outside
/// the block, beside the track.
pub(super) fn signal_pos_at(gen: &Gen, line: RailLine, c: Vec2, side: u8) -> Vec3 {
    let dir = if side % 2 == 0 { 1.0 } else { -1.0 };
    let sc = line.along(c.x, c.y);
    let (x, z) = line.point(sc - dir * (BLOCK_R + 1.0), -dir * 2.6);
    Vec3::new(x, gen.rail_level(x, z) + 1.0, z)
}

pub(super) fn signal_pos(gen: &Gen, (a, b): Block, side: u8) -> Vec3 {
    let c = crate::world::gen::crossing(a, b);
    signal_pos_at(gen, if side < 2 { a } else { b }, c, side) + Vec3::Y * 2.8
}

impl Train {
    /// The very front of the train, on the ground plane.
    pub(super) fn head(&self) -> Vec2 {
        match &self.front {
            Front::Line { line, s, .. } => Vec2::from(line.point(*s, 0.0)),
            Front::Chord { chord, u, .. } => chord.at(*u),
        }
    }

    /// The point `back` behind the front, following the trail.
    pub(super) fn behind(&self, back: f32) -> Vec2 {
        let mut p = self.head();
        let mut left = back;
        for &q in self.trail.iter().rev() {
            let d = p.distance(q);
            if d >= left && d > 1e-5 {
                return p + (q - p) * (left / d);
            }
            left -= d;
            p = q;
        }
        p
    }

    pub(super) fn car_back(k: usize) -> f32 {
        k as f32 * CAR_SPACING + CAR_SPACING * 0.5
    }

    pub(super) fn car_pos(&self, gen: &Gen, k: usize) -> Vec3 {
        // the same smooth height the track geometry is built from
        let p = self.behind(Self::car_back(k));
        Vec3::new(p.x, gen.rail_level(p.x, p.y) + 1.26, p.y)
    }

    /// Where every carriage is, locomotive first, for picking a train out by click.
    pub fn car_points(&self, gen: &Gen) -> Vec<Vec3> {
        (0..self.cars.len()).map(|k| self.car_pos(gen, k)).collect()
    }

    /// Heading of car `k`: along the trail between its two bogies.
    pub(super) fn car_yaw(&self, k: usize) -> f32 {
        let b = Self::car_back(k);
        yaw_of(self.behind(b - 1.6) - self.behind(b + 1.6))
    }

    pub fn front(&self, gen: &Gen) -> Vec3 {
        self.car_pos(gen, 0)
    }

    /// Standing at a platform: the line, the station, seconds until it
    /// leaves, and how far along the line each carriage is.
    pub fn at_station(&self) -> Option<(RailLine, f32, f32, Vec<f32>)> {
        let Front::Line { line, .. } = self.front else { return None };
        if self.stop_t <= 0.0 {
            return None;
        }
        let cars = (0..self.cars.len()).map(|k| {
            let p = self.behind(Self::car_back(k));
            line.along(p.x, p.y)
        });
        // the locomotive (car 0) carries no passengers
        Some((line, self.served, self.stop_t, cars.skip(1).collect()))
    }

    /// Point to follow with the camera: the middle of the train.
    pub fn middle(&self, gen: &Gen) -> Vec3 {
        self.car_pos(gen, self.cars.len() / 2)
    }

    pub(super) fn length(&self) -> f32 {
        self.cars.len() as f32 * CAR_SPACING
    }

    /// Points along the whole train, front to back.
    pub(super) fn body(&self) -> Vec<Vec2> {
        let len = self.length();
        (0..=(len / 2.0) as usize).map(|i| self.behind(i as f32 * 2.0)).collect()
    }

    /// The next crossing ahead (not yet entered), its distance to the block
    /// edge, and which of its four signals faces this train.
    pub(super) fn next_block(&self, gen: &Gen) -> Option<(Block, f32, u8)> {
        let Front::Line { line, s, dir } = self.front else { return None };
        let h = self.head();
        let r = (BLOCK_R + SIGNAL_SIGHT) as i32 + 10;
        gen.crossings_near(h.x as i32 - r, h.y as i32 - r, h.x as i32 + r, h.y as i32 + r)
            .into_iter()
            .filter(|(a, b, _)| *a == line || *b == line)
            .map(|(a, b, c)| {
                let sc = line.along(c.x, c.y);
                let d = (sc - s) * dir - BLOCK_R;
                let side = if line == a { 0 } else { 2 } + if dir > 0.0 { 0 } else { 1 };
                ((a, b), d, side)
            })
            .filter(|(_, d, _)| *d > -0.5 && *d < SIGNAL_SIGHT)
            .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap())
    }

    /// Does this train take this junction? Fixed per train and junction.
    pub(super) fn takes(&self, c: &Chord) -> bool {
        hash_u32(self.id ^ c.id.rotate_left(7)) % 2 == 0
    }

    /// The sets of points this train is coming to, as (chord, which end, will
    /// it take the branch).
    pub fn points_ahead(&self, gen: &Gen, within: f32) -> Vec<(u32, u8, bool)> {
        let Front::Line { line, s, dir } = self.front else { return Vec::new() };
        let h = self.head();
        let r = within as i32 + 60;
        let mut out = Vec::new();
        for c in gen.chords_near(h.x as i32 - r, h.y as i32 - r, h.x as i32 + r, h.y as i32 + r) {
            if c.a == line {
                let ahead = (c.a_s - s) * dir;
                if ahead >= -2.0 && ahead < within && (c.cross_a - c.a_s) * dir > 0.0 {
                    out.push((c.id, 0, self.takes(&c)));
                }
            }
            if c.b == line {
                let ahead = (c.b_s - s) * dir;
                if ahead >= -2.0 && ahead < within && (c.cross_b - c.b_s) * dir > 0.0 {
                    out.push((c.id, 1, self.takes(&c)));
                }
            }
        }
        out
    }

    /// Move the front `dist` along the track, turning onto any junction
    /// this train takes, and lay down trail behind it.
    pub(super) fn advance(&mut self, gen: &Gen, dist: f32, busy: &[RailLine]) {
        let crosses = |a: f32, b: f32, x: f32| a != b && (a - x) * (b - x) <= 0.0;
        let away = |from: f32, cross: f32| if from >= cross { 1.0 } else { -1.0 };
        let mut joined: Option<(RailLine, f32, f32)> = None;
        let next = match &self.front {
            Front::Line { line, s, dir } => {
                let (s0, s1) = (*s, *s + dir * dist);
                let h = self.head();
                let near = gen.chords_near(h.x as i32 - 50, h.y as i32 - 50, h.x as i32 + 50, h.y as i32 + 50);
                let mut turn = None;
                for c in near {
                    // points facing us: heading towards the crossing past them
                    // never onto a line another train is using: it's single track
                    if c.a == *line && crosses(s0, s1, c.a_s) && (c.cross_a - c.a_s) * dir > 0.0 && self.takes(&c) && !busy.contains(&c.b) {
                        let over = (s1 - c.a_s).abs();
                        turn = Some(Front::Chord { chord: c, u: over, forward: true });
                        break;
                    }
                    if c.b == *line && crosses(s0, s1, c.b_s) && (c.cross_b - c.b_s) * dir > 0.0 && self.takes(&c) && !busy.contains(&c.a) {
                        let over = (s1 - c.b_s).abs();
                        let len = c.length();
                        turn = Some(Front::Chord { chord: c, u: len - over, forward: false });
                        break;
                    }
                }
                turn.unwrap_or(Front::Line { line: *line, s: s1, dir: *dir })
            }
            Front::Chord { chord, u, forward } => {
                let u1 = if *forward { u + dist } else { u - dist };
                if *forward && u1 >= chord.length() {
                    let dir = away(chord.b_s, chord.cross_b);
                    joined = Some((chord.b, chord.b_s + dir * (u1 - chord.length()), dir));
                    Front::Line { line: chord.b, s: chord.b_s + dir * (u1 - chord.length()), dir }
                } else if !*forward && u1 <= 0.0 {
                    let dir = away(chord.a_s, chord.cross_a);
                    joined = Some((chord.a, chord.a_s + dir * -u1, dir));
                    Front::Line { line: chord.a, s: chord.a_s + dir * -u1, dir }
                } else {
                    Front::Chord { chord: chord.clone(), u: u1, forward: *forward }
                }
            }
        };
        self.front = next;
        if let Some((line, s, dir)) = joined {
            // onto a new line: its stations are what matter now
            self.stations = if self.freight { Vec::new() } else { stations_for(gen, line, s, dir) };
            self.served = f32::MAX;
            self.approach_horn = false;
        }
        let h = self.head();
        if self.trail.back().map_or(true, |q| q.distance(h) >= TRAIL_STEP) {
            self.trail.push_back(h);
        }
        let keep = ((self.length() + 12.0) / TRAIL_STEP) as usize + 8;
        while self.trail.len() > keep {
            self.trail.pop_front();
        }
    }
}
