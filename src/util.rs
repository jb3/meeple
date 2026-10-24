//! Hashing, random numbers and noise. Everything procedural in Meeple is
//! derived from these so that the same seed always yields the same world.

#[inline]
pub fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

#[inline]
pub fn hash2(seed: u32, x: i32, z: i32) -> u32 {
    let a = hash_u32((x as u32).wrapping_mul(0x9E37_79B1) ^ seed);
    hash_u32(a ^ (z as u32).wrapping_mul(0x85EB_CA6B).wrapping_add(0x27d4_eb2f))
}

#[inline]
pub fn hash3(seed: u32, x: i32, y: i32, z: i32) -> u32 {
    hash2(hash_u32(seed ^ (y as u32).wrapping_mul(0xC2B2_AE35)), x, z)
}

/// Map a hash to [0, 1).
#[inline]
pub fn unit(h: u32) -> f32 {
    (h >> 8) as f32 / 16_777_216.0
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Exponential approach factor for frame-rate independent smoothing.
#[inline]
pub fn approach(rate: f32, dt: f32) -> f32 {
    1.0 - (-rate * dt).exp()
}

/// Small, fast PCG-style generator for gameplay randomness.
#[derive(Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut r = Rng(seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407));
        r.next_u32();
        r
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.0;
        self.0 = old.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    pub fn f32(&mut self) -> f32 {
        unit(self.next_u32())
    }

    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f32()
    }

    pub fn int(&mut self, a: i32, b: i32) -> i32 {
        if b <= a {
            return a;
        }
        a + (self.next_u32() % (b - a) as u32) as i32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.next_u32() as usize % items.len()]
    }
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn grad(h: u32, x: f32, z: f32) -> f32 {
    match h & 7 {
        0 => x + z,
        1 => x - z,
        2 => -x + z,
        3 => -x - z,
        4 => x * 1.41,
        5 => -x * 1.41,
        6 => z * 1.41,
        _ => -z * 1.41,
    }
}

/// 2D gradient noise, roughly in [-1, 1].
pub fn perlin(seed: u32, x: f32, z: f32) -> f32 {
    let xi = x.floor();
    let zi = z.floor();
    let xf = x - xi;
    let zf = z - zi;
    let (ix, iz) = (xi as i32, zi as i32);
    let u = fade(xf);
    let v = fade(zf);
    let a = grad(hash2(seed, ix, iz), xf, zf);
    let b = grad(hash2(seed, ix + 1, iz), xf - 1.0, zf);
    let c = grad(hash2(seed, ix, iz + 1), xf, zf - 1.0);
    let d = grad(hash2(seed, ix + 1, iz + 1), xf - 1.0, zf - 1.0);
    lerp(lerp(a, b, u), lerp(c, d, u), v) * 0.75
}

/// Fractal noise.
pub fn fbm(seed: u32, x: f32, z: f32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for o in 0..octaves {
        sum += perlin(seed.wrapping_add(o * 1013), x * freq, z * freq) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.03;
    }
    sum / norm
}

/// Ridged fractal noise in [0, 1], peaks along ridge lines.
pub fn ridged(seed: u32, x: f32, z: f32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for o in 0..octaves {
        let n = 1.0 - perlin(seed.wrapping_add(o * 7919), x * freq, z * freq).abs();
        sum += n * n * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.1;
    }
    sum / norm
}
