//! Life on station platforms: passengers wait with their luggage, board
//! when a train comes in, others get off and head away, and the guard
//! waves the green flag before it leaves.

use std::collections::HashMap;
use std::rc::Rc;

use glam::{Vec2, Vec3};

use super::people::Sound;
use crate::render::boxes::{yaw_of, Boxes, Rig};
use crate::util::*;
use crate::world::gen::{Gen, RailLine, Town};

/// The platform runs from s-4 to s+3 along the line, between these
/// distances from the track centre (the canopy posts, bench and lamps
/// stand along the back, beyond BACK).
const EDGE: f32 = 2.4;
const BACK: f32 = 2.95;
const ENDS: (f32, f32) = (-3.6, 3.2);
const MAX_WAITING: usize = 6;
const WALK: f32 = 1.3;
const FADE_RATE: f32 = 1.7;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Doing {
    /// Walking onto the platform to a spot to wait.
    Arriving,
    Waiting,
    /// Off to a carriage door, to vanish aboard.
    Boarding,
    /// Just off the train, heading for the way out.
    Leaving,
}

struct Passenger {
    /// Position as (distance along the line, distance from its centre).
    s: f32,
    p: f32,
    to: (f32, f32),
    doing: Doing,
    yaw: f32,
    phase: f32,
    fade: f32,
    gone: bool,
    coat: [f32; 3],
    legs: [f32; 3],
    skin: [f32; 3],
    hat: Option<[f32; 3]>,
    /// A suitcase, a carpet bag, or nothing.
    luggage: u8,
}

pub struct Station {
    line: RailLine,
    s: f32,
    y: f32,
    people: Vec<Passenger>,
    rng: Rng,
    spawn_t: f32,
    /// The train being served and seconds left before it goes, if any.
    train: Option<(f32, Vec<f32>)>,
    /// Cells taken by the canopy posts, the bench and the lamps.
    blocked: Vec<(i32, i32)>,
    /// How many have come and gone today, for the newsletter.
    pub boarded: u32,
    pub alighted: u32,
}

impl Station {
    pub fn new(gen: &Gen, town: Rc<Town>) -> Option<Station> {
        let (line, s) = (town.rail?, town.station_s()?);
        let y = gen.rail_bed(line, s.floor()) as f32 + 2.0;
        let mut st = Station {
            line,
            s: s.floor(),
            y,
            people: Vec::new(),
            rng: Rng::new(town.id as u64 ^ 0x57a7),
            spawn_t: 0.0,
            train: None,
            blocked: Vec::new(),
            boarded: 0,
            alighted: 0,
        };
        // the same cells build_station puts posts, bench and lamps in
        let s0 = st.s as i32;
        st.blocked = [s0 - 2, s0 + 1, s0, s0 - 4, s0 + 3]
            .iter()
            .map(|&a| {
                let (x, z) = line.point(a as f32 + 0.5, 3.5);
                (x.floor() as i32, z.floor() as i32)
            })
            .collect();
        // a few already waiting
        for _ in 0..st.rng.int(1, 4) {
            let mut p = st.passenger();
            p.doing = Doing::Waiting;
            (p.s, p.p) = p.to;
            p.fade = 1.0;
            st.people.push(p);
        }
        Some(st)
    }

    fn at(&self, s: f32, p: f32) -> Vec2 {
        Vec2::from(self.line.point(self.s + s, p))
    }

    fn blocked(&self, s: f32, p: f32) -> bool {
        let at = self.at(s, p);
        self.blocked.contains(&(at.x.floor() as i32, at.y.floor() as i32))
    }

    /// Somewhere clear to stand along the platform.
    fn spot(&mut self) -> (f32, f32) {
        for _ in 0..12 {
            let to = (self.rng.range(ENDS.0 + 0.6, ENDS.1 - 0.6), self.rng.range(EDGE + 0.3, BACK));
            if !self.blocked(to.0, to.1) {
                return to;
            }
        }
        (0.0, EDGE + 0.2)
    }

    fn passenger(&mut self) -> Passenger {
        let to = self.spot();
        let start = self.rng.range(EDGE + 0.3, BACK);
        let r = &mut self.rng;
        let coats = [[0.25, 0.3, 0.45], [0.45, 0.28, 0.2], [0.3, 0.38, 0.3], [0.5, 0.48, 0.44], [0.6, 0.22, 0.25], [0.2, 0.2, 0.22]];
        let skins = [[0.96, 0.8, 0.66], [0.85, 0.65, 0.5], [0.62, 0.45, 0.32], [0.45, 0.3, 0.22]];
        let end = if r.chance(0.5) { ENDS.0 } else { ENDS.1 };
        Passenger {
            s: end,
            p: start,
            to,
            doing: Doing::Arriving,
            yaw: 0.0,
            phase: r.range(0.0, 6.0),
            fade: 0.0,
            gone: false,
            coat: *r.pick(&coats),
            legs: [0.18, 0.17, 0.2],
            skin: *r.pick(&skins),
            hat: r.chance(0.5).then(|| [0.15, 0.13, 0.12]),
            luggage: r.int(0, 2) as u8,
        }
    }

    /// `train`: a train standing here (seconds until it leaves, and where
    /// its carriages are along the line).
    pub fn update(&mut self, dt: f32, train: Option<(f32, Vec<f32>)>, hour: f32, sounds: &mut Vec<Sound>) {
        // a train has just come in: some get off, and the waiting board
        if let (None, Some((_, cars))) = (&self.train, &train) {
            let doors: Vec<f32> = cars.iter().map(|c| c - self.s).filter(|d| (ENDS.0..ENDS.1).contains(d)).collect();
            if !doors.is_empty() {
                let off = self.rng.int(0, 3);
                for _ in 0..off {
                    let mut p = self.passenger();
                    p.s = *self.rng.pick(&doors) + self.rng.range(-0.6, 0.6);
                    p.p = EDGE;
                    p.fade = 0.0;
                    p.doing = Doing::Leaving;
                    p.to = (if p.s < 0.0 { ENDS.0 } else { ENDS.1 }, EDGE + 0.2);
                    self.people.push(p);
                    self.alighted += 1;
                }
                for p in self.people.iter_mut().filter(|p| matches!(p.doing, Doing::Waiting | Doing::Arriving)) {
                    let d = doors.iter().copied().min_by(|a, b| (a - p.s).abs().partial_cmp(&(b - p.s).abs()).unwrap()).unwrap();
                    p.doing = Doing::Boarding;
                    p.to = (d, EDGE - 0.6);
                    self.boarded += 1;
                }
                let at = self.at(0.0, EDGE);
                sounds.push(Sound { name: "door", pos: Vec3::new(at.x, self.y, at.y), vol: 0.5, pitch: 0.9 });
            }
        }
        self.train = train;

        // new passengers turn up through the day
        let busy = if (6.5..21.0).contains(&hour) { 1.0 } else { 0.15 };
        self.spawn_t -= dt * busy;
        let waiting = self.people.iter().filter(|p| matches!(p.doing, Doing::Arriving | Doing::Waiting)).count();
        if self.spawn_t <= 0.0 && self.train.is_none() {
            self.spawn_t = self.rng.range(8.0, 22.0);
            if waiting < MAX_WAITING {
                let p = self.passenger();
                self.people.push(p);
            }
        }

        let line = self.line;
        let s0 = self.s;
        let blocked = |s: f32, p: f32| {
            let (x, z) = line.point(s0 + s, p);
            self.blocked.contains(&(x.floor() as i32, z.floor() as i32))
        };
        for p in &mut self.people {
            let target = Vec2::new(p.to.0, p.to.1);
            let here = Vec2::new(p.s, p.p);
            let d = target - here;
            let walking = matches!(p.doing, Doing::Arriving | Doing::Boarding | Doing::Leaving) && d.length() > 0.05;
            if walking {
                let mut step = d.clamp_length_max(WALK * dt);
                // round a post or lamp: slide along it one way or the other,
                // else out towards the edge (stepping out and straight back in
                // made people shudder against the post)
                if blocked(p.s + step.x, p.p + step.y) {
                    let out = Vec2::new(0.0, -WALK * dt);
                    let full = step.length();
                    step = [Vec2::new(step.x, 0.0), Vec2::new(0.0, step.y)]
                        .into_iter()
                        .find(|c| c.length() > full * 0.5 && !blocked(p.s + c.x, p.p + c.y))
                        .unwrap_or(out);
                }
                p.s += step.x;
                p.p += step.y;
                p.phase += dt * WALK * 4.5;
                let (tx, tz) = line.tangent(s0 + p.s);
                let fwd = Vec2::new(tx, tz);
                let side = Vec2::new(-fwd.y, fwd.x);
                p.yaw = yaw_of(fwd * step.x + side * step.y);
            }
            let fading_out = match p.doing {
                Doing::Arriving if !walking => {
                    p.doing = Doing::Waiting;
                    false
                }
                Doing::Waiting => {
                    // face the track
                    let (tx, tz) = line.tangent(s0 + p.s);
                    p.yaw = yaw_of(Vec2::new(tz, -tx));
                    false
                }
                Doing::Boarding | Doing::Leaving => !walking,
                _ => false,
            };
            if fading_out {
                p.fade -= dt * FADE_RATE;
                p.gone = p.fade <= 0.0;
            } else {
                p.fade = (p.fade + dt * FADE_RATE).min(1.0);
            }
        }
        self.people.retain(|p| !p.gone);
    }

    pub fn draw(&self, b: &mut Boxes, view: Vec2, view_r: f32, time: f32) {
        if self.at(0.0, 3.0).distance(view) > view_r + 8.0 {
            return;
        }
        for p in &self.people {
            let at = self.at(p.s, p.p);
            draw_passenger(b, p, Vec3::new(at.x, self.y, at.y));
        }
        // the guard, by the rear of the train, with a green flag up as it's
        // about to go
        if let Some((left, cars)) = &self.train {
            // carriages run front to back: the last one alongside
            let rear = cars.iter().map(|c| c - self.s).filter(|d| (ENDS.0..ENDS.1).contains(d)).last();
            let gs = rear.unwrap_or(ENDS.1 - 0.5).clamp(ENDS.0 + 0.4, ENDS.1 - 0.4);
            let at = self.at(gs, EDGE + 0.3);
            let (tx, tz) = self.line.tangent(self.s + gs);
            let yaw = yaw_of(Vec2::new(tx, tz));
            let flag = *left < 2.6;
            draw_guard(b, Vec3::new(at.x, self.y, at.y), yaw, flag, time);
        }
    }
}

fn draw_passenger(b: &mut Boxes, p: &Passenger, pos: Vec3) {
    let mut r = Rig::new(b, pos, p.yaw, 1.18);
    r.alpha = p.fade;
    let walking = matches!(p.doing, Doing::Arriving | Doing::Boarding | Doing::Leaving);
    let sw = if walking { p.phase.sin() } else { 0.0 };
    let bob = if walking { (p.phase * 2.0).sin().abs() * 0.035 } else { 0.0 };
    r.part(Vec3::new(0.11, 0.28, sw * 0.13), Vec3::new(0.09, 0.28, 0.09), p.legs);
    r.part(Vec3::new(-0.11, 0.28, -sw * 0.13), Vec3::new(0.09, 0.28, 0.09), p.legs);
    let ty = 0.84 + bob;
    // a long travelling coat
    r.part(Vec3::new(0.0, ty - 0.06, 0.0), Vec3::new(0.23, 0.32, 0.15), p.coat);
    let arm = [p.coat[0] * 0.85, p.coat[1] * 0.85, p.coat[2] * 0.85];
    r.part(Vec3::new(-0.3, ty - 0.02, -sw * 0.14), Vec3::new(0.07, 0.24, 0.07), arm);
    r.part(Vec3::new(0.3, ty - 0.08, 0.0), Vec3::new(0.07, 0.22, 0.07), arm);
    match p.luggage {
        1 => r.part(Vec3::new(0.36, 0.3, 0.0), Vec3::new(0.07, 0.17, 0.24), [0.5, 0.32, 0.18]),
        2 => r.part(Vec3::new(0.36, 0.34, 0.0), Vec3::new(0.08, 0.13, 0.16), [0.55, 0.2, 0.22]),
        _ => {}
    }
    let hy = 1.3 + bob;
    r.part(Vec3::new(0.0, hy, 0.0), Vec3::splat(0.17), p.skin);
    if let Some(h) = p.hat {
        r.part(Vec3::new(0.0, hy + 0.19, 0.0), Vec3::new(0.24, 0.02, 0.24), h);
        r.part(Vec3::new(0.0, hy + 0.27, 0.0), Vec3::new(0.15, 0.08, 0.15), h);
    }
}

fn draw_guard(b: &mut Boxes, pos: Vec3, yaw: f32, flag: bool, time: f32) {
    let navy = [0.12, 0.14, 0.26];
    let mut r = Rig::new(b, pos, yaw, 1.18);
    for x in [-0.11, 0.11] {
        r.part(Vec3::new(x, 0.28, 0.0), Vec3::new(0.09, 0.28, 0.09), navy);
    }
    r.part(Vec3::new(0.0, 0.82, 0.0), Vec3::new(0.22, 0.26, 0.14), navy);
    r.part(Vec3::new(-0.3, 0.8, 0.0), Vec3::new(0.07, 0.24, 0.07), navy);
    if flag {
        // arm up, flag waving
        let wave = (time * 7.0).sin() * 0.06;
        r.part(Vec3::new(0.3, 1.28, 0.0), Vec3::new(0.07, 0.24, 0.07), navy);
        r.part(Vec3::new(0.3, 1.62, 0.0), Vec3::new(0.02, 0.2, 0.02), [0.5, 0.4, 0.3]);
        r.part(Vec3::new(0.3, 1.72, 0.17 + wave), Vec3::new(0.015, 0.1, 0.15), [0.15, 0.7, 0.25]);
    } else {
        r.part(Vec3::new(0.3, 0.8, 0.0), Vec3::new(0.07, 0.24, 0.07), navy);
    }
    r.part(Vec3::new(0.0, 1.27, 0.0), Vec3::splat(0.17), [0.94, 0.78, 0.64]);
    // peaked cap
    r.part(Vec3::new(0.0, 1.47, 0.0), Vec3::new(0.19, 0.05, 0.19), navy);
    r.part(Vec3::new(0.0, 1.43, 0.2), Vec3::new(0.14, 0.02, 0.07), [0.08, 0.08, 0.1]);
}

/// Stations near the camera, keyed by town.
#[derive(Default)]
pub struct Stations {
    pub by_town: HashMap<u32, Station>,
}

impl Stations {
    pub fn sync(&mut self, gen: &Gen, towns: &[Rc<Town>]) {
        for t in towns {
            if !self.by_town.contains_key(&t.id) {
                if let Some(st) = Station::new(gen, t.clone()) {
                    self.by_town.insert(t.id, st);
                }
            }
        }
        self.by_town.retain(|id, _| towns.iter().any(|t| t.id == *id));
    }

    /// `standing`: trains at platforms, as (line, station, seconds left, carriages).
    pub fn update(&mut self, dt: f32, standing: &[(RailLine, f32, f32, Vec<f32>)], hour: f32, sounds: &mut Vec<Sound>) {
        for st in self.by_town.values_mut() {
            let train = standing
                .iter()
                .find(|(l, s, _, _)| *l == st.line && (s.floor() - st.s).abs() < 1.5)
                .map(|(_, _, left, cars)| (*left, cars.clone()));
            st.update(dt, train, hour, sounds);
        }
    }

    pub fn draw(&self, b: &mut Boxes, view: Vec2, view_r: f32, time: f32) {
        for st in self.by_town.values() {
            st.draw(b, view, view_r, time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::blocks::PLATFORM;
    use crate::world::chunk::{generate, CS};

    /// Passengers stand on the platform, board an arriving train and are
    /// gone when it leaves; others get off and walk away.
    #[test]
    fn passengers_wait_board_and_alight() {
        let gen = Gen::new(1234);
        let town = (-6..=6)
            .flat_map(|rx| (-6..=6).map(move |rz| (rx, rz)))
            .filter_map(|(rx, rz)| gen.town(rx, rz))
            .find(|t| t.station_s().is_some())
            .expect("no station towns");
        let mut st = Station::new(&gen, town).unwrap();
        let mut sounds = Vec::new();
        let dt = 1.0 / 30.0;
        for _ in 0..(60.0 / dt) as i32 {
            st.update(dt, None, 10.0, &mut sounds);
        }
        let waiting = st.people.iter().filter(|p| p.doing == Doing::Waiting).count();
        assert!(waiting >= 2, "only {waiting} waiting after a minute");
        // everyone is standing on the platform itself
        for p in &st.people {
            let at = st.at(p.s, p.p);
            let (x, z) = (at.x.floor() as i32, at.y.floor() as i32);
            let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
            assert_eq!(ch.get_w(x, st.y as i32 - 1, z), PLATFORM, "passenger off the platform at {at}");
            assert_eq!(ch.get_w(x, st.y as i32, z), crate::world::blocks::AIR, "passenger inside something at {at}");
        }
        let cars: Vec<f32> = (0..3).map(|k| st.s + 2.5 - k as f32 * 4.7).collect();
        for train in 0..4 {
            let mut left = 9.0;
            while left > 0.0 {
                st.update(dt, Some((left, cars.clone())), 10.0, &mut sounds);
                left -= dt;
            }
            if train == 0 {
                assert!(st.boarded as usize >= waiting, "{} boarded of {waiting}", st.boarded);
            }
            assert!(!st.people.iter().any(|p| p.doing == Doing::Boarding), "someone missed the train");
            for _ in 0..(30.0 / dt) as i32 {
                st.update(dt, None, 10.0, &mut sounds);
            }
            assert!(!st.people.iter().any(|p| p.doing == Doing::Leaving), "arrivals never left the platform");
        }
        assert!(st.alighted > 0, "nobody ever got off");
        println!("{} boarded, {} got off", st.boarded, st.alighted);
    }
}
