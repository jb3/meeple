//! What's overhead and what's new in the age: airships and biplanes,
//! eclipses of the sun, and the telegraph poles that arrive along the
//! railways as the years go by.

use std::f32::consts::{PI, TAU};

use glam::{Vec2, Vec3};

use super::people::Sound;
use crate::render::boxes::{Boxes, Rig, MAT_GLOW, MAT_LIT};
use crate::util::*;
use crate::world::gen::{Gen, SEA};

/// How far out from the view aircraft come in from.
fn edge_of(view_r: f32) -> f32 {
    view_r + 70.0
}

/// How often a balloon's burner roars, in seconds, and for how long.
const BURN_EVERY: f32 = 9.0;
const BURN_FOR: f32 = 1.3;

/// Days an age lasts before the next one dawns.
const ERA_DAYS: i64 = 36;

pub const ERA_NAMES: [&str; 3] = ["the age of horse and lantern", "the age of steam and telegraph", "the age of motor and electric light"];

/// Which age the world is in: a different one for each world to begin with,
/// moving on as the days pass.
pub fn era(seed: u32, first_day: i64, day: i64) -> u8 {
    let base = match hash_u32(seed ^ 0xe7a) % 5 {
        0 | 1 => 0,
        2 | 3 => 1,
        _ => 2,
    };
    (base + (day - first_day).max(0) / ERA_DAYS).min(2) as u8
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Airship,
    Plane,
    /// Gaily striped, drifting in groups on calm mornings and evenings.
    Balloon,
}

struct Craft {
    kind: Kind,
    pos: Vec2,
    vel: Vec2,
    alt: f32,
    age: f32,
    seed: u32,
}

pub struct Skies {
    rng: Rng,
    craft: Vec<Craft>,
    air_t: f32,
    plane_t: f32,
    hum_t: f32,
    balloon_t: f32,
    /// Seconds running, which the balloons' burners keep time by.
    t: f32,
}

impl Skies {
    pub fn new(seed: u32) -> Self {
        Skies { rng: Rng::new(seed as u64 ^ 0x5c1e5), craft: Vec::new(), air_t: 20.0, plane_t: 40.0, hum_t: 0.0, balloon_t: 25.0, t: 0.0 }
    }

    /// How much of the sun is covered, 0..1, in the rare eclipse that happens
    /// at noon every month or so.
    pub fn eclipse(seed: u32, days: f64) -> f32 {
        let day = days.floor() as i64;
        let on = (hash_u32(seed ^ (day.div_euclid(24) as u32).wrapping_mul(0x9e37)) % 3 == 0)
            && day.rem_euclid(24) == (hash_u32(seed ^ 0xec1) % 24) as i64;
        if !on {
            return 0.0;
        }
        let hour = (days.fract() * 24.0) as f32;
        let x = (hour - 12.4).abs();
        (1.0 - smoothstep(0.1, 0.9, x)).clamp(0.0, 1.0)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, dt: f32, view: Vec2, view_r: f32, era: u8, calm: bool, hour: f32, winter: bool, sounds: &mut Vec<Sound>) {
        let was = self.t;
        self.t += dt;
        self.balloon_t -= dt;
        // balloons go up in a bunch on still mornings and evenings
        let air_hours = (6.0..10.0).contains(&hour) || (16.5..19.0).contains(&hour);
        if calm && !winter && air_hours && self.balloon_t <= 0.0 && !self.craft.iter().any(|c| c.kind == Kind::Balloon) {
            self.balloon_t = self.rng.range(240.0, 600.0);
            let a = self.rng.range(0.0, TAU);
            let across = Vec2::new(a.cos(), a.sin());
            let side = Vec2::new(-across.y, across.x);
            let drift = -across * self.rng.range(1.1, 1.8);
            for k in 0..self.rng.int(2, 4) {
                let off = side * (k as f32 * self.rng.range(13.0, 20.0) - 14.0) + across * (k as f32 * self.rng.range(-6.0, 6.0));
                self.craft.push(Craft { kind: Kind::Balloon, pos: view + across * (edge_of(view_r) + 10.0) + off, vel: drift, alt: self.rng.range(16.0, 38.0), age: 0.0, seed: self.rng.next_u32() });
            }
        }
        // each burner roars for a moment every nine seconds or so
        for c in self.craft.iter().filter(|c| c.kind == Kind::Balloon) {
            let off = (c.seed % 90) as f32 * 0.1;
            if (was + off) % BURN_EVERY > (self.t + off) % BURN_EVERY && c.pos.distance(view) < view_r + 30.0 {
                sounds.push(Sound { name: "burner", pos: Vec3::new(c.pos.x, c.alt, c.pos.y), vol: 0.45, pitch: 1.0 });
            }
        }
        self.air_t -= dt;
        self.plane_t -= dt;
        let edge = edge_of(view_r);
        if era >= 1 && calm && self.air_t <= 0.0 && !self.craft.iter().any(|c| c.kind == Kind::Airship) {
            self.air_t = self.rng.range(90.0, 220.0);
            let a = self.rng.range(0.0, TAU);
            let across = Vec2::new(a.cos(), a.sin());
            let side = Vec2::new(-across.y, across.x) * self.rng.range(-0.5, 0.5) * view_r;
            let pos = view + across * edge + side;
            self.craft.push(Craft { kind: Kind::Airship, pos, vel: -across * self.rng.range(3.0, 4.5), alt: self.rng.range(30.0, 40.0), age: 0.0, seed: self.rng.next_u32() });
        }
        if era >= 2 && calm && self.plane_t <= 0.0 && !self.craft.iter().any(|c| c.kind == Kind::Plane) {
            self.plane_t = self.rng.range(120.0, 300.0);
            let a = self.rng.range(0.0, TAU);
            let across = Vec2::new(a.cos(), a.sin());
            let side = Vec2::new(-across.y, across.x) * self.rng.range(-0.6, 0.6) * view_r;
            let pos = view + across * (edge + 30.0) + side;
            self.craft.push(Craft { kind: Kind::Plane, pos, vel: -across * self.rng.range(11.0, 15.0), alt: self.rng.range(22.0, 34.0), age: 0.0, seed: self.rng.next_u32() });
        }
        self.hum_t -= dt;
        let hum = self.hum_t <= 0.0;
        if hum {
            self.hum_t = 0.45;
        }
        for c in &mut self.craft {
            c.age += dt;
            let wobble = if c.kind == Kind::Plane { Vec2::new(-c.vel.y, c.vel.x).normalize_or_zero() * (c.age * 0.6).sin() * 1.5 } else { Vec2::ZERO };
            c.pos += (c.vel + wobble) * dt;
            if hum && c.kind != Kind::Balloon && c.pos.distance(view) < view_r + 30.0 {
                let (name, vol, pitch) = if c.kind == Kind::Airship { ("drone", 0.35, 0.6) } else { ("drone", 0.5, 1.0) };
                sounds.push(Sound { name, pos: Vec3::new(c.pos.x, c.alt, c.pos.y), vol, pitch });
            }
        }
        self.craft.retain(|c| c.age < 5.0 || c.pos.distance(view) < edge + 130.0);
    }

    pub fn draw(&self, b: &mut Boxes, time: f32, night: f32, ground: &dyn Fn(Vec2) -> f32) {
        for c in &self.craft {
            let heading = c.vel.normalize_or_zero();
            let yaw = crate::render::boxes::yaw_of(heading);
            let base = ground(c.pos);
            let pos = Vec3::new(c.pos.x, base + c.alt, c.pos.y);
            match c.kind {
                Kind::Airship => {
                    let bob = (time * 0.3 + (c.seed % 100) as f32).sin() * 0.3;
                    let mut r = Rig::new(b, pos + Vec3::Y * bob, yaw, 1.0);
                    // a tapered envelope of cream segments with a red band
                    let n = 9;
                    for i in 0..n {
                        let f = i as f32 / (n - 1) as f32 * 2.0 - 1.0;
                        let rad = 2.4 * (1.0 - f * f).max(0.0).sqrt().max(0.18);
                        let col = if i == 4 { [0.8, 0.18, 0.15] } else { [0.94, 0.92, 0.84] };
                        r.part(Vec3::new(0.0, 0.0, f * 7.0), Vec3::new(rad, rad * 0.92, 0.95), col);
                    }
                    r.part(Vec3::new(0.0, -2.9, 0.5), Vec3::new(0.7, 0.45, 1.6), [0.28, 0.22, 0.2]);
                    r.part(Vec3::new(0.0, -2.45, 0.5), Vec3::new(0.05, 0.1, 1.4), [0.3, 0.3, 0.3]);
                    for (x, y) in [(1.5, 0.0), (-1.5, 0.0), (0.0, 1.5), (0.0, -1.5)] {
                        r.part(Vec3::new(x * 0.9, y * 0.9, -7.2), Vec3::new(0.15 + y.abs() * 0.5, 0.15 + x.abs() * 0.5, 0.7), [0.8, 0.18, 0.15]);
                    }
                    let spin = time * 20.0;
                    r.part(Vec3::new(0.0, -2.8, -1.6), Vec3::new(0.1, 0.1, 0.1), [0.2, 0.2, 0.2]);
                    r.part(Vec3::new(0.0, -2.8 + spin.sin() * 0.6, -1.7), Vec3::new(0.05, 0.6, 0.02), [0.2, 0.2, 0.2]);
                    if night > 0.4 {
                        r.part_mat(Vec3::new(2.4, 0.0, 0.0), Vec3::splat(0.14), [1.0, 0.2, 0.2], MAT_GLOW);
                        r.part_mat(Vec3::new(-2.4, 0.0, 0.0), Vec3::splat(0.14), [0.2, 1.0, 0.3], MAT_GLOW);
                    }
                }
                Kind::Balloon => {
                    let bob = (time * 0.35 + (c.seed % 100) as f32).sin() * 0.4;
                    let top = pos + Vec3::Y * bob;
                    let palette = [[0.85, 0.2, 0.2], [0.95, 0.75, 0.2], [0.2, 0.45, 0.8], [0.95, 0.95, 0.9], [0.25, 0.65, 0.4], [0.85, 0.4, 0.15]];
                    let (c1, c2) = (palette[c.seed as usize % palette.len()], palette[(c.seed >> 8) as usize % palette.len()]);
                    // the envelope: stacked octagons, striped, fullest at the shoulder
                    let layers = 9;
                    for i in 0..layers {
                        let f = i as f32 / (layers - 1) as f32;
                        let rad = 3.0 * (1.0 - (f * 2.0 - 0.75).powi(2) * 0.85).max(0.12).sqrt().min(1.0);
                        let col = if i % 2 == 0 { c1 } else { c2 };
                        let y = top.y + 1.0 + f * 5.6;
                        for yw in [0.0, PI * 0.25] {
                            b.push(Vec3::new(top.x, y, top.z), Vec3::new(rad * 0.93, 0.34, rad * 0.93), yw, col, MAT_LIT);
                        }
                    }
                    // ropes, basket, burner and a pair of passengers
                    for (dx, dz) in [(-0.45, -0.45), (0.45, -0.45), (-0.45, 0.45), (0.45, 0.45)] {
                        b.push(top + Vec3::new(dx, -1.2, dz), Vec3::new(0.025, 1.4, 0.025), 0.0, [0.4, 0.3, 0.2], MAT_LIT);
                    }
                    let bask = top + Vec3::Y * -2.9;
                    b.push(bask, Vec3::new(0.62, 0.38, 0.62), yaw, [0.62, 0.45, 0.28], MAT_LIT);
                    b.push(bask + Vec3::Y * 0.5, Vec3::new(0.12, 0.12, 0.12), 0.0, [0.93, 0.78, 0.62], MAT_LIT);
                    b.push(bask + Vec3::new(0.25, 0.48, 0.15), Vec3::new(0.1, 0.1, 0.1), 0.0, [0.9, 0.72, 0.58], MAT_LIT);
                    // the burner's flame flares in time with its roar
                    let ph = (self.t + (c.seed % 90) as f32 * 0.1) % BURN_EVERY;
                    let flare = if ph < BURN_FOR { (ph / BURN_FOR * PI).sin() } else { 0.0 };
                    let flame = 0.12 + flare * 0.9 + 0.05 * (time * 14.0).sin();
                    b.push_alpha(top + Vec3::new(0.0, -1.5 + flame * 0.5, 0.0), Vec3::new(0.12 + flare * 0.08, flame * 0.5, 0.12 + flare * 0.08), 0.0, [1.0, 0.65, 0.25], MAT_GLOW, 0.5 + flare * 0.5);
                }
                Kind::Plane => {
                    let bank = (c.age * 0.6).cos() * 0.0;
                    let mut r = Rig::new(b, pos, yaw + bank, 1.0);
                    let cloth = [0.85, 0.8, 0.62];
                    let red = [0.75, 0.15, 0.12];
                    r.part(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.35, 0.35, 1.6), red);
                    r.part(Vec3::new(0.0, 0.0, -1.7), Vec3::new(0.06, 0.45, 0.4), red);
                    r.part(Vec3::new(0.0, 0.1, -1.6), Vec3::new(0.9, 0.04, 0.3), cloth);
                    for y in [-0.25, 0.65] {
                        r.part(Vec3::new(0.0, y, 0.4), Vec3::new(2.3, 0.05, 0.55), cloth);
                    }
                    for x in [-1.2, 1.2] {
                        r.part(Vec3::new(x, 0.2, 0.4), Vec3::new(0.04, 0.45, 0.04), [0.3, 0.25, 0.2]);
                    }
                    let spin = time * 40.0;
                    r.part(Vec3::new(0.0, 0.0, 1.75), Vec3::new(0.1, 0.1, 0.1), [0.3, 0.3, 0.3]);
                    r.part(Vec3::new(spin.sin() * 0.7, spin.cos() * 0.7, 1.82), Vec3::new(0.05, 0.05, 0.02), [0.25, 0.2, 0.15]);
                    r.part(Vec3::new(0.0, 0.42, 0.15), Vec3::new(0.12, 0.12, 0.12), [0.93, 0.78, 0.62]);
                }
            }
        }
    }
}

/// Telegraph poles beside the railways (from the second age on), strung
/// with wire.
pub fn draw_poles(b: &mut Boxes, gen: &Gen, view: Vec2, view_r: f32, time: f32) {
    let r = view_r as i32 + 12;
    for l in gen.rails_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r) {
        let s0 = l.along(view.x, view.y);
        let spacing = 26.0;
        let first = ((s0 - view_r - 12.0) / spacing).floor() as i32;
        let last = ((s0 + view_r + 12.0) / spacing).ceil() as i32;
        let side: f32 = if hash_u32(l.coord as u32 ^ 0x7e1) & 1 == 0 { 4.2 } else { -4.2 };
        let mut prev: Option<Vec3> = None;
        for k in first..=last {
            let s = k as f32 * spacing;
            let (cx, cz) = l.point(s, 0.0);
            if Vec2::new(cx, cz).distance(view) > view_r + 10.0 {
                prev = None;
                continue;
            }
            let level = gen.rail_level(cx, cz);
            let ground = gen.raw_height(cx, cz);
            // in a tunnel there's nothing to see; on a viaduct or over water
            // the pole stands on the deck's edge rather than in mid-air
            if ground - level > 6.0 {
                prev = None;
                continue;
            }
            let bridged = level - ground > 3.5 || ground < (SEA + 1) as f32;
            let (x, z) = l.point(s, if bridged { side.signum() * 2.0 } else { side });
            let y = level.floor() + 1.0;
            let top = Vec3::new(x, y + 5.0, z);
            let (tx, tz) = l.tangent(s);
            let yaw = crate::render::boxes::yaw_of(Vec2::new(tx, tz));
            let wood = [0.38, 0.28, 0.2];
            b.push(Vec3::new(x, y + 2.5, z), Vec3::new(0.09, 2.5, 0.09), yaw, wood, MAT_LIT);
            b.push(top, Vec3::new(0.7, 0.05, 0.05), yaw, wood, MAT_LIT);
            for dx in [-0.6, 0.6] {
                b.push(top + Vec3::new(tz * dx * -1.0, 0.1, tx * dx), Vec3::new(0.05, 0.07, 0.05), 0.0, [0.7, 0.8, 0.85], MAT_LIT);
            }
            if let Some(p) = prev {
                for w in [-0.6f32, 0.6] {
                    let a = p + Vec3::new(-tz * w, 0.1, tx * w);
                    let c = top + Vec3::new(-tz * w, 0.1, tx * w);
                    // a continuous sagging wire: short lengths laid end to end,
                    // each pointing along the way it runs
                    let n = ((a.distance(c) / 0.5).ceil() as i32).max(8);
                    let sag = |f: f32| a.lerp(c, f) - Vec3::Y * (f * PI).sin() * 0.45;
                    for i in 0..n {
                        let (f0, f1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
                        let (p0, p1) = (sag(f0), sag(f1));
                        let mid = (p0 + p1) * 0.5;
                        let d = p1 - p0;
                        b.push(mid, Vec3::new(0.022, 0.022, Vec2::new(d.x, d.z).length() * 0.55 + 0.03), crate::render::boxes::yaw_of(Vec2::new(d.x, d.z)), [0.12, 0.12, 0.14], MAT_LIT);
                    }
                    // a message in flight: a bright spark running down the wire
                    // with a short fading tail, one pulse at a time along the line
                    let span = spacing;
                    for m in 0..2 {
                        let at = (time * 22.0 + m as f32 * 260.0 + (l.coord as f32 * 7.0)).rem_euclid(520.0);
                        let u = (at - (k - 1) as f32 * span).rem_euclid(520.0);
                        if u < span && w < 0.0 {
                            for tail in 0..4 {
                                let f = ((u - tail as f32 * 1.1) / span).clamp(0.0, 1.0);
                                let q = a.lerp(c, f) - Vec3::Y * (f * PI).sin() * 0.45;
                                b.push_alpha(q, Vec3::splat(0.09 - tail as f32 * 0.016), 0.0, [1.0, 0.92, 0.55], MAT_GLOW, 1.0 - tail as f32 * 0.25);
                            }
                        }
                    }
                }
            }
            prev = Some(top);
        }
    }
}
