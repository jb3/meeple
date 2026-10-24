//! The terrain noise four points at a time. On wasm built with `simd128`
//! (see `.cargo/config.toml`) each `f32x4` is one 128-bit register; elsewhere
//! `wide` falls back to SSE, NEON or plain code, so the results agree with the
//! one-at-a-time versions in `util` to rounding.

use wide::f32x4;

#[inline]
fn fade4(t: f32x4) -> f32x4 {
    t * t * t * (t * (t * f32x4::splat(6.0) - f32x4::splat(15.0)) + f32x4::splat(10.0))
}

#[inline]
fn lerp4(a: f32x4, b: f32x4, t: f32x4) -> f32x4 {
    a + (b - a) * t
}

/// `util::perlin` at four points.
///
/// The lattice hashes are worked out one at a time (and only once when all
/// four points share a cell, as neighbouring samples of the slow terrain
/// fields do); the interpolation, which is most of the arithmetic, is done
/// four lanes at once.
pub fn perlin4(seed: u32, x: f32x4, z: f32x4) -> f32x4 {
    use crate::util::hash2;
    const GX: [f32; 8] = [1.0, 1.0, -1.0, -1.0, 1.41, -1.41, 0.0, 0.0];
    const GZ: [f32; 8] = [1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 1.41, -1.41];
    let xi = x.floor();
    let zi = z.floor();
    let (xf, zf) = (x - xi, z - zi);
    let one = f32x4::ONE;
    let ix = xi.to_array().map(|v| v as i32);
    let iz = zi.to_array().map(|v| v as i32);
    let same = ix[0] == ix[3] && iz[0] == iz[3] && ix[0] == ix[1] && ix[0] == ix[2] && iz[0] == iz[1] && iz[0] == iz[2];
    let corner = |dx: i32, dz: i32| -> (f32x4, f32x4) {
        if same {
            let h = (hash2(seed, ix[0] + dx, iz[0] + dz) & 7) as usize;
            (f32x4::splat(GX[h]), f32x4::splat(GZ[h]))
        } else {
            let h = [0, 1, 2, 3].map(|l| (hash2(seed, ix[l] + dx, iz[l] + dz) & 7) as usize);
            (f32x4::from(h.map(|k| GX[k])), f32x4::from(h.map(|k| GZ[k])))
        }
    };
    let (ax, az) = corner(0, 0);
    let (bx, bz) = corner(1, 0);
    let (cx, cz) = corner(0, 1);
    let (dx, dz) = corner(1, 1);
    let u = fade4(xf);
    let v = fade4(zf);
    let a = ax * xf + az * zf;
    let b = bx * (xf - one) + bz * zf;
    let c = cx * xf + cz * (zf - one);
    let d = dx * (xf - one) + dz * (zf - one);
    lerp4(lerp4(a, b, u), lerp4(c, d, u), v) * f32x4::splat(0.75)
}

/// `util::fbm` at four points.
pub fn fbm4(seed: u32, x: f32x4, z: f32x4, octaves: u32) -> f32x4 {
    let (mut sum, mut amp, mut freq, mut norm) = (f32x4::ZERO, 1.0f32, 1.0f32, 0.0f32);
    for o in 0..octaves {
        sum += perlin4(seed.wrapping_add(o * 1013), x * f32x4::splat(freq), z * f32x4::splat(freq)) * f32x4::splat(amp);
        norm += amp;
        amp *= 0.5;
        freq *= 2.03;
    }
    sum / f32x4::splat(norm)
}

/// `util::ridged` at four points.
pub fn ridged4(seed: u32, x: f32x4, z: f32x4, octaves: u32) -> f32x4 {
    let (mut sum, mut amp, mut freq, mut norm) = (f32x4::ZERO, 1.0f32, 1.0f32, 0.0f32);
    for o in 0..octaves {
        let n = f32x4::ONE - perlin4(seed.wrapping_add(o * 7919), x * f32x4::splat(freq), z * f32x4::splat(freq)).abs();
        sum += n * n * f32x4::splat(amp);
        norm += amp;
        amp *= 0.5;
        freq *= 2.1;
    }
    sum / f32x4::splat(norm)
}

/// `util::smoothstep` at four points (either edge order).
#[inline]
pub fn smoothstep4(e0: f32, e1: f32, x: f32x4) -> f32x4 {
    let t = ((x - f32x4::splat(e0)) / f32x4::splat(e1 - e0)).max(f32x4::ZERO).min(f32x4::ONE);
    t * t * (f32x4::splat(3.0) - f32x4::splat(2.0) * t)
}

/// `?bench`: time the terrain noise one at a time against four at a time, in
/// whatever this browser makes of the wasm, and say so on the console.
pub fn bench(seed: u32, now: &dyn Fn() -> f64) -> String {
    let gen = crate::world::gen::Gen::new(seed);
    let (mut scalar, mut simd) = (f64::MAX, f64::MAX);
    let mut sink = 0.0;
    for rep in 0..6 {
        let o = rep * 5000;
        let t = now();
        for dz in 0..128 {
            for dx in 0..128 {
                sink += gen.raw_height(std::hint::black_box((dx + o) as f32 + 0.5), std::hint::black_box((dz - o) as f32 + 0.5));
            }
        }
        scalar = scalar.min(now() - t);
        let t = now();
        sink += gen.noise_block(o, -o, 128, 128).at(3, 3).0;
        simd = simd.min(now() - t);
    }
    // and what a whole chunk costs (terrain, features, then meshing), so the
    // noise can be seen in proportion
    let t = now();
    let mut quads = 0;
    for k in 0..12 {
        let ch = crate::world::chunk::generate(&gen, k * 3 - 20, k * 2 - 9);
        let (solid, _water) = crate::world::mesh::build(&ch);
        quads += solid.verts.len();
    }
    let chunk = (now() - t) / 12.0;
    format!("meeple bench: 16384 terrain columns: {scalar:.2} ms one at a time, {simd:.2} ms four at a time ({sink:.1}); a whole chunk (generate and mesh) {chunk:.2} ms ({quads})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{fbm, perlin, ridged};
    use crate::world::gen::Gen;

    #[test]
    fn four_at_a_time_agrees_with_one_at_a_time() {
        let xs = [-3012.37f32, 0.5, 17.25, 98123.7];
        let zs = [441.9f32, -0.5, -5000.1, 12.0];
        let (x, z) = (f32x4::from(xs), f32x4::from(zs));
        for seed in [0u32, 1234, 99_999] {
            let (p, f, r) = (perlin4(seed, x / f32x4::splat(40.0), z / f32x4::splat(40.0)).to_array(), fbm4(seed, x / f32x4::splat(700.0), z / f32x4::splat(700.0), 4).to_array(), ridged4(seed, x / f32x4::splat(240.0), z / f32x4::splat(240.0), 4).to_array());
            for i in 0..4 {
                assert!((p[i] - perlin(seed, xs[i] / 40.0, zs[i] / 40.0)).abs() < 1e-5, "perlin lane {i}");
                assert!((f[i] - fbm(seed, xs[i] / 700.0, zs[i] / 700.0, 4)).abs() < 1e-5, "fbm lane {i}");
                assert!((r[i] - ridged(seed, xs[i] / 240.0, zs[i] / 240.0, 4)).abs() < 1e-5, "ridged lane {i}");
            }
        }
    }

    #[test]
    fn terrain_blocks_match_the_scalar_terrain() {
        let gen = Gen::new(535);
        let b = gen.noise_block(-77, 301, 34, 5);
        for dz in 0..5 {
            for dx in 0..34 {
                let (x, z) = (-77 + dx, 301 + dz);
                let (raw, tn, mn) = b.at(dx, dz);
                let want = gen.raw_height(x as f32 + 0.5, z as f32 + 0.5);
                assert!((raw - want).abs() < 2e-3, "height at {x},{z}: {raw} vs {want}");
                let (wt, wm) = gen.biome_noise(x as f32 + 0.5, z as f32 + 0.5);
                assert!((tn - wt).abs() < 1e-5 && (mn - wm).abs() < 1e-5);
            }
        }
    }

    #[test]
    #[ignore]
    fn bench_perlin() {
        let n = 200_000;
        let t = std::time::Instant::now();
        let mut s = 0.0f32;
        for i in 0..n {
            let (x, z) = (i as f32 * 0.37, i as f32 * 0.11);
            for k in 0..4 {
                s += perlin(7, x + k as f32 * 0.9, z);
            }
        }
        let scalar = t.elapsed();
        let t = std::time::Instant::now();
        let mut s4 = f32x4::ZERO;
        for i in 0..n {
            let (x, z) = (i as f32 * 0.37, i as f32 * 0.11);
            s4 += perlin4(7, f32x4::from([x, x + 0.9, x + 1.8, x + 2.7]), f32x4::splat(z));
        }
        println!("perlin x{}: scalar {scalar:?} ({s}), four-wide {:?} ({:?})", n * 4, t.elapsed(), s4.to_array());
    }

    /// `cargo test --release bench_terrain -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn bench_terrain() {
        let gen = Gen::new(535);
        let mut s = 0.0;
        let mut scalar = std::time::Duration::MAX;
        let mut simd = std::time::Duration::MAX;
        let mut b = gen.noise_block(0, 0, 4, 4);
        // best of several runs, over varied ground, so caches and clocks settle
        for rep in 0..8 {
            let o = rep * 5000;
            let t = std::time::Instant::now();
            let mut acc = 0.0;
            for dz in 0..128 {
                for dx in 0..128 {
                    acc += gen.raw_height(std::hint::black_box((dx + o) as f32 + 0.5), std::hint::black_box((dz - o) as f32 + 0.5));
                }
            }
            s = acc;
            scalar = scalar.min(t.elapsed());
            let t = std::time::Instant::now();
            b = gen.noise_block(o, -o, 128, 128);
            simd = simd.min(t.elapsed());
        }
        println!("16384 heights: scalar {scalar:?} ({s}), four-wide {simd:?} ({})", b.at(5, 5).0);
    }
}
