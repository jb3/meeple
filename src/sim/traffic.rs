//! Steam trains on the railway lines and horse carts on the roads.

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use glam::{Vec2, Vec3};

use super::people::Sound;
use crate::render::boxes::{rgb, yaw_of, Boxes, Rig, MAT_GLOW, MAT_NIGHT_WINDOW};
use crate::util::*;
use crate::world::gen::{Chord, Ctx, Gen, RailLine, JUNCTION_REACH, STREET};

mod train;
pub use self::train::*;
mod draw;
mod carts;
pub(crate) use self::carts::entry_point;
#[cfg(test)]
use self::carts::*;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod train_tests;

const CAR_SPACING: f32 = 4.7;
const CRUISE: f32 = 9.0;

#[derive(Clone, Copy)]
enum Car {
    Loco([f32; 3]),
    Coach([f32; 3]),
    Freight([f32; 3]),
    Logs,
}

/// Where a train's locomotive is: running along a line, or round a
/// junction chord from one line onto another.
enum Front {
    Line { line: RailLine, s: f32, dir: f32 },
    Chord { chord: Rc<Chord>, u: f32, forward: bool },
}

/// How far apart the points of a train's trail are.
const TRAIL_STEP: f32 = 0.5;
/// Trains ease off to this speed round a junction curve.
const JUNCTION_SPEED: f32 = 5.5;
/// Each crossing (and its junction curve) is one signalled block of this
/// radius: only one train in it at a time. Signals stand just outside, and
/// a driver sees one this far off.
const BLOCK_R: f32 = JUNCTION_REACH + 6.0;
const SIGNAL_SIGHT: f32 = 70.0;
/// Braking used to pull up at a signal.
const SIGNAL_BRAKE: f32 = 1.8;

/// A crossing, keyed by its two lines.
type Block = (RailLine, RailLine);

impl Train {
}

struct Cart {
    pts: Vec<Vec2>,
    seg: usize,
    pos: Vec2,
    yaw: f32,
    y: f32,
    phase: f32,
    ctx: Ctx,
    body: [f32; 3],
    horse: [f32; 3],
    shirt: [f32; 3],
    cargo: u8,
    clop_t: f32,
    /// Seconds left standing at journey's end before turning round for home.
    wait: f32,
    returned: bool,
}

pub struct Traffic {
    pub trains: Vec<Train>,
    /// Which train each crossing is set for, and which signals are off
    /// (showing clear): crossing and the signal's side.
    reserved: HashMap<Block, u32>,
    clear: Vec<(Block, u8)>,
    line_t: HashMap<RailLine, f32>,
    /// How each set of points lies: (chord, which end), true when set for the branch.
    points: HashMap<(u32, u8), bool>,
    next_id: u32,
    carts: Vec<Cart>,
    cart_t: f32,
    /// How good the harvest is: more goods on the roads in a good year.
    plenty: f32,
    /// The age: from the third on, motor cars replace the horse carts.
    pub era: u8,
    rng: Rng,
}

pub struct Emit {
    pub smoke: Vec<(Vec3, f32)>,
}

impl Traffic {
    pub fn new(seed: u32) -> Self {
        Traffic { trains: Vec::new(), reserved: HashMap::new(), clear: Vec::new(), line_t: HashMap::new(), points: HashMap::new(), next_id: seed, carts: Vec::new(), cart_t: 3.0, plenty: 1.0, era: 0, rng: Rng::new(seed as u64 ^ 0x7a1f) }
    }

    /// The signalman's work: each crossing is a block that only one train
    /// may be in. A train nearing one gets the road if it's free (nearest
    /// first), otherwise its signal stays at danger and it must stop there.
    /// Returns, per train, how far it may go before a signal at danger.
    fn signals(&mut self, gen: &Gen) -> Vec<Option<f32>> {
        let bodies: Vec<Vec<Vec2>> = self.trains.iter().map(|t| t.body()).collect();
        let ahead: Vec<Option<(Block, f32, u8)>> = self.trains.iter().map(|t| t.next_block(gen)).collect();
        // which blocks each train is in
        let mut inside: Vec<Vec<Block>> = Vec::new();
        for body in &bodies {
            let (lo, hi) = body.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            let r = BLOCK_R as i32 + 2;
            let blocks = gen
                .crossings_near(lo.x as i32 - r, lo.y as i32 - r, hi.x as i32 + r, hi.y as i32 + r)
                .into_iter()
                .filter(|(_, _, c)| body.iter().any(|p| p.distance(*c) < BLOCK_R))
                .map(|(a, b, _)| (a, b))
                .collect();
            inside.push(blocks);
        }
        // let go of blocks a train has left, or no longer heads for
        let ids: Vec<u32> = self.trains.iter().map(|t| t.id).collect();
        self.reserved.retain(|blk, id| {
            ids.iter().position(|i| i == id).is_some_and(|k| inside[k].contains(blk) || ahead[k].is_some_and(|a| a.0 == *blk))
        });
        // anyone already in a block holds it
        for (k, blocks) in inside.iter().enumerate() {
            for blk in blocks {
                self.reserved.entry(*blk).or_insert(ids[k]);
            }
        }
        // set the road for the nearest train waiting at each block
        let mut order: Vec<usize> = (0..self.trains.len()).filter(|&k| ahead[k].is_some()).collect();
        order.sort_by(|a, b| ahead[*a].unwrap().1.partial_cmp(&ahead[*b].unwrap().1).unwrap());
        self.clear.clear();
        let mut limits = vec![None; self.trains.len()];
        for k in order {
            let (blk, d, side) = ahead[k].unwrap();
            let someone_in = inside.iter().enumerate().any(|(j, b)| j != k && b.contains(&blk));
            let holder = *self.reserved.entry(blk).or_insert(ids[k]);
            if holder == ids[k] && !someone_in {
                self.clear.push((blk, side));
            } else {
                limits[k] = Some(d);
            }
        }
        limits
    }

    /// `people` are positions carts stop for.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        gen: &Gen,
        dt: f32,
        view: Vec2,
        view_r: f32,
        people: &[Vec2],
        sounds: &mut Vec<Sound>,
        emit: &mut Emit,
    ) {
        // spawn trains on visible lines
        let r = view_r as i32 + 80;
        let lines = gen.rails_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r);
        for l in lines {
            let t = self.line_t.entry(l).or_insert_with(|| 2.0 + unit(hash_u32(l.coord as u32)) * 6.0);
            *t -= dt;
            if *t <= 0.0 {
                *t = self.rng.range(14.0, 40.0);
                let dir = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
                let s0 = l.along(view.x, view.y) - dir * (view_r + 140.0);
                // it's single track: another train may follow one going the same
                // way, a good way behind, but never meets one coming the other
                let blocked = self.trains.iter().any(|o| match &o.front {
                    Front::Line { line, s, dir: od } if *line == l => *od != dir || (s - s0).abs() < 260.0,
                    Front::Chord { chord, .. } => chord.a == l || chord.b == l,
                    _ => false,
                });
                if blocked {
                    continue;
                }
                self.next_id = self.next_id.wrapping_add(1);
                let train = self.make_train(gen, l, s0, dir, self.next_id);
                self.trains.push(train);
            }
        }

        // the points are set for the next train to come at them, the way it will go
        for t in &self.trains {
            for (id, end, take) in t.points_ahead(gen, 90.0) {
                self.points.insert((id, end), take);
            }
        }
        let was = self.clear.clone();
        let limits = self.signals(gen);
        // every arm that moves, off or back to danger, clunks
        for &(blk, side) in self.clear.iter().filter(|c| !was.contains(c)) {
            sounds.push(Sound { name: "semaphore", pos: signal_pos(gen, blk, side), vol: 0.9, pitch: 1.1 });
        }
        for &(blk, side) in was.iter().filter(|c| !self.clear.contains(c)) {
            sounds.push(Sound { name: "semaphore", pos: signal_pos(gen, blk, side), vol: 0.9, pitch: 0.9 });
        }
        let lines: Vec<Vec<RailLine>> = (0..self.trains.len())
            .map(|i| {
                let mut v = Vec::new();
                for (j, o) in self.trains.iter().enumerate() {
                    if j != i {
                        match &o.front {
                            Front::Line { line, .. } => v.push(*line),
                            Front::Chord { chord, .. } => v.extend([chord.a, chord.b]),
                        }
                    }
                }
                v
            })
            .collect();
        // where every train is on its line, for each to keep clear of the one ahead
        let runs: Vec<Option<(RailLine, f32, f32, f32)>> = self
            .trains
            .iter()
            .map(|t| match &t.front {
                Front::Line { line, s, dir } => Some((*line, *s, *dir, t.length())),
                Front::Chord { .. } => None,
            })
            .collect();
        for (i, t) in self.trains.iter_mut().enumerate() {
            // next station ahead (only while running along a line)
            let (on_line, ts, tdir) = match t.front {
                Front::Line { s, dir, .. } => (true, s, dir),
                Front::Chord { .. } => (false, 0.0, 1.0),
            };
            let stop_at = t
                .stations
                .iter()
                .map(|st| st + tdir * 7.0)
                .filter(|sp| on_line && (sp - ts) * tdir > -0.5 && (sp - tdir * 7.0 - t.served).abs() > 1.0)
                .min_by(|a, b| ((a - ts) * tdir).partial_cmp(&((b - ts) * tdir)).unwrap());
            let mut target = if on_line { CRUISE } else { JUNCTION_SPEED };
            if t.stop_t > 0.0 {
                t.stop_t -= dt;
                target = 0.0;
                if t.stop_t <= 0.0 {
                    sounds.push(Sound { name: "horn", pos: t.front(gen), vol: 1.0, pitch: 1.0 });
                    sounds.push(Sound { name: "whistle", pos: t.front(gen), vol: 0.6, pitch: 1.0 });
                }
            } else if let Some(sp) = stop_at {
                let d = (sp - ts) * tdir;
                if d < 60.0 {
                    target = (2.0 * 1.1 * d.max(0.0)).sqrt().min(CRUISE).max(0.5);
                    if !t.approach_horn && d < 45.0 {
                        t.approach_horn = true;
                        sounds.push(Sound { name: "horn", pos: t.front(gen), vol: 0.8, pitch: 1.0 });
                    }
                    if d < 0.4 {
                        t.stop_t = 9.0;
                        t.speed = 0.0;
                        t.served = sp - tdir * 7.0;
                        t.approach_horn = false;
                    }
                }
            }
            // a signal at danger ahead: pull up at it
            // trains stuck behind each other's signals at a tangle of crossings
            // give way after a while rather than waiting for ever
            if limits[i].is_some() && t.speed < 0.1 {
                t.wait_t += dt;
            } else if limits[i].is_none() {
                t.wait_t = 0.0;
            }
            let mut limit = if t.wait_t > 30.0 { None } else { limits[i] };
            // a train ahead on the same line, going the same way: keep a gap
            if let Some((line, s, dir, _)) = runs[i] {
                let gap = runs
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .filter_map(|(_, r)| *r)
                    .filter(|(l, sj, d, _)| *l == line && *d == dir && (sj - s) * dir > 0.0)
                    .map(|(_, sj, _, len)| (sj - s) * dir - len - 24.0)
                    .fold(f32::MAX, f32::min);
                if gap < f32::MAX && (gap < 120.0) {
                    limit = Some(limit.map_or(gap, |l| l.min(gap)));
                }
            }
            if let Some(d) = limit {
                target = target.min((2.0 * SIGNAL_BRAKE * (d - 1.0).max(0.0)).sqrt());
                if d < 0.6 {
                    t.speed = 0.0;
                }
            }
            let acc = if target > t.speed { 1.2 } else { 2.5 };
            t.speed += (target - t.speed).clamp(-acc * dt, acc * dt);
            t.advance(gen, t.speed * dt, &lines[i]);

            t.smoke_t -= dt * (0.6 + t.speed * 0.5);
            if t.smoke_t <= 0.0 {
                t.smoke_t = 0.35;
                let yaw = t.car_yaw(0);
                let fwd = Vec3::new(yaw.sin(), 0.0, yaw.cos());
                // out of the top of the chimney (drawn at 1.45 forward, 2.42 up)
                emit.smoke.push((t.front(gen) + fwd * 1.45 + Vec3::Y * 2.5, 0.25 + t.speed * 0.02));
            }
        }
        self.trains.retain(|t| t.head().distance(view) < view_r + 600.0);

        // carts on roads
        self.cart_t -= dt;
        if self.cart_t <= 0.0 && self.carts.len() < 2 + (self.plenty * 2.0) as usize {
            self.cart_t = self.rng.range(15.0, 45.0) / self.plenty.max(0.4);
            self.spawn_cart(gen, view, view_r);
        }
        for c in &mut self.carts {
            if c.seg + 1 >= c.pts.len() {
                // arrived: stand a few moments, then turn round and go back
                // the way it came, rather than blocking the street for good
                if !c.returned {
                    c.wait -= dt;
                    if c.wait <= 0.0 {
                        c.pts.reverse();
                        c.seg = 0;
                        c.returned = true;
                    }
                }
                continue;
            }
            // wait for anyone standing in the road ahead
            let fwd = Vec2::new(c.yaw.sin(), c.yaw.cos());
            let blocked = people.iter().any(|&p| {
                let r = p - c.pos;
                let along = r.dot(fwd);
                along > -0.5 && along < 3.4 && r.perp_dot(fwd).abs() < 1.1
            });
            if blocked {
                continue;
            }
            let target = c.pts[c.seg + 1];
            let d = target - c.pos;
            let dist = d.length();
            let step = 2.4 * dt;
            if dist <= step {
                c.pos = target;
                c.seg += 1;
            } else {
                c.pos += d / dist * step;
                let mut turn = (yaw_of(d) - c.yaw).rem_euclid(std::f32::consts::TAU);
                if turn > std::f32::consts::PI {
                    turn -= std::f32::consts::TAU;
                }
                c.yaw += turn * approach(4.0, dt);
            }
            let gy = gen.walk_y(&c.ctx, c.pos.x, c.pos.y);
            c.y += (gy - c.y) * approach(8.0, dt);
            c.phase += dt * 6.0;
            c.clop_t -= dt;
            if c.clop_t <= 0.0 {
                c.clop_t = 0.42;
                sounds.push(Sound { name: "clop", pos: Vec3::new(c.pos.x, c.y, c.pos.y), vol: 0.35, pitch: 1.0 });
            }
        }
        // a cart that has arrived stays parked until it's out of sight
        self.carts.retain(|c| {
            let d = c.pos.distance(view);
            if c.seg + 1 >= c.pts.len() {
                d < view_r + 12.0
            } else {
                d < view_r + 500.0
            }
        });
    }

    fn make_train(&mut self, gen: &Gen, l: RailLine, s0: f32, dir: f32, id: u32) -> Train {
        let locos = [rgb(40, 90, 60), rgb(150, 40, 40), rgb(40, 50, 90), rgb(40, 40, 45)];
        let coaches = [rgb(130, 40, 40), rgb(50, 100, 70), rgb(210, 190, 140), rgb(60, 80, 140)];
        let mut cars = vec![Car::Loco(*self.rng.pick(&locos))];
        let freight = self.rng.chance(0.35);
        let coach = *self.rng.pick(&coaches);
        for _ in 0..self.rng.int(2, 6) {
            cars.push(if freight {
                if self.rng.chance(0.4) {
                    Car::Logs
                } else {
                    Car::Freight(*self.rng.pick(&[rgb(140, 90, 60), rgb(110, 110, 100), rgb(160, 70, 50)]))
                }
            } else {
                Car::Coach(coach)
            });
        }
        // lay the trail behind the train as if it had been running all along
        let len = cars.len() as f32 * CAR_SPACING + 12.0;
        let trail = (0..=(len / TRAIL_STEP) as usize)
            .rev()
            .map(|i| Vec2::from(l.point(s0 - dir * i as f32 * TRAIL_STEP, 0.0)))
            .collect();
        Train {
            id,
            front: Front::Line { line: l, s: s0, dir },
            speed: CRUISE,
            cars,
            freight,
            stations: if freight { Vec::new() } else { stations_for(gen, l, s0, dir) },
            served: f32::MAX,
            stop_t: 0.0,
            approach_horn: false,
            smoke_t: 0.0,
            wait_t: 0.0,
            trail,
        }
    }

}
