//! Particles (smoke, fountain spray, sparks) and stateless weather/ambient
//! effects (rain, snow, fireflies, butterflies, falling leaves).

use glam::{Vec2, Vec3};

use crate::render::boxes::{Boxes, MAT_GLOW, MAT_LIT, MAT_UNLIT};
use crate::util::*;
use crate::world::gen::{Gen, SEA};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Smoke,
    Spray,
    /// A fountain's splash where a drop lands.
    Mist,
    Spark,
    /// A firework's rocket, climbing.
    Rocket,
    /// A spark of a firework burst; `shade` picks its colour.
    Burst,
}

struct Particle {
    pos: Vec3,
    vel: Vec3,
    life: f32,
    max: f32,
    size: f32,
    kind: Kind,
    shade: f32,
}

pub struct Fx {
    parts: Vec<Particle>,
    rng: Rng,
    /// Rockets waiting to be sent up: seconds to go, where.
    pending: Vec<(f32, Vec3)>,
    /// Bursts that went off this frame, for the bang.
    pub booms: Vec<Vec3>,
}

pub struct FxEnv {
    pub time: f32,
    pub dt: f32,
    pub wind: Vec2,
    pub rain: f32,
    pub snowing: bool,
    pub night: f32,
    pub brightness: f32,
    pub season: f32,
    pub hour: f32,
    /// World size of one rendered pixel.
    pub texel: f32,
    /// Rain, snow and falling leaves are drawn on the GPU instead.
    pub gpu_ambient: bool,
    /// Steam rising off wet ground when the sun comes out (0..1).
    pub steam: f32,
}

const MAX_PARTS: usize = 2500;
/// Side of the world tiles ambient particles are anchored to.
const TILE: f32 = 48.0;

impl Fx {
    pub fn new(seed: u32) -> Self {
        Fx { parts: Vec::new(), rng: Rng::new(seed as u64 ^ 0xf00d), pending: Vec::new(), booms: Vec::new() }
    }

    /// Send a rocket up from `from` after `delay` seconds; it bursts overhead.
    pub fn firework(&mut self, from: Vec3, delay: f32) {
        self.pending.push((delay, from));
    }

    pub fn smoke(&mut self, p: Vec3, size: f32) {
        if self.parts.len() >= MAX_PARTS {
            return;
        }
        let shade = self.rng.range(0.6, 0.78);
        self.parts.push(Particle {
            pos: p + Vec3::new(self.rng.range(-0.1, 0.1), 0.0, self.rng.range(-0.1, 0.1)),
            vel: Vec3::new(self.rng.range(-0.1, 0.1), self.rng.range(0.7, 1.1), self.rng.range(-0.1, 0.1)),
            life: 0.0,
            max: self.rng.range(4.0, 7.0),
            size,
            kind: Kind::Smoke,
            shade,
        });
    }

    /// A hammer blow on an anvil: a spray of hot sparks off the top.
    pub fn strike(&mut self, at: Vec3) {
        for _ in 0..9 {
            if self.parts.len() >= MAX_PARTS {
                return;
            }
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let s = self.rng.range(0.4, 1.6);
            self.parts.push(Particle {
                pos: at,
                vel: Vec3::new(a.cos() * s, self.rng.range(1.0, 2.6), a.sin() * s),
                life: 0.0,
                max: self.rng.range(0.3, 0.7),
                size: 0.035,
                kind: Kind::Spark,
                shade: 1.0,
            });
        }
    }

    /// Emit chimney smoke, fountain spray and forge sparks near the camera.
    pub fn emitters(&mut self, env: &FxEnv, chimneys: &[[f32; 3]], fountains: &[[f32; 3]], forges: &[[f32; 3]]) {
        let winter = (2.9..3.95).contains(&env.season);
        let h = env.hour;
        let cooking = (6.0..9.0).contains(&h) || (17.0..22.0).contains(&h);
        let active = 0.15 + cooking as u8 as f32 * 0.45 + winter as u8 as f32 * 0.4;
        for c in chimneys {
            let id = hash3(0xc4, c[0] as i32, c[1] as i32, c[2] as i32);
            if unit(id) > active {
                continue;
            }
            if self.rng.chance(env.dt * 1.6) {
                self.smoke(Vec3::from(*c), 0.13);
            }
        }
        for f in fountains {
            if env.snowing && winter {
                continue;
            }
            // `shade` carries the height of the water the drop lands in
            let floor = f[1] - 2.1;
            // a tight jet up the middle, and a wider fan that arcs out over the basin
            for (rate, spread, lift, size) in [(14.0, 0.25, (4.2, 5.0), 0.045), (22.0, 1.6, (2.4, 3.8), 0.04)] {
                if self.parts.len() < MAX_PARTS && self.rng.chance((env.dt * rate).min(1.0)) {
                    let a = self.rng.range(0.0, std::f32::consts::TAU);
                    let s = self.rng.range(0.15, 1.0) * spread;
                    self.parts.push(Particle {
                        pos: Vec3::from(*f) + Vec3::Y * 0.1,
                        vel: Vec3::new(a.cos() * s, self.rng.range(lift.0, lift.1), a.sin() * s),
                        life: 0.0,
                        max: 2.0,
                        size: size * self.rng.range(0.8, 1.3),
                        kind: Kind::Spray,
                        shade: floor,
                    });
                }
            }
        }
        for f in forges {
            // a wisp of smoke from the coals now and then
            if self.rng.chance(env.dt * 0.8) {
                self.smoke(Vec3::from(*f) + Vec3::Y * 0.2, 0.1);
            }
            if self.parts.len() < MAX_PARTS && self.rng.chance(env.dt * 6.0) {
                self.parts.push(Particle {
                    pos: Vec3::from(*f) + Vec3::new(self.rng.range(-0.25, 0.25), 0.15, self.rng.range(-0.25, 0.25)),
                    vel: Vec3::new(self.rng.range(-0.5, 0.5), self.rng.range(1.2, 2.4), self.rng.range(-0.5, 0.5)),
                    life: 0.0,
                    max: 0.8,
                    size: 0.04,
                    kind: Kind::Spark,
                    shade: 1.0,
                });
            }
        }
    }

    pub fn update(&mut self, env: &FxEnv) {
        let dt = env.dt;
        self.booms.clear();
        for i in 0..self.pending.len() {
            self.pending[i].0 -= dt;
            if self.pending[i].0 <= 0.0 {
                let from = self.pending[i].1;
                self.parts.push(Particle { pos: from, vel: Vec3::new(self.rng.range(-0.6, 0.6), 17.0, self.rng.range(-0.6, 0.6)), life: 0.0, max: 1.15, size: 0.12, kind: Kind::Rocket, shade: 0.0 });
            }
        }
        self.pending.retain(|p| p.0 > 0.0);
        // a rocket that has run out of climb bursts into a ball of colour
        let mut bursts = Vec::new();
        for p in &mut self.parts {
            if p.kind == Kind::Rocket && p.life + dt >= p.max {
                bursts.push(p.pos);
            }
        }
        for c in bursts {
            self.booms.push(c);
            let hue = self.rng.range(0.0, 1.0);
            for _ in 0..90 {
                if self.parts.len() >= MAX_PARTS {
                    break;
                }
                let (a, b) = (self.rng.range(0.0, std::f32::consts::TAU), self.rng.range(-1.0, 1.0));
                let r = (1.0 - b * b).sqrt();
                let sp = self.rng.range(3.5, 7.5);
                let shade = (hue + self.rng.range(-0.06, 0.06)).rem_euclid(1.0);
                self.parts.push(Particle { pos: c, vel: Vec3::new(a.cos() * r, b, a.sin() * r) * sp, life: 0.0, max: self.rng.range(1.4, 2.4), size: 0.13, kind: Kind::Burst, shade });
            }
        }
        let wind = Vec3::new(env.wind.x, 0.0, env.wind.y);
        for p in &mut self.parts {
            p.life += dt;
            match p.kind {
                Kind::Smoke => {
                    p.vel += (wind * 2.6 - p.vel * Vec3::new(1.0, 0.0, 1.0)) * approach(0.6, dt);
                    p.vel.y *= 1.0 - dt * 0.12;
                }
                Kind::Spray => p.vel.y -= 9.0 * dt,
                Kind::Mist => p.vel.y -= 6.0 * dt,
                Kind::Spark => {
                    p.vel.y -= 2.0 * dt;
                }
                Kind::Rocket => p.vel.y *= 1.0 - dt * 0.9,
                Kind::Burst => {
                    p.vel.y -= 3.2 * dt;
                    p.vel *= 1.0 - dt * 1.2;
                }
            }
            p.pos += p.vel * dt;
        }
        // drops that reach the water die in a little splash
        let mut splashes = Vec::new();
        for p in &self.parts {
            if p.kind == Kind::Spray && p.pos.y < p.shade && p.vel.y < 0.0 && self.rng.chance(0.35) {
                splashes.push(Vec3::new(p.pos.x, p.shade, p.pos.z));
            }
        }
        for pos in splashes {
            if self.parts.len() < MAX_PARTS {
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                self.parts.push(Particle { pos, vel: Vec3::new(a.cos() * 0.5, self.rng.range(0.6, 1.2), a.sin() * 0.5), life: 0.0, max: 0.3, size: 0.03, kind: Kind::Mist, shade: pos.y - 1.0 });
            }
        }
        self.parts.retain(|p| p.life < p.max && !(p.kind == Kind::Spray && p.pos.y < p.shade && p.vel.y < 0.0));
    }

    pub fn draw(&self, b: &mut Boxes, env: &FxEnv, gen: &Gen, view: Vec3, view_r: f32) {
        let br = env.brightness;
        for p in &self.parts {
            let t = p.life / p.max;
            match p.kind {
                Kind::Smoke => {
                    let s = p.size * (1.0 + t * 2.5) * (1.0 - t * t * 0.7);
                    let c = p.shade + t * 0.08;
                    b.push(p.pos, Vec3::splat(s), p.life * 0.3, [c, c, c * 1.02], MAT_LIT);
                }
                Kind::Spray => {
                    // drops stretch as they fall fast and whiten near the top of the arc
                    let foam = (1.0 - p.vel.y.abs() / 4.5).clamp(0.0, 1.0) * 0.3;
                    let fall = (p.vel.y.abs() * 0.04).min(0.12);
                    b.push(p.pos, Vec3::new(p.size, p.size + fall, p.size), 0.0, [(0.62 + foam) * br, (0.8 + foam * 0.7) * br, 0.98 * br], MAT_UNLIT);
                }
                Kind::Mist => {
                    let s = p.size * (1.0 + t);
                    b.push(p.pos, Vec3::splat(s), 0.0, [0.85 * br, 0.93 * br, br], MAT_UNLIT);
                }
                Kind::Spark => b.push(p.pos, Vec3::splat(p.size), 0.0, [1.0, 0.6, 0.2], MAT_GLOW),
                Kind::Rocket => b.push(p.pos, Vec3::splat(p.size), 0.0, [1.0, 0.85, 0.5], MAT_GLOW),
                Kind::Burst => {
                    let h = p.shade * 6.0;
                    let c = [(h - 3.0).abs() - 1.0, 2.0 - (h - 2.0).abs(), 2.0 - (h - 4.0).abs()].map(|v| v.clamp(0.0, 1.0) * 0.8 + 0.3);
                    let fade = 1.0 - t * t;
                    b.push(p.pos, Vec3::splat(p.size * (0.4 + fade * 0.6)), 0.0, [c[0] * fade, c[1] * fade, c[2] * fade], MAT_GLOW);
                }
            }
        }

        // Ambient particles are laid out per fixed world tile, seeded by the
        // tile's coordinates, so panning or zooming only reveals and hides
        // tiles instead of reshuffling every drop.
        let r = view_r * 1.05;
        let (tx0, tz0) = (((view.x - r) / TILE).floor() as i32, ((view.z - r) / TILE).floor() as i32);
        let (tx1, tz1) = (((view.x + r) / TILE).floor() as i32, ((view.z + r) / TILE).floor() as i32);
        let season = env.season as u32 % 4;
        let calm = env.rain < 0.2;
        let fireflies = env.night > 0.6 && season < 2 && calm;
        let butterflies = env.night < 0.3 && season < 2 && calm;
        let leaves = season == 2 && env.rain < 0.6 && !env.gpu_ambient;
        // drops keep their world size, but never get thinner than about a
        // rendered pixel or they'd flicker in and out
        let thin = env.texel * 0.6;

        for tz in tz0..=tz1 {
            for tx in tx0..=tx1 {
                let (ox, oz) = (tx as f32 * TILE, tz as f32 * TILE);
                let seed = hash2(0xa1b2, tx, tz);
                let at = |h: u32| Vec2::new(ox + unit(h) * TILE, oz + unit(hash_u32(h ^ 0x33)) * TILE);
                let ground = gen.raw_height(ox + TILE * 0.5, oz + TILE * 0.5).max(SEA as f32 + 1.0);

                // rain and snow, falling in curtains that drift with the wind
                if env.rain > 0.02 && !env.gpu_ambient {
                    let snow = env.snowing;
                    let drift = env.wind * env.time * 3.0;
                    let mid = Vec2::new(ox + TILE * 0.5, oz + TILE * 0.5) - drift;
                    let curtain = smoothstep(-0.25, 0.3, fbm(0xc0a7, mid.x / 160.0, mid.y / 160.0, 2) + env.rain * 0.6 - 0.4);
                    let n = (TILE * TILE * if snow { 0.08 } else { 0.12 } * env.rain * (0.2 + 0.8 * curtain)) as u32;
                    let height = 32.0;
                    let fall = if snow { 2.2 } else { 26.0 };
                    for i in 0..n {
                        let h = hash_u32(seed ^ i.wrapping_mul(0x9e37_79b1));
                        let ph = unit(hash_u32(h ^ 0x77)) * height;
                        let fy = (ph + env.time * fall * (0.85 + unit(h >> 3) * 0.3)) % height;
                        let y = ground + height - 6.0 - fy;
                        let drift = env.wind * (height - fy) * if snow { 0.9 } else { 0.25 };
                        let mut q = at(h) + drift;
                        if snow {
                            q += Vec2::new((env.time * 0.8 + i as f32).sin(), (env.time * 0.7 + i as f32 * 1.3).cos()) * 0.6;
                        }
                        let p = Vec3::new(q.x, y, q.y);
                        if snow {
                            b.push(p, Vec3::splat(thin.max(0.05)), 0.0, [0.95 * br, 0.97 * br, br], MAT_UNLIT);
                        } else {
                            let c = 0.75 * br;
                            let w = thin.max(0.025);
                            b.push(p, Vec3::new(w, 0.4, w), 0.0, [c * 0.92, c * 0.97, c * 1.05], MAT_UNLIT);
                        }
                    }
                }

                // steam lifting off wet ground in the sunshine
                if env.steam > 0.02 {
                    for i in 0..(14.0 * env.steam) as u32 {
                        let h = hash_u32(seed ^ 0x57ea ^ i.wrapping_mul(0x85eb_ca6b));
                        let q = at(h) + env.wind * ((env.time * 0.25 + unit(h >> 3) * 4.0) % 4.0);
                        let rise = (env.time * 0.25 + unit(h >> 3) * 4.0) % 4.0 / 4.0;
                        let gy = gen.raw_height(q.x, q.y).max(SEA as f32 + 1.0).floor() + 1.0;
                        let s = 0.12 + rise * 0.25;
                        b.push_alpha(Vec3::new(q.x, gy + rise * 2.2, q.y), Vec3::splat(s), 0.0, [0.92 * br, 0.93 * br, 0.95 * br], MAT_UNLIT, 0.35 * (1.0 - rise));
                    }
                }

                // fireflies on warm nights
                if fireflies {
                    for i in 0..15u32 {
                        let h = hash_u32(seed ^ 0xf1f1 ^ i.wrapping_mul(0x85eb_ca6b));
                        let q = at(h)
                            + Vec2::new((env.time * 0.3 + i as f32).sin(), (env.time * 0.27 + i as f32 * 2.0).cos()) * 1.2;
                        let blink = ((env.time * 1.7 + unit(h >> 8) * 20.0).sin() * 0.5 + 0.5).powf(3.0);
                        if blink < 0.08 {
                            continue;
                        }
                        let gy = gen.raw_height(q.x, q.y).max(SEA as f32 + 1.0).floor() + 1.0;
                        let y = gy + 0.8 + (env.time * 0.5 + i as f32).sin() * 0.5;
                        let c = [0.8 * blink, 1.0 * blink, 0.3 * blink];
                        b.push(Vec3::new(q.x, y, q.y), Vec3::splat(0.05), 0.0, c, MAT_GLOW);
                    }
                }

                // butterflies by day
                if butterflies {
                    let cols = [[1.0, 0.85, 0.3], [0.95, 0.95, 1.0], [0.4, 0.6, 1.0], [1.0, 0.55, 0.3]];
                    for i in 0..6u32 {
                        let h = hash_u32(seed ^ 0xb7b7 ^ i.wrapping_mul(0x85eb_ca6b));
                        let q = at(h)
                            + Vec2::new((env.time * 0.9 + i as f32).sin(), (env.time * 0.7 + i as f32).cos()) * 2.0;
                        let gy = gen.raw_height(q.x, q.y).max(SEA as f32 + 1.0).floor() + 1.0;
                        let y = gy + 0.7 + (env.time * 2.0 + i as f32).sin().abs() * 0.8;
                        let flap = (env.time * 18.0 + i as f32).sin().abs() * 0.05;
                        let c = cols[(h >> 12) as usize % 4];
                        let c = [c[0] * br, c[1] * br, c[2] * br];
                        let wing = Vec3::new(0.07, 0.01 + flap, 0.05);
                        b.push(Vec3::new(q.x + 0.07, y, q.y), wing, 0.0, c, MAT_UNLIT);
                        b.push(Vec3::new(q.x - 0.07, y, q.y), wing, 0.0, c, MAT_UNLIT);
                    }
                }

                // autumn leaves on the wind
                if leaves {
                    let cols = [[0.9, 0.5, 0.15], [0.8, 0.25, 0.1], [0.95, 0.75, 0.2]];
                    let height = 20.0;
                    for i in 0..27u32 {
                        let h = hash_u32(seed ^ 0x1eaf ^ i.wrapping_mul(0x85eb_ca6b));
                        let fy = (unit(h >> 2) * height + env.time * 0.9) % height;
                        let drift = (env.wind + Vec2::new(0.3, 0.2)) * (height - fy) * 1.2;
                        let q = at(h) + drift + Vec2::new((env.time * 1.3 + i as f32).sin() * 0.7, 0.0);
                        let y = ground + height - 4.0 - fy;
                        let c = cols[(h >> 12) as usize % 3];
                        let c = [c[0] * br, c[1] * br, c[2] * br];
                        b.push(Vec3::new(q.x, y, q.y), Vec3::new(0.09, 0.02, 0.07), env.time * 2.0 + i as f32, c, MAT_UNLIT);
                    }
                }
            }
        }
    }
}
