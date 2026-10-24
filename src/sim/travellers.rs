//! People on the road between towns: a peddler with his donkey, a string of
//! pilgrims, a shepherd driving a flock. They walk the roads the carts use,
//! appearing and vanishing out of sight, and rest a while at the far end.

use glam::{Vec2, Vec3};

use super::people::happenings::extra_clad;
use super::people::Sound;
use super::traffic::entry_point;
use crate::render::boxes::{rgb, yaw_of, Boxes, Rig};
use crate::util::*;
use crate::world::gen::{Ctx, Gen, STREET};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Peddler,
    Pilgrims,
    Shepherd,
}

impl Kind {
    pub fn describe(self) -> &'static str {
        match self {
            Kind::Peddler => "A peddler and his donkey on the road",
            Kind::Pilgrims => "Pilgrims on the road",
            Kind::Shepherd => "A shepherd driving his flock",
        }
    }

    fn speed(self) -> f32 {
        match self {
            Kind::Peddler => 1.5,
            Kind::Pilgrims => 1.3,
            Kind::Shepherd => 0.95,
        }
    }
}

struct Party {
    id: u32,
    kind: Kind,
    pts: Vec<Vec2>,
    seg: usize,
    pos: Vec2,
    yaw: f32,
    y: f32,
    phase: f32,
    ctx: Ctx,
    seed: u32,
    /// Seconds left resting at the far end before turning round.
    wait: f32,
    returned: bool,
    bell_t: f32,
    /// How long it's been held up behind someone, and how far it has stepped
    /// aside (to its right) to get round them.
    blocked_t: f32,
    dodge: f32,
}

impl Party {
    fn arrived(&self) -> bool {
        self.seg + 1 >= self.pts.len()
    }

    fn forward(&self) -> Vec2 {
        Vec2::new(self.yaw.sin(), self.yaw.cos())
    }
}

/// How many parties may be on the roads near the view at once.
const MAX_PARTIES: usize = 6;

pub struct Travellers {
    parties: Vec<Party>,
    spawn_t: f32,
    next_id: u32,
    rng: Rng,
}

impl Travellers {
    pub fn new(seed: u32) -> Self {
        Travellers { parties: Vec::new(), spawn_t: 12.0, next_id: 1, rng: Rng::new(seed as u64 ^ 0x7ac3_11e5) }
    }

    /// Parties on the road: (id, where, what), for the camera to take an interest in.
    pub fn about(&self) -> Vec<(u32, Vec2, Kind)> {
        self.parties.iter().filter(|p| !p.arrived() || !p.returned).map(|p| (p.id, p.pos, p.kind)).collect()
    }

    /// Where the members of every party are, for carts to keep clear of.
    pub fn points(&self) -> Vec<Vec2> {
        self.parties.iter().flat_map(members).collect()
    }

    pub fn find(&self, id: u32) -> Option<Vec2> {
        self.parties.iter().find(|p| p.id == id).map(|p| p.pos)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, gen: &Gen, dt: f32, view: Vec2, view_r: f32, hour: f32, people: &[Vec2], sounds: &mut Vec<Sound>) {
        self.spawn_t -= dt;
        if self.spawn_t <= 0.0 {
            self.spawn_t = self.rng.range(14.0, 40.0);
            if (5.5..19.5).contains(&hour) && self.parties.len() < MAX_PARTIES {
                self.spawn(gen, view, view_r);
            }
        }
        // where everyone else's members are, for each party to keep clear of
        let others: Vec<Vec<Vec2>> = (0..self.parties.len()).map(|i| self.parties.iter().enumerate().filter(|(j, _)| *j != i).flat_map(|(_, q)| members(q)).collect()).collect();
        for (idx, p) in self.parties.iter_mut().enumerate() {
            if p.arrived() {
                if !p.returned {
                    p.wait -= dt;
                    if p.wait <= 0.0 {
                        p.pts.reverse();
                        p.seg = 0;
                        p.returned = true;
                    }
                }
                continue;
            }
            // wait for anyone in the way ahead: villagers, carts, and the members
            // of other parties; held up for a while, step aside to get round
            let fwd = p.forward();
            let right = Vec2::new(fwd.y, -fwd.x);
            let me = p.pos + right * p.dodge;
            let (reach, half) = (front_reach(p) + 1.4, lane_half(p.kind) + 0.3);
            let in_way = |q: Vec2| {
                let r = q - me;
                let along = r.dot(fwd);
                along > -0.3 && along < reach && r.dot(right).abs() < half
            };
            let blocked = people.iter().any(|&q| in_way(q)) || others[idx].iter().any(|&q| in_way(q));
            p.blocked_t = if blocked { p.blocked_t + dt } else { (p.blocked_t - dt * 2.0).max(0.0) };
            // once stepped aside, stay there until they've gone right by
            let lane = |q: Vec2| {
                let r = q - p.pos;
                let along = r.dot(fwd);
                along > -2.5 && along < reach + 1.0 && r.dot(right).abs() < half + 0.4
            };
            let still_there = p.dodge > 0.3 && (people.iter().any(|&q| lane(q)) || others[idx].iter().any(|&q| lane(q)));
            let want = if p.blocked_t > 1.5 || still_there { sidestep(p.kind) } else { 0.0 };
            p.dodge += (want - p.dodge) * approach(1.5, dt);
            // stand and wait until they've stepped well clear
            if blocked && p.dodge < sidestep(p.kind) * 0.9 {
                continue;
            }
            let target = p.pts[p.seg + 1];
            let d = target - p.pos;
            let dist = d.length();
            let step = p.kind.speed() * dt;
            if dist <= step {
                p.pos = target;
                p.seg += 1;
            } else {
                p.pos += d / dist * step;
                let mut turn = (yaw_of(d) - p.yaw).rem_euclid(std::f32::consts::TAU);
                if turn > std::f32::consts::PI {
                    turn -= std::f32::consts::TAU;
                }
                p.yaw += turn * approach(4.0, dt);
            }
            let gy = gen.walk_y(&p.ctx, p.pos.x, p.pos.y);
            p.y += (gy - p.y) * approach(8.0, dt);
            p.phase += dt * 4.4;
            p.bell_t -= dt;
            if p.bell_t <= 0.0 {
                p.bell_t = 2.5 + unit(hash_u32(p.seed ^ (p.phase as u32))) * 2.5;
                let name = match p.kind {
                    Kind::Shepherd => "sheepbell",
                    Kind::Peddler => "clop",
                    Kind::Pilgrims => continue,
                };
                sounds.push(Sound { name, pos: Vec3::new(p.pos.x, p.y, p.pos.y), vol: 0.3, pitch: 1.0 });
            }
        }
        // out of sight and done with: gone
        self.parties.retain(|p| {
            let d = p.pos.distance(view);
            if p.arrived() {
                d < view_r + 12.0
            } else {
                d < view_r + 500.0
            }
        });
    }

    fn spawn(&mut self, gen: &Gen, view: Vec2, view_r: f32) {
        let r = view_r as i32 + 60;
        let towns = gen.towns_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r, 0);
        if towns.is_empty() {
            return;
        }
        let kind = *self.rng.pick(&[Kind::Peddler, Kind::Pilgrims, Kind::Shepherd]);
        let t = self.rng.pick(&towns).clone();
        let links = gen.links(&t);
        if links.is_empty() {
            return;
        }
        let u = self.rng.pick(&links).clone();
        let mut pts: Vec<Vec2> = Gen::road_path(&t, &u).iter().map(|&(x, z)| Vec2::new(x as f32 + 0.5, z as f32 + 0.5)).collect();
        if pts.len() < 2 {
            return;
        }
        if self.rng.chance(0.5) {
            pts.reverse();
        }
        // stop at the quay where the road meets water too wide to bridge
        let from_end = self.rng.chance(0.5);
        let Some(dry) = gen.dry_part(&pts, from_end) else { return };
        let mut pts = dry;
        if kind == Kind::Shepherd {
            // a flock is too wide for the streets: it keeps to the open road
            // and turns back where the town begins
            let (mut x0, mut z0, mut x1, mut z1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
            for q in &pts {
                x0 = x0.min(q.x as i32);
                z0 = z0.min(q.y as i32);
                x1 = x1.max(q.x as i32);
                z1 = z1.max(q.y as i32);
            }
            let ctx = gen.ctx(x0 - 8, z0 - 8, x1 + 8, z1 + 8);
            let near_town = |q: Vec2| {
                [(0, 0), (6, 0), (-6, 0), (0, 6), (0, -6)].iter().any(|&(dx, dz)| gen.column(&ctx, q.x as i32 + dx, q.y as i32 + dz).town.is_some())
            };
            let mut best: (usize, usize) = (0, 0);
            let mut i = 0;
            while i < pts.len() {
                if near_town(pts[i]) {
                    i += 1;
                    continue;
                }
                let start = i;
                while i < pts.len() && !near_town(pts[i]) {
                    i += 1;
                }
                if i - start > best.1 - best.0 {
                    best = (start, i);
                }
            }
            if best.1 - best.0 < 2 {
                return;
            }
            pts = pts[best.0..best.1].to_vec();
        } else {
            // start and finish at the plaza's edge
            let edge = (STREET - 1) as f32;
            let n = pts.len();
            for (end, next) in [(0, 1), (n - 1, n - 2)] {
                let d = pts[next] - pts[end];
                pts[end] += d.normalize_or_zero() * d.length().min(edge);
            }
        }
        // keep to the verge on the right, outside the carts' lane (which is also on
        // their right), so oncoming traffic passes on the other side of the road
        let verge = |pts: &[Vec2]| -> Vec<Vec2> {
            (0..pts.len())
                .map(|i| {
                    let d = if i + 1 < pts.len() { pts[i + 1] - pts[i] } else { pts[i] - pts[i - 1] };
                    pts[i] + Vec2::new(-d.y, d.x).normalize_or_zero() * -2.3
                })
                .collect()
        };
        let reach = view_r + 12.0;
        let mut off = verge(&pts);
        let mut entry = entry_point(&off, view, reach);
        if entry.is_none() {
            pts.reverse();
            off = verge(&pts);
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
        self.next_id += 1;
        self.parties.push(Party {
            id: self.next_id,
            kind,
            pos: start,
            yaw: yaw_of(off[seg + 1] - start),
            pts: off,
            seg,
            y,
            phase: 0.0,
            ctx,
            seed: self.rng.next_u32(),
            wait: self.rng.range(15.0, 35.0),
            returned: false,
            bell_t: 1.0,
            blocked_t: 0.0,
            dodge: 0.0,
        });
    }

    pub fn draw(&self, b: &mut Boxes, view: Vec2, view_r: f32, time: f32) {
        for p in &self.parties {
            if p.pos.distance(view) > view_r {
                continue;
            }
            match p.kind {
                Kind::Peddler => peddler(b, p, time),
                Kind::Pilgrims => pilgrims(b, p, time),
                Kind::Shepherd => shepherd(b, p, time),
            }
        }
    }
}

/// How far a party steps aside when it's held up: enough to clear its own width.
fn sidestep(kind: Kind) -> f32 {
    match kind {
        Kind::Shepherd => 2.0,
        _ => 1.3,
    }
}

/// Where each member of a party stands, as (distance behind the leader, to
/// the right of them): the peddler's donkey behind him, pilgrims in a string,
/// the shepherd's flock ahead (negative behind). The leader is first.
fn layout(p: &Party) -> Vec<(f32, f32)> {
    match p.kind {
        Kind::Peddler => vec![(0.0, 0.0), (2.4, 0.0)],
        Kind::Pilgrims => (0..4).map(|k| (k as f32 * 1.35, if k % 2 == 0 { 0.0 } else { 0.5 })).collect(),
        Kind::Shepherd => {
            let mut v = vec![(0.0, 0.0)];
            v.extend((0..8u32).map(|k| {
                let h = hash_u32(p.seed ^ (k * 977));
                (-(2.0 + (k / 3) as f32 * 1.2 + unit(h) * 0.8), (k % 3) as f32 * 1.0 - 1.0 + (unit(h >> 8) - 0.5) * 0.5)
            }));
            v
        }
    }
}

/// How far ahead of its leader a party reaches (the shepherd's flock).
fn front_reach(p: &Party) -> f32 {
    layout(p).iter().map(|(back, _)| -back).fold(0.0, f32::max)
}

/// Half the width a party takes up, plus its dodge.
fn lane_half(kind: Kind) -> f32 {
    match kind {
        Kind::Shepherd => 1.7,
        Kind::Pilgrims => 0.6,
        Kind::Peddler => 0.5,
    }
}

/// Where a party's members are in the world, for others to keep clear of.
fn members(p: &Party) -> Vec<Vec2> {
    layout(p).into_iter().map(|(back, side)| {
        let v = at(p, back, side);
        Vec2::new(v.x, v.z)
    }).collect()
}

/// A point `back` behind a party along its way and `side` to its right (and
/// however far it's stepped aside).
fn at(p: &Party, back: f32, side: f32) -> Vec3 {
    let f = p.forward();
    let q = p.pos - f * back + Vec2::new(f.y, -f.x) * (side + p.dodge);
    Vec3::new(q.x, p.y, q.y)
}

fn moving(p: &Party) -> bool {
    !p.arrived()
}

fn donkey(b: &mut Boxes, pos: Vec3, yaw: f32, phase: f32, moving: bool, load: [f32; 3]) {
    let mut r = Rig::new(b, pos, yaw, 0.8);
    let coat = [0.5, 0.46, 0.42];
    let sw = if moving { phase.sin() * 0.22 } else { 0.0 };
    r.part(Vec3::new(0.0, 0.95, 0.0), Vec3::new(0.24, 0.26, 0.58), coat);
    r.part(Vec3::new(0.0, 1.3, 0.55), Vec3::new(0.12, 0.26, 0.14), coat);
    r.part(Vec3::new(0.0, 1.45, 0.82), Vec3::new(0.11, 0.12, 0.2), coat);
    for x in [-0.07, 0.07] {
        r.part(Vec3::new(x, 1.72, 0.7), Vec3::new(0.035, 0.12, 0.03), coat);
    }
    r.part(Vec3::new(0.0, 1.0, -0.62), Vec3::new(0.04, 0.2, 0.04), [0.25, 0.2, 0.18]);
    for (x, z, s) in [(0.13, 0.38, 1.0), (-0.13, 0.38, -1.0), (0.13, -0.38, -1.0), (-0.13, -0.38, 1.0)] {
        r.part(Vec3::new(x, 0.34, z + sw * s), Vec3::new(0.055, 0.34, 0.055), coat);
    }
    // panniers either side
    for x in [-0.34, 0.34] {
        r.part(Vec3::new(x, 1.02, -0.05), Vec3::new(0.12, 0.2, 0.3), load);
    }
    r.part(Vec3::new(0.0, 1.28, -0.05), Vec3::new(0.3, 0.05, 0.32), [0.55, 0.3, 0.2]);
}

fn peddler(b: &mut Boxes, p: &Party, time: f32) {
    let m = moving(p);
    let me = at(p, 0.0, 0.0);
    extra_clad(b, me, p.yaw, p.phase, p.seed, if m { 1 } else { 0 }, time, rgb(120, 70, 50), rgb(60, 50, 45), true, 1.0);
    // his pack: a big bundle with a bedroll and a pot dangling off it
    let mut r = Rig::new(b, me, p.yaw, 1.0);
    let sway = if m { (p.phase * 2.0).sin() * 0.02 } else { 0.0 };
    r.part(Vec3::new(0.0, 1.05 + sway, -0.3), Vec3::new(0.26, 0.32, 0.16), rgb(150, 110, 70));
    r.part(Vec3::new(0.0, 1.43, -0.3), Vec3::new(0.3, 0.07, 0.1), rgb(90, 110, 140));
    r.part(Vec3::new(0.2, 0.7, -0.42), Vec3::new(0.07, 0.07, 0.07), rgb(70, 70, 75));
    // the donkey follows a few steps behind
    donkey(b, at(p, 2.4, 0.0), p.yaw, p.phase * 0.9, m, rgb(180, 140, 80));
}

fn pilgrims(b: &mut Boxes, p: &Party, time: f32) {
    let m = moving(p);
    let cloaks = [rgb(110, 95, 80), rgb(80, 85, 95), rgb(120, 100, 70), rgb(95, 80, 90)];
    for k in 0..4usize {
        let pos = at(p, k as f32 * 1.35, if k % 2 == 0 { 0.0 } else { 0.5 });
        let ph = p.phase + k as f32 * 0.9;
        extra_clad(b, pos, p.yaw, ph, p.seed ^ k as u32, if m { 1 } else { 0 }, time, cloaks[k], rgb(60, 55, 50), k != 2, 1.0);
        // a long staff in the hand
        let mut r = Rig::new(b, pos, p.yaw, 1.0);
        r.part(Vec3::new(0.36, 0.95, 0.12), Vec3::new(0.025, 0.95, 0.025), rgb(110, 80, 50));
    }
}

fn sheep(b: &mut Boxes, pos: Vec3, yaw: f32, phase: f32, moving: bool) {
    let mut r = Rig::new(b, pos, yaw, 0.85);
    let wool = [0.88, 0.86, 0.8];
    let sw = if moving { phase.sin() * 0.18 } else { 0.0 };
    r.part(Vec3::new(0.0, 0.55, 0.0), Vec3::new(0.26, 0.24, 0.36), wool);
    r.part(Vec3::new(0.0, 0.7, 0.46), Vec3::new(0.12, 0.13, 0.13), [0.25, 0.2, 0.2]);
    for (x, z, s) in [(0.12, 0.22, 1.0), (-0.12, 0.22, -1.0), (0.12, -0.22, -1.0), (-0.12, -0.22, 1.0)] {
        r.part(Vec3::new(x, 0.18, z + sw * s), Vec3::new(0.04, 0.18, 0.04), [0.25, 0.2, 0.2]);
    }
}

fn shepherd(b: &mut Boxes, p: &Party, time: f32) {
    let m = moving(p);
    let me = at(p, 0.0, 0.0);
    extra_clad(b, me, p.yaw, p.phase, p.seed, if m { 1 } else { 0 }, time, rgb(100, 120, 80), rgb(70, 60, 45), true, 1.0);
    // his crook
    let mut r = Rig::new(b, me, p.yaw, 1.0);
    r.part(Vec3::new(0.36, 1.0, 0.18), Vec3::new(0.025, 1.0, 0.025), rgb(120, 85, 50));
    r.part(Vec3::new(0.36, 2.02, 0.26), Vec3::new(0.025, 0.025, 0.1), rgb(120, 85, 50));
    // the flock ahead of him, jostling along
    for k in 0..8u32 {
        let h = hash_u32(p.seed ^ (k * 977));
        let along = 2.0 + (k / 3) as f32 * 1.2 + unit(h) * 0.8;
        let side = (k % 3) as f32 * 1.0 - 1.0 + (unit(h >> 8) - 0.5) * 0.5;
        let pos = at(p, -along, side + (p.phase * 0.5 + k as f32).sin() * 0.12);
        sheep(b, pos, p.yaw + (unit(h >> 16) - 0.5) * 0.4, p.phase * 1.3 + k as f32, m);
    }
    // the dog working the flank
    let d = (p.phase * 0.25).sin();
    let dog = at(p, -2.5 - d.abs() * 1.5, d * 2.2);
    let mut r = Rig::new(b, dog, p.yaw + d * 0.4, 1.0);
    let sw = if m { (p.phase * 2.0).sin() * 0.14 } else { 0.0 };
    r.part(Vec3::new(0.0, 0.34, 0.0), Vec3::new(0.1, 0.1, 0.24), [0.15, 0.13, 0.12]);
    r.part(Vec3::new(0.0, 0.44, 0.28), Vec3::new(0.08, 0.08, 0.09), [0.15, 0.13, 0.12]);
    for (x, z, s) in [(0.06, 0.16, 1.0), (-0.06, 0.16, -1.0), (0.06, -0.16, -1.0), (-0.06, -0.16, 1.0)] {
        r.part(Vec3::new(x, 0.14, z + sw * s), Vec3::new(0.025, 0.14, 0.025), [0.15, 0.13, 0.12]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn party(gen: &Gen, id: u32, kind: Kind, pts: Vec<Vec2>, seed: u32) -> Party {
        let ctx = gen.ctx(-40, -40, 120, 40);
        Party { id, kind, yaw: yaw_of(pts[1] - pts[0]), pos: pts[0], pts, seg: 0, y: 0.0, phase: 0.0, ctx, seed, wait: 1.0, returned: true, bell_t: 9.0, blocked_t: 0.0, dodge: 0.0 }
    }

    /// Two parties walking straight at each other along the very same line,
    /// of every sort, get round one another: no two of their members ever
    /// overlap, and both reach the far end.
    #[test]
    fn parties_pass_each_other_without_passing_through() {
        let gen = Gen::new(1234);
        let kinds = [Kind::Peddler, Kind::Pilgrims, Kind::Shepherd];
        let mut worst = f32::MAX;
        for (i, ka) in kinds.iter().enumerate() {
            for (j, kb) in kinds.iter().enumerate() {
                let mut t = Travellers::new(5);
                let line = |a: f32, b: f32| vec![Vec2::new(a, 0.0), Vec2::new((a + b) * 0.5, 0.0), Vec2::new(b, 0.0)];
                t.parties.push(party(&gen, 1, *ka, line(0.0, 80.0), 11 + i as u32));
                t.parties.push(party(&gen, 2, *kb, line(80.0, 0.0), 77 + j as u32));
                let mut sounds = Vec::new();
                for _ in 0..(300.0 / 0.05) as i32 {
                    t.update(&gen, 0.05, Vec2::new(40.0, 0.0), 500.0, 12.0, &[], &mut sounds);
                    let (a, b) = (members(&t.parties[0]), members(&t.parties[1]));
                    for p in &a {
                        for q in &b {
                            worst = worst.min(p.distance(*q));
                            assert!(p.distance(*q) > 0.4, "{ka:?} and {kb:?} walked through each other at {p}: A {:?} blocked {} dodge {} seg {}; B {:?} blocked {} dodge {}", t.parties[0].pos, t.parties[0].blocked_t, t.parties[0].dodge, t.parties[0].seg, t.parties[1].pos, t.parties[1].blocked_t, t.parties[1].dodge);
                        }
                    }
                    if t.parties.iter().all(|p| p.arrived()) {
                        break;
                    }
                }
                assert!(t.parties.iter().all(|p| p.arrived()), "{ka:?} and {kb:?} never got past each other");
            }
        }
        println!("closest approach between members: {worst}");
    }

    /// A cart's horse in the road holds a party up rather than being walked through.
    #[test]
    fn a_party_waits_for_a_cart_in_the_road() {
        let gen = Gen::new(1234);
        let mut t = Travellers::new(5);
        t.parties.push(party(&gen, 1, Kind::Pilgrims, vec![Vec2::new(0.0, 0.0), Vec2::new(20.0, 0.0), Vec2::new(40.0, 0.0)], 3));
        let cart = [Vec2::new(10.0, 0.0), Vec2::new(11.3, 0.0), Vec2::new(12.6, 0.0)];
        let mut sounds = Vec::new();
        let mut start_blocked = 0.0;
        for k in 0..200 {
            t.update(&gen, 0.05, Vec2::new(20.0, 0.0), 500.0, 12.0, &cart, &mut sounds);
            assert!(members(&t.parties[0]).iter().all(|p| cart.iter().all(|c| p.distance(*c) > 0.4)), "walked into the cart");
            if k == 199 {
                start_blocked = t.parties[0].blocked_t;
            }
        }
        assert!(start_blocked > 0.0 || t.parties[0].dodge > 0.1, "never noticed the cart");
    }
}
