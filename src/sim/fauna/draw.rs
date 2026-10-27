use super::*;

pub(super) fn draw_sheep(b: &mut Boxes, p: Vec3, a: &Animal) {
    let mut r = Rig::new(b, p, a.yaw, 1.0);
    let head_dn = if a.graze > 0.0 { 0.25 } else { 0.0 };
    r.part(Vec3::new(0.0, 0.62, 0.0), Vec3::new(0.32, 0.27, 0.45), a.tint);
    r.part(Vec3::new(0.0, 0.72 - head_dn, 0.5), Vec3::new(0.14, 0.15, 0.15), [0.22, 0.2, 0.22]);
    for (x, z) in [(0.18, 0.28), (-0.18, 0.28), (0.18, -0.28), (-0.18, -0.28)] {
        r.part(Vec3::new(x, 0.18, z), Vec3::new(0.06, 0.18, 0.06), [0.22, 0.2, 0.22]);
    }
}

pub(super) fn draw_cow(b: &mut Boxes, p: Vec3, a: &Animal) {
    let mut r = Rig::new(b, p, a.yaw, 1.0);
    let head_dn = if a.graze > 0.0 { 0.35 } else { 0.0 };
    let spot = [0.18, 0.16, 0.16];
    r.part(Vec3::new(0.0, 0.95, 0.0), Vec3::new(0.38, 0.32, 0.72), a.tint);
    r.part(Vec3::new(0.39, 1.0, 0.1), Vec3::new(0.01, 0.15, 0.2), spot);
    r.part(Vec3::new(-0.39, 0.9, -0.3), Vec3::new(0.01, 0.12, 0.18), spot);
    r.part(Vec3::new(0.0, 1.1 - head_dn, 0.85), Vec3::new(0.2, 0.2, 0.2), a.tint);
    r.part(Vec3::new(0.0, 1.02 - head_dn, 1.05), Vec3::new(0.15, 0.1, 0.04), [0.9, 0.7, 0.7]);
    for x in [0.22, -0.22] {
        r.part(Vec3::new(x, 1.32 - head_dn, 0.82), Vec3::new(0.04, 0.06, 0.04), [0.9, 0.88, 0.8]);
    }
    for (x, z) in [(0.24, 0.5), (-0.24, 0.5), (0.24, -0.5), (-0.24, -0.5)] {
        r.part(Vec3::new(x, 0.32, z), Vec3::new(0.08, 0.32, 0.08), a.tint);
    }
}

pub(super) fn draw_duck(b: &mut Boxes, p: Vec3, a: &Animal, time: f32) {
    let bob = (time * 2.0 + a.phase).sin() * 0.03;
    let mut r = Rig::new(b, p + Vec3::Y * bob, a.yaw, 1.0);
    r.part(Vec3::new(0.0, 0.05, 0.0), Vec3::new(0.12, 0.08, 0.18), a.tint);
    r.part(Vec3::new(0.0, 0.2, 0.14), Vec3::new(0.07, 0.07, 0.07), [0.15, 0.45, 0.25]);
    r.part(Vec3::new(0.0, 0.18, 0.23), Vec3::new(0.035, 0.02, 0.05), [0.95, 0.7, 0.15]);
}

pub(super) fn draw_boat(b: &mut Boxes, p: Vec3, a: &Animal, time: f32) {
    let bob = (time * 1.3 + a.phase).sin() * 0.06;
    let mut r = Rig::new(b, p + Vec3::Y * (bob - 0.2), a.yaw, 1.0);
    let wood = [0.5, 0.34, 0.22];
    r.part(Vec3::new(0.0, 0.2, 0.0), Vec3::new(0.55, 0.22, 1.3), wood);
    r.part(Vec3::new(0.0, 0.2, 1.4), Vec3::new(0.35, 0.2, 0.15), wood);
    r.part(Vec3::new(0.0, 0.43, 0.0), Vec3::new(0.48, 0.02, 1.2), [0.65, 0.5, 0.32]);
    r.part(Vec3::new(0.0, 1.9, 0.2), Vec3::new(0.05, 1.5, 0.05), [0.4, 0.3, 0.2]);
    r.part(Vec3::new(0.0, 2.0, -0.35), Vec3::new(0.03, 1.1, 0.5), a.tint);
    r.part(Vec3::new(0.0, 0.75, -0.6), Vec3::new(0.14, 0.3, 0.12), [0.25, 0.4, 0.7]);
    r.part(Vec3::new(0.0, 1.15, -0.6), Vec3::new(0.12, 0.12, 0.12), [0.93, 0.78, 0.62]);
}

pub(super) fn draw_swan(b: &mut Boxes, p: Vec3, a: &Animal, time: f32) {
    let bob = (time * 1.6 + a.phase).sin() * 0.03;
    let mut r = Rig::new(b, p + Vec3::Y * bob, a.yaw, 1.0);
    let w = a.tint;
    r.part(Vec3::new(0.0, 0.1, 0.0), Vec3::new(0.2, 0.12, 0.32), w);
    r.part(Vec3::new(0.0, 0.22, -0.12), Vec3::new(0.16, 0.08, 0.2), w);
    r.part(Vec3::new(0.0, 0.42, 0.26), Vec3::new(0.05, 0.22, 0.05), w);
    r.part(Vec3::new(0.0, 0.64, 0.32), Vec3::new(0.06, 0.06, 0.09), w);
    r.part(Vec3::new(0.0, 0.62, 0.43), Vec3::new(0.035, 0.03, 0.05), [0.95, 0.5, 0.15]);
}

pub(super) fn draw_heron(b: &mut Boxes, p: Vec3, a: &Animal) {
    let mut r = Rig::new(b, p, a.yaw, 1.0);
    let stab = if a.graze > 0.0 { 0.3 } else { 0.0 };
    for x in [-0.06, 0.06] {
        r.part(Vec3::new(x, 0.45, 0.0), Vec3::new(0.02, 0.45, 0.02), [0.5, 0.45, 0.35]);
    }
    r.part(Vec3::new(0.0, 1.05, 0.0), Vec3::new(0.13, 0.16, 0.28), a.tint);
    r.part(Vec3::new(0.0, 1.35 - stab, 0.22 + stab * 0.6), Vec3::new(0.04, 0.2, 0.04), [0.85, 0.86, 0.9]);
    r.part(Vec3::new(0.0, 1.55 - stab * 1.6, 0.28 + stab), Vec3::new(0.06, 0.06, 0.08), a.tint);
    r.part(Vec3::new(0.0, 1.53 - stab * 1.6, 0.44 + stab), Vec3::new(0.025, 0.02, 0.12), [0.9, 0.75, 0.2]);
}

pub(super) fn draw_deer(b: &mut Boxes, p: Vec3, a: &Animal, alpha: f32) {
    let mut r = Rig::new(b, p, a.yaw, 1.0);
    r.alpha = alpha;
    let moving = a.graze <= 0.0 && a.pos.distance(a.target) > 0.1;
    let sw = if moving { (a.phase * 8.0).sin() * 0.1 } else { 0.0 };
    let head = if a.graze > 0.0 { 0.35 } else { 0.0 };
    r.part(Vec3::new(0.0, 0.85, 0.0), Vec3::new(0.2, 0.2, 0.48), a.tint);
    r.part(Vec3::new(0.0, 0.92, -0.5), Vec3::new(0.06, 0.07, 0.05), [0.95, 0.93, 0.9]);
    r.part(Vec3::new(0.0, 1.15 - head, 0.5), Vec3::new(0.07, 0.2, 0.08), a.tint);
    r.part(Vec3::new(0.0, 1.32 - head * 1.6, 0.62), Vec3::new(0.08, 0.08, 0.13), a.tint);
    if a.phase as i32 % 2 == 0 {
        for x in [-0.08, 0.08] {
            r.part(Vec3::new(x, 1.55 - head * 1.6, 0.58), Vec3::new(0.02, 0.14, 0.02), [0.45, 0.35, 0.25]);
            r.part(Vec3::new(x * 1.6, 1.66 - head * 1.6, 0.56), Vec3::new(0.06, 0.02, 0.02), [0.45, 0.35, 0.25]);
        }
    }
    for (x, z, s) in [(0.12, 0.35, 1.0), (-0.12, 0.35, -1.0), (0.12, -0.35, -1.0), (-0.12, -0.35, 1.0)] {
        r.part(Vec3::new(x, 0.33, z + sw * s), Vec3::new(0.04, 0.33, 0.04), a.tint);
    }
}

pub(super) fn draw_fox(b: &mut Boxes, p: Vec3, a: &Animal, alpha: f32) {
    let mut r = Rig::new(b, p, a.yaw, 1.0);
    r.alpha = alpha;
    let sw = (a.phase * 10.0).sin() * 0.08;
    r.part(Vec3::new(0.0, 0.32, 0.0), Vec3::new(0.12, 0.11, 0.3), a.tint);
    r.part(Vec3::new(0.0, 0.42, 0.34), Vec3::new(0.1, 0.09, 0.1), a.tint);
    r.part(Vec3::new(0.0, 0.38, 0.47), Vec3::new(0.04, 0.04, 0.06), [0.95, 0.92, 0.88]);
    for x in [-0.06, 0.06] {
        r.part(Vec3::new(x, 0.54, 0.32), Vec3::new(0.025, 0.05, 0.02), [0.25, 0.15, 0.1]);
    }
    r.part(Vec3::new(0.0, 0.32, -0.42), Vec3::new(0.07, 0.07, 0.16), a.tint);
    r.part(Vec3::new(0.0, 0.32, -0.6), Vec3::new(0.06, 0.06, 0.04), [0.95, 0.93, 0.9]);
    for (x, z, s) in [(0.07, 0.18, 1.0), (-0.07, 0.18, -1.0), (0.07, -0.18, -1.0), (-0.07, -0.18, 1.0)] {
        r.part(Vec3::new(x, 0.11, z + sw * s), Vec3::new(0.03, 0.11, 0.03), [0.25, 0.15, 0.1]);
    }
}

pub(super) fn draw_rabbit(b: &mut Boxes, p: Vec3, a: &Animal, alpha: f32) {
    let moving = a.pos.distance(a.target) > 0.1 && a.graze <= 0.0;
    let hop = if moving { (a.phase * 9.0).sin().abs() * 0.18 } else { 0.0 };
    let mut r = Rig::new(b, p + Vec3::Y * hop, a.yaw, 1.0);
    r.alpha = alpha;
    r.part(Vec3::new(0.0, 0.13, 0.0), Vec3::new(0.09, 0.09, 0.13), a.tint);
    r.part(Vec3::new(0.0, 0.22, 0.12), Vec3::new(0.06, 0.06, 0.06), a.tint);
    for x in [-0.03, 0.03] {
        r.part(Vec3::new(x, 0.36, 0.1), Vec3::new(0.015, 0.08, 0.02), a.tint);
    }
    r.part(Vec3::new(0.0, 0.16, -0.14), Vec3::new(0.035, 0.035, 0.03), [0.95, 0.95, 0.92]);
}

pub(super) fn draw_owl(b: &mut Boxes, fx: &mut Boxes, p: Vec3, a: &Animal, alpha: f32, time: f32) {
    let flying = a.flee > 0.0;
    let mut r = Rig::new(b, p, a.yaw, 1.0);
    r.alpha = alpha;
    r.part(Vec3::new(0.0, 0.2, 0.0), Vec3::new(0.13, 0.17, 0.12), a.tint);
    r.part(Vec3::new(0.0, 0.42, 0.0), Vec3::new(0.12, 0.1, 0.11), a.tint);
    for x in [-0.07, 0.07] {
        r.part(Vec3::new(x, 0.54, 0.0), Vec3::new(0.025, 0.04, 0.025), a.tint);
    }
    if flying {
        let flap = (time * 10.0).sin() * 0.15;
        r.part(Vec3::new(0.32, 0.26 + flap, 0.0), Vec3::new(0.22, 0.02, 0.1), a.tint);
        r.part(Vec3::new(-0.32, 0.26 + flap, 0.0), Vec3::new(0.22, 0.02, 0.1), a.tint);
    }
    // glowing eyes, blinking now and then
    if (time * 0.4 + a.phase).fract() > 0.06 {
        let mut e = Rig::new(fx, p, a.yaw, 1.0);
        e.alpha = alpha;
        for x in [-0.045, 0.045] {
            e.part_mat(Vec3::new(x, 0.44, 0.11), Vec3::new(0.025, 0.025, 0.01), [1.0, 0.85, 0.3], MAT_GLOW);
        }
    }
}

pub(super) fn draw_wolf(b: &mut Boxes, p: Vec3, a: &Animal, alpha: f32) {
    let mut r = Rig::new(b, p, a.yaw, 1.0);
    r.alpha = alpha;
    let moving = a.pos.distance(a.target) > 0.1 && a.graze <= 0.0;
    let sw = if moving { (a.phase * 9.0).sin() * 0.1 } else { 0.0 };
    let head = if a.graze > 0.0 { 0.18 } else { 0.0 };
    r.part(Vec3::new(0.0, 0.52, 0.0), Vec3::new(0.14, 0.16, 0.42), a.tint);
    r.part(Vec3::new(0.0, 0.62 - head, 0.5), Vec3::new(0.11, 0.11, 0.13), a.tint);
    r.part(Vec3::new(0.0, 0.56 - head, 0.68), Vec3::new(0.06, 0.05, 0.08), [0.3, 0.28, 0.28]);
    for x in [-0.06, 0.06] {
        r.part(Vec3::new(x, 0.76 - head, 0.46), Vec3::new(0.03, 0.06, 0.02), a.tint);
    }
    r.part(Vec3::new(0.0, 0.5, -0.55), Vec3::new(0.06, 0.07, 0.2), a.tint);
    for (x, z, s) in [(0.08, 0.3, 1.0), (-0.08, 0.3, -1.0), (0.08, -0.3, -1.0), (-0.08, -0.3, 1.0)] {
        r.part(Vec3::new(x, 0.2, z + sw * s), Vec3::new(0.04, 0.2, 0.04), [0.35, 0.34, 0.36]);
    }
}

/// A salmon leaping a weir: most of the time it's under the water and out of
/// sight, then it breaks the surface below the weir and arcs up over the sill.
/// Some tries fall short and drop back. `a.graze` is how often it tries.
pub(super) fn draw_salmon(b: &mut Boxes, p: Vec3, a: &Animal) {
    const LEAP: f32 = 1.1;
    let period = a.graze.max(1.0);
    let cycle = a.phase / period;
    let t = cycle.fract() * period / LEAP;
    if t > 1.0 {
        return;
    }
    let fails = hash_u32(a.graze.to_bits() ^ (cycle.floor() as i32 as u32).wrapping_mul(0x9E37_79B1)) % 3 == 0;
    let reach = if fails { 0.4 } else { 1.0 };
    let fwd = Vec3::new(a.yaw.sin(), 0.0, a.yaw.cos());
    let k = t;
    let up = (k * std::f32::consts::PI).sin();
    let at = p + fwd * (k * 2.0 * WEIR_FOOT * reach) + Vec3::Y * (up * 1.1 - 0.15);
    let mut r = Rig::new(b, at, a.yaw, 1.0);
    // nose up on the way up, down on the way back
    let pitch = 0.5 - k;
    r.part(Vec3::new(0.0, 0.12 + 0.1 * pitch, 0.0), Vec3::new(0.05, 0.06, 0.2), a.tint);
    r.part(Vec3::new(0.0, 0.12 + 0.18 * pitch, -0.24), Vec3::new(0.02, 0.07, 0.06), a.tint);
    // splash where it breaks the surface and where it comes down
    for s in [0.0, 1.0] {
        let d = (k - s).abs();
        if d < 0.12 {
            let spot = p + fwd * (s * 2.0 * WEIR_FOOT * reach);
            let w = 0.12 + d * 2.0;
            r.b.push(Vec3::new(spot.x, p.y - 0.04, spot.z), Vec3::new(w, 0.03, w), 0.0, [0.92, 0.96, 1.0], 0.0);
        }
    }
}

impl Fauna {
    /// `tide` is how far the water stands above its usual level; whatever floats rides up with it.
    pub fn draw(&self, b: &mut Boxes, fx: &mut Boxes, view: Vec2, view_r: f32, time: f32, tide: f32) {
        for g in self.groups.values().filter(|g| !g.hidden && g.active > 0.02) {
            let alpha = g.active;
            for a in g.members.iter().take(g.shown()) {
                if a.pos.distance(view) > view_r {
                    continue;
                }
                let floats = matches!(g.kind, Kind::Duck | Kind::Boat | Kind::Swan | Kind::Salmon);
                let p = Vec3::new(a.pos.x, a.y + if floats { tide } else { 0.0 }, a.pos.y);
                match g.kind {
                    Kind::Sheep => draw_sheep(b, p, a),
                    Kind::Cow => draw_cow(b, p, a),
                    Kind::Duck => draw_duck(b, p, a, time),
                    Kind::Boat => draw_boat(b, p, a, time),
                    Kind::Swan => draw_swan(b, p, a, time),
                    Kind::Heron => draw_heron(b, p, a),
                    Kind::Deer => draw_deer(b, p, a, alpha),
                    Kind::Fox => draw_fox(b, p, a, alpha),
                    Kind::Rabbit => draw_rabbit(b, p, a, alpha),
                    Kind::Owl => draw_owl(b, fx, p, a, alpha, time),
                    Kind::Salmon => draw_salmon(b, p, a),
                    Kind::Wolf => draw_wolf(b, p, a, alpha),
                }
            }
        }
        // gulls wheeling over the fishing boats by day
        if self.night < 0.6 {
            for g in self.groups.values().filter(|g| !g.hidden && g.kind == Kind::Boat && g.active > 0.02) {
                for (i, a) in g.members.iter().enumerate() {
                    if a.pos.distance(view) > view_r {
                        continue;
                    }
                    // fewer by the quay at dusk: boats that are tied up draw no gulls
                    for k in 0..3u32 {
                        let w = 0.55 + 0.17 * k as f32;
                        let ang = time * w + a.phase * 2.0 + k as f32 * 2.1 + i as f32;
                        let rad = 3.2 + 1.6 * k as f32;
                        let p = Vec3::new(a.pos.x + ang.cos() * rad, a.y + tide + 4.5 + (time * 0.9 + k as f32).sin() * 0.8, a.pos.y + ang.sin() * rad);
                        let yaw = yaw_of(Vec2::new(-ang.sin(), ang.cos()));
                        let flap = (time * 8.0 + k as f32 * 1.7).sin() * 0.2;
                        let mut r = Rig::new(b, p, yaw, 1.0);
                        r.part(Vec3::ZERO, Vec3::new(0.07, 0.06, 0.2), [0.96, 0.96, 0.95]);
                        r.part(Vec3::new(0.0, 0.0, 0.16), Vec3::new(0.04, 0.04, 0.05), [0.95, 0.75, 0.2]);
                        r.part(Vec3::new(0.28, flap, 0.0), Vec3::new(0.26, 0.02, 0.09), [0.8, 0.82, 0.85]);
                        r.part(Vec3::new(-0.28, flap, 0.0), Vec3::new(0.26, 0.02, 0.09), [0.8, 0.82, 0.85]);
                    }
                }
            }
        }
        for h in &self.herds {
            for a in &h.members {
                if a.pos.distance(view) <= view_r {
                    draw_deer(b, Vec3::new(a.pos.x, a.y, a.pos.y), a, 1.0);
                }
            }
        }
        for nb in &self.narrowboats {
            let p = Vec3::new(nb.x, SEA as f32 + 0.6 + tide, nb.canal as f32 + 0.5);
            if Vec2::new(p.x, p.z).distance(view) > view_r + 4.0 {
                continue;
            }
            let mut r = Rig::new(b, p, yaw_of(Vec2::new(nb.dir, 0.0)), 1.0);
            r.part(Vec3::new(0.0, 0.2, 0.0), Vec3::new(0.55, 0.3, 3.2), nb.hull);
            r.part(Vec3::new(0.0, 0.48, 0.0), Vec3::new(0.56, 0.04, 3.21), nb.trim);
            r.part(Vec3::new(0.0, 0.8, -0.4), Vec3::new(0.5, 0.32, 2.2), nb.hull);
            r.part(Vec3::new(0.0, 1.14, -0.4), Vec3::new(0.52, 0.04, 2.25), [0.25, 0.22, 0.2]);
            for z in [-1.8, -0.9, 0.0, 0.9] {
                r.part(Vec3::new(0.51, 0.85, z), Vec3::new(0.02, 0.12, 0.2), [0.95, 0.85, 0.5]);
                r.part(Vec3::new(-0.51, 0.85, z), Vec3::new(0.02, 0.12, 0.2), [0.95, 0.85, 0.5]);
            }
            r.part(Vec3::new(0.0, 1.45, 1.2), Vec3::new(0.07, 0.28, 0.07), [0.2, 0.2, 0.22]);
            // the steerer at the tiller
            r.part(Vec3::new(0.0, 0.9, -2.9), Vec3::new(0.18, 0.25, 0.12), nb.trim);
            r.part(Vec3::new(0.0, 1.32, -2.9), Vec3::new(0.15, 0.15, 0.15), [0.93, 0.78, 0.62]);
        }
        for f in &self.flocks {
            let yaw = yaw_of(f.dir);
            let right = Vec2::new(f.dir.y, -f.dir.x);
            let col = if f.dark { [0.2, 0.2, 0.24] } else { [0.95, 0.95, 0.95] };
            for bd in &f.birds {
                let o = right * bd.off.x + f.dir * bd.off.y;
                let p = f.pos + Vec3::new(o.x, (bd.phase * 0.3).sin() * 0.3, o.y);
                let flap = bd.phase.sin() * 0.25;
                let mut r = Rig::new(b, p, yaw, 1.0);
                r.part(Vec3::ZERO, Vec3::new(0.08, 0.07, 0.22), col);
                r.part(Vec3::new(0.3, flap, 0.0), Vec3::new(0.26, 0.02, 0.1), col);
                r.part(Vec3::new(-0.3, flap, 0.0), Vec3::new(0.26, 0.02, 0.1), col);
            }
        }
    }
}
