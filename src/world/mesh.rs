//! Turns chunk voxels into GPU-ready vertex data: culled faces with
//! per-vertex ambient occlusion, plus small boxes for special blocks.

use glam::{Vec2, Vec3};

use super::blocks::*;
use super::chunk::{Chunk, CH, CS};
use crate::util::hash3;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    /// rgb + material id
    pub col: [u8; 4],
    /// normal index (0..6), ambient occlusion (0..3), how long a leaf clings in
    /// autumn (see `leaf_hold`), unused
    pub info: [u8; 4],
}

#[derive(Default)]
pub struct MeshData {
    pub verts: Vec<Vertex>,
    pub idx: Vec<u32>,
}

impl MeshData {
    pub fn bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.verts.as_ptr() as *const u8,
                self.verts.len() * std::mem::size_of::<Vertex>(),
            )
        }
    }
    pub fn idx_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.idx.as_ptr() as *const u8, self.idx.len() * 4) }
    }
}

// face directions: +x, -x, +y, -y, +z, -z
const DIRS: [[i32; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

fn jitter(c: [u8; 3], h: u32, amt: f32) -> [u8; 3] {
    let f = 1.0 + ((h % 1000) as f32 / 1000.0 - 0.5) * amt;
    [
        (c[0] as f32 * f).min(255.0) as u8,
        (c[1] as f32 * f).min(255.0) as u8,
        (c[2] as f32 * f).min(255.0) as u8,
    ]
}

/// Emit one quad for face `d` of the box [lo, hi].
#[allow(clippy::too_many_arguments)]
fn quad(m: &mut MeshData, lo: [f32; 3], hi: [f32; 3], d: usize, col: [u8; 3], mat: u8, ao: [u8; 4]) {
    let a = d / 2;
    let pos = d % 2 == 0;
    let u = (a + 1) % 3;
    let v = (a + 2) % 3;
    let mut corners = [(0u8, 0u8), (1, 0), (1, 1), (0, 1)];
    let mut aos = ao;
    if !pos {
        corners.reverse();
        aos.reverse();
    }
    let base = m.verts.len() as u32;
    for (i, &(cu, cv)) in corners.iter().enumerate() {
        let mut p = [0.0f32; 3];
        p[a] = if pos { hi[a] } else { lo[a] };
        p[u] = if cu == 1 { hi[u] } else { lo[u] };
        p[v] = if cv == 1 { hi[v] } else { lo[v] };
        let vm = if mat == M_CROP && p[1] == hi[1] { M_CROP_TOP } else { mat };
        m.verts.push(Vertex { pos: p, col: [col[0], col[1], col[2], vm], info: [d as u8, aos[i], 0, 0] });
    }
    if aos[0] as u32 + aos[2] as u32 > aos[1] as u32 + aos[3] as u32 {
        m.idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    } else {
        m.idx.extend_from_slice(&[base + 1, base + 2, base + 3, base + 1, base + 3, base]);
    }
}

fn add_box(m: &mut MeshData, lo: [f32; 3], hi: [f32; 3], col: [u8; 3], mat: u8) {
    for d in 0..6 {
        if d == 3 {
            continue; // bottoms are never visible from above
        }
        quad(m, lo, hi, d, col, mat, [3, 3, 3, 3]);
    }
}

/// How long a leaf clings on when the leaves fall: 1 for those touching a
/// trunk down to 0 for those far out, so a bare tree keeps a few twigs by
/// the trunk rather than loose blocks floating around it.
fn leaf_hold(ch: &Chunk, lx: i32, y: i32, lz: i32) -> f32 {
    for r in 1..=3i32 {
        for dy in -r..=r {
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx.abs().max(dy.abs()).max(dz.abs()) != r {
                        continue;
                    }
                    if matches!(ch.get(lx + dx, y + dy, lz + dz), LOG | BIRCH) {
                        return (4 - r) as f32 / 3.0;
                    }
                }
            }
        }
    }
    0.0
}

/// Walls, roofs and timber, trees: what can stand between the camera and
/// someone, as opposed to the ground itself.
fn is_structure(b: u8) -> bool {
    matches!(
        b,
        LOG | LEAVES | PINE | BIRCH | PLANK | BRICK | ROOF_RED | ROOF_BLUE | ROOF_BROWN | ROOF_GREEN | ROOF_SLATE | WINDOW | DOOR | PLASTER | PLASTER_Y | PLASTER_P | TIMBER | STONEBRICK | WALLSTONE | HAY
    )
}

pub fn build(ch: &Chunk) -> (MeshData, MeshData) {
    let mut solid = MeshData::default();
    let mut water = MeshData::default();
    let ox = ch.cx * CS;
    let oz = ch.cz * CS;

    for y in 0..CH {
        for lz in 0..CS {
            for lx in 0..CS {
                let b = ch.get(lx, y, lz);
                if b == AIR {
                    continue;
                }
                let (wx, wz) = (ox + lx, oz + lz);
                let fx = wx as f32;
                let fy = y as f32;
                let fz = wz as f32;
                match shape(b) {
                    Shape::Full => {
                        let h = hash3(0x5eed, wx, y, wz);
                        // info.z: 0 = no preference, else 1..=255 from "falls first" to "clings"
                        let hold = if b == LEAVES { 1 + (leaf_hold(ch, lx, y, lz) * 254.0) as u8 } else { 0 };
                        for (d, dir) in DIRS.iter().enumerate() {
                            if d == 3 && y == 0 {
                                continue;
                            }
                            let n = ch.get(lx + dir[0], y + dir[1], lz + dir[2]);
                            // a cap on the block standing on the ground (the bottom of a wall or
                            // trunk) under another of the same sort: hidden in the solid, it's
                            // what shows when the see-through effect takes the blocks above away
                            let below = ch.get(lx, y - 1, lz);
                            let cap = d == 2 && is_structure(b) && is_structure(n) && opaque(below) && !is_structure(below);
                            // leaves can be discarded in winter, so they never hide neighbours
                            if opaque(n) && n != LEAVES && !cap {
                                continue;
                            }
                            let (mut c, mut mat) = (color(b), material(b));
                            if b == GRASS {
                                if d == 2 {
                                    mat = M_GRASS;
                                } else {
                                    c = color(DIRT);
                                }
                            }
                            if d == 2 && matches!(b, COBBLE | PATH | GRAVEL) {
                                mat = M_ROAD;
                            }
                            let ao = if cap { [3, 3, 3, 3] } else { face_ao(ch, lx, y, lz, d) };
                            let c = jitter(c, h.wrapping_add(d as u32 * 7), 0.12);
                            quad(&mut solid, [fx, fy, fz], [fx + 1.0, fy + 1.0, fz + 1.0], d, c, mat, ao);
                            // bit 4: part of a building or tree, not the ground
                            // bit 6: not standing on the ground, so it can be taken away
                            let loose = is_structure(b) && !(opaque(below) && !is_structure(below));
                            let mask = if is_structure(b) { 16 } else { 0 } | if cap { 32 } else { 0 } | if loose { 64 } else { 0 };
                            let n = solid.verts.len();
                            for v in &mut solid.verts[n - 4..] {
                                v.info[2] = hold;
                                v.info[3] = mask;
                            }
                        }
                    }
                    Shape::Water => {
                        let above = ch.get(lx, y + 1, lz);
                        let top = if above == WATER { 1.0 } else { 0.86 };
                        let c = color(WATER);
                        for (d, dir) in DIRS.iter().enumerate() {
                            if d == 3 {
                                continue;
                            }
                            let n = ch.get(lx + dir[0], y + dir[1], lz + dir[2]);
                            if n == WATER || opaque(n) {
                                continue;
                            }
                            quad(
                                &mut water,
                                [fx, fy, fz],
                                [fx + 1.0, fy + top, fz + 1.0],
                                d,
                                c,
                                M_WATER,
                                [3, 3, 3, 3],
                            );
                            // water that isn't at sea level (a fountain's basin, a pool)
                            // is still: tides and floods don't lift it
                            if y != crate::world::gen::SEA {
                                let n = water.verts.len();
                                for v in &mut water.verts[n - 4..] {
                                    v.info[2] = 1;
                                }
                            }
                        }
                    }
                    Shape::Special => special(&mut solid, ch, b, lx, y, lz, fx, fy, fz),
                    Shape::Empty => {}
                }
            }
        }
    }
    for r in &ch.rails {
        rail_seg(&mut solid, r);
    }
    (solid, water)
}

/// Push a quad from four corners already in counter-clockwise order.
fn quad4(m: &mut MeshData, c: [[f32; 3]; 4], normal: u8, col: [u8; 3]) {
    let base = m.verts.len() as u32;
    for p in c {
        m.verts.push(Vertex { pos: p, col: [col[0], col[1], col[2], M_SOLID], info: [normal, 3, 0, 0] });
    }
    m.idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Nearest axis-aligned normal index for a direction (the shaders light
/// faces by these six normals).
fn axis_index(n: Vec3) -> u8 {
    let a = n.abs();
    if a.x >= a.y && a.x >= a.z {
        if n.x > 0.0 { 0 } else { 1 }
    } else if a.y >= a.z {
        if n.y > 0.0 { 2 } else { 3 }
    } else if n.z > 0.0 {
        4
    } else {
        5
    }
}

/// Quad from four corners in order round the edge, wound to face `out`.
fn face(m: &mut MeshData, mut c: [Vec3; 4], out: Vec3, col: [u8; 3]) {
    if (c[1] - c[0]).cross(c[2] - c[0]).dot(out) < 0.0 {
        c.reverse();
    }
    quad4(m, c.map(|v| v.to_array()), axis_index(out), col);
}

/// A strip running from `a` to `b`, spanning `lo..hi` sideways (along the
/// side directions `na`/`nb`), `rise` above the given centre heights and
/// reaching down to `bottom` (absolute) or `depth` below its top. Emits the
/// top and both long sides.
#[allow(clippy::too_many_arguments)]
fn strip(m: &mut MeshData, a: Vec3, b: Vec3, na: Vec2, nb: Vec2, (lo, hi): (f32, f32), rise: f32, depth: f32, bottom: Option<f32>, col: [u8; 3]) {
    let at = |p: Vec3, n: Vec2, off: f32, y: f32| Vec3::new(p.x + n.x * off, y, p.z + n.y * off);
    let (ta, tb) = (a.y + rise, b.y + rise);
    let (ba, bb) = match bottom {
        Some(f) => (f.min(ta), f.min(tb)),
        None => (ta - depth, tb - depth),
    };
    face(m, [at(a, na, lo, ta), at(a, na, hi, ta), at(b, nb, hi, tb), at(b, nb, lo, tb)], Vec3::Y, col);
    let n = (na + nb).normalize_or_zero();
    for (off, sign) in [(lo, -1.0), (hi, 1.0)] {
        let out = Vec3::new(n.x * sign, 0.0, n.y * sign);
        face(m, [at(a, na, off, ta), at(b, nb, off, tb), at(b, nb, off, bb), at(a, na, off, ba)], out, col);
    }
}

fn rail_seg(m: &mut MeshData, r: &super::chunk::RailSeg) {
    let h = hash3(0x5eed, r.a.x as i32, 7, r.a.z as i32);
    let ballast = jitter(color(GRAVEL), h, 0.1);
    // ballast bed, its sides exposed down to the voxel terrain
    strip(m, r.a, r.b, r.na, r.nb, (-1.0, 1.0), 0.0, 0.0, Some(r.bed), ballast);
    // a sleeper across the middle of the step
    let ta = (r.b - r.a).normalize_or_zero();
    let mid = (r.a + r.b) * 0.5;
    let n = (r.na + r.nb).normalize_or_zero();
    let (s0, s1) = (mid - ta * 0.2, mid + ta * 0.2);
    strip(m, s0, s1, n, n, (-1.0, 1.0), 0.12, 0.12, None, color(RAIL_XA));
    let end = |p: Vec3| [p + Vec3::new(n.x, 0.12, n.y), p + Vec3::new(-n.x, 0.12, -n.y), p - Vec3::new(n.x, 0.0, n.y), p + Vec3::new(n.x, 0.0, n.y)];
    face(m, end(s0), -ta, color(RAIL_XA));
    face(m, end(s1), ta, color(RAIL_XA));
    // steel rails
    for off in [-0.65, 0.65] {
        strip(m, r.a, r.b, r.na, r.nb, (off - 0.07, off + 0.07), 0.26, 0.14, None, [150, 152, 164]);
    }
}

fn face_ao(ch: &Chunk, x: i32, y: i32, z: i32, d: usize) -> [u8; 4] {
    let a = d / 2;
    let u = (a + 1) % 3;
    let v = (a + 2) % 3;
    let mut f = [x, y, z];
    f[a] += if d % 2 == 0 { 1 } else { -1 };
    let occ = |p: [i32; 3]| opaque(ch.get(p[0], p[1], p[2])) as u8;
    let mut out = [3u8; 4];
    for (i, &(cu, cv)) in [(0i32, 0i32), (1, 0), (1, 1), (0, 1)].iter().enumerate() {
        let du = cu * 2 - 1;
        let dv = cv * 2 - 1;
        let mut s1 = f;
        s1[u] += du;
        let mut s2 = f;
        s2[v] += dv;
        let mut c = f;
        c[u] += du;
        c[v] += dv;
        let (a1, a2, ac) = (occ(s1), occ(s2), occ(c));
        out[i] = if a1 == 1 && a2 == 1 { 0 } else { 3 - (a1 + a2 + ac) };
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn special(m: &mut MeshData, ch: &Chunk, b: u8, lx: i32, y: i32, lz: i32, x: f32, fy: f32, z: f32) {
    let bx = |m: &mut MeshData, x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32, c: [u8; 3], mat: u8| {
        add_box(m, [x + x0, fy + y0, z + z0], [x + x1, fy + y1, z + z1], c, mat);
    };
    let h = hash3(0x5eed, x as i32, y, z as i32);
    let steel = [150, 152, 164];
    match b {
        LAMP => {
            bx(m, 0.3, 0.0, 0.3, 0.7, 0.42, 0.7, color(LAMP), M_LAMP);
            bx(m, 0.24, 0.42, 0.24, 0.76, 0.54, 0.76, color(LAMPPOST), M_SOLID);
        }
        LAMPPOST => bx(m, 0.42, 0.0, 0.42, 0.58, 1.0, 0.58, color(LAMPPOST), M_SOLID),
        CROP => {
            let c = jitter(color(CROP), h, 0.2);
            bx(m, 0.12, 0.0, 0.12, 0.88, 0.9, 0.88, c, M_CROP);
        }
        RAIL_XA | RAIL_XB => {
            bx(m, 0.3, 0.0, 0.0, 0.7, 0.12, 1.0, color(b), M_SOLID);
            let (z0, z1) = if b == RAIL_XA { (0.28, 0.42) } else { (0.58, 0.72) };
            bx(m, 0.0, 0.12, z0, 1.0, 0.26, z1, steel, M_SOLID);
        }
        RAIL_ZA | RAIL_ZB => {
            bx(m, 0.0, 0.0, 0.3, 1.0, 0.12, 0.7, color(b), M_SOLID);
            let (x0, x1) = if b == RAIL_ZA { (0.28, 0.42) } else { (0.58, 0.72) };
            bx(m, x0, 0.12, 0.0, x1, 0.26, 1.0, steel, M_SOLID);
        }
        FENCE => {
            let c = color(FENCE);
            bx(m, 0.38, 0.0, 0.38, 0.62, 0.9, 0.62, c, M_SOLID);
            let conn = |dx: i32, dz: i32| {
                let n = ch.get(lx + dx, y, lz + dz);
                n == FENCE || opaque(n)
            };
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if !conn(dx, dz) {
                    continue;
                }
                let (x0, x1) = match dx {
                    1 => (0.5, 1.0),
                    -1 => (0.0, 0.5),
                    _ => (0.45, 0.55),
                };
                let (z0, z1) = match dz {
                    1 => (0.5, 1.0),
                    -1 => (0.0, 0.5),
                    _ => (0.45, 0.55),
                };
                bx(m, x0, 0.45, z0, x1, 0.57, z1, c, M_SOLID);
                bx(m, x0, 0.7, z0, x1, 0.8, z1, c, M_SOLID);
            }
        }
        BLUEBELL => {
            // a nodding stem of little bells
            let ox = (h % 5) as f32 * 0.06 - 0.12;
            let mat = material(b);
            bx(m, 0.47 + ox, 0.0, 0.47, 0.53 + ox, 0.45, 0.53, [70, 130, 60], mat);
            bx(m, 0.52 + ox, 0.3, 0.44, 0.62 + ox, 0.4, 0.56, color(b), mat);
            bx(m, 0.5 + ox, 0.2, 0.44, 0.6 + ox, 0.29, 0.56, color(b), mat);
            bx(m, 0.25, 0.0, 0.3, 0.31, 0.3, 0.36, [70, 130, 60], mat);
            bx(m, 0.28, 0.18, 0.28, 0.36, 0.26, 0.36, color(b), mat);
        }
        POPPY => {
            let ox = (h % 5) as f32 * 0.06 - 0.12;
            let mat = material(b);
            bx(m, 0.47 + ox, 0.0, 0.47, 0.53 + ox, 0.5, 0.53, [80, 140, 60], mat);
            bx(m, 0.36 + ox, 0.5, 0.36, 0.64 + ox, 0.6, 0.64, color(b), mat);
            bx(m, 0.46 + ox, 0.6, 0.46, 0.54 + ox, 0.63, 0.54, [30, 26, 30], mat);
        }
        FLOWER_R | FLOWER_Y | FLOWER_B => {
            let ox = (h % 5) as f32 * 0.05 - 0.1;
            bx(m, 0.46 + ox, 0.0, 0.46, 0.54 + ox, 0.4, 0.54, [70, 140, 60], M_FLOWER);
            bx(m, 0.36 + ox, 0.4, 0.36, 0.64 + ox, 0.62, 0.64, color(b), M_FLOWER);
        }
        TALLGRASS => {
            let c = jitter(color(TALLGRASS), h, 0.2);
            bx(m, 0.2, 0.0, 0.4, 0.3, 0.5, 0.5, c, M_BLADE);
            bx(m, 0.55, 0.0, 0.2, 0.65, 0.7, 0.3, c, M_BLADE);
            bx(m, 0.45, 0.0, 0.65, 0.55, 0.4, 0.75, c, M_BLADE);
        }
        CACTUS => bx(m, 0.14, 0.0, 0.14, 0.86, 1.0, 0.86, jitter(color(CACTUS), h, 0.1), M_SOLID),
        BENCH => {
            let c = color(BENCH);
            bx(m, 0.05, 0.32, 0.2, 0.95, 0.42, 0.75, c, M_SOLID);
            bx(m, 0.05, 0.42, 0.65, 0.95, 0.8, 0.75, c, M_SOLID);
            bx(m, 0.1, 0.0, 0.3, 0.2, 0.32, 0.7, [90, 70, 50], M_SOLID);
            bx(m, 0.8, 0.0, 0.3, 0.9, 0.32, 0.7, [90, 70, 50], M_SOLID);
        }
        ANVIL => {
            let c = color(ANVIL);
            bx(m, 0.32, 0.0, 0.32, 0.68, 0.35, 0.68, c, M_SOLID);
            bx(m, 0.15, 0.35, 0.3, 0.85, 0.6, 0.7, c, M_SOLID);
        }
        PUMPKIN => {
            bx(m, 0.2, 0.0, 0.2, 0.8, 0.5, 0.8, jitter(color(PUMPKIN), h, 0.15), M_SOLID);
            bx(m, 0.45, 0.5, 0.45, 0.55, 0.62, 0.55, [90, 120, 50], M_SOLID);
        }
        BUSH => bx(m, 0.08, 0.0, 0.08, 0.92, 0.7, 0.92, jitter(color(BUSH), h, 0.15), M_LEAF),
        FORGE => {
            // stone hearth with a raised rim around a bed of glowing coals
            let stone = [104, 98, 98];
            let rim = [82, 76, 76];
            bx(m, 0.04, 0.0, 0.04, 0.96, 0.5, 0.96, stone, M_SOLID);
            bx(m, 0.04, 0.5, 0.04, 0.96, 0.68, 0.18, rim, M_SOLID);
            bx(m, 0.04, 0.5, 0.82, 0.96, 0.68, 0.96, rim, M_SOLID);
            bx(m, 0.04, 0.5, 0.18, 0.18, 0.68, 0.82, rim, M_SOLID);
            bx(m, 0.82, 0.5, 0.18, 0.96, 0.68, 0.82, rim, M_SOLID);
            bx(m, 0.18, 0.5, 0.18, 0.82, 0.6, 0.82, color(FORGE), M_FIRE);
            // a few coals heaped higher in the middle
            bx(m, 0.34, 0.6, 0.3, 0.56, 0.67, 0.52, [255, 170, 60], M_FIRE);
            bx(m, 0.5, 0.6, 0.48, 0.68, 0.65, 0.68, [255, 150, 50], M_FIRE);
        }
        _ => {}
    }
}
