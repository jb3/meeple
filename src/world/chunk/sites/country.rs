//! Quiet places in the country: orchards, apiaries, monasteries, shrines,
//! hill forts and travellers' camps.

use super::*;

/// The trees of an orchard, in rows, as offsets from its centre.
pub(crate) fn orchard_trees(st: &Site) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for iz in -2..=2 {
        for ix in -2..=2 {
            // a gap for the gate and path down the middle column
            if ix == 0 && iz > 0 {
                continue;
            }
            let h = hash_u32(st.seed ^ ((ix + 3) * 31 + iz + 3) as u32);
            if h % 7 != 0 {
                out.push((st.x + (ix as f32 * 3.5).round() as i32 + (h >> 8) as i32 % 2, st.z + (iz as f32 * 3.5).round() as i32));
            }
        }
    }
    out
}

pub(in crate::world::chunk) fn build_orchard(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z) = (st.x, st.z);
    // a low fence round it, with a gap at the south for the gate
    for k in -8..=8 {
        for (fx, fz) in [(x + k, z - 8), (x + k, z + 8), (x - 8, z + k), (x + 8, z + k)] {
            let gate = fz == z + 8 && (x - 1..=x + 1).contains(&fx);
            if !gate && ch.core_or_pad(fx, fz) {
                ch.set(fx, ground(fx, fz) + 1, fz, FENCE);
            }
        }
    }
    for gz in z + 1..=z + 8 {
        if ch.core_or_pad(x, gz) {
            ch.set(x, ground(x, gz), gz, PATH);
        }
    }
    for (tx, tz) in orchard_trees(st) {
        if !ch.core_or_pad(tx, tz) {
            continue;
        }
        let g = ground(tx, tz);
        ch.fill_box(tx, g + 1, tz, tx, g + 2, tz, LOG);
        ch.fill_box(tx - 1, g + 3, tz - 1, tx + 1, g + 4, tz + 1, LEAVES);
        ch.set(tx, g + 5, tz, LEAVES);
    }
    // a hay cart's worth of crates by the gate
    for (cx, cz) in [(x + 3, z + 6), (x + 4, z + 6), (x + 3, z + 7)] {
        if ch.core_or_pad(cx, cz) {
            ch.set(cx, ground(cx, cz) + 1, cz, PLANK);
        }
    }
}

/// Where an apiary's hives stand.
pub(crate) fn apiary_hives(st: &Site) -> Vec<(i32, i32)> {
    (0..6).map(|k| (st.x - 3 + (k % 3) * 3, st.z - 2 + (k / 3) * 3)).collect()
}

pub(in crate::world::chunk) fn build_apiary(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z) = (st.x, st.z);
    for (hx, hz) in apiary_hives(st) {
        if !ch.core_or_pad(hx, hz) {
            continue;
        }
        let g = ground(hx, hz);
        ch.set(hx, g + 1, hz, PLANK);
        ch.set(hx, g + 2, hz, HAY);
        ch.set(hx, g + 3, hz, ROOF_BROWN);
    }
    // a bed of flowers all round for the bees
    for k in 0..28u32 {
        let h = hash_u32(st.seed ^ k.wrapping_mul(0x9e37));
        let a = unit(h) * std::f32::consts::TAU;
        let d = 4.4 + unit(h >> 9) * 1.4;
        let (fx, fz) = (x + (a.cos() * d).round() as i32, z + (a.sin() * d).round() as i32);
        if ch.core_or_pad(fx, fz) {
            ch.set_air(fx, ground(fx, fz) + 1, fz, [FLOWER_Y, FLOWER_B, FLOWER_R, FLOWER_Y][(h >> 17) as usize % 4]);
        }
    }
    // the beekeeper's shed and a honey stall
    for sz in 3..=5 {
        for sx in 4..=6 {
            let (wx, wz) = (x + sx, z + sz);
            if ch.core_or_pad(wx, wz) {
                let g = ground(wx, wz);
                let wall = !(sx == 5 && sz == 4);
                if wall {
                    ch.fill_box(wx, g + 1, wz, wx, g + 2, wz, PLANK);
                }
                ch.set(wx, g + 3, wz, ROOF_RED);
            }
        }
    }
    ch.set(x + 5, ground(x + 5, z + 3) + 2, z + 3, WINDOW);
    for tx in [x - 3, x - 2] {
        if ch.core_or_pad(tx, z + 5) {
            let g = ground(tx, z + 5);
            ch.set(tx, g + 1, z + 5, PLANK);
            ch.set(tx, g + 3, z + 5, AWNING_W);
        }
    }
}

/// The belfry of a monastery.
pub(in crate::world::chunk) fn build_monastery(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z, g) = (st.x, st.z, st.level);
    let y0 = g + 1;
    let (x0, z0, x1, z1) = (x - 8, z - 6, x + 8, z + 6);
    // cloister wall with a gate at the south
    for zz in z0..=z1 {
        for xx in x0..=x1 {
            let edge = xx == x0 || xx == x1 || zz == z0 || zz == z1;
            let gate = zz == z1 && (x - 1..=x + 1).contains(&xx);
            if edge && !gate {
                ch.fill_box(xx, y0, zz, xx, y0 + 2, zz, PLASTER);
                ch.set(xx, y0 + 3, zz, ROOF_BROWN);
            } else if !edge {
                ch.set(xx, g, zz, COBBLE);
            }
        }
    }
    // the church along the north wall
    ch.fill_box(x - 4, y0, z0 + 1, x + 4, y0 + 4, z0 + 5, STONEBRICK);
    for (i, y) in (y0 + 5..=y0 + 7).enumerate() {
        ch.fill_box(x - 4, y, z0 + 1 + i as i32, x + 4, y, z0 + 5 - i as i32, ROOF_SLATE);
    }
    ch.set(x, y0, z0 + 5, DOOR);
    ch.set(x, y0 + 1, z0 + 5, DOOR);
    for wx in [x - 3, x - 2, x + 2, x + 3] {
        ch.set(wx, y0 + 2, z0 + 5, WINDOW);
    }
    // the bell tower beside it
    let (tx, tz) = (x + 6, z0 + 1);
    ch.fill_box(tx - 1, y0, tz, tx + 1, y0 + 6, tz + 2, STONEBRICK);
    for (px, pz) in [(tx - 1, tz), (tx + 1, tz), (tx - 1, tz + 2), (tx + 1, tz + 2)] {
        ch.fill_box(px, y0 + 7, pz, px, y0 + 8, pz, STONEBRICK);
    }
    ch.fill_box(tx - 1, y0 + 9, tz, tx + 1, y0 + 9, tz + 2, ROOF_SLATE);
    ch.set(tx, y0 + 10, tz + 1, ROOF_SLATE);
    ch.set(tx, y0 + 3, tz + 3, WINDOW);
    if ch.core(tx, tz + 1) {
        ch.feat.bells.push([tx as f32 + 0.5, (y0 + 7) as f32 + 0.8, (tz + 1) as f32 + 0.5]);
    }
    // monks' cells down the east side, a vegetable garden to the west
    ch.fill_box(x + 5, y0, z - 1, x + 7, y0 + 2, z + 4, PLASTER);
    ch.fill_box(x + 5, y0 + 3, z - 1, x + 7, y0 + 3, z + 4, ROOF_BROWN);
    ch.set(x + 5, y0, z + 1, DOOR);
    ch.set(x + 5, y0 + 1, z + 1, DOOR);
    for zz in z - 1..=z + 4 {
        for xx in x - 7..=x - 2 {
            ch.set(xx, g, zz, SOIL);
            ch.set(xx, y0, zz, if (xx + zz) % 3 == 0 { BUSH } else { CROP });
        }
    }
    // a well in the courtyard and lamps at the gate
    ch.fill_box(x, y0, z + 2, x, y0 + 1, z + 2, STONEBRICK);
    for lx in [x - 2, x + 2] {
        ch.lamp(lx, y0 + 2, z1);
    }
    let _ = ground;
}

pub(in crate::world::chunk) fn build_shrine(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z, y0) = (st.x, st.z, st.level + 1);
    ch.fill_box(x - 1, y0, z - 1, x + 1, y0, z + 1, STONEBRICK);
    for (px, pz) in [(x - 1, z - 1), (x + 1, z - 1), (x - 1, z + 1), (x + 1, z + 1)] {
        ch.fill_box(px, y0 + 1, pz, px, y0 + 2, pz, TIMBER);
    }
    ch.fill_box(x - 1, y0 + 3, z - 1, x + 1, y0 + 3, z + 1, ROOF_RED);
    ch.set(x, y0 + 4, z, ROOF_RED);
    ch.set(x, y0 + 1, z, STONEBRICK);
    ch.lamp(x, y0 + 2, z);
    for k in 0..8u32 {
        let h = hash_u32(st.seed ^ k.wrapping_mul(77));
        let (fx, fz) = (x - 3 + (h % 7) as i32, z - 3 + ((h >> 8) % 7) as i32);
        if (fx - x).abs() > 1 || (fz - z).abs() > 1 {
            if ch.core_or_pad(fx, fz) {
                ch.set_air(fx, ground(fx, fz) + 1, fz, [FLOWER_Y, FLOWER_B, FLOWER_R][(h >> 16) as usize % 3]);
            }
        }
    }
    ch.set(x, y0, z + 3, BENCH);
}

pub(in crate::world::chunk) fn build_hill_fort(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z, y0) = (st.x, st.z, st.level + 1);
    let (ox, oz) = site_out(st.face);
    // a ring bank with a palisade on top, and a gate on the side it faces
    let r = 11.0f32;
    for dz in -12..=12 {
        for dx in -12..=12 {
            let d = ((dx * dx + dz * dz) as f32).sqrt();
            if (d - r).abs() > 0.95 {
                continue;
            }
            let (wx, wz) = (x + dx, z + dz);
            if !ch.core_or_pad(wx, wz) {
                continue;
            }
            let gate = (dx * ox + dz * oz) as f32 > d * 0.93;
            let g = ground(wx, wz);
            if gate {
                ch.set(wx, g, wz, PATH);
                continue;
            }
            let h = hash3(st.seed, wx, 0, wz);
            ch.fill_box(wx, g + 1, wz, wx, g + 2, wz, COBBLE);
            ch.set(wx, g + 3, wz, if h % 4 == 0 { AIR } else { FENCE });
        }
    }
    // gate posts
    for s in [-1, 1] {
        let (gx, gz) = (x + ox * 11 + oz.abs() * s * 2, z + oz * 11 + ox.abs() * s * 2);
        if ch.core_or_pad(gx, gz) {
            let g = ground(gx, gz);
            ch.fill_box(gx, g + 1, gz, gx, g + 4, gz, TIMBER);
            ch.lamp(gx, g + 5, gz);
        }
    }
    // the keep
    ch.fill_box(x - 2, y0 - 1, z - 2, x + 2, y0 + 7, z + 2, STONEBRICK);
    for xx in x - 2..=x + 2 {
        for zz in z - 2..=z + 2 {
            if (xx + zz).rem_euclid(2) == 0 && (xx - x).abs().max((zz - z).abs()) == 2 {
                ch.set(xx, y0 + 8, zz, STONEBRICK);
            }
        }
    }
    for (wx, wz) in [(x + 2, z), (x - 2, z), (x, z + 2), (x, z - 2)] {
        ch.set(wx, y0 + 4, wz, WINDOW);
    }
    ch.set(x + ox * 2, y0, z + oz * 2, DOOR);
    ch.set(x + ox * 2, y0 + 1, z + oz * 2, DOOR);
    ch.lamp(x, y0 + 8, z);
    // a pair of timber halls inside the wall
    for (hx, hz) in [(x - 7, z - 5), (x + 6, z + 5)] {
        ch.fill_box(hx, y0, hz, hx + 2, y0 + 1, hz + 2, PLANK);
        ch.fill_box(hx - 1, y0 + 2, hz - 1, hx + 3, y0 + 2, hz + 3, ROOF_BROWN);
        ch.set(hx + 1, y0, hz + 3, AIR);
    }
}

pub(in crate::world::chunk) fn build_camp(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z, y0) = (st.x, st.z, st.level + 1);
    // the fire with logs round it
    ch.set(x, y0, z, FORGE);
    ch.chimney(x, y0 + 1, z);
    for (lx, lz) in [(x - 2, z), (x + 2, z), (x, z + 2)] {
        ch.set(lx, y0, lz, LOG);
    }
    // two tents
    for (i, (tx, tz)) in [(x - 4, z - 3), (x + 3, z - 4)].into_iter().enumerate() {
        let c = if i == 0 { AWNING_R } else { AWNING_W };
        ch.fill_box(tx - 1, y0, tz, tx - 1, y0, tz + 2, c);
        ch.fill_box(tx + 1, y0, tz, tx + 1, y0, tz + 2, c);
        ch.fill_box(tx, y0 + 1, tz, tx, y0 + 1, tz + 2, c);
    }
    // a painted caravan on the other side
    let (cx, cz) = (x + 2, z + 4);
    if ch.core_or_pad(cx, cz) {
        ch.fill_box(cx, y0, cz, cx + 3, y0 + 1, cz + 1, PLANK);
        ch.fill_box(cx, y0 + 2, cz, cx + 3, y0 + 2, cz + 1, ROOF_GREEN);
        ch.set(cx + 1, y0 + 1, cz, WINDOW);
        ch.set(cx + 2, y0 + 1, cz + 1, WINDOW);
        for wx in [cx, cx + 3] {
            ch.set(wx, y0 - 1, cz - 1, LOG);
        }
        ch.set(cx - 1, y0, cz, FENCE);
    }
    let _ = ground;
}
