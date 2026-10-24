//! Chain ferries over water too wide to bridge: a flat barge hauled hand over
//! hand along a chain from one plank landing to the other, with a pause at
//! each quay. Some crossings carry passengers, some a load of crates.

use std::f32::consts::PI;

use glam::{Vec2, Vec3};

use super::people::happenings::extra_clad;
use crate::render::boxes::{rgb, yaw_of, Boxes, Rig, MAT_GLOW, MAT_LIT};
use crate::util::*;
use crate::world::gen::SEA;
use crate::world::World;

/// How fast the barge is hauled, and how long it waits at each landing.
const SPEED: f32 = 0.9;
const WAIT: f32 = 11.0;
/// How far the barge's centre keeps from the first and last water cell's centre.
const MARGIN: f32 = 1.3;

/// Where a ferry is along its crossing (cells from the first water cell
/// from the low bank) at `time`, whether it's under way, and which crossing
/// number it is (so each trip can carry something different).
pub fn position(span: f32, seed: u32, time: f32) -> (f32, bool, u32) {
    let (lo, hi) = (MARGIN, span - 1.0 - MARGIN);
    let travel = ((hi - lo) / SPEED).max(1.0);
    let leg = WAIT + travel;
    let period = 2.0 * leg;
    let t = (time + (seed % 997) as f32 * 0.37).rem_euclid(period);
    let trip = ((time + (seed % 997) as f32 * 0.37) / leg).floor().max(0.0) as u32;
    let (from, to, u) = if t < leg { (lo, hi, t - WAIT) } else { (hi, lo, t - leg - WAIT) };
    if u <= 0.0 {
        (from, false, trip)
    } else {
        let f = (u / travel).clamp(0.0, 1.0);
        (from + (to - from) * smoothstep(0.0, 1.0, f), f < 1.0, trip)
    }
}

pub fn draw(world: &World, b: &mut Boxes, view: Vec2, view_r: f32, time: f32, night: f32) {
    for (_, f) in world.features_near(view.x, view.y, view_r + 20.0) {
        for &[x, z, ex, ez] in &f.ferries {
            let start = Vec2::new(x, z);
            let end = Vec2::new(ex, ez);
            let span = start.distance(end) + 1.0;
            let d = (end - start) / (span - 1.0).max(0.1);
            if (start + d * span * 0.5).distance(view) > view_r + 12.0 {
                continue;
            }
            let seed = hash_u32((x as i32 * 73856093) as u32 ^ (z as i32 * 19349663) as u32);
            // the water's top is at 0.86 into its block and the landings' planks fill theirs:
            // the deck rides level with the planks
            let water = SEA as f32 + 0.9;
            // the chain, strung from landing to landing, dipping a little between
            let chain = [0.2, 0.2, 0.23];
            let yaw = yaw_of(d);
            let mut s = -0.8;
            while s < span - 0.2 {
                let p = start + d * s;
                let sag = ((s / span.max(1.0)) * PI).sin() * 0.12;
                b.push(Vec3::new(p.x, water + 0.75 - sag, p.y), Vec3::new(0.025, 0.025, 0.5), yaw, chain, MAT_LIT);
                s += 0.95;
            }
            let (s, moving, trip) = position(span, seed, time);
            let c = start + d * s;
            let base = Vec3::new(c.x, water, c.y);
            let mut r = Rig::new(b, base, yaw, 1.0);
            // hull and deck
            r.part(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.2, 0.3, 1.75), rgb(75, 56, 42));
            r.part(Vec3::new(0.0, 0.3, 0.0), Vec3::new(1.12, 0.06, 1.68), rgb(158, 118, 76));
            for sx in [-1.12f32, 1.12] {
                r.part(Vec3::new(sx, 0.55, 0.0), Vec3::new(0.04, 0.2, 1.65), rgb(110, 80, 52));
            }
            // posts at each end for lamps, and the chain bracket
            for sz in [-1.55f32, 1.55] {
                r.part(Vec3::new(0.0, 0.7, sz), Vec3::new(0.05, 0.45, 0.05), rgb(80, 58, 40));
                if night > 0.4 {
                    r.part_mat(Vec3::new(0.0, 1.35, sz), Vec3::splat(0.1), [1.0, 0.8, 0.4], MAT_GLOW);
                }
            }
            let deck = water + 0.36;
            // the ferryman, hauling on the chain
            let side = Vec2::new(d.y, -d.x) * 0.6;
            let man = c + side;
            extra_clad(b, Vec3::new(man.x, deck, man.y), yaw, time * 2.2, seed, if moving { 1 } else { 0 }, time, rgb(90, 80, 60), rgb(55, 50, 45), true, 1.0);
            // what this trip carries
            match hash_u32(seed ^ trip.wrapping_mul(0x9e37)) % 3 {
                1 => {
                    for (k, off) in [(-0.3f32, 0.6f32), (0.4, -0.4)].iter().enumerate() {
                        let p = c - side * 0.55 + d * off.1 + Vec2::new(d.y, -d.x) * off.0 * 0.4;
                        extra_clad(b, Vec3::new(p.x, deck, p.y), yaw + PI * if k == 0 { 0.5 } else { 1.4 }, 0.0, seed ^ (k as u32 * 99 + trip), 0, time, [rgb(180, 90, 70), rgb(70, 120, 170)][k], rgb(60, 55, 50), k == 0, 1.0);
                    }
                }
                2 => {
                    let mut r = Rig::new(b, Vec3::new(c.x, deck, c.y), yaw, 1.0);
                    for (lx, lz, h) in [(-0.45f32, -0.7f32, 0.3f32), (0.45, -0.7, 0.3), (-0.45, 0.0, 0.3), (0.0, -0.7, 0.75), (0.5, 0.55, 0.3)] {
                        r.part(Vec3::new(lx, h * 0.5 + 0.1, lz), Vec3::new(0.3, h * 0.5 + 0.1, 0.3), rgb(150, 112, 70));
                    }
                    r.part(Vec3::new(0.4, 0.4, 0.3), Vec3::new(0.2, 0.3, 0.2), rgb(120, 85, 55));
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ferry_shuttles_between_the_landings_and_waits_at_each() {
        for span in [10.0f32, 14.0, 25.0] {
            let (lo, hi) = (MARGIN, span - 1.0 - MARGIN);
            let (mut at_lo, mut at_hi, mut moving_seen) = (0.0, 0.0, false);
            let mut prev = position(span, 5, 0.0).0;
            for i in 0..4000 {
                let (s, moving, _) = position(span, 5, i as f32 * 0.1);
                assert!(s >= lo - 1e-3 && s <= hi + 1e-3, "span {span}: ferry at {s} off its crossing");
                assert!((s - prev).abs() < 0.2, "ferry jumped from {prev} to {s}");
                prev = s;
                if (s - lo).abs() < 0.01 {
                    at_lo += 0.1;
                }
                if (s - hi).abs() < 0.01 {
                    at_hi += 0.1;
                }
                moving_seen |= moving;
            }
            assert!(moving_seen && at_lo > 5.0 && at_hi > 5.0, "span {span}: lo {at_lo} hi {at_hi}");
        }
    }
}
