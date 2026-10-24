//! Working buildings: mills, the manor, quarry, mine, lighthouse and harbour.

use super::*;

pub(in crate::world::chunk) fn build_windmill(ch: &mut Chunk, st: &Site) {
    let (x, z, y0) = (st.x, st.z, st.level + 1);
    // face east or south, where the camera usually looks from
    let face = 1 + (st.seed % 2) as u8;
    let (ox, oz) = site_out(face);
    ch.fill_box(x - 2, y0 - 1, z - 2, x + 2, y0 - 1, z + 2, STONEBRICK);
    ch.fill_box(x - 2, y0, z - 2, x + 2, y0 + 1, z + 2, STONEBRICK);
    ch.fill_box(x - 1, y0 + 2, z - 1, x + 1, y0 + 8, z + 1, PLASTER);
    for (wy, (wx, wz)) in [(y0 + 4, (1, 0)), (y0 + 6, (0, 1)), (y0 + 5, (-1, 0))] {
        ch.set(x + wx, wy, z + wz, WINDOW);
    }
    ch.fill_box(x - 1, y0 + 9, z - 1, x + 1, y0 + 9, z + 1, ROOF_BROWN);
    ch.set(x, y0 + 10, z, ROOF_BROWN);
    ch.set(x + ox * 2, y0, z + oz * 2, DOOR);
    ch.set(x + ox * 2, y0 + 1, z + oz * 2, DOOR);
    if ch.core(x, z) {
        let hub = [x as f32 + 0.5 + ox as f32 * 1.7, y0 as f32 + 7.5, z as f32 + 0.5 + oz as f32 * 1.7];
        let yaw = (ox as f32).atan2(oz as f32);
        ch.feat.sails.push((hub, yaw));
    }
}

pub(in crate::world::chunk) fn build_manor(ch: &mut Chunk, st: &Site) {
    let (x, z, ground) = (st.x, st.z, st.level);
    let y0 = ground + 1;
    // garden wall with a gate at the south, a gravel drive, clipped bushes
    let (wx0, wz0, wx1, wz1) = (x - 9, z - 7, x + 8, z + 7);
    for zz in wz0..=wz1 {
        for xx in wx0..=wx1 {
            let edge = xx == wx0 || xx == wx1 || zz == wz0 || zz == wz1;
            let gate = zz == wz1 && (x - 1..=x + 1).contains(&xx);
            if edge && !gate {
                ch.set(xx, y0, zz, STONEBRICK);
            }
        }
    }
    for zz in z + 4..=wz1 {
        for xx in x - 1..=x + 1 {
            ch.set(xx, ground, zz, GRAVEL);
        }
    }
    for (bx, bz) in [(wx0 + 1, wz0 + 1), (wx1 - 1, wz0 + 1), (wx0 + 1, wz1 - 1), (wx1 - 1, wz1 - 1), (x - 3, wz1 - 1), (x + 3, wz1 - 1)] {
        ch.set(bx, y0, bz, BUSH);
    }
    for gx in [x - 2, x + 2] {
        ch.fill_box(gx, y0, wz1, gx, y0 + 1, wz1, STONEBRICK);
        ch.lamp(gx, y0 + 2, wz1);
    }
    let wall = if st.seed % 2 == 0 { PLASTER } else { STONEBRICK };
    let mut lot = Lot {
        kind: LotKind::Manor,
        x0: x - 5,
        z0: z - 4,
        x1: x + 4,
        z1: z + 3,
        face: 2,
        door: (x, z + 3),
        front: (x as f32 + 0.5, z as f32 + 4.5),
        street: (x as f32 + 0.5, z as f32 + 6.5),
        floors: 2,
        wall,
        roof: ROOF_SLATE,
        seed: st.seed,
    };
    lot.door = (x, lot.z1);
    build_lot(ch, &lot, ground, false);
}

pub(in crate::world::chunk) fn build_watermill(ch: &mut Chunk, st: &Site) {
    let (x, z, ground) = (st.x, st.z, st.level);
    let (ox, oz) = site_out(st.face);
    let mut lot = Lot {
        kind: LotKind::House,
        x0: x - 3,
        z0: z - 3,
        x1: x + 3,
        z1: z + 3,
        face: (st.face + 2) % 4,
        door: (0, 0),
        front: (0.0, 0.0),
        street: (0.0, 0.0),
        floors: 2,
        wall: if st.seed % 2 == 0 { WALLSTONE } else { PLANK },
        roof: ROOF_BROWN,
        seed: st.seed,
    };
    // door on the landward side
    let (dx, dz) = site_out(lot.face);
    lot.door = (x + dx * 2, z + dz * 2);
    build_lot(ch, &lot, ground, false);
    if ch.core(x, z) {
        let hub = [x as f32 + 0.5 + ox as f32 * 4.6, (SEA + 1) as f32 + 0.6, z as f32 + 0.5 + oz as f32 * 4.6];
        // the wheel turns alongside the bank, face-on to the river, its
        // axle running out from the mill
        let yaw = (ox as f32).atan2(oz as f32);
        ch.feat.wheels.push((hub, yaw));
    }
}

pub(in crate::world::chunk) fn build_quarry(ch: &mut Chunk, st: &Site, ground: &dyn Fn(i32, i32) -> i32) {
    let (x, z) = (st.x, st.z);
    let r = st.r as i32;
    // a timber crane on the rim with a block on its rope
    let (cx, cz) = (x + r - 1, z);
    if ch.core_or_pad(cx, cz) {
        let g = ground(cx, cz);
        ch.fill_box(cx, g + 1, cz, cx, g + 5, cz, LOG);
        ch.fill_box(cx - 4, g + 5, cz, cx - 1, g + 5, cz, PLANK);
        ch.set(cx - 4, g + 4, cz, LAMPPOST);
        ch.set(cx - 4, g + 3, cz, STONE);
    }
    // a shed and stacked blocks on the other side
    let (sx, sz) = (x - r - 1, z - 2);
    if ch.core_or_pad(sx, sz) {
        let g = ground(sx, sz);
        ch.fill_box(sx - 1, g + 1, sz - 1, sx + 1, g + 2, sz + 1, PLANK);
        ch.fill_box(sx, g + 1, sz, sx, g + 2, sz, AIR);
        ch.set(sx + 1, g + 1, sz, DOOR);
        ch.fill_box(sx - 1, g + 3, sz - 1, sx + 1, g + 3, sz + 1, ROOF_BROWN);
    }
    let (bx, bz) = (x - 2, z + r);
    if ch.core_or_pad(bx, bz) {
        let g = ground(bx, bz);
        ch.fill_box(bx, g + 1, bz, bx + 1, g + 2, bz, STONE);
        ch.set(bx + 2, g + 1, bz, STONE);
    }
}

pub(in crate::world::chunk) fn build_lighthouse(ch: &mut Chunk, st: &Site) {
    let (x, z, y0) = (st.x, st.z, st.level + 1);
    ch.fill_box(x - 2, y0 - 1, z - 2, x + 2, y0 + 1, z + 2, STONEBRICK);
    for y in y0 + 2..=y0 + 11 {
        let band = if (y - y0) / 2 % 2 == 0 { PLASTER } else { ROOF_RED };
        ch.fill_box(x - 1, y, z - 1, x + 1, y, z + 1, band);
    }
    ch.set(x, y0 + 4, z + 1, WINDOW);
    ch.set(x + 1, y0 + 8, z, WINDOW);
    // gallery, lantern room and cap
    ch.fill_box(x - 2, y0 + 12, z - 2, x + 2, y0 + 12, z + 2, PLANK);
    for (fx, fz) in [(-2, -2), (2, -2), (-2, 2), (2, 2), (0, -2), (0, 2), (-2, 0), (2, 0)] {
        ch.set(x + fx, y0 + 13, z + fz, FENCE);
    }
    ch.fill_box(x - 1, y0 + 13, z - 1, x + 1, y0 + 14, z + 1, WINDOW);
    ch.set(x, y0 + 13, z, AIR);
    ch.set(x, y0 + 14, z, AIR);
    ch.lamp(x, y0 + 13, z);
    ch.fill_box(x - 1, y0 + 15, z - 1, x + 1, y0 + 15, z + 1, ROOF_RED);
    ch.set(x, y0 + 16, z, ROOF_RED);
    if ch.core(x, z) {
        ch.feat.beacons.push([x as f32 + 0.5, (y0 + 13) as f32 + 0.4, z as f32 + 0.5]);
    }
}

pub(in crate::world::chunk) fn build_harbour(ch: &mut Chunk, st: &Site) {
    let (ox, oz) = site_out(st.face);
    let (sx, sz) = (-oz, ox);
    let deck = SEA + 1;
    let len = 13;
    for k in 0..=len {
        for w in 0..2 {
            let (x, z) = (st.x + ox * k + sx * w, st.z + oz * k + sz * w);
            ch.set(x, deck, z, PLANK);
            for y in deck + 1..deck + 4 {
                ch.set(x, y, z, AIR);
            }
            if k % 3 == 0 && k > 1 {
                for y in SEA - 4..deck {
                    ch.set(x, y, z, LOG);
                }
            }
        }
    }
    let (ex, ez) = (st.x + ox * len, st.z + oz * len);
    ch.fill_box(ex, deck + 1, ez, ex, deck + 2, ez, LAMPPOST);
    ch.lamp(ex, deck + 3, ez);
    ch.set(st.x + ox * (len - 2) + sx, deck + 1, st.z + oz * (len - 2) + sz, HAY);
    if ch.core(ex, ez) {
        ch.feat.piers.push([ex as f32 + 0.5 + ox as f32 * 2.0, SEA as f32 + 0.9, ez as f32 + 0.5 + oz as f32 * 2.0]);
    }
}

/// Where a point `u` along the site's outward direction and `v` to its
/// right lies, in voxels.
pub(crate) fn mine_at(st: &Site, u: i32, v: i32) -> (i32, i32) {
    let (ox, oz) = site_out(st.face);
    (st.x + ox * u - oz * v, st.z + oz * u + ox * v)
}

pub(in crate::world::chunk) fn build_mine(ch: &mut Chunk, st: &Site) {
    let y0 = st.level + 1;
    let g = st.level;
    let at = |u: i32, v: i32| mine_at(st, u, v);
    let fill = |ch: &mut Chunk, u0: i32, v0: i32, u1: i32, v1: i32, y_a: i32, y_b: i32, b: u8| {
        for u in u0.min(u1)..=u0.max(u1) {
            for v in v0.min(v1)..=v0.max(v1) {
                let (x, z) = at(u, v);
                for y in y_a..=y_b {
                    ch.set(x, y, z, b);
                }
            }
        }
    };
    // trodden gravel yard
    fill(ch, -8, -6, 10, 6, g, g, GRAVEL);
    // engine house: stone walls, slate roof, door towards the track
    fill(ch, -3, -3, 3, 3, y0, y0 + 3, STONEBRICK);
    fill(ch, -2, -2, 2, 2, y0, y0 + 2, AIR);
    fill(ch, -3, -3, 3, 3, y0 + 4, y0 + 4, ROOF_SLATE);
    fill(ch, -2, -2, 2, 2, y0 + 4, y0 + 4, ROOF_SLATE);
    fill(ch, 3, 0, 3, 0, y0, y0 + 1, DOOR);
    for v in [-3, 3] {
        let (x, z) = at(0, v);
        ch.set(x, y0 + 1, z, WINDOW);
    }
    let (cx, cz) = at(-2, 2);
    ch.fill_box(cx, y0 + 5, cz, cx, y0 + 8, cz, STONEBRICK);
    // the headframe over the shaft: two timber towers either side of the
    // wheel, joined by beams at the top and braced; the wheel turns in `wonders`
    for v in [-2, 2] {
        for u in [-11, -5] {
            fill(ch, u, v, u, v, y0, y0 + 10, LOG);
        }
        fill(ch, -11, v, -5, v, y0 + 10, y0 + 10, LOG);
        fill(ch, -11, v, -5, v, y0 + 4, y0 + 4, PLANK);
    }
    for u in [-11, -5] {
        fill(ch, u, -2, u, 2, y0 + 10, y0 + 10, LOG);
    }
    // the axle the wheel turns on, and a rail round the shaft mouth
    fill(ch, -8, -2, -8, 2, y0 + 7, y0 + 7, FENCE);
    fill(ch, -9, -2, -7, 2, y0, y0, GRAVEL);
    for (u, v) in [(-9, -1), (-9, 0), (-9, 1), (-7, -1), (-7, 0), (-7, 1), (-8, -1), (-8, 1)] {
        fill(ch, u, v, u, v, y0, y0, FENCE);
    }
    // the tunnel: a rock face with the mouth cut into it and a timber arch
    fill(ch, 9, -5, 13, 5, y0, y0 + 5, STONE);
    fill(ch, 9, -1, 13, 1, y0, y0 + 2, AIR);
    fill(ch, 8, -2, 8, -2, y0, y0 + 3, LOG);
    fill(ch, 8, 2, 8, 2, y0, y0 + 3, LOG);
    fill(ch, 8, -2, 8, 2, y0 + 3, y0 + 3, LOG);
    let (lx, lz) = at(8, 0);
    ch.lamp(lx, y0 + 2, lz);
    // the track from tunnel to engine house, and a spoil heap beside it
    fill(ch, 4, 0, 12, 0, g, g, PLANK);
    for k in 0..4 {
        fill(ch, 3 + k, 4 + k, 7 - k, 8 - k, y0 + k, y0 + k, if k % 2 == 0 { STONE } else { DIRT });
    }
    // a lantern by the door for the night shift
    let (dx, dz) = at(4, 2);
    ch.lamp(dx, y0 + 2, dz);
    ch.fill_box(dx, y0, dz, dx, y0 + 1, dz, LAMPPOST);
}
