use super::*;

/// A weir: a stone sill across the river at the water's surface, with a
/// small stone pier where it meets each bank. (The water spilling over it
/// is drawn by the renderer, so it can move.)
pub(super) fn build_weir(ch: &mut Chunk, w: &Weir) {
    let mut t = -w.a - 1.5;
    while t <= w.b + 1.5 {
        let on_water = t > -w.a && t < w.b;
        // two cells thick, so a sill running at an angle has no gaps
        for s in [-0.45, 0.45] {
            let q = w.c + w.cross * t + w.flow * s;
            let (x, z) = (q.x.floor() as i32, q.y.floor() as i32);
            if !ch.core_or_pad(x, z) {
                continue;
            }
            if on_water {
                if ch.get_w(x, SEA, z) == WATER {
                    ch.set(x, SEA, z, STONEBRICK);
                }
            } else if let Some(top) = (SEA..SEA + 6).rev().find(|&y| ch.get_w(x, y, z) != AIR) {
                if matches!(ch.get_w(x, top, z), GRASS | DIRT | SAND | GRAVEL | SNOW | STONE) {
                    ch.set(x, top, z, STONEBRICK);
                    ch.set(x, top + 1, z, STONEBRICK);
                }
            }
        }
        t += 0.5;
    }
    if ch.core(w.c.x.floor() as i32, w.c.y.floor() as i32) {
        ch.feat.weirs.push(*w);
    }
}
