use super::*;

/// A house plot: dug footings, corner pegs, and timber and stone piled ready.
pub(super) fn build_plot(ch: &mut Chunk, lot: &Lot, ground: i32) {
    let (fx0, fz0, fx1, fz1) = lot.footprint();
    let y0 = ground + 1;
    for z in fz0..=fz1 {
        for x in fx0..=fx1 {
            let edge = x == fx0 || x == fx1 || z == fz0 || z == fz1;
            ch.set(x, ground, z, if edge { GRAVEL } else { DIRT });
        }
    }
    for (x, z) in [(fx0, fz0), (fx1, fz0), (fx0, fz1), (fx1, fz1)] {
        ch.set(x, y0, z, FENCE);
    }
    // a stack of planks and a pile of stone
    let (cx, cz) = (fx0 + 1, fz0 + 1);
    ch.fill_box(cx, y0, cz, cx + 2, y0, cz, PLANK);
    ch.set(cx + 1, y0 + 1, cz, PLANK);
    ch.fill_box(fx1 - 2, y0, fz1 - 1, fx1 - 1, y0, fz1 - 1, COBBLE);
    ch.set(fx1 - 1, y0 + 1, fz1 - 1, COBBLE);
}

/// A house left to fall down: broken walls of uneven height, ivy over the
/// tops, rubble and long grass where the floors were.
pub(super) fn build_derelict(ch: &mut Chunk, lot: &Lot, ground: i32) {
    let (fx0, fz0, fx1, fz1) = lot.footprint();
    let y0 = ground + 1;
    let wall_h = lot.floors * 3;
    let pick = |x: i32, z: i32, k: i32| hash3(lot.seed, x, z, k);
    for z in fz0..=fz1 {
        for x in fx0..=fx1 {
            let (ex, ez) = (x == fx0 || x == fx1, z == fz0 || z == fz1);
            if ex || ez {
                // each stretch of wall has stood to a different height
                let h = (pick(x, z, 0xa1) % (wall_h as u32 + 1)).min(wall_h as u32) as i32;
                let h = if (x, z) == lot.door { 0 } else { h };
                for y in 0..h {
                    ch.set(x, y0 + y, z, if pick(x, z + y, 0xa2) % 4 == 0 { COBBLE } else { lot.wall });
                }
                if h > 0 && pick(x, z, 0xa3) % 3 != 0 {
                    ch.set(x, y0 + h, z, LEAVES);
                }
            } else {
                let r = pick(x, z, 0xa4) % 10;
                if r < 2 {
                    ch.set(x, y0, z, COBBLE);
                } else if r < 5 {
                    ch.set(x, y0, z, TALLGRASS);
                } else if r == 5 {
                    ch.set(x, y0, z, BUSH);
                }
                ch.set(x, ground, z, if r < 7 { GRASS } else { DIRT });
            }
        }
    }
}

/// A house going up: the frame and walls rising (`p` of the way) inside
/// scaffolding, no roof yet.
pub(super) fn build_frame(ch: &mut Chunk, lot: &Lot, ground: i32, p: f32) {
    let (fx0, fz0, fx1, fz1) = lot.footprint();
    let y0 = ground + 1;
    let wall_h = lot.floors * 3;
    let top = y0 + wall_h - 1;
    let walls = y0 + ((wall_h as f32 * p).ceil() as i32).clamp(1, wall_h) - 1;
    for z in fz0..=fz1 {
        for x in fx0..=fx1 {
            ch.set(x, ground, z, PLANK);
        }
    }
    for y in y0..=top {
        for z in fz0..=fz1 {
            for x in fx0..=fx1 {
                let (ex, ez) = (x == fx0 || x == fx1, z == fz0 || z == fz1);
                if !(ex || ez) {
                    continue;
                }
                if ex && ez {
                    // the frame's corner posts stand full height first
                    ch.set(x, y, z, TIMBER);
                } else if y <= walls && (x, z) != lot.door {
                    let along = if ex { z - fz0 } else { x - fx0 };
                    ch.set(x, y, z, if along % 2 == 1 && (y - y0) % 3 == 1 { AIR } else { lot.wall });
                }
            }
        }
    }
    // scaffolding: poles out at the corners, a boarded walkway at each floor
    let (sx0, sz0, sx1, sz1) = (fx0 - 1, fz0 - 1, fx1 + 1, fz1 + 1);
    for (x, z) in [(sx0, sz0), (sx1, sz0), (sx0, sz1), (sx1, sz1)] {
        ch.fill_box(x, y0, z, x, top + 1, z, FENCE);
    }
    // boards only as high as the walls have got
    let mut lvl = y0 + 2;
    while lvl <= walls + 1 && lvl <= top {
        for x in sx0..=sx1 {
            for z in [sz0, sz1] {
                ch.set(x, lvl, z, PLANK);
            }
        }
        for z in sz0..=sz1 {
            for x in [sx0, sx1] {
                ch.set(x, lvl, z, PLANK);
            }
        }
        lvl += 3;
    }
    // keep the way in clear
    let (dx, dz) = lot.door;
    let (ox, oz) = lot.outward();
    ch.fill_box(dx + ox, y0, dz + oz, dx + ox, y0 + 1, dz + oz, AIR);
}

pub(super) fn build_lot(ch: &mut Chunk, lot: &Lot, ground: i32, city: bool) {
    let y0 = ground + 1;
    match lot.kind {
        LotKind::Farm => return build_farm(ch, lot, ground),
        LotKind::Garden => return build_garden(ch, lot, y0),
        _ => {}
    }
    let (fx0, fz0, fx1, fz1) = lot.footprint();
    let church = lot.kind == LotKind::Church;
    let wall_h = if church { 7 } else { lot.floors * 3 };
    let top = y0 + wall_h - 1;

    // floor + walls
    for z in fz0..=fz1 {
        for x in fx0..=fx1 {
            ch.set(x, ground, z, PLANK);
        }
    }
    let framed = matches!(lot.kind, LotKind::House | LotKind::Tavern | LotKind::Bakery | LotKind::Shop)
        && lot.wall != BRICK
        && lot.wall != STONEBRICK;
    for y in y0..=top {
        let fy = (y - y0) % 3;
        for z in fz0..=fz1 {
            for x in fx0..=fx1 {
                let ex = x == fx0 || x == fx1;
                let ez = z == fz0 || z == fz1;
                if !(ex || ez) {
                    continue;
                }
                let corner = ex && ez;
                let along = if ex { z - fz0 } else { x - fx0 };
                let b = if corner && framed {
                    TIMBER
                } else if !corner
                    && along % 2 == 1
                    && (fy == 1 || (church && (2..=4).contains(&(y - y0))) || (lot.kind == LotKind::Library && fy == 2))
                {
                    WINDOW
                } else if framed && fy == 2 && !city {
                    TIMBER
                } else {
                    lot.wall
                };
                ch.set(x, y, z, b);
            }
        }
    }
    // door (and open front for the smithy)
    let (dx, dz) = lot.door;
    if lot.kind == LotKind::Smithy {
        let (ax, az, bx, bz) = match lot.face {
            0 => (fx0 + 1, fz0, fx1 - 1, fz0),
            2 => (fx0 + 1, fz1, fx1 - 1, fz1),
            1 => (fx1, fz0 + 1, fx1, fz1 - 1),
            _ => (fx0, fz0 + 1, fx0, fz1 - 1),
        };
        ch.fill_box(ax, y0, az, bx, y0 + 1, bz, AIR);
        // the forge sits out in the open yard near the anvil, where the
        // camera can see it glow (the smith works on the anvil's other side)
        let (fx, fz) = (lot.front.0.floor() as i32, lot.front.1.floor() as i32);
        ch.set(fx, y0, fz, ANVIL);
        let (gx, gz) = lot.forge();
        ch.set(gx, y0, gz, FORGE);
        if ch.core(gx, gz) {
            ch.feat.forges.push([gx as f32 + 0.5, y0 as f32 + 0.5, gz as f32 + 0.5]);
        }
    } else {
        ch.set(dx, y0, dz, DOOR);
        ch.set(dx, y0 + 1, dz, DOOR);
        if church {
            ch.set(dx, y0 + 2, dz, DOOR);
        }
    }

    // roof
    if matches!(lot.kind, LotKind::Apartment | LotKind::Library) {
        ch.fill_box(fx0, top + 1, fz0, fx1, top + 1, fz1, ROOF_SLATE);
        for z in fz0..=fz1 {
            for x in fx0..=fx1 {
                if x == fx0 || x == fx1 || z == fz0 || z == fz1 {
                    ch.set(x, top + 2, z, lot.wall);
                }
            }
        }
        ch.set(fx0 + 2, top + 2, fz0 + 2, BRICK);
        ch.set(fx0 + 2, top + 3, fz0 + 2, BRICK);
        ch.chimney(fx0 + 2, top + 4, fz0 + 2);
    } else {
        let ridge_x = lot.face == 0 || lot.face == 2;
        let roof = if church { ROOF_SLATE } else { lot.roof };
        let (a0, a1, p0, p1) = if ridge_x {
            (fx0 - 1, fx1 + 1, fz0 - 1, fz1 + 1)
        } else {
            (fz0 - 1, fz1 + 1, fx0 - 1, fx1 + 1)
        };
        let at = |a: i32, p: i32| if ridge_x { (a, p) } else { (p, a) };
        let mut k = 0;
        loop {
            let lo = p0 + k;
            let hi = p1 - k;
            if lo > hi {
                break;
            }
            let y = top + 1 + k;
            for a in a0..=a1 {
                for p in [lo, hi] {
                    let (x, z) = at(a, p);
                    ch.set(x, y, z, roof);
                }
                if hi - lo <= 1 {
                    continue;
                }
                // gable end walls
                if a == a0 + 1 || a == a1 - 1 {
                    for p in lo + 1..hi {
                        let (x, z) = at(a, p);
                        ch.set(x, y, z, lot.wall);
                    }
                }
            }
            k += 1;
        }
        let layers = k;
        if matches!(lot.kind, LotKind::House | LotKind::Bakery | LotKind::Tavern | LotKind::Smithy | LotKind::Manor) {
            let (cx, cz) = at(a1 - 2, p0 + 2);
            let cy = top + layers + 1 + (lot.kind == LotKind::Bakery) as i32;
            ch.fill_box(cx, top + 1, cz, cx, cy, cz, BRICK);
            ch.chimney(cx, cy + 1, cz);
            if lot.kind == LotKind::Manor {
                let (cx, cz) = at(a0 + 2, p1 - 2);
                ch.fill_box(cx, top + 1, cz, cx, cy, cz, BRICK);
                ch.chimney(cx, cy + 1, cz);
            }
        }
        if lot.kind == LotKind::School {
            // a little bell cupola on the ridge
            let (cx, cz) = at((a0 + a1) / 2, (p0 + p1) / 2);
            let ry = top + layers;
            ch.fill_box(cx, ry, cz, cx, ry + 1, cz, PLANK);
            ch.set(cx, ry + 2, cz, ROOF_RED);
        }
    }

    match lot.kind {
        LotKind::Church => {
            // bell tower at the back corner
            let (tx, tz) = match lot.face {
                0 => (fx1 - 2, fz1 - 2),
                2 => (fx0, fz0),
                1 => (fx0, fz1 - 2),
                _ => (fx1 - 2, fz0),
            };
            let tt = y0 + 13;
            for y in y0..=tt {
                for z in tz..tz + 3 {
                    for x in tx..tx + 3 {
                        let edge = x == tx || x == tx + 2 || z == tz || z == tz + 2;
                        if edge {
                            let win = y >= tt - 2 && y < tt && (x == tx + 1 || z == tz + 1);
                            ch.set(x, y, z, if win { AIR } else { STONEBRICK });
                        }
                    }
                }
            }
            ch.fill_box(tx, tt + 1, tz, tx + 2, tt + 1, tz + 2, ROOF_SLATE);
            ch.set(tx + 1, tt + 2, tz + 1, ROOF_SLATE);
            ch.set(tx + 1, tt + 3, tz + 1, LAMPPOST);
        }
        LotKind::School => {
            // flagpole in the front corner of the yard
            let (fx, fz) = (lot.x0, lot.z0);
            ch.fill_box(fx, y0, fz, fx, y0 + 4, fz, FENCE);
            let (ox, oz) = if lot.face % 2 == 0 { (1, 0) } else { (0, 1) };
            ch.set(fx + ox, y0 + 4, fz + oz, AWNING_R);
            ch.set(fx + ox * 2, y0 + 4, fz + oz * 2, AWNING_W);
            ch.set(fx + ox, y0 + 3, fz + oz, AWNING_W);
            ch.set(fx + ox * 2, y0 + 3, fz + oz * 2, AWNING_R);
        }
        LotKind::Shop => {
            let (fx, fz) = lot.front;
            let (fx, fz) = (fx.floor() as i32, fz.floor() as i32);
            for i in -1..=1 {
                let (x, z) = if lot.face == 0 || lot.face == 2 { (fx + i, fz) } else { (fx, fz + i) };
                ch.set(x, y0 + 2, z, if i == 0 { AWNING_W } else { AWNING_R });
            }
            ch.set(fx + if lot.face % 2 == 0 { 2 } else { 0 }, y0, fz + if lot.face % 2 == 1 { 2 } else { 0 }, PUMPKIN);
        }
        LotKind::Tavern => {
            let (fx, fz) = lot.front;
            let (fx, fz) = (fx.floor() as i32, fz.floor() as i32);
            let side = if lot.face == 0 || lot.face == 2 { (2, 0) } else { (0, 2) };
            ch.lamp(fx + side.0, y0 + 2, fz + side.1);
            ch.lamp(fx - side.0, y0 + 2, fz - side.1);
            ch.set(fx + side.0 * 2, y0, fz + side.1 * 2, BENCH);
        }
        _ => {}
    }

    // street lamp on the lot corner
    if lot.footprint().0 > lot.x0 && hash_u32(lot.seed ^ 0x1a) % 2 == 0 {
        ch.fill_box(lot.x0, y0, lot.z0, lot.x0, y0 + 2, lot.z0, LAMPPOST);
        ch.lamp(lot.x0, y0 + 3, lot.z0);
    }
}

pub(super) fn build_farm(ch: &mut Chunk, lot: &Lot, ground: i32) {
    let y0 = ground + 1;
    let rows_x = lot.face == 1 || lot.face == 3;
    let pumpkins = hash_u32(lot.seed ^ 0x9f) % 4 == 0;
    let (gx, gz) = lot.door;
    for z in lot.z0..=lot.z1 {
        for x in lot.x0..=lot.x1 {
            let edge = x == lot.x0 || x == lot.x1 || z == lot.z0 || z == lot.z1;
            if edge {
                let gate = if lot.face % 2 == 0 { (x - gx).abs() <= 1 } else { (z - gz).abs() <= 1 };
                let on_face = match lot.face {
                    0 => z == lot.z0,
                    2 => z == lot.z1,
                    1 => x == lot.x1,
                    _ => x == lot.x0,
                };
                if !(gate && on_face) {
                    ch.set(x, y0, z, FENCE);
                }
                continue;
            }
            let row = if rows_x { x } else { z };
            if row % 2 == 0 {
                ch.set(x, ground, z, SOIL);
                ch.set(x, y0, z, if pumpkins { PUMPKIN } else { CROP });
            } else {
                ch.set(x, ground, z, PATH);
            }
        }
    }
    ch.set(lot.x0 + 1, y0, lot.z0 + 1, HAY);
}

pub(super) fn build_garden(ch: &mut Chunk, lot: &Lot, y0: i32) {
    for z in lot.z0..=lot.z1 {
        for x in lot.x0..=lot.x1 {
            let r = hash2(lot.seed, x, z) % 9;
            let b = match r {
                0 => FLOWER_R,
                1 => FLOWER_Y,
                2 => FLOWER_B,
                3 => TALLGRASS,
                _ => AIR,
            };
            if b != AIR {
                ch.set(x, y0, z, b);
            }
        }
    }
    let (cx, cz) = ((lot.x0 + lot.x1) / 2, (lot.z0 + lot.z1) / 2);
    ch.fill_box(cx - 1, y0, cz - 1, cx + 1, y0, cz + 1, AIR);
    oak(ch, cx, y0, cz, lot.seed);
    let (fx, fz) = lot.front;
    let (bx, bz) = (
        (fx.floor() as i32).clamp(lot.x0 + 1, lot.x1 - 1),
        (fz.floor() as i32).clamp(lot.z0 + 1, lot.z1 - 1),
    );
    ch.set(bx, y0, bz, BENCH);
}
