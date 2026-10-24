//! Landmarks out in the country: ruins, standing stones, chalk figures,
//! geysers and oases, ski resorts.

use super::*;

pub(in crate::world::chunk) fn build_ruin(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z) = (st.x, st.z);
    let (x0, z0, x1, z1) = (x - 3, z - 2, x + 3, z + 2);
    for zz in z0..=z1 {
        for xx in x0..=x1 {
            if !(xx == x0 || xx == x1 || zz == z0 || zz == z1) || !ch.core_or_pad(xx, zz) {
                continue;
            }
            let h = hash3(st.seed, xx, 0, zz);
            let tall = (h % 4) as i32;
            let g = ground(xx, zz);
            for y in g + 1..=g + tall {
                ch.set(xx, y, zz, if (h >> 4) % 3 == 0 { STONEBRICK } else { COBBLE });
            }
            if tall > 0 && (h >> 8) % 3 == 0 {
                ch.set(xx, g + tall + 1, zz, BUSH);
            }
        }
    }
    // a broken arch still standing on one side
    let g = ground(x, z0);
    for px in [x - 1, x + 1] {
        ch.fill_box(px, g + 1, z0, px, g + 3, z0, STONEBRICK);
    }
    ch.fill_box(x - 1, g + 4, z0, x + 1, g + 4, z0, STONEBRICK);
    ch.set(x, g + 1, z0, AIR);
    // fallen stones
    for k in 0..4u32 {
        let h = hash_u32(st.seed ^ (k * 77));
        let (fx, fz) = (x - 5 + (h % 11) as i32, z - 4 + ((h >> 8) % 9) as i32);
        if ch.core_or_pad(fx, fz) {
            ch.set_air(fx, ground(fx, fz) + 1, fz, COBBLE);
        }
    }
}

/// Twelve standing stones round an altar stone, two of them capped with a
/// lintel where the midsummer sun comes up.
pub(in crate::world::chunk) fn build_stone_circle(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z) = (st.x, st.z);
    for k in 0..12 {
        let a = k as f32 * std::f32::consts::TAU / 12.0;
        let (sx, sz) = (x + (a.cos() * 6.0).round() as i32, z + (a.sin() * 6.0).round() as i32);
        if !ch.core_or_pad(sx, sz) {
            continue;
        }
        let g = ground(sx, sz);
        let tall = 3 + (hash_u32(st.seed ^ k as u32) % 2) as i32;
        let (wx, wz) = if k % 2 == 0 { (1, 0) } else { (0, 1) };
        for y in g + 1..=g + tall {
            ch.set(sx, y, sz, STONE);
            if hash_u32(st.seed ^ (k as u32 * 7)) % 3 == 0 {
                ch.set(sx + wx, y.min(g + tall - 1), sz + wz, STONE);
            }
        }
        if k % 4 == 0 {
            ch.set(sx, g + tall + 1, sz, COBBLE);
        }
    }
    // the trilithon on the sunrise side
    let g = ground(x + 6, z);
    for dz in [-1, 1] {
        if ch.core_or_pad(x + 6, z + dz * 2) {
            let gg = ground(x + 6, z + dz * 2);
            ch.fill_box(x + 6, gg + 1, z + dz * 2, x + 6, gg + 4, z + dz * 2, STONE);
        }
    }
    let _ = g;
    for dz in -2..=2 {
        if ch.core_or_pad(x + 6, z + dz) {
            ch.set(x + 6, ground(x + 6, z + 2) + 5, z + dz, STONEBRICK);
        }
    }
    for (dx, dz) in [(0, 0), (1, 0)] {
        if ch.core_or_pad(x + dx, z + dz) {
            let gg = ground(x + dx, z + dz);
            ch.set(x + dx, gg + 1, z + dz, STONEBRICK);
        }
    }
}

/// A figure cut into the turf, seen from above: white chalk where the
/// grass is stripped away.
pub(in crate::world::chunk) fn build_chalk_figure(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    const FIGURE: [&str; 15] = [
        ".....###.....",
        "....#####....",
        "....#####....",
        ".....###.....",
        "..#########..",
        ".##.#####.##.",
        "##..#####..##",
        "#...#####...#",
        "....#####....",
        "....#####....",
        "...##...##...",
        "...##...##...",
        "..##.....##..",
        "..##.....##..",
        ".##.......##.",
    ];
    let (ox, oz) = (st.x - 13, st.z - 15);
    for (j, row) in FIGURE.iter().enumerate() {
        for (i, c) in row.chars().enumerate() {
            if c != '#' {
                continue;
            }
            for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (x, z) = (ox + i as i32 * 2 + dx, oz + j as i32 * 2 + dz);
                if ch.core(x, z) {
                    ch.set(x, ground(x, z), z, PLASTER);
                }
            }
        }
    }
}

/// A round pool sunk into level ground. A geyser has a rim of rock and
/// steams; an oasis has palms and reeds.
pub(in crate::world::chunk) fn build_pool(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32, geyser: bool) {
    let (x, z) = (st.x, st.z);
    for dz in -7..=7 {
        for dx in -7..=7 {
            let (px, pz) = (x + dx, z + dz);
            if !ch.core(px, pz) {
                continue;
            }
            let d = Vec2::new(dx as f32 + 0.5 - 0.5, dz as f32).length();
            let g = ground(px, pz);
            if d < 4.4 {
                for y in g + 1..=st.level {
                    ch.set(px, y, pz, WATER);
                }
            } else if d < 5.6 {
                let h = hash3(st.seed, px, 0, pz);
                if geyser {
                    ch.set(px, g, pz, if h % 3 == 0 { COBBLE } else { STONE });
                    if h % 4 == 0 {
                        ch.set(px, g + 1, pz, STONE);
                    }
                } else {
                    ch.set(px, g, pz, GRASS);
                    if h % 2 == 0 {
                        ch.set(px, g + 1, pz, TALLGRASS);
                    }
                }
            }
        }
    }
    if geyser {
        if ch.core(x, z) {
            ch.feat.steams.push([x as f32 + 0.5, st.level as f32 + 1.2, z as f32 + 0.5]);
        }
        return;
    }
    // palms leaning over the water
    for k in 0..6 {
        let a = k as f32 * std::f32::consts::TAU / 6.0 + unit(st.seed) * 2.0;
        let (px, pz) = (x + (a.cos() * 6.4).round() as i32, z + (a.sin() * 6.4).round() as i32);
        if !ch.core_or_pad(px, pz) {
            continue;
        }
        let g = ground(px, pz);
        let th = 5 + (hash_u32(st.seed ^ k as u32) % 3) as i32;
        for i in 1..=th {
            ch.set(px, g + i, pz, LOG);
        }
        let top = g + th + 1;
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for l in 1..=3 {
                ch.set_air(px + dx * l, top - (l == 3) as i32, pz + dz * l, LEAVES);
            }
        }
        ch.set_air(px, top, pz, LEAVES);
        ch.set_air(px + 1, top, pz + 1, LEAVES);
        ch.set_air(px - 1, top, pz - 1, LEAVES);
    }
}

/// A few chalets, a lodge with a chimney, a lamp or two.
/// Level a footprint for a building: fill the low cells up to `g` and cut the
/// high ones down, then lay a plank floor.
pub(in crate::world::chunk) fn terrace(ch: &mut Chunk, cells: &[(i32, i32)], g: i32, ground: &dyn Fn(i32, i32) -> i32) {
    for &(x, z) in cells {
        if !ch.core_or_pad(x, z) {
            continue;
        }
        let cg = ground(x, z);
        for y in cg + 1..=g {
            ch.set(x, y, z, STONEBRICK);
        }
        for y in g + 1..=cg {
            ch.set(x, y, z, AIR);
        }
        ch.set(x, g, z, PLANK);
    }
}

/// A ski resort, premade: the terrain is already laid as an even slope (see
/// `Gen::column`); here goes the groomed piste, the lift's two stations, and
/// the village of chalets round a plaza at the foot, all in the run's own frame
/// (`u` up the slope from the middle, `v` across it).
pub(in crate::world::chunk) fn build_ski_resort(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (ox, oz) = {
        let o = site_out(st.face);
        (o.0, o.1)
    };
    let cell = |u: i32, v: i32| (st.x + ox * u - oz * v, st.z + oz * u + ox * v);
    let half = SKI_HALF_LEN as i32;
    // the groomed piste: packed snow all down the middle of the run, fenced either
    // side; cobbles on the plaza at the foot of it
    for u in -half..=half {
        for v in -(SKI_HALF_WIDTH as i32)..=SKI_HALF_WIDTH as i32 {
            let (x, z) = cell(u, v);
            if !ch.core_or_pad(x, z) {
                continue;
            }
            let g = ground(x, z);
            if v.abs() <= 9 && u >= -30 {
                ch.set(x, g, z, SNOW);
            } else if v.abs() <= 8 && u < -30 {
                ch.set(x, g, z, COBBLE);
            }
            if v.abs() == 11 && (-30..=40).contains(&u) && u % 2 == 0 {
                ch.set(x, g + 1, z, FENCE);
            }
        }
    }
    // lamp posts along the plaza
    for (u, v) in [(-41, -9), (-41, 9), (-35, -9), (-35, 9), (-29, -9), (-29, 9)] {
        let (x, z) = cell(u, v);
        if ch.core(x, z) {
            let g = ground(x, z);
            ch.fill_box(x, g + 1, z, x, g + 3, z, LAMPPOST);
            ch.lamp(x, g + 4, z);
        }
    }
    // chalets either side of the plaza and up the first of the slope, doors
    // turned to face the middle of the run
    let across = (-oz, ox);
    for (k, (u, v)) in [(-37, -13), (-37, 13), (-26, -14), (-26, 14), (-15, -13), (-15, 13)].into_iter().enumerate() {
        let (cx, cz) = cell(u, v);
        let dir = if v < 0 { 1 } else { -1 };
        let (door_x, door_z) = (across.0 * dir, across.1 * dir);
        let cells: Vec<(i32, i32)> = (-3..=3).flat_map(|dz| (-3..=3).map(move |dx| (cx + dx, cz + dz))).collect();
        let g = ground(cx, cz);
        terrace(ch, &cells, g, ground);
        let y0 = g + 1;
        let (w, d) = (2, 2);
        for zz in cz - d..=cz + d {
            for xx in cx - w..=cx + w {
                let edge = xx == cx - w || xx == cx + w || zz == cz - d || zz == cz + d;
                if edge {
                    for y in y0..y0 + 3 {
                        let corner = (xx == cx - w || xx == cx + w) && (zz == cz - d || zz == cz + d);
                        let win = y == y0 + 1 && !corner && (xx + zz).rem_euclid(2) == 0;
                        ch.set(xx, y, zz, if win { WINDOW } else { LOG });
                    }
                }
            }
        }
        ch.set(cx + door_x * w, y0, cz + door_z * d, DOOR);
        ch.set(cx + door_x * w, y0 + 1, cz + door_z * d, DOOR);
        // a steep roof: layers narrowing towards a ridge
        for l in 0..3 {
            ch.fill_box(cx - w - 1, y0 + 3 + l, cz - d + l, cx + w + 1, y0 + 3 + l, cz + d - l, if l == 2 { ROOF_BROWN } else { ROOF_RED });
        }
        if k % 2 == 0 {
            ch.fill_box(cx + w - 1, y0 + 3, cz, cx + w - 1, y0 + 6, cz, STONEBRICK);
            ch.chimney(cx + w - 1, y0 + 7, cz);
        }
    }
    // the lift's stations: a little hut at the foot and another at the top
    for u in [-36, 37] {
        let (cx, cz) = cell(u, 0);
        let cells: Vec<(i32, i32)> = (-2..=2).flat_map(|dz| (-2..=2).map(move |dx| (cx + dx, cz + dz))).collect();
        let g = ground(cx, cz);
        terrace(ch, &cells, g, ground);
        for zz in cz - 1..=cz + 1 {
            for xx in cx - 1..=cx + 1 {
                if xx == cx && zz == cz {
                    continue;
                }
                ch.fill_box(xx, g + 1, zz, xx, g + 3, zz, PLANK);
            }
        }
        ch.fill_box(cx - 2, g + 4, cz - 2, cx + 2, g + 4, cz + 2, ROOF_RED);
        ch.lamp(cx, g + 5, cz);
    }
}
