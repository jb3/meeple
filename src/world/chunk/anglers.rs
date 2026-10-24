use super::*;

/// Side of the grid cells that each may hold one angler.
const FISH_CELL: i32 = 8;

/// Blocks a person can stand among: nothing solid, nothing built.
fn open(b: u8) -> bool {
    matches!(b, AIR | TALLGRASS | FLOWER_R | FLOWER_Y | FLOWER_B | BLUEBELL | POPPY)
}

/// Somebody sitting or standing on a river bank with a line in the water:
/// at most one per grid cell, on a bank spot that really is open ground in
/// the finished chunk (not under a tree, in a wall or on a road).
pub(super) fn place_anglers(ch: &mut Chunk, gen: &Gen) {
    let (x0, z0) = (ch.cx * CS, ch.cz * CS);
    for cz in z0.div_euclid(FISH_CELL)..(z0 + CS).div_euclid(FISH_CELL) {
        for cx in x0.div_euclid(FISH_CELL)..(x0 + CS).div_euclid(FISH_CELL) {
            let h = hash2(0xf15b, cx, cz);
            if unit(h) > 0.9 {
                continue;
            }
            let mut spots = Vec::new();
            let step = 2;
            for iz in 0..FISH_CELL / step {
                for ix in 0..FISH_CELL / step {
                    let (wx, wz) = (cx * FISH_CELL + ix * step, cz * FISH_CELL + iz * step);
                    // the bank: the topmost block of the column, a few above the water
                    let Some(top) = (SEA + 1..SEA + 6).rev().find(|&y| ch.get_w(wx, y, wz) != AIR) else { continue };
                    if !matches!(ch.get_w(wx, top, wz), GRASS | DIRT | SAND | SNOW | GRAVEL) || top < SEA + 1 {
                        continue;
                    }
                    let p = Vec2::new(wx as f32 + 0.5, wz as f32 + 0.5);
                    for i in 0..8 {
                        let a = i as f32 * std::f32::consts::FRAC_PI_4;
                        let d = Vec2::new(a.cos(), a.sin());
                        let wet = |m: f32| gen.raw_height(p.x + d.x * m, p.y + d.y * m) < SEA as f32 - 0.5;
                        if wet(2.5) && wet(3.5) && !wet(0.6) {
                            // room to stand and to cast: no block in the way
                            let cell = |m: f32, y: i32| ch.get_w((p.x + d.x * m).floor() as i32, y, (p.y + d.y * m).floor() as i32);
                            let free = (0..3).all(|k| open(ch.get_w(wx, top + 1 + k, wz)))
                                && [1.0, 2.0].iter().all(|&m| open(cell(m, top + 1)) && open(cell(m, top + 2)));
                            if free {
                                spots.push((p, d, top));
                            }
                            break;
                        }
                    }
                }
            }
            if spots.is_empty() {
                continue;
            }
            for k in 0..3u32 {
                let hk = hash_u32(h ^ k.wrapping_mul(0x9E37_79B1));
                let (p, d, top) = spots[hk as usize % spots.len()];
                if ch.core(p.x as i32, p.y as i32) && gen.standing_clear(p.x as i32, p.y as i32) {
                    ch.feat.anglers.push(([p.x, top as f32 + 1.0, p.y], d.x.atan2(d.y), hk));
                    break;
                }
            }
        }
    }
}
