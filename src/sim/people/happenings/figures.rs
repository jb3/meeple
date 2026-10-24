use super::*;

pub(super) fn garb(seed: u32, shirt: Option<[f32; 3]>, pants: Option<[f32; 3]>) -> Garb {
    let h = |k: u32| hash_u32(seed ^ k) as usize;
    Garb {
        shirt: shirt.unwrap_or_else(|| col(SHIRTS[h(1) % SHIRTS.len()])),
        pants: pants.unwrap_or_else(|| col(PANTS[h(2) % PANTS.len()])),
        skin: col(SKIN[h(3) % SKIN.len()]),
        hair: col(HAIR[h(4) % HAIR.len()]),
        hat: Hat::None,
        hat_col: BLACK,
        scale: 1.0,
    }
}

/// Where a walker has got to after `d` along a path, which way they face,
/// and whether they're still on the move.
pub(super) fn walk(path: &[Vec2], d: f32) -> (Vec2, Vec2, bool) {
    let mut left = d.max(0.0);
    for w in path.windows(2) {
        let seg = w[1] - w[0];
        let len = seg.length();
        if left <= len {
            let dir = if len > 1e-4 { seg / len } else { Vec2::Y };
            return (w[0] + dir * left, dir, true);
        }
        left -= len;
    }
    let n = path.len();
    let dir = if n >= 2 { (path[n - 1] - path[n - 2]).normalize_or_zero() } else { Vec2::Y };
    (path[n - 1], dir, false)
}

pub(super) fn path_len(path: &[Vec2]) -> f32 {
    path.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// A limb swung from `pivot` (its top) through `ang` radians about the sideways
/// axis, positive being forward. Boxes can't tilt, so it's a short chain of
/// small blocks laid along the swing, which reads as a rotating arm or leg.
pub(super) fn limb(r: &mut Rig, pivot: Vec3, len: f32, ang: f32, hx: f32, col: [f32; 3]) {
    let n = if ang.abs() < 0.04 { 1 } else { 4 };
    let seg = len / n as f32;
    for i in 0..n {
        let d = (i as f32 + 0.5) * seg;
        let c = pivot + Vec3::new(0.0, -d * ang.cos(), d * ang.sin());
        r.part(c, Vec3::new(hx, seg * 0.5 + if n > 1 { 0.03 } else { 0.0 }, hx), col);
    }
}

/// A person, drawn from a few boxes.
pub(super) fn figure(b: &mut Boxes, pos: Vec3, yaw: f32, phase: f32, g: &Garb, pose: Pose, prop: Prop, time: f32, alpha: f32) {
    let mut r = Rig::new(b, pos, yaw, g.scale);
    r.squash = FLATTEN.with(|f| f.get());
    r.alpha = alpha;
    let dark = [g.shirt[0] * 0.85, g.shirt[1] * 0.85, g.shirt[2] * 0.85];
    let drunk = pose == Pose::Drunk;
    // an uneven gait, a heavy lurch at every step, the body swaying over the
    // feet from side to side and weaving forwards and back
    let sw = match pose {
        Pose::Walk => phase.sin(),
        Pose::Drunk => (phase.sin() + 0.5 * (phase * 1.7 + 1.0).sin()).clamp(-1.2, 1.2),
        _ => 0.0,
    };
    let bob = match pose {
        Pose::Walk => (phase * 2.0).sin().abs() * 0.035,
        Pose::Drunk => (phase * 2.0).sin().abs() * 0.07,
        Pose::Lift => (phase * 2.0).sin().abs() * 0.025,
        _ => 0.0,
    };
    let lat = if drunk { 0.14 * (phase * 0.5).sin() + 0.06 * (phase * 1.3 + 0.7).sin() } else { 0.0 };
    let wide = if drunk || pose == Pose::Lift { 0.16 } else { 0.11 };
    let kneel = pose == Pose::Kneel;
    let low = if kneel { 0.3 } else { 0.0 };
    // a skater's stroke: one leg straight down to glide on while the other
    // kicks back, then, slowly, the other way about; the arms swing against
    // the legs through the same stroke
    let stroke = if pose == Pose::Skate { (phase * 0.9).sin() } else { 0.0 };
    if pose == Pose::Skate {
        let kick = |u: f32| -0.7 * u.max(0.0) + 0.12 * (-u).max(0.0);
        limb(&mut r, Vec3::new(0.1, 0.56, 0.0), 0.56, kick(stroke), 0.09, g.pants);
        limb(&mut r, Vec3::new(-0.1, 0.56, 0.0), 0.56, kick(-stroke), 0.09, g.pants);
    } else if kneel {
        r.part(Vec3::new(0.1, 0.2, 0.15), Vec3::new(0.09, 0.2, 0.18), g.pants);
        r.part(Vec3::new(-0.1, 0.2, 0.15), Vec3::new(0.09, 0.2, 0.18), g.pants);
    } else {
        // legs swing from the hip
        let swing = if drunk { 0.65 } else { 0.5 };
        limb(&mut r, Vec3::new(wide, 0.56, 0.0), 0.56, sw * swing, 0.09, g.pants);
        limb(&mut r, Vec3::new(-wide, 0.56, 0.0), 0.56, -sw * swing, 0.09, g.pants);
    }
    let lean = if pose == Pose::Skate {
        0.1
    } else if drunk {
        0.1 * (phase * 0.5 + 1.2).sin()
    } else {
        0.0
    };
    let ty = 0.82 + bob - low;
    r.part(Vec3::new(lat, ty, lean), Vec3::new(0.22, 0.26, 0.14), g.shirt);
    match pose {
        Pose::Walk => {
            let held = matches!(prop, Prop::Bag | Prop::Posy | Prop::Baby | Prop::Mug);
            limb(&mut r, Vec3::new(0.3, ty + 0.22, 0.0), 0.48, -sw * 0.5, 0.07, dark);
            if held {
                r.part(Vec3::new(-0.24, ty + 0.05, 0.15), Vec3::new(0.07, 0.07, 0.17), dark);
            } else {
                limb(&mut r, Vec3::new(-0.3, ty + 0.22, 0.0), 0.48, sw * 0.5, 0.07, dark);
            }
        }
        Pose::Drunk => {
            // arms flung about for balance: one swings right up past the
            // shoulder and back, the other swings against it (or holds the mug
            // close, if there is one)
            let flail = (phase * 0.9).sin();
            limb(&mut r, Vec3::new(0.3 + lat, ty + 0.22, 0.0), 0.48, 0.15 + 1.9 * flail.max(0.0) - 0.4 * (-flail).max(0.0), 0.07, dark);
            if matches!(prop, Prop::Mug) {
                r.part(Vec3::new(-0.24 + lat, ty + 0.05, 0.15), Vec3::new(0.07, 0.07, 0.17), dark);
            } else {
                limb(&mut r, Vec3::new(-0.3 + lat, ty + 0.22, 0.0), 0.48, -0.6 * flail, 0.07, dark);
            }
        }
        Pose::Lift => {
            // dumbbell curls, the arms taking turns: the forearm comes up from
            // the hip as the weight rises
            let up = |ph: f32| 0.5 + 0.5 * ph.sin();
            for (x, ph) in [(0.3, phase), (-0.3, phase + PI)] {
                let ang = 0.12 + 2.1 * up(ph);
                limb(&mut r, Vec3::new(x, ty + 0.22, 0.0), 0.48, ang, 0.07, g.skin);
                let end = Vec3::new(x, ty + 0.22 - 0.58 * ang.cos(), 0.58 * ang.sin());
                r.part(end, Vec3::new(0.15, 0.045, 0.045), [0.17, 0.17, 0.2]);
                for dx in [-0.15, 0.15] {
                    r.part(end + Vec3::new(dx, 0.0, 0.0), Vec3::new(0.03, 0.1, 0.1), [0.1, 0.1, 0.12]);
                }
            }
        }
        Pose::Stand => {
            for x in [-0.3, 0.3] {
                r.part(Vec3::new(x, ty - 0.02, 0.0), Vec3::new(0.07, 0.24, 0.07), dark);
            }
        }
        Pose::Cheer => {
            let s = (time * 9.0 + phase).sin() * 0.08;
            r.part(Vec3::new(0.3, ty + 0.38 + s, 0.0), Vec3::new(0.07, 0.24, 0.07), dark);
            r.part(Vec3::new(-0.3, ty + 0.38 - s, 0.0), Vec3::new(0.07, 0.24, 0.07), dark);
        }
        Pose::Dig => {
            let s = (time * 3.2 + phase).sin() * 0.5 + 0.5;
            for x in [-0.22, 0.22] {
                r.part(Vec3::new(x, ty - 0.02, 0.2), Vec3::new(0.07, 0.07, 0.2), dark);
            }
            r.part(Vec3::new(0.0, ty - 0.15 + s * 0.3, 0.45), Vec3::new(0.03, 0.4, 0.03), [0.55, 0.4, 0.25]);
            r.part(Vec3::new(0.0, ty - 0.52 + s * 0.3, 0.52), Vec3::new(0.14, 0.03, 0.08), [0.6, 0.62, 0.66]);
        }
        Pose::Pass => {
            let s = (time * 2.0 + phase).sin() * 0.1;
            for x in [-0.2, 0.2] {
                r.part(Vec3::new(x, ty - 0.02, 0.22 + s), Vec3::new(0.07, 0.07, 0.22), dark);
            }
        }
        Pose::Play => {
            let s = (time * 4.5 + phase).sin();
            let wood = [0.62, 0.36, 0.2];
            r.part(Vec3::new(-0.08, ty + 0.2, 0.2), Vec3::new(0.09, 0.04, 0.16), wood);
            r.part(Vec3::new(-0.22, ty + 0.14, 0.18), Vec3::new(0.07, 0.07, 0.2), dark);
            r.part(Vec3::new(0.24, ty - 0.02 + s * 0.04, 0.12 + s * 0.05), Vec3::new(0.07, 0.07, 0.2), dark);
            r.part(Vec3::new(0.0 + s * 0.12, ty + 0.12, 0.36), Vec3::new(0.2, 0.012, 0.012), [0.9, 0.85, 0.7]);
        }
        Pose::Skate => {
            // arms at the sides, swinging against the legs: the arm on the side
            // of the kicking leg goes forward
            limb(&mut r, Vec3::new(0.3, ty + 0.22, 0.0), 0.48, 0.75 * stroke, 0.07, dark);
            limb(&mut r, Vec3::new(-0.3, ty + 0.22, 0.0), 0.48, -0.75 * stroke, 0.07, dark);
        }
        Pose::Swing => {
            // phase 0..1 through the stroke
            let a = phase;
            r.part(Vec3::new(0.0, ty + 0.1 - a * 0.25, 0.3), Vec3::new(0.09, 0.07, 0.22), dark);
            r.part(Vec3::new(0.4 - a * 0.8, ty + 0.35 - a * 0.4, 0.28 + a * 0.35), Vec3::new(0.04, 0.4, 0.08), [0.82, 0.7, 0.45]);
        }
        Pose::Kneel => {
            for x in [-0.2, 0.2] {
                r.part(Vec3::new(x, ty - 0.12, 0.28), Vec3::new(0.07, 0.07, 0.24), dark);
            }
        }
        Pose::Reach => {
            let s = (time * 2.0 + phase).sin() * 0.1;
            r.part(Vec3::new(0.26, ty + 0.4 + s, 0.1), Vec3::new(0.07, 0.26, 0.07), dark);
            r.part(Vec3::new(-0.26, ty - 0.02, 0.14), Vec3::new(0.07, 0.07, 0.18), dark);
        }
    }
    match prop {
        Prop::None => {}
        Prop::Bag => r.part(Vec3::new(-0.34, ty - 0.35, 0.0), Vec3::new(0.12, 0.1, 0.07), [0.12, 0.1, 0.1]),
        Prop::Pole => {
            r.part(Vec3::new(0.34, ty + 0.5, 0.16), Vec3::new(0.025, 1.05, 0.025), [0.45, 0.32, 0.2]);
            let flick = 0.85 + 0.15 * (time * 5.0 + phase).sin();
            r.part_mat(Vec3::new(0.34, ty + 1.62, 0.16), Vec3::new(0.07, 0.08, 0.07), [1.0 * flick, 0.7 * flick, 0.3], MAT_GLOW);
        }
        Prop::Bat => r.part(Vec3::new(0.3, ty - 0.25, 0.05), Vec3::new(0.03, 0.3, 0.07), [0.82, 0.7, 0.45]),
        Prop::Baby => {
            r.part(Vec3::new(0.0, ty - 0.02, 0.28), Vec3::new(0.13, 0.08, 0.2), [0.98, 0.97, 0.94]);
            r.part(Vec3::new(0.0, ty + 0.06, 0.4), Vec3::new(0.07, 0.07, 0.07), g.skin);
        }
        Prop::Posy => r.part(Vec3::new(-0.2, ty + 0.05, 0.34), Vec3::new(0.08, 0.08, 0.06), [0.95, 0.55, 0.65]),
        Prop::Mug => r.part(Vec3::new(-0.24 + lat, ty + 0.1, 0.3), Vec3::new(0.05, 0.07, 0.05), [0.85, 0.7, 0.3]),
    }
    let hy = 1.27 + bob - low;
    // a drunk's head lolls a little further than the body sways
    let hx = lat * 1.5;
    r.part(Vec3::new(hx, hy, lean), Vec3::new(0.17, 0.17, 0.17), g.skin);
    for x in [-0.07, 0.07] {
        r.part(Vec3::new(x + hx, hy + 0.02, 0.17 + lean), Vec3::new(0.025, 0.035, 0.01), [0.1, 0.08, 0.08]);
    }
    r.part(Vec3::new(hx, hy + 0.16, -0.02 + lean), Vec3::new(0.18, 0.05, 0.18), g.hair);
    r.part(Vec3::new(hx, hy + 0.02, -0.16 + lean), Vec3::new(0.18, 0.14, 0.04), g.hair);
    match g.hat {
        Hat::Straw => {
            r.part(Vec3::new(hx, hy + 0.19, lean), Vec3::new(0.32, 0.025, 0.32), g.hat_col);
            r.part(Vec3::new(hx, hy + 0.25, lean), Vec3::new(0.17, 0.06, 0.17), g.hat_col);
        }
        Hat::Chef => r.part(Vec3::new(hx, hy + 0.3, lean), Vec3::new(0.16, 0.14, 0.16), g.hat_col),
        Hat::Cap => {
            r.part(Vec3::new(hx, hy + 0.2, lean), Vec3::new(0.19, 0.05, 0.19), g.hat_col);
            r.part(Vec3::new(hx, hy + 0.16, 0.2 + lean), Vec3::new(0.15, 0.02, 0.08), g.hat_col);
        }
        Hat::Bun => r.part(Vec3::new(hx, hy + 0.24, -0.1 + lean), Vec3::new(0.08, 0.08, 0.08), g.hair),
        Hat::None => {}
    }
}

pub(super) fn horse(b: &mut Boxes, pos: Vec3, yaw: f32, phase: f32, moving: bool, coat: [f32; 3]) {
    let mut r = Rig::new(b, pos, yaw, 1.0);
    let sw = if moving { phase.sin() * 0.22 } else { 0.0 };
    r.part(Vec3::new(0.0, 1.05, 0.0), Vec3::new(0.26, 0.28, 0.62), coat);
    r.part(Vec3::new(0.0, 1.45, 0.6), Vec3::new(0.14, 0.28, 0.16), coat);
    r.part(Vec3::new(0.0, 1.55, 0.88), Vec3::new(0.12, 0.13, 0.22), coat);
    r.part(Vec3::new(0.0, 1.2, -0.66), Vec3::new(0.05, 0.2, 0.05), [0.15, 0.1, 0.08]);
    for (x, z, s) in [(0.14, 0.4, 1.0), (-0.14, 0.4, -1.0), (0.14, -0.4, -1.0), (-0.14, -0.4, 1.0)] {
        r.part(Vec3::new(x, 0.38, z + sw * s), Vec3::new(0.06, 0.38, 0.06), coat);
    }
}

/// A passer-by for scenes elsewhere: standing (0), walking (1) or on skis (2).
pub fn extra(b: &mut Boxes, pos: Vec3, yaw: f32, phase: f32, seed: u32, how: u8, time: f32) {
    let g = garb(seed, None, None).hat(if seed % 3 == 0 { Hat::Cap } else { Hat::None }, [0.7, 0.2, 0.2]);
    match how {
        0 => figure(b, pos, yaw, phase, &g, Pose::Stand, Prop::None, time, 1.0),
        1 => figure(b, pos, yaw, phase, &g, Pose::Walk, Prop::None, time, 1.0),
        _ => {
            figure(b, pos + Vec3::Y * 0.06, yaw, 0.0, &g, Pose::Skate, Prop::None, time, 1.0);
            for x in [-0.14, 0.14] {
                let mut r = Rig::new(b, pos, yaw, 1.0);
                r.part(Vec3::new(x, 0.03, 0.2), Vec3::new(0.04, 0.025, 0.9), [0.85, 0.2, 0.2]);
            }
        }
    }
}

/// A wooden pail with iron hoops, brimming with water if it's full.
pub(super) fn pail(b: &mut Boxes, p: Vec3, full: bool) {
    let wood = [0.62, 0.44, 0.26];
    b.push(p, Vec3::new(0.17, 0.14, 0.17), 0.0, wood, MAT_LIT);
    b.push(p + Vec3::Y * 0.1, Vec3::new(0.2, 0.025, 0.2), 0.0, [0.2, 0.2, 0.22], MAT_LIT);
    b.push(p - Vec3::Y * 0.08, Vec3::new(0.19, 0.025, 0.19), 0.0, [0.2, 0.2, 0.22], MAT_LIT);
    b.push(p + Vec3::Y * 0.2, Vec3::new(0.14, 0.01, 0.01), 0.0, [0.25, 0.25, 0.27], MAT_LIT);
    if full {
        b.push(p + Vec3::Y * 0.14, Vec3::new(0.15, 0.02, 0.15), 0.0, [0.35, 0.62, 0.92], MAT_UNLIT);
    }
}

pub(super) fn ease(x: f32) -> f32 {
    smoothstep(0.0, 1.0, x.clamp(0.0, 1.0))
}

/// How fierce a fire is, `t` seconds in: it takes hold, burns, and is put
/// out by the bucket chain and the pump.
pub(super) fn fire_level(t: f32) -> f32 {
    if t < 0.0 {
        0.0
    } else if t < 10.0 {
        t / 10.0
    } else if t < 60.0 {
        1.0
    } else if t < 96.0 {
        1.0 - (t - 60.0) / 36.0
    } else {
        0.0
    }
}

impl Garb {
    pub(super) fn hat(mut self, hat: Hat, c: [f32; 3]) -> Garb {
        self.hat = hat;
        self.hat_col = c;
        self
    }

    pub(super) fn kid(mut self) -> Garb {
        self.scale = 0.68;
        self
    }
}

/// A resident at home, for the view through a cut-away roof.
pub(in crate::sim::people) fn resident(b: &mut Boxes, p: &Person, pos: Vec3, yaw: f32, time: f32, moving: bool) {
    let g = Garb { shirt: p.look.shirt, pants: p.look.pants, skin: p.look.skin, hair: p.look.hair, hat: Hat::None, hat_col: p.look.hat_col, scale: p.look.scale };
    figure(b, pos, yaw, p.phase + time * 4.0, &g, if moving { Pose::Walk } else { Pose::Stand }, Prop::None, time, 1.0);
}

/// A passer-by in particular clothes, for scenes elsewhere: `how` picks the
/// pose (0 standing, 1 walking, 3 reaching up, 4 kneeling, 5 walking with a
/// torch), `straw` gives them a straw hat, and `alpha` below 1 makes them see-through.
#[allow(clippy::too_many_arguments)]
pub fn extra_clad(b: &mut Boxes, pos: Vec3, yaw: f32, phase: f32, seed: u32, how: u8, time: f32, shirt: [f32; 3], pants: [f32; 3], straw: bool, alpha: f32) {
    let mut g = garb(seed, Some(shirt), Some(pants));
    if straw {
        g = g.hat(Hat::Straw, [0.85, 0.75, 0.43]);
    }
    let (pose, prop) = match how {
        1 => (Pose::Walk, Prop::None),
        3 => (Pose::Reach, Prop::None),
        4 => (Pose::Kneel, Prop::None),
        5 => (Pose::Walk, Prop::Pole),
        _ => (Pose::Stand, Prop::None),
    };
    figure(b, pos, yaw, phase, &g, pose, prop, time, alpha);
}
