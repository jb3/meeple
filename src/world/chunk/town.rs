use super::*;

pub(super) fn build_town(ch: &mut Chunk, gen: &Gen, t: &Town) {
    let ground = t.level;
    let y0 = ground + 1;
    let x0 = ch.cx * CS - 1;
    let z0 = ch.cz * CS - 1;
    let (x1, z1) = (x0 + CS + 1, z0 + CS + 1);
    let near = |ax: i32, az: i32, bx: i32, bz: i32| bx >= x0 && ax <= x1 && bz >= z0 && az <= z1;

    let r = t.radius + 2;
    if near(t.cx - r, t.cz - r, t.cx + r, t.cz + r) {
        build_plaza(ch, t, y0);
        for lot in gen.lots(t).iter() {
            if near(lot.x0 - 1, lot.z0 - 1, lot.x1 + 1, lot.z1 + 1) {
                match gen.stage(lot) {
                    Stage::Done => build_lot(ch, lot, ground, t.city),
                    Stage::Plot => build_plot(ch, lot, ground),
                    Stage::Frame(p) => build_frame(ch, lot, ground, p),
                    Stage::Ruin => build_derelict(ch, lot, ground),
                    Stage::Vacant => {}
                }
            }
        }
    }
    if let (Some(l), Some(s)) = (t.rail, t.station_s()) {
        let s = s as i32;
        let (ax, az) = l.point(s as f32 - 6.0, 0.0);
        let (bx, bz) = l.point(s as f32 + 6.0, 5.0);
        let (lo_x, hi_x) = (ax.min(bx) as i32 - 2, ax.max(bx) as i32 + 2);
        let (lo_z, hi_z) = (az.min(bz) as i32 - 2, az.max(bz) as i32 + 2);
        if near(lo_x, lo_z, hi_x, hi_z) {
            build_station(ch, gen, l, s);
        }
    }
}

pub(super) fn build_plaza(ch: &mut Chunk, t: &Town, y: i32) {
    let (cx, cz) = (t.cx, t.cz);
    // fountain: stone basin (5x5, rim only at ground level) with flower planters on the
    // corners, stepping stones to a tiered central column
    for dz in -2i32..=2 {
        for dx in -2i32..=2 {
            let edge = dx.abs() == 2 || dz.abs() == 2;
            let b = if !edge {
                WATER
            } else if dx.abs() == 2 && dz.abs() == 2 {
                COBBLE
            } else {
                STONEBRICK
            };
            ch.set(cx + dx, y, cz + dz, b);
        }
    }
    for (i, (dx, dz)) in [(-2, -2), (2, -2), (2, 2), (-2, 2)].into_iter().enumerate() {
        ch.set(cx + dx, y + 1, cz + dz, STONEBRICK);
        ch.set(cx + dx, y + 2, cz + dz, [FLOWER_R, FLOWER_Y, FLOWER_B, POPPY][i]);
    }
    for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        ch.set(cx + dx, y, cz + dz, COBBLE);
    }
    ch.set(cx, y, cz, STONEBRICK);
    ch.set(cx, y + 1, cz, STONEBRICK);
    ch.set(cx, y + 2, cz, COBBLE);
    if ch.core(cx, cz) {
        ch.feat.fountains.push([cx as f32 + 0.5, y as f32 + 3.0, cz as f32 + 0.5]);
    }
    for (dx, dz) in [(-5, -3), (5, 3), (-3, 5), (3, -5)] {
        ch.set(cx + dx, y, cz + dz, BENCH);
    }
    for (dx, dz) in [(-4, -4), (4, 4), (-4, 4), (4, -4)] {
        ch.fill_box(cx + dx, y, cz + dz, cx + dx, y + 2, cz + dz, LAMPPOST);
        ch.lamp(cx + dx, y + 3, cz + dz);
    }
    for (dx, dz) in [(-7, -7), (7, 7), (-7, 7), (7, -7)] {
        oak(ch, cx + dx, y, cz + dz, t.id ^ (dx * 31 + dz) as u32);
    }
    if t.city {
        for (sx, sz, a, b) in [(-7, 3, AWNING_R, AWNING_W), (4, -8, ROOF_BLUE, AWNING_W)] {
            market_stall(ch, cx + sx, y, cz + sz, a, b);
        }
    }
}

pub(super) fn market_stall(ch: &mut Chunk, x: i32, y: i32, z: i32, a: u8, b: u8) {
    for (dx, dz) in [(0, 0), (3, 0), (0, 2), (3, 2)] {
        ch.fill_box(x + dx, y, z + dz, x + dx, y + 1, z + dz, FENCE);
    }
    for dx in 0..=3 {
        for dz in 0..=2 {
            ch.set(x + dx, y + 2, z + dz, if dx % 2 == 0 { a } else { b });
        }
    }
    ch.set(x + 1, y, z + 1, HAY);
    ch.set(x + 2, y, z + 1, PUMPKIN);
}

pub(super) fn build_station(ch: &mut Chunk, gen: &Gen, l: RailLine, s: i32) {
    let bed = gen.rail_bed(l, s as f32);
    // sample the curve at half steps so a bending platform has no gaps
    let cell = |a: f32, p: f32| {
        let (x, z) = l.point(a, p);
        (x.floor() as i32, z.floor() as i32)
    };
    let rows = [2.2, 2.7, 3.2, 3.7];
    let span = |from: i32, to: i32| (from * 2..=to * 2 + 1).map(|k| k as f32 * 0.5);
    // platform between street lines: s-4 ..= s+3
    for a in span(s - 4, s + 3) {
        for p in rows {
            let (x, z) = cell(a, p);
            ch.set(x, bed, z, STONEBRICK);
            ch.set(x, bed + 1, z, PLATFORM);
            for y in bed + 2..bed + 6 {
                ch.set(x, y, z, AIR);
            }
        }
    }
    for a in [s - 2, s + 1] {
        let (x, z) = cell(a as f32 + 0.5, 3.5);
        ch.fill_box(x, bed + 2, z, x, bed + 3, z, TIMBER);
    }
    // a short canopy over the middle, leaving the ends open to the sky (and
    // the camera, so passengers can be seen waiting)
    for a in span(s - 2, s + 1) {
        for p in rows {
            let (x, z) = cell(a, p);
            ch.set(x, bed + 4, z, ROOF_RED);
        }
    }
    let (x, z) = cell(s as f32 + 0.5, 3.5);
    ch.set(x, bed + 2, z, BENCH);
    for a in [s - 4, s + 3] {
        let (x, z) = cell(a as f32 + 0.5, 3.5);
        ch.fill_box(x, bed + 2, z, x, bed + 3, z, LAMPPOST);
        ch.lamp(x, bed + 4, z);
    }
}
