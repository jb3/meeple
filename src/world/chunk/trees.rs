use super::*;

/// How high above the trunk's foot an owl sits on each sort of tree: on top of
/// the crown, and (for the trees that lose their leaves) on the bare trunk's end.
pub(super) fn oak_top(k: u32) -> (f32, f32) {
    let th = 3 + (k % 3) as i32;
    let r = 2 + ((k >> 3) % 2) as i32;
    ((th + r) as f32, th as f32 + 1.0)
}

pub(super) fn birch_top(k: u32) -> (f32, f32) {
    let th = 5 + (k % 3) as i32;
    (th as f32 + 2.0, th as f32 + 1.0)
}

pub(super) fn pine_top(k: u32) -> (f32, f32) {
    let th = 6 + (k % 4) as i32;
    let top = th as f32 + 2.0;
    (top, top)
}

pub(super) fn oak(ch: &mut Chunk, x: i32, y: i32, z: i32, k: u32) {
    let th = 3 + (k % 3) as i32;
    for i in 0..th {
        ch.set(x, y + i, z, LOG);
    }
    let r = 2 + ((k >> 3) % 2) as i32;
    let cy = y + th;
    for dy in -1..=r {
        for dz in -r..=r {
            for dx in -r..=r {
                let d = (dx * dx + dz * dz) as f32 + (dy * dy) as f32 * 1.6;
                let j = unit(hash3(k, dx, dy, dz)) * 1.6;
                if d <= (r * r) as f32 + 0.5 - j {
                    ch.set_air(x + dx, cy + dy, z + dz, LEAVES);
                }
            }
        }
    }
}

pub(super) fn birch(ch: &mut Chunk, x: i32, y: i32, z: i32, k: u32) {
    let th = 5 + (k % 3) as i32;
    for i in 0..th {
        ch.set(x, y + i, z, BIRCH);
    }
    for dy in -2..=1 {
        let r = if dy == 1 { 0 } else { 1 + (dy > -2) as i32 };
        for dz in -r..=r {
            for dx in -r..=r {
                if dx.abs() + dz.abs() <= r + 1 {
                    ch.set_air(x + dx, y + th + dy, z + dz, LEAVES);
                }
            }
        }
    }
}

pub(super) fn pine(ch: &mut Chunk, x: i32, y: i32, z: i32, k: u32) {
    let th = 6 + (k % 4) as i32;
    for i in 0..th {
        ch.set(x, y + i, z, LOG);
    }
    ch.set(x, y + th, z, PINE);
    ch.set(x, y + th + 1, z, PINE);
    let mut i = th - 1;
    let mut r = 1;
    while i >= 2 {
        for dz in -r..=r {
            for dx in -r..=r {
                if dx * dx + dz * dz <= r * r + 1 {
                    ch.set_air(x + dx, y + i, z + dz, PINE);
                }
            }
        }
        i -= 1;
        if (th - i) % 2 == 0 && r < 3 {
            r += 1;
        }
    }
}

pub(super) fn cactus(ch: &mut Chunk, x: i32, y: i32, z: i32, k: u32) {
    let h = 2 + (k % 3) as i32;
    for i in 0..h {
        ch.set(x, y + i, z, CACTUS);
    }
}
