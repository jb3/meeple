//! The moving parts of the countryside's rarer places: a camel caravan at
//! the oasis, the geyser's eruptions, a ski lift and its skiers, the
//! midsummer dawn through the standing stones, and the foghorn at the
//! lighthouse.

use std::f32::consts::{PI, TAU};

use glam::{Vec2, Vec3};

use super::people::happenings::extra;
use super::people::Sound;
use crate::render::boxes::{yaw_of, Boxes, Rig, MAT_GLOW, MAT_LIT, MAT_UNLIT};
use crate::util::*;
use crate::world::gen::{Gen, Site, SiteKind};
use crate::world::World;

mod places;
use self::places::Hours;
pub use self::places::{ghost_at, haunted};

#[derive(Default)]
pub struct Wonders {
    horn_t: f32,
    last_hour: i32,
    ghost_t: f32,
}

/// How long the geyser plays, and how often.
const ERUPT_EVERY: f32 = 48.0;
const ERUPT_FOR: f32 = 8.0;

fn surface(gen: &Gen, p: Vec2) -> f32 {
    gen.raw_height(p.x, p.y).floor() + 1.0
}

impl Wonders {
    /// The lighthouse sounds its horn in fog.
    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, dt: f32, world: &World, view: Vec2, view_r: f32, fog: f32, hour: f32, night: f32, sounds: &mut Vec<Sound>) {
        // the monastery bell rings dawn, noon and dusk
        let hr = hour as i32;
        if hr != self.last_hour {
            self.last_hour = hr;
            if matches!(hr, 6 | 12 | 18) {
                for (_, f) in world.features_near(view.x, view.y, view_r + 90.0) {
                    for b in &f.bells {
                        if Vec2::new(b[0], b[2]).distance(view) < view_r + 90.0 {
                            for pitch in [1.0, 1.26, 0.84] {
                                sounds.push(Sound { name: "bell", pos: Vec3::from(*b), vol: 0.8, pitch });
                            }
                        }
                    }
                }
            }
        }
        // a moan out of the dark from a haunted ruin
        self.ghost_t -= dt;
        if self.ghost_t <= 0.0 {
            self.ghost_t = 8.0 + (hash_u32((hour * 100.0) as u32 ^ 0x6057) % 14) as f32;
            if night + fog * 0.5 > 0.5 {
                let gen = &world.gen;
                let r = view_r as i32 + 40;
                for st in gen.sites_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r) {
                    if st.kind == SiteKind::Ruin && places::haunted(&st) && st.centre().distance(view) < view_r + 30.0 {
                        sounds.push(Sound { name: "ghost", pos: Vec3::new(st.x as f32, st.level as f32 + 2.0, st.z as f32), vol: 1.4, pitch: 1.0 });
                        break;
                    }
                }
            }
        }
        self.horn_t -= dt;
        // in fog, and now and then through the night
        if (fog > 0.15 || night > 0.6) && self.horn_t <= 0.0 {
            for (_, f) in world.features_near(view.x, view.y, view_r + 90.0) {
                for b in &f.beacons {
                    if Vec2::new(b[0], b[2]).distance(view) < view_r + 90.0 {
                        sounds.push(Sound { name: "foghorn", pos: Vec3::from(*b), vol: 0.9, pitch: 1.0 });
                        self.horn_t = 17.0 + (hash_u32((hour * 100.0) as u32 ^ 0x4083) % 20) as f32;
                    }
                }
            }
        }
    }

    /// Steam off the hot springs (position, size).
    pub fn steam(world: &World, view: Vec2, view_r: f32) -> Vec<(Vec3, f32)> {
        let mut out = Vec::new();
        for (_, f) in world.features_near(view.x, view.y, view_r + 20.0) {
            for s in &f.steams {
                if Vec2::new(s[0], s[2]).distance(view) < view_r + 10.0 {
                    out.push((Vec3::from(*s), 0.32));
                }
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, b: &mut Boxes, fx: &mut Boxes, gen: &Gen, view: Vec2, view_r: f32, time: f32, day: i64, sun_rise: f32, hour: f32, season: f32, night: f32, fog: f32) {
        let hours = Hours { time, day, hour, season, night, fog };
        let r = view_r as i32 + 40;
        for st in gen.sites_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r) {
            if st.centre().distance(view) > view_r + 40.0 {
                continue;
            }
            match st.kind {
                SiteKind::Oasis => caravan(b, gen, &st, time),
                SiteKind::Geyser => geyser(b, fx, &st, time),
                SiteKind::SkiResort => ski_lift(b, &st, time),
                SiteKind::Mine => mine(b, fx, &st, time),
                SiteKind::StoneCircle => sunrise(fx, &st, day, hour, sun_rise, season),
                _ => places::draw(b, fx, gen, &st, &hours),
            }
        }
    }
}

fn camel(b: &mut Boxes, pos: Vec3, yaw: f32, phase: f32, k: u32) {
    let mut r = Rig::new(b, pos, yaw, 1.0);
    let coat = [0.72, 0.58, 0.38];
    let sw = phase.sin() * 0.25;
    r.part(Vec3::new(0.0, 1.35, 0.0), Vec3::new(0.34, 0.34, 0.85), coat);
    r.part(Vec3::new(0.0, 1.85, 0.1), Vec3::new(0.2, 0.22, 0.25), coat);
    r.part(Vec3::new(0.0, 1.95, 0.9), Vec3::new(0.14, 0.4, 0.14), coat);
    r.part(Vec3::new(0.0, 2.45, 1.15), Vec3::new(0.14, 0.14, 0.24), coat);
    r.part(Vec3::new(0.0, 1.4, -0.9), Vec3::new(0.04, 0.3, 0.04), [0.4, 0.3, 0.2]);
    for (x, z, s) in [(0.22, 0.55, 1.0), (-0.22, 0.55, -1.0), (0.22, -0.55, -1.0), (-0.22, -0.55, 1.0)] {
        r.part(Vec3::new(x, 0.55, z + sw * s), Vec3::new(0.08, 0.55, 0.08), coat);
    }
    // a pack saddle in bright cloth
    let cloth = [[0.8, 0.2, 0.2], [0.2, 0.4, 0.75], [0.9, 0.7, 0.2]][k as usize % 3];
    r.part(Vec3::new(0.0, 1.78, -0.35), Vec3::new(0.4, 0.15, 0.4), cloth);
    r.part(Vec3::new(0.0, 2.0, -0.35), Vec3::new(0.26, 0.12, 0.28), [0.55, 0.42, 0.3]);
}

fn caravan(b: &mut Boxes, gen: &Gen, st: &Site, time: f32) {
    let c = st.centre();
    // three camels and two drovers circling the pool, a slow lap
    let rad = 15.0;
    let w = 0.065;
    for k in 0..5u32 {
        let a = time * w / 1.0 - k as f32 * 0.19 + unit(st.seed) * TAU;
        let r = rad + if k >= 3 { 1.8 } else { 0.0 };
        let p = c + Vec2::new(a.cos(), a.sin()) * r;
        let tang = Vec2::new(-a.sin(), a.cos());
        let y = surface(gen, p);
        if k < 3 {
            camel(b, Vec3::new(p.x, y, p.y), yaw_of(tang), time * 4.0 + k as f32, k);
        } else {
            let q = p;
            extra(b, Vec3::new(q.x, y, q.y), yaw_of(tang), time * 3.5 + k as f32, st.seed ^ k, 1, time);
        }
    }
    // a tent by the water
    let tp = c + Vec2::new(-8.0, 6.0);
    let y = surface(gen, tp);
    b.push(Vec3::new(tp.x, y + 0.9, tp.y), Vec3::new(1.5, 0.9, 1.2), 0.3, [0.85, 0.78, 0.62], MAT_LIT);
    b.push(Vec3::new(tp.x, y + 1.9, tp.y), Vec3::new(0.7, 0.2, 1.0), 0.3, [0.7, 0.2, 0.2], MAT_LIT);
}

fn geyser(b: &mut Boxes, fx: &mut Boxes, st: &Site, time: f32) {
    let c = st.centre();
    let y = st.level as f32 + 1.0;
    // the odd bather's head above the water, steam in their hair
    for k in 0..3 {
        let a = k as f32 * 2.1 + unit(st.seed) * 6.0;
        let p = c + Vec2::new(a.cos(), a.sin()) * 2.4;
        let bob = (time * 0.7 + k as f32).sin() * 0.03;
        b.push(Vec3::new(p.x, y + 0.02 + bob, p.y), Vec3::new(0.16, 0.16, 0.16), 0.0, [0.9, 0.72, 0.58], MAT_LIT);
        b.push(Vec3::new(p.x, y + 0.14 + bob, p.y), Vec3::new(0.17, 0.05, 0.17), 0.0, [0.3 + k as f32 * 0.2, 0.22, 0.18], MAT_LIT);
    }
    // an eruption: a column of water and steam that climbs and collapses
    let t = (time + (st.seed % 48) as f32) % ERUPT_EVERY;
    if t < ERUPT_FOR {
        let f = t / ERUPT_FOR;
        let height = 11.0 * (f * PI).sin().powf(0.6);
        let n = (height / 0.5) as usize;
        for i in 0..n {
            let h = i as f32 * 0.5;
            let sway = (time * 5.0 + i as f32).sin() * 0.12 * (1.0 + h * 0.1);
            let wide = 0.2 + h * 0.03;
            fx.push_alpha(Vec3::new(c.x + sway, y + h, c.y + sway * 0.6), Vec3::new(wide, 0.28, wide), time * 2.0 + i as f32, [0.9, 0.95, 1.0], MAT_UNLIT, 0.7 - h / (height + 1.0) * 0.4);
        }
        for i in 0..14u32 {
            let hk = hash_u32(i * 77 + st.seed);
            let a = unit(hk) * TAU;
            let rr = 0.5 + unit(hk >> 8) * 1.8 * f;
            let fall = (time * 1.5 + unit(hk >> 16)).fract();
            fx.push_alpha(Vec3::new(c.x + a.cos() * rr, y + height * (1.0 - fall * 0.8), c.y + a.sin() * rr), Vec3::splat(0.08), 0.0, [0.8, 0.9, 1.0], MAT_UNLIT, 0.8 * (1.0 - fall));
        }
    }
}

/// The mine at work: the winding wheel turning over the shaft, and trucks
/// of ore running out of the tunnel and back on a slow cycle.
fn mine(b: &mut Boxes, fx: &mut Boxes, st: &Site, time: f32) {
    let at = |u: f32, v: f32| {
        let (ox, oz) = [(0.0f32, -1.0f32), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)][st.face as usize % 4];
        Vec2::new(st.x as f32 + 0.5 + ox * u - oz * v, st.z as f32 + 0.5 + oz * u + ox * v)
    };
    let y0 = st.level as f32 + 1.0;
    let yaw = yaw_of(at(0.0, 1.0) - at(0.0, 0.0));
    let iron = [0.2, 0.2, 0.22];
    // the winding wheel: a drum of two rims joined by spokes, turning on the
    // axle between the towers, in step with the cage it hauls
    let steel = [0.58, 0.6, 0.65];
    const R: f32 = 2.2;
    let lift = ((time * 0.15).sin() * 0.5 + 0.5) * 4.0;
    let spin = lift / R;
    let hub = at(-8.0, 0.0);
    let out = at(-7.0, 0.0) - hub;
    let hub_y = y0 + 7.5;
    let spot = |ang: f32, r: f32, v: f32| {
        let o = at(-8.0, v) - hub;
        let p = hub + out * (ang.cos() * r) + o;
        Vec3::new(p.x, hub_y + ang.sin() * r, p.y)
    };
    for v in [-0.55f32, 0.55] {
        // the rim: a ring of small blocks close enough to read as a circle
        for k in 0..56 {
            let ang = spin + k as f32 * TAU / 56.0;
            b.push(spot(ang, R, v), Vec3::splat(0.12), 0.0, steel, MAT_LIT);
        }
    }
    // spokes: a line of blocks out from the hub on each rim
    for v in [-0.55f32, 0.55] {
        for k in 0..8 {
            let ang = spin + k as f32 * PI / 4.0;
            for j in 1..9 {
                b.push(spot(ang, R * j as f32 / 9.0, v), Vec3::splat(0.09), 0.0, [0.62, 0.46, 0.3], MAT_LIT);
            }
        }
    }
    // slats across the drum, where the rope winds on
    for k in 0..40 {
        let ang = spin + k as f32 * TAU / 40.0;
        for v in [-0.3f32, 0.0, 0.3] {
            b.push(spot(ang, R, v), Vec3::splat(0.08), 0.0, [0.4, 0.28, 0.18], MAT_LIT);
        }
    }
    b.push(Vec3::new(hub.x, hub_y, hub.y), Vec3::new(0.2, 0.2, 0.9), yaw, iron, MAT_LIT);
    // the rope from the drum's foot down the shaft to the cage riding on it
    let cage_top = y0 + 0.7 + lift;
    let rope_top = hub_y - R;
    b.push(Vec3::new(hub.x, (cage_top + rope_top) * 0.5, hub.y), Vec3::new(0.03, (rope_top - cage_top) * 0.5, 0.03), 0.0, iron, MAT_LIT);
    b.push(Vec3::new(hub.x, y0 + 0.4 + lift, hub.y), Vec3::new(0.35, 0.3, 0.35), 0.0, [0.45, 0.32, 0.2], MAT_LIT);
    // trucks running out of the tunnel to the engine house and back
    for k in 0..2 {
        let f = (time * 0.04 + k as f32 * 0.5) % 1.0;
        let tri = if f < 0.5 { f * 2.0 } else { 2.0 - f * 2.0 };
        let u = 12.0 - tri * 8.0;
        let p = at(u, 0.0);
        b.push(Vec3::new(p.x, y0 + 0.45, p.y), Vec3::new(0.45, 0.28, 0.45), yaw, [0.4, 0.28, 0.2], MAT_LIT);
        b.push(Vec3::new(p.x, y0 + 0.75, p.y), Vec3::new(0.38, 0.1, 0.38), yaw, [0.1, 0.1, 0.12], MAT_LIT);
    }
    // smoke from the engine-house chimney
    let ch = at(-2.0, 2.0);
    for k in 0..4 {
        let t = (time * 0.35 + k as f32 * 0.25).fract();
        fx.push_alpha(Vec3::new(ch.x + t * 1.2, y0 + 9.0 + t * 4.0, ch.y), Vec3::splat(0.25 + t * 0.6), 0.0, [0.78, 0.78, 0.8], MAT_UNLIT, 0.6 * (1.0 - t));
    }
}

/// The lift and the skiers on a resort's premade run. Everything is placed in
/// the run's own frame (`u` up the slope, `v` across), standing on the same
/// surface height the terrain is laid at, so nothing hangs in the air.
fn ski_lift(b: &mut Boxes, st: &Site, time: f32) {
    let wood = [0.42, 0.3, 0.2];
    let (lo, hi) = (-36.0f32, 37.0f32);
    let len = hi - lo;
    let side_yaw = yaw_of(st.ski_at(0.0, 1.0) - st.ski_at(0.0, 0.0)) + PI * 0.5;
    let up_yaw = yaw_of(st.ski_at(1.0, 0.0) - st.ski_at(0.0, 0.0));
    let at = |u: f32, v: f32| st.ski_at(u, v);
    let ground = |u: f32| st.ski_surface(u);
    // pylons up the line
    for k in 0..=6 {
        let u = lo + 6.0 + k as f32 * 10.0;
        let p = at(u, 0.0);
        let y = ground(u);
        b.push(Vec3::new(p.x, y + 2.8, p.y), Vec3::new(0.12, 2.8, 0.12), 0.0, wood, MAT_LIT);
        b.push(Vec3::new(p.x, y + 5.5, p.y), Vec3::new(0.12, 0.08, 1.0), side_yaw, wood, MAT_LIT);
    }
    // the cable, a thin line of blocks each side following the slope
    for lat in [-0.9f32, 0.9] {
        let mut u = lo;
        while u < hi {
            let p = at(u, lat);
            b.push(Vec3::new(p.x, ground(u) + 5.7, p.y), Vec3::splat(0.04), 0.0, [0.1, 0.1, 0.12], MAT_LIT);
            u += 1.4;
        }
    }
    // chairs: up one side and down the other
    let n = 7;
    for k in 0..n {
        let f = ((time * 0.035 + k as f32 / n as f32) % 1.0).abs();
        let (u, lat) = if f < 0.5 { (lo + f * 2.0 * len, 0.9) } else { (lo + (1.0 - f) * 2.0 * len, -0.9) };
        let p = at(u, lat);
        let y = ground(u);
        b.push(Vec3::new(p.x, y + 5.7, p.y), Vec3::new(0.02, 0.4, 0.02), 0.0, [0.1, 0.1, 0.12], MAT_LIT);
        b.push(Vec3::new(p.x, y + 5.2, p.y), Vec3::new(0.35, 0.05, 0.25), up_yaw, [0.8, 0.2, 0.2], MAT_LIT);
        if k % 2 == 0 && f < 0.5 {
            b.push(Vec3::new(p.x, y + 5.55, p.y), Vec3::new(0.14, 0.2, 0.1), up_yaw, [0.2, 0.4, 0.8], MAT_LIT);
            b.push(Vec3::new(p.x, y + 5.9, p.y), Vec3::new(0.1, 0.1, 0.1), 0.0, [0.93, 0.78, 0.62], MAT_LIT);
        }
    }
    // skiers carving down the piste beside the lift, in turns
    for k in 0..5u32 {
        let f = (time * 0.04 + k as f32 / 5.0).fract();
        let u = hi - 2.0 - f * (len - 10.0);
        let swing = |f: f32| (f * 16.0 + k as f32 * 1.7).sin() * 5.5;
        let v = 2.0 + swing(f) + (k % 2) as f32 * 1.2;
        let p = at(u, v);
        let ahead = at(u - 1.0, 2.0 + swing(f + 0.01) + (k % 2) as f32 * 1.2);
        extra(b, Vec3::new(p.x, ground(u), p.y), yaw_of(ahead - p), 0.0, st.seed ^ k, 2, time);
    }
}

/// At dawn on midsummer's and midwinter's days a shaft of sun runs through
/// the trilithon onto the altar stone.
fn sunrise(fx: &mut Boxes, st: &Site, day: i64, hour: f32, rise: f32, season: f32) {
    let solstice = day.rem_euclid(12) == 4 || day.rem_euclid(12) == 10;
    let _ = season;
    if !solstice || hour < rise - 0.3 || hour > rise + 1.3 {
        return;
    }
    let k = 1.0 - ((hour - rise - 0.3) / 1.0).abs().min(1.0);
    let c = st.centre();
    let y0 = st.level as f32 + 1.0;
    let mut d = 1.0;
    while d < 20.0 {
        fx.push_alpha(Vec3::new(c.x + d, y0 + 0.4 + d * 0.22, c.y), Vec3::new(0.7, 0.45, 0.35), PI * 0.5, [1.0, 0.82, 0.5], MAT_GLOW, 0.32 * k);
        d += 0.7;
    }
    fx.push(Vec3::new(c.x + 0.5, y0 + 1.25, c.y), Vec3::new(0.4, 0.12, 0.4), 0.0, [1.0, 0.8, 0.45], MAT_GLOW);
}
