//! Moving parts of buildings: windmill sails turning in the wind, water
//! wheels turning in the stream, lighthouse beams sweeping round at night.

use glam::{Vec2, Vec3};

use crate::render::boxes::{rgb, Boxes, Rig, MAT_GLOW};
use crate::world::gen::{Weir, SEA};
use crate::world::World;

/// Sails turn at `spin` (an angle the caller advances smoothly with the
/// wind); beams only sweep at night.
#[allow(clippy::too_many_arguments)]
pub fn draw(b: &mut Boxes, fx: &mut Boxes, world: &World, view: Vec2, view_r: f32, time: f32, spin: f32, night: f32, tide: f32) {
    let near = |p: [f32; 3], pad: f32| Vec2::new(p[0], p[2]).distance(view) < view_r + pad;
    for (_, f) in world.features_near(view.x, view.y, view_r + 30.0) {
        for &(hub, yaw) in &f.sails {
            if near(hub, 6.0) {
                sails(b, Vec3::from(hub), yaw, spin);
            }
        }
        for &(hub, yaw) in &f.wheels {
            if near(hub, 4.0) {
                wheel(b, Vec3::from(hub), yaw, -time * 0.8);
            }
        }
        for w in &f.weirs {
            if near([w.c.x, 0.0, w.c.y], w.a.max(w.b) + 4.0) {
                weir_water(b, w, time, night, tide);
            }
        }
        if night > 0.4 {
            for &lamp in &f.beacons {
                if near(lamp, 30.0) {
                    beam(fx, Vec3::from(lamp), time * 0.9, (night - 0.4) / 0.6);
                }
            }
        }
    }
    fishers(b, world, view, view_r, time, night, tide);
}

/// Anglers on the river banks near the view, each with a rod, line and
/// bobber; now and then one gets a bite and the rod tip dips.
fn fishers(b: &mut Boxes, world: &World, view: Vec2, view_r: f32, time: f32, night: f32, tide: f32) {
    if night > 0.85 {
        return;
    }
    for (_, f) in world.features_near(view.x, view.y, view_r + 6.0) {
        for &(pos, yaw, seed) in &f.anglers {
            if Vec2::new(pos[0], pos[2]).distance(view) < view_r + 2.0 {
                angler(b, &Angler { pos: Vec3::from(pos), yaw, seed }, time, view_r, tide);
            }
        }
    }
}

struct Angler {
    pos: Vec3,
    yaw: f32,
    seed: u32,
}

fn angler(b: &mut Boxes, a: &Angler, time: f32, view_r: f32, tide: f32) {
    let pick = |n: &[[f32; 3]]| n[(a.seed >> 4) as usize % n.len()];
    let shirt = pick(&[rgb(180, 70, 60), rgb(70, 110, 160), rgb(90, 140, 80), rgb(200, 170, 80), rgb(120, 90, 140)]);
    let pants = pick(&[rgb(60, 60, 80), rgb(90, 75, 55), rgb(50, 70, 60)]);
    let skin = pick(&[rgb(235, 190, 160), rgb(200, 150, 115), rgb(140, 95, 70)]);
    let hat = pick(&[rgb(210, 190, 120), rgb(70, 90, 70), rgb(140, 60, 50)]);
    let sitting = a.seed & 1 == 0;
    let mut r = Rig::new(b, a.pos, a.yaw, 1.0);
    let ty = if sitting { 0.62 } else { 0.82 };
    // the odd bite: the rod tip dips for a moment
    let phase = time * 0.35 + (a.seed % 997) as f32;
    let bite = (phase.fract() > 0.93) as i32 as f32 * ((time * 14.0).sin() * 0.5 + 0.5);
    if sitting {
        for x in [-0.11, 0.11] {
            r.part(Vec3::new(x, 0.14, 0.16), Vec3::new(0.09, 0.08, 0.2), pants);
            r.part(Vec3::new(x, 0.12, 0.38), Vec3::new(0.08, 0.1, 0.08), pants);
        }
        r.part(Vec3::new(0.0, 0.06, -0.08), Vec3::new(0.16, 0.06, 0.14), rgb(110, 80, 50));
    } else {
        for x in [-0.11, 0.11] {
            r.part(Vec3::new(x, 0.28, 0.0), Vec3::new(0.09, 0.28, 0.09), pants);
        }
    }
    r.part(Vec3::new(0.0, ty, 0.0), Vec3::new(0.22, 0.26, 0.14), shirt);
    let sleeve = [shirt[0] * 0.85, shirt[1] * 0.85, shirt[2] * 0.85];
    for x in [-0.25, 0.25] {
        r.part(Vec3::new(x, ty + 0.0, 0.2), Vec3::new(0.07, 0.07, 0.2), sleeve);
    }
    let hy = ty + 0.45;
    r.part(Vec3::new(0.0, hy, 0.0), Vec3::new(0.17, 0.17, 0.17), skin);
    r.part(Vec3::new(0.0, hy + 0.19, 0.0), Vec3::new(0.32, 0.025, 0.32), hat);
    r.part(Vec3::new(0.0, hy + 0.25, 0.0), Vec3::new(0.17, 0.06, 0.17), hat);
    // rod: a long slim pole out over the water, tip dipping on a bite
    let tip_y = ty + 1.0 - bite * 0.25;
    let (tx, tz) = (0.28, 2.4);
    // (a box can't tilt, so the rod is a close string of small blocks along its slope,
    // thinning towards the tip)
    let segs = 28;
    for i in 0..segs {
        let t = (i as f32 + 0.5) / segs as f32;
        let y = ty + 0.05 + (tip_y - ty - 0.05) * t;
        let w = 0.02 - 0.009 * t;
        r.part(Vec3::new(tx * t, y, 0.3 + (tz - 0.3) * t), Vec3::new(w, w, 0.06), rgb(120, 85, 50));
    }
    // line down to a red-and-white bobber on the water
    // the bobber rides the water, so it rises and falls with the tide
    let bob_y = SEA as f32 + 0.86 + tide - a.pos.y;
    let n = 6;
    // a hair would be narrower than a screen pixel and blink in and out as
    // the view zooms, so the line is kept about a pixel wide
    let line = (view_r * 0.0035).clamp(0.01, 0.028);
    for i in 0..n {
        let y = tip_y - (i as f32 + 0.5) / n as f32 * (tip_y - bob_y);
        r.part(Vec3::new(tx, y, tz), Vec3::new(line, (tip_y - bob_y) / n as f32 / 2.0, line), rgb(235, 235, 225));
    }
    let ripple = (time * 2.0 + a.seed as f32).sin() * 0.02 - bite * 0.05;
    r.part(Vec3::new(tx, bob_y + 0.04 + ripple, tz), Vec3::new(0.05, 0.05, 0.05), rgb(220, 50, 40));
    r.part(Vec3::new(tx, bob_y + 0.1 + ripple, tz), Vec3::new(0.03, 0.03, 0.03), rgb(245, 245, 240));
}

/// A water wheel: a ring of paddles round an axle.
fn wheel(b: &mut Boxes, hub: Vec3, yaw: f32, angle: f32) {
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let wood = [0.42, 0.3, 0.2];
    b.push(hub, Vec3::new(0.12, 0.12, 0.12), yaw, [0.25, 0.22, 0.2], 0.0);
    // spokes and rim as strings of overlapping blocks (a box can only turn
    // about the vertical), then a paddle at the end of each spoke
    for k in 0..10 {
        let a = angle + k as f32 * std::f32::consts::TAU / 10.0;
        let dir = right * a.cos() + Vec3::Y * a.sin();
        let mut r = 0.15;
        while r < 1.5 {
            b.push(hub + dir * r, Vec3::new(0.07, 0.07, 0.3), yaw, wood, 0.0);
            r += 0.11;
        }
        b.push(hub + dir * 1.62, Vec3::new(0.18, 0.18, 0.42), yaw, wood, 0.0);
    }
    for k in 0..48 {
        let a = angle + k as f32 * std::f32::consts::TAU / 48.0;
        let dir = right * a.cos() + Vec3::Y * a.sin();
        b.push(hub + dir * 1.45, Vec3::new(0.09, 0.09, 0.36), yaw, wood, 0.0);
    }
}

/// A cheap hash of a number to 0..1, for scattering spray.
fn scatter(x: f32) -> f32 {
    (x.sin() * 43758.547).fract().abs()
}

/// The water running over a weir: streaks quickening towards the crest, a
/// glassy lip, a curtain that flares out as it falls, flung droplets, foam
/// churning where it lands and a little mist rising off it.
fn weir_water(b: &mut Boxes, w: &Weir, time: f32, night: f32, tide: f32) {
    let dim = 1.0 - night * 0.55;
    let surface = SEA as f32 + 0.86 + tide;
    let crest = SEA as f32 + 1.0;
    let drop = (crest - surface).max(0.1);
    let mut t = -w.a + 0.25;
    while t < w.b {
        let base = w.c + w.cross * t;
        let at = |s: f32, y: f32| Vec3::new(base.x + w.flow.x * s, y, base.y + w.flow.y * s);
        let seed = t * 12.9 + w.c.x * 0.37 + w.c.y * 0.71;
        // pale streaks drawing in over the upstream water, faster as they near the lip
        for k in 0..2 {
            let ph = (time * 0.35 + scatter(seed + k as f32 * 3.7)).fract();
            let s = -2.6 + ph * ph * 2.2;
            let fade = (ph * 4.0).min(1.0) * (1.0 - ph * 0.3);
            b.push_alpha(at(s, surface + 0.03), Vec3::new(0.2, 0.012, 0.09), 0.0, [0.9 * dim, 0.96 * dim, dim], 0.0, 0.55 * fade);
        }
        // the glassy lip over the crest
        let glint = 0.9 + 0.1 * (time * 3.0 + t * 1.7).sin();
        b.push_alpha(at(0.0, crest + 0.02), Vec3::new(0.27, 0.03, 0.27), 0.0, [0.72 * glint * dim, 0.88 * glint * dim, dim], 0.0, 0.9);
        // the curtain, flaring out as it falls, with streaks racing down it
        let steps = 6;
        for k in 0..steps {
            let f = (k as f32 + 0.5) / steps as f32;
            let y = crest - f * drop;
            let race = 0.5 + 0.5 * (time * 9.0 + seed * 2.0 - f * 8.0).sin();
            let white = 0.62 + 0.3 * f + 0.2 * race;
            let c = [(0.6 + 0.4 * white) * dim, (0.78 + 0.22 * white) * dim, dim];
            b.push_alpha(at(0.95 + f * f * 0.55, y), Vec3::new(0.27, drop / steps as f32 * 0.55, 0.2), 0.0, c, 0.0, 0.85);
        }
        // droplets flung out and falling back
        for k in 0..2 {
            let id = seed + k as f32 * 5.3;
            let ph = (time * 0.8 + scatter(id)).fract();
            let tau = ph * 0.5;
            let vx = 0.7 + scatter(id + 1.0) * 0.9;
            let vy = 1.0 + scatter(id + 2.0) * 1.1;
            let y = surface + 0.15 + vy * tau - 4.5 * tau * tau;
            if y > surface {
                let lat = (scatter(id + 3.0) - 0.5) * 0.4;
                let p = at(1.3 + vx * tau, y) + Vec3::new(w.cross.x, 0.0, w.cross.y) * lat;
                b.push(p, Vec3::splat(0.035), 0.0, [0.9 * dim, 0.96 * dim, dim], 0.0);
            }
        }
        // foam churning below, patchy and always shifting
        for (i, s) in [1.35, 1.9, 2.5, 3.2].into_iter().enumerate() {
            let flick = (time * (4.0 + i as f32) + t * 3.1 + s * 5.0).sin();
            if flick > -0.35 {
                let size = (0.2 + 0.08 * flick) * (1.0 - i as f32 * 0.12);
                let c = 0.88 + 0.12 * flick;
                let a = if i == 3 { 0.5 } else { 0.95 };
                b.push_alpha(at(s + 0.1 * flick, surface + 0.02), Vec3::new(size, 0.03, size), 0.0, [c * dim, c * dim, dim], 0.0, a);
            }
        }
        // mist drifting up off the foot
        let ph = (time * 0.3 + scatter(seed + 9.0)).fract();
        b.push_alpha(at(1.5 + ph * 0.6, surface + 0.2 + ph * 0.9), Vec3::splat(0.2 + ph * 0.18), 0.0, [0.92 * dim, 0.96 * dim, dim], 0.0, 0.32 * (1.0 - ph));
        t += 0.5;
    }
}

/// A lighthouse beam: two glowing shafts sweeping round, fading outward.
fn beam(fx: &mut Boxes, lamp: Vec3, angle: f32, strength: f32) {
    for k in [0.0, std::f32::consts::PI] {
        let a = angle + k;
        let dir = Vec3::new(a.cos(), -0.04, a.sin());
        let yaw = dir.x.atan2(dir.z);
        let mut r = 0.8;
        while r < 26.0 {
            let fade = (1.0 - r / 26.0) * strength;
            fx.push_alpha(lamp + dir * r, Vec3::new(0.18 + r * 0.03, 0.18 + r * 0.03, 0.4), yaw, [1.0, 0.93, 0.7], MAT_GLOW, 0.55 * fade);
            r += 0.8;
        }
    }
}

/// Four lattice sails on a hub facing `yaw`, turned to `angle`. Boxes only
/// turn about the vertical, so each sail is laid out as a lattice of small
/// blocks along its arm.
fn sails(b: &mut Boxes, hub: Vec3, yaw: f32, angle: f32) {
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let fwd = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let wood = [0.42, 0.3, 0.2];
    let cloth = [0.93, 0.9, 0.82];
    b.push(hub, Vec3::splat(0.22), yaw, wood, 0.0);
    for k in 0..4 {
        let a = angle + k as f32 * std::f32::consts::FRAC_PI_2;
        let dir = right * a.cos() + Vec3::Y * a.sin();
        let side = right * -a.sin() + Vec3::Y * a.cos();
        let mut r = 0.35;
        while r < 4.2 {
            b.push(hub + dir * r + fwd * 0.12, Vec3::new(0.07, 0.07, 0.05), yaw, wood, 0.0);
            if r > 0.9 {
                for (w, col) in [(0.3, cloth), (0.6, cloth), (0.88, wood)] {
                    b.push(hub + dir * r + side * w + fwd * 0.08, Vec3::new(0.13, 0.13, 0.03), yaw, col, 0.0);
                }
            }
            r += 0.28;
        }
    }
}
