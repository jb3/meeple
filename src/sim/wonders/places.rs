//! Life at the countryside's quieter places: apple pickers in the orchard,
//! bees and a beekeeper at the apiary, monks in the cloister, a pilgrim at the
//! shrine, travellers round their campfire, a sentry on the hill fort, and
//! lights and a ghost among the haunted ruins.

use std::f32::consts::{PI, TAU};

use glam::{Vec2, Vec3};

use crate::render::boxes::{yaw_of, Boxes, MAT_GLOW, MAT_LIT, MAT_UNLIT};
use crate::sim::people::happenings::extra_clad;
use crate::util::*;
use crate::world::chunk::{apiary_hives, orchard_trees};
use crate::world::gen::{Gen, Site, SiteKind};

/// What the time is, for deciding who's about.
#[derive(Clone, Copy)]
pub(super) struct Hours {
    pub time: f32,
    pub day: i64,
    pub hour: f32,
    pub season: f32,
    /// 0 (day) to 1 (night).
    pub night: f32,
    pub fog: f32,
}

impl Hours {
    fn winter(&self) -> bool {
        (2.95..4.0).contains(&self.season)
    }

    /// Late summer into autumn, when the apples are ripe.
    fn apples(&self) -> bool {
        (1.85..2.95).contains(&self.season)
    }
}

/// Is this ruin a haunted one? Most are.
pub fn haunted(st: &Site) -> bool {
    st.seed % 10 < 7
}

/// A point `d` of the way round a closed loop of corners.
fn along_loop(pts: &[Vec2], d: f32) -> (Vec2, Vec2) {
    let total: f32 = (0..pts.len()).map(|i| pts[i].distance(pts[(i + 1) % pts.len()])).sum();
    let mut left = d.rem_euclid(total);
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let len = a.distance(b);
        if left <= len {
            let dir = (b - a) / len.max(1e-4);
            return (a + dir * left, dir);
        }
        left -= len;
    }
    (pts[0], Vec2::X)
}

pub(super) fn draw(b: &mut Boxes, fx: &mut Boxes, _gen: &Gen, st: &Site, k: &Hours) {
    match st.kind {
        SiteKind::Orchard => orchard(b, st, k),
        SiteKind::Apiary => apiary(b, st, k),
        SiteKind::Monastery => monastery(b, st, k),
        SiteKind::Shrine => shrine(b, st, k),
        SiteKind::Camp => camp(b, fx, st, k),
        SiteKind::HillFort => hill_fort(b, st, k),
        SiteKind::Ruin => ruin(b, fx, st, k),
        _ => {}
    }
}

const APPLE: [f32; 3] = [0.82, 0.12, 0.1];

fn orchard(b: &mut Boxes, st: &Site, k: &Hours) {
    if !k.apples() {
        return;
    }
    let trees = orchard_trees(st);
    for (i, &(tx, tz)) in trees.iter().enumerate() {
        let c = Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5);
        // (the orchard's ground is levelled to the site's height, not the raw land's)
        let g = st.level as f32 + 1.0;
        // apples hanging round the edge of the canopy, fewer as they're picked
        let picked = ((k.hour - 8.0) / 9.0).clamp(0.0, 1.0);
        for a in 0..7u32 {
            let h = hash_u32(st.seed ^ (i as u32 * 131 + a * 17));
            if unit(h >> 20) < picked * 0.6 {
                continue;
            }
            let ang = unit(h) * TAU;
            let r = 1.55;
            let p = Vec3::new(c.x + ang.cos() * r, g + 2.6 + unit(h >> 8) * 1.1, c.y + ang.sin() * r);
            b.push(p, Vec3::splat(0.13), 0.0, APPLE, MAT_LIT);
        }
        // windfalls in the grass
        for a in 0..2u32 {
            let h = hash_u32(st.seed ^ (i as u32 * 71 + a * 5 + 9));
            let ang = unit(h) * TAU;
            let r = 0.8 + unit(h >> 8) * 1.4;
            b.push(Vec3::new(c.x + ang.cos() * r, g + 0.1, c.y + ang.sin() * r), Vec3::splat(0.08), 0.0, APPLE, MAT_LIT);
        }
    }
    // pickers on ladders through the working day
    if (8.0..17.0).contains(&k.hour) && k.fog < 0.6 && !trees.is_empty() {
        for p in 0..3usize {
            let i = (hash_u32(st.seed ^ (p as u32 * 977)) as usize) % trees.len();
            let (tx, tz) = trees[i];
            let c = Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5);
            let g = st.level as f32 + 1.0;
            // a ladder against the canopy and the picker at the foot of it
            let lx = c.x + 1.75;
            for z in [-0.18f32, 0.18] {
                b.push(Vec3::new(lx, g + 1.6, c.y + z), Vec3::new(0.025, 1.6, 0.025), 0.0, [0.5, 0.36, 0.22], MAT_LIT);
            }
            for r in 0..6 {
                b.push(Vec3::new(lx, g + 0.5 + r as f32 * 0.5, c.y), Vec3::new(0.025, 0.025, 0.2), 0.0, [0.5, 0.36, 0.22], MAT_LIT);
            }
            let stand = Vec3::new(lx + 0.7, g, c.y + 0.1);
            let face = yaw_of(Vec2::new(-1.0, 0.0));
            extra_clad(b, stand, face, k.time * 1.5 + p as f32, st.seed ^ p as u32, 3, k.time + p as f32 * 1.7, [0.6, 0.55, 0.42], [0.35, 0.3, 0.25], true, 1.0);
            // the basket beside them, heaped with apples
            b.push(Vec3::new(stand.x + 0.5, g + 0.15, stand.z), Vec3::new(0.22, 0.15, 0.22), 0.0, [0.62, 0.44, 0.26], MAT_LIT);
            for a in 0..4 {
                b.push(Vec3::new(stand.x + 0.4 + 0.1 * (a % 2) as f32, g + 0.33, stand.z - 0.08 + 0.1 * (a / 2) as f32), Vec3::splat(0.08), 0.0, APPLE, MAT_LIT);
            }
        }
    }
    // crates by the gate filling up through the day
    let filled = ((k.hour - 8.0) / 9.0).clamp(0.0, 1.0);
    for (n, (cx, cz)) in [(st.x + 3, st.z + 6), (st.x + 4, st.z + 6), (st.x + 3, st.z + 7)].into_iter().enumerate() {
        if filled > n as f32 / 3.0 {
            let y = st.level as f32 + 2.12;
            for a in 0..3 {
                b.push(Vec3::new(cx as f32 + 0.25 + 0.25 * a as f32, y, cz as f32 + 0.5), Vec3::splat(0.1), 0.0, APPLE, MAT_LIT);
            }
        }
    }
}

fn apiary(b: &mut Boxes, st: &Site, k: &Hours) {
    let y0 = st.level as f32 + 1.0;
    let hives = apiary_hives(st);
    // honey jars on the stall
    for j in 0..4 {
        b.push(Vec3::new(st.x as f32 - 2.65 + j as f32 * 0.3, y0 + 1.15, st.z as f32 + 5.5), Vec3::new(0.09, 0.12, 0.09), 0.0, [0.85, 0.52, 0.08], MAT_LIT);
    }
    let warm = !k.winter() && (7.0..19.0).contains(&k.hour) && k.fog < 0.5;
    if !warm {
        return;
    }
    // bees looping round each hive and out to the flowers
    for (n, &(hx, hz)) in hives.iter().enumerate() {
        for bee in 0..4u32 {
            let ph = k.time * (2.0 + 0.3 * bee as f32) + n as f32 * 1.9 + bee as f32 * 1.7;
            let r = 0.55 + 0.45 * (k.time * 0.7 + bee as f32).sin().abs() + (bee % 2) as f32 * 0.8;
            let p = Vec3::new(hx as f32 + 0.5 + ph.cos() * r, y0 + 3.3 + 0.25 * (ph * 1.7).sin(), hz as f32 + 0.5 + (ph * 2.0).sin() * r * 0.7);
            b.push(p, Vec3::splat(0.035), 0.0, [0.95, 0.78, 0.12], MAT_LIT);
        }
    }
    // the beekeeper in a pale suit, strolling round the hives
    let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
    let route = [Vec2::new(x - 4.2, z - 3.4), Vec2::new(x + 4.2, z - 3.4), Vec2::new(x + 4.2, z + 2.4), Vec2::new(x - 4.2, z + 2.4)];
    let (p, dir) = along_loop(&route, k.time * 0.55 + (st.seed % 20) as f32);
    extra_clad(b, Vec3::new(p.x, y0, p.y), yaw_of(dir), k.time * 3.0, st.seed, 1, k.time, [0.95, 0.95, 0.92], [0.9, 0.9, 0.86], true, 1.0);
}

fn monastery(b: &mut Boxes, st: &Site, k: &Hours) {
    if !(5.5..20.5).contains(&k.hour) || k.fog > 0.85 {
        return;
    }
    let y0 = st.level as f32 + 1.0;
    let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
    // monks pacing the cloister walk, two of them a little apart
    let route = [Vec2::new(x - 1.5, z - 0.3), Vec2::new(x + 4.2, z - 0.3), Vec2::new(x + 4.2, z + 4.8), Vec2::new(x - 1.5, z + 4.8)];
    for m in 0..2u32 {
        let (p, dir) = along_loop(&route, k.time * 0.7 + m as f32 * 8.5);
        extra_clad(b, Vec3::new(p.x, y0, p.y), yaw_of(dir), k.time * 2.6 + m as f32, st.seed ^ m, 1, k.time, [0.42, 0.3, 0.2], [0.38, 0.27, 0.18], false, 1.0);
    }
    // another tending the vegetable garden
    let tend = (k.hour > 7.0 && k.hour < 17.0) as u8;
    if tend == 1 {
        extra_clad(b, Vec3::new(x - 4.5, y0 + 0.0, z + 1.5), 1.3, 0.0, st.seed ^ 77, 4, k.time, [0.42, 0.3, 0.2], [0.38, 0.27, 0.18], false, 1.0);
    }
}

fn shrine(b: &mut Boxes, st: &Site, k: &Hours) {
    let y0 = st.level as f32 + 1.0;
    let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
    // a pilgrim kneels at dawn or dusk, on some days
    let at_prayer = (6.0..7.5).contains(&k.hour) || (17.5..19.0).contains(&k.hour);
    if at_prayer && hash_u32(st.seed ^ k.day as u32) % 2 == 0 && k.fog < 0.7 {
        extra_clad(b, Vec3::new(x, y0, z + 2.3), PI, 0.0, st.seed ^ k.day as u32, 4, k.time, [0.5, 0.42, 0.55], [0.3, 0.28, 0.35], false, 1.0);
    }
}

fn camp(b: &mut Boxes, fx: &mut Boxes, st: &Site, k: &Hours) {
    let y0 = st.level as f32 + 1.0;
    let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
    // sparks off the fire, burning whenever someone's about
    let about = k.hour >= 17.0 || k.hour < 7.5;
    if about {
        for s in 0..5u32 {
            let t = (k.time * 0.6 + s as f32 * 0.2).fract();
            let sway = (k.time * 3.0 + s as f32 * 2.0).sin() * 0.25;
            fx.push_alpha(Vec3::new(x + sway, y0 + 0.7 + t * 2.2, z + sway * 0.6), Vec3::splat(0.045), 0.0, [1.0, 0.65, 0.2], MAT_GLOW, 1.0 - t);
        }
        // three travellers sitting up round it
        for p in 0..3u32 {
            let a = p as f32 * 2.1 + (st.seed % 7) as f32;
            let c = Vec2::new(x + a.cos() * 2.3, z + a.sin() * 2.3);
            let how = if p == 1 { 4 } else { 0 };
            extra_clad(b, Vec3::new(c.x, y0, c.y), yaw_of(Vec2::new(x, z) - c), 0.0, st.seed ^ (p * 31), how, k.time + p as f32, [[0.7, 0.3, 0.25], [0.3, 0.5, 0.6], [0.75, 0.6, 0.25]][p as usize], [0.3, 0.25, 0.2], p == 2, 1.0);
        }
    }
}

fn hill_fort(b: &mut Boxes, st: &Site, k: &Hours) {
    let y0 = st.level as f32 + 1.0;
    let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
    // a pennant on the keep, streaming in the wind
    let col = [[0.75, 0.15, 0.15], [0.15, 0.3, 0.7], [0.85, 0.7, 0.15]][st.seed as usize % 3];
    b.push(Vec3::new(x, y0 + 10.4, z), Vec3::new(0.04, 1.4, 0.04), 0.0, [0.4, 0.3, 0.2], MAT_LIT);
    for s in 0..5 {
        let w = (k.time * 3.0 - s as f32 * 0.8).sin() * 0.12 * (s as f32 * 0.5 + 0.3);
        b.push(Vec3::new(x + 0.3 + s as f32 * 0.28, y0 + 11.4 + w * 0.3, z + w), Vec3::new(0.15, 0.2 - s as f32 * 0.03, 0.025), 0.0, col, MAT_LIT);
    }
    // the sentry on his rounds inside the palisade, with a torch after dark
    if k.hour >= 5.5 || k.hour < 1.0 {
        let a = k.time * 0.09 + unit(st.seed) * TAU;
        let c = Vec2::new(x + a.cos() * 9.2, z + a.sin() * 9.2);
        let tang = Vec2::new(-a.sin(), a.cos());
        extra_clad(b, Vec3::new(c.x, y0, c.y), yaw_of(tang), k.time * 2.4, st.seed, if k.night > 0.5 { 5 } else { 1 }, k.time, [0.55, 0.2, 0.2], [0.3, 0.3, 0.35], false, 1.0);
    }
}

/// Wisps of cold light wander the haunted ruins after dark, and now and then
/// a pale figure walks through the broken arch.
fn ruin(b: &mut Boxes, fx: &mut Boxes, st: &Site, k: &Hours) {
    if !haunted(st) {
        return;
    }
    let dark = (k.night + k.fog * 0.5).min(1.0);
    if dark < 0.55 {
        return;
    }
    let y0 = st.level as f32 + 1.0;
    let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
    let fade = smoothstep(0.55, 0.9, dark);
    for w in 0..3u32 {
        let a = k.time * (0.35 + 0.1 * w as f32) + w as f32 * 2.2 + unit(st.seed) * TAU;
        let p = Vec3::new(x + a.cos() * (2.4 + w as f32 * 0.5), y0 + 0.9 + 0.5 * (a * 1.7).sin() + w as f32 * 0.25, z + a.sin() * (1.5 + w as f32 * 0.4));
        let flick = 0.6 + 0.4 * (k.time * 7.0 + w as f32 * 3.0).sin();
        fx.push_alpha(p, Vec3::splat(0.2), 0.0, [0.55, 0.95, 1.0], MAT_GLOW, flick * fade);
        fx.push_alpha(p, Vec3::splat(0.55), 0.0, [0.4, 0.8, 1.0], MAT_UNLIT, 0.22 * flick * fade);
    }
    if let Some((p, vis)) = ghost_at(st, k.time) {
        extra_clad(b, p, yaw_of(Vec2::new(1.0, 0.1)), 0.0, st.seed, 0, k.time, [0.85, 0.9, 0.95], [0.8, 0.85, 0.9], false, 0.8 * vis * fade);
    }
}

/// Where the pale figure is hovering at this time (and how visible, 0..1),
/// if it's out: here for a stretch of each minute, drifting across the arch.
pub fn ghost_at(st: &Site, time: f32) -> Option<(Vec3, f32)> {
    if !haunted(st) {
        return None;
    }
    let (x, z) = (st.x as f32 + 0.5, st.z as f32 + 0.5);
    let cycle = (time * 0.03 + unit(st.seed >> 3) * 10.0).rem_euclid(1.0);
    if !(0.2..0.7).contains(&cycle) {
        return None;
    }
    let f = (cycle - 0.2) / 0.5;
    let p = Vec2::new(x - 2.5 + f * 5.0, z - 1.2 + (f * 3.0).sin() * 0.5);
    Some((Vec3::new(p.x, st.level as f32 + 1.0 + 0.15 + 0.1 * (time * 2.0).sin(), p.y), (f * PI).sin()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(st: &Site, gen: &Gen, k: &Hours) -> usize {
        let (mut b, mut fx) = (Boxes::default(), Boxes::default());
        draw(&mut b, &mut fx, gen, st, k);
        b.len() + fx.len()
    }

    fn find(kind: SiteKind, pred: &dyn Fn(&Site) -> bool) -> (Gen, std::rc::Rc<Site>) {
        for seed in [1234u32, 7, 99, 2024, 5, 31] {
            let gen = Gen::new(seed);
            for rx in -8..=8 {
                for rz in -8..=8 {
                    if let Some(st) = gen.sites(rx, rz).iter().find(|s| s.kind == kind && pred(s)) {
                        let st = st.clone();
                        return (gen, st);
                    }
                }
            }
        }
        panic!("no {kind:?} found");
    }

    /// The quiet places are alive at the right times: apples and pickers only
    /// in autumn daylight, bees in warm weather, ghosts only in the dark.
    #[test]
    fn places_draw_what_belongs_to_the_time() {
        let k = |season: f32, hour: f32, night: f32| Hours { time: 100.0, day: 7, hour, season, night, fog: 0.0 };
        let any = |_: &Site| true;
        let (gen, orchard) = find(SiteKind::Orchard, &any);
        assert!(count(&orchard, &gen, &k(2.2, 12.0, 0.0)) > 20, "orchard in autumn noon");
        assert_eq!(count(&orchard, &gen, &k(0.5, 12.0, 0.0)), 0, "no apples in spring");
        let (gen, apiary) = find(SiteKind::Apiary, &any);
        assert!(count(&apiary, &gen, &k(1.0, 12.0, 0.0)) > count(&apiary, &gen, &k(3.3, 12.0, 0.0)), "bees in summer, not winter");
        let (gen, monastery) = find(SiteKind::Monastery, &any);
        assert!(count(&monastery, &gen, &k(1.0, 10.0, 0.0)) > count(&monastery, &gen, &k(1.0, 2.0, 1.0)), "monks about by day");
        let (gen, camp) = find(SiteKind::Camp, &any);
        assert!(count(&camp, &gen, &k(1.0, 21.0, 1.0)) > count(&camp, &gen, &k(1.0, 12.0, 0.0)), "travellers by the fire at night");
        let (gen, fort) = find(SiteKind::HillFort, &any);
        assert!(count(&fort, &gen, &k(1.0, 12.0, 0.0)) > 0, "the fort's pennant flies");
        let (gen, ruin) = find(SiteKind::Ruin, &|s| haunted(s));
        assert_eq!(count(&ruin, &gen, &k(1.0, 12.0, 0.0)), 0, "no ghosts by day");
        assert!(count(&ruin, &gen, &k(1.0, 23.0, 1.0)) > 0, "lights over the ruin at night");
    }
}
