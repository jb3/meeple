use super::*;

pub(super) fn wheels(r: &mut Rig, zs: &[f32]) {
    let dark = [0.16, 0.15, 0.17];
    for &z in zs {
        for x in [-0.82, 0.82] {
            r.part(Vec3::new(x, 0.32, z), Vec3::new(0.07, 0.3, 0.3), dark);
        }
    }
    r.part(Vec3::new(0.0, 0.62, 0.0), Vec3::new(0.9, 0.1, 2.15), dark);
}

pub(super) fn draw_car(b: &mut Boxes, car: Car, p: Vec3, yaw: f32, night: f32) {
    let mut r = Rig::new(b, p, yaw, 1.0);
    let dark = [0.16, 0.15, 0.17];
    match car {
        Car::Loco(c) => {
            wheels(&mut r, &[-1.3, -0.2, 0.9]);
            r.part(Vec3::new(0.0, 1.28, 0.55), Vec3::new(0.66, 0.56, 1.5), c);
            r.part(Vec3::new(0.0, 1.28, 2.06), Vec3::new(0.6, 0.5, 0.04), dark);
            r.part(Vec3::new(0.0, 1.5, -1.35), Vec3::new(0.86, 0.86, 0.72), c);
            r.part(Vec3::new(0.0, 2.42, -1.35), Vec3::new(0.95, 0.07, 0.82), dark);
            r.part(Vec3::new(0.87, 1.8, -1.35), Vec3::new(0.02, 0.22, 0.35), [1.0, 0.85, 0.5]);
            r.part(Vec3::new(-0.87, 1.8, -1.35), Vec3::new(0.02, 0.22, 0.35), [1.0, 0.85, 0.5]);
            r.part(Vec3::new(0.0, 2.1, 1.45), Vec3::new(0.18, 0.32, 0.18), dark);
            r.part(Vec3::new(0.0, 1.95, 0.4), Vec3::new(0.22, 0.14, 0.22), [0.85, 0.7, 0.3]);
            r.part(Vec3::new(0.0, 0.62, 2.2), Vec3::new(0.85, 0.1, 0.06), [0.75, 0.15, 0.12]);
            let lamp = if night > 0.3 { MAT_GLOW } else { 0.0 };
            r.part_mat(Vec3::new(0.0, 1.62, 2.12), Vec3::new(0.14, 0.14, 0.04), [1.0, 0.9, 0.6], lamp);
        }
        Car::Coach(c) => {
            wheels(&mut r, &[-1.4, 1.4]);
            r.part(Vec3::new(0.0, 1.45, 0.0), Vec3::new(0.95, 0.72, 2.1), c);
            r.part(Vec3::new(0.0, 2.22, 0.0), Vec3::new(1.0, 0.08, 2.15), [c[0] * 0.5, c[1] * 0.5, c[2] * 0.5]);
            r.part(Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.97, 0.06, 2.12), [0.85, 0.8, 0.65]);
            for z in [-1.45, -0.48, 0.48, 1.45] {
                for x in [-0.96, 0.96] {
                    r.part_mat(Vec3::new(x, 1.62, z), Vec3::new(0.02, 0.22, 0.3), [1.0, 0.8, 0.45], MAT_NIGHT_WINDOW);
                }
            }
        }
        Car::Freight(c) => {
            wheels(&mut r, &[-1.4, 1.4]);
            r.part(Vec3::new(0.0, 1.35, 0.0), Vec3::new(0.9, 0.62, 2.05), c);
            r.part(Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.95, 0.06, 2.1), [0.3, 0.28, 0.27]);
            r.part(Vec3::new(0.91, 1.3, 0.0), Vec3::new(0.02, 0.45, 0.5), [c[0] * 0.7, c[1] * 0.7, c[2] * 0.7]);
            r.part(Vec3::new(-0.91, 1.3, 0.0), Vec3::new(0.02, 0.45, 0.5), [c[0] * 0.7, c[1] * 0.7, c[2] * 0.7]);
        }
        Car::Logs => {
            wheels(&mut r, &[-1.4, 1.4]);
            r.part(Vec3::new(0.0, 0.8, 0.0), Vec3::new(0.95, 0.08, 2.1), [0.45, 0.32, 0.22]);
            for (x, y) in [(-0.5, 1.15), (0.0, 1.15), (0.5, 1.15), (-0.25, 1.55), (0.25, 1.55)] {
                r.part(Vec3::new(x, y, 0.0), Vec3::new(0.23, 0.22, 2.0), [0.5, 0.36, 0.24]);
            }
        }
    }
}

/// A vintage motor car in a cart's place: black or coloured body, running
/// boards, a folded hood and brass lamps.
pub(super) fn draw_motor(b: &mut Boxes, c: &Cart, night: f32) {
    let mut r = Rig::new(b, Vec3::new(c.pos.x, c.y, c.pos.y), c.yaw, 1.0);
    let body = c.body;
    let dark = [0.14, 0.14, 0.16];
    r.part(Vec3::new(0.0, 0.62, 0.25), Vec3::new(0.52, 0.28, 1.15), body);
    r.part(Vec3::new(0.0, 0.8, 1.3), Vec3::new(0.4, 0.3, 0.5), body);
    r.part(Vec3::new(0.0, 1.12, 1.3), Vec3::new(0.34, 0.03, 0.46), dark);
    r.part(Vec3::new(0.0, 1.1, -0.35), Vec3::new(0.5, 0.025, 0.4), dark);
    r.part(Vec3::new(0.0, 1.4, 0.05), Vec3::new(0.46, 0.06, 0.55), [0.2, 0.19, 0.2]);
    for x in [-0.46, 0.46] {
        r.part(Vec3::new(x, 1.25, 0.3), Vec3::new(0.03, 0.15, 0.03), dark);
        r.part(Vec3::new(x, 1.25, -0.3), Vec3::new(0.03, 0.15, 0.03), dark);
    }
    for (x, z) in [(-0.55, 1.2), (0.55, 1.2), (-0.55, -0.5), (0.55, -0.5)] {
        r.part(Vec3::new(x, 0.36, z), Vec3::new(0.07, 0.36, 0.36), dark);
    }
    r.part(Vec3::new(0.0, 0.62, 1.88), Vec3::new(0.3, 0.2, 0.03), [0.8, 0.7, 0.3]);
    for x in [-0.3, 0.3] {
        r.part_mat(Vec3::new(x, 0.85, 1.9), Vec3::splat(0.08), [1.0, 0.9, 0.55], if night > 0.4 { MAT_GLOW } else { 0.0 });
    }
    r.part(Vec3::new(0.0, 1.15, 0.25), Vec3::new(0.18, 0.22, 0.12), c.shirt);
    r.part(Vec3::new(0.0, 1.55, 0.25), Vec3::new(0.15, 0.15, 0.15), [0.93, 0.78, 0.62]);
    r.part(Vec3::new(0.0, 1.72, 0.25), Vec3::new(0.2, 0.05, 0.2), [0.15, 0.15, 0.2]);
}

pub(super) fn draw_cart(b: &mut Boxes, c: &Cart) {
    let mut r = Rig::new(b, Vec3::new(c.pos.x, c.y, c.pos.y), c.yaw, 1.0);
    let sw = c.phase.sin() * 0.12;
    let h = c.horse;
    // horse
    r.part(Vec3::new(0.0, 0.95, 1.2), Vec3::new(0.22, 0.25, 0.55), h);
    r.part(Vec3::new(0.0, 1.32, 1.72), Vec3::new(0.12, 0.25, 0.14), h);
    r.part(Vec3::new(0.0, 1.5, 1.95), Vec3::new(0.11, 0.11, 0.2), h);
    r.part(Vec3::new(0.0, 1.42, 1.58), Vec3::new(0.04, 0.22, 0.08), [0.2, 0.15, 0.12]);
    r.part(Vec3::new(0.0, 1.0, 0.62), Vec3::new(0.05, 0.2, 0.05), [0.2, 0.15, 0.12]);
    for (x, z, s) in [(0.13, 1.6, 1.0), (-0.13, 1.6, -1.0), (0.13, 0.8, -1.0), (-0.13, 0.8, 1.0)] {
        r.part(Vec3::new(x, 0.36, z + sw * s), Vec3::new(0.06, 0.36, 0.06), h);
    }
    // shafts + cart
    for x in [-0.32, 0.32] {
        r.part(Vec3::new(x, 0.85, 0.75), Vec3::new(0.03, 0.03, 0.5), [0.4, 0.3, 0.2]);
    }
    r.part(Vec3::new(0.0, 0.78, -0.35), Vec3::new(0.6, 0.12, 0.7), c.body);
    r.part(Vec3::new(0.6, 1.0, -0.35), Vec3::new(0.03, 0.18, 0.7), c.body);
    r.part(Vec3::new(-0.6, 1.0, -0.35), Vec3::new(0.03, 0.18, 0.7), c.body);
    r.part(Vec3::new(0.0, 1.0, -1.03), Vec3::new(0.6, 0.18, 0.03), c.body);
    for x in [-0.68, 0.68] {
        r.part(Vec3::new(x, 0.38, -0.4), Vec3::new(0.05, 0.36, 0.36), [0.35, 0.25, 0.18]);
    }
    match c.cargo {
        0 => r.part(Vec3::new(0.0, 1.2, -0.45), Vec3::new(0.5, 0.3, 0.5), [0.87, 0.75, 0.35]),
        1 => {
            for z in [-0.75, -0.2] {
                r.part(Vec3::new(0.0, 1.2, z), Vec3::new(0.24, 0.3, 0.24), [0.5, 0.33, 0.2]);
            }
        }
        _ => {}
    }
    // driver
    r.part(Vec3::new(0.0, 1.35, 0.15), Vec3::new(0.2, 0.25, 0.13), c.shirt);
    r.part(Vec3::new(0.0, 1.78, 0.15), Vec3::new(0.16, 0.16, 0.16), [0.93, 0.78, 0.62]);
    r.part(Vec3::new(0.0, 1.95, 0.15), Vec3::new(0.28, 0.025, 0.28), [0.85, 0.75, 0.43]);
}

/// A semaphore signal: a post with an arm that stands out level at danger
/// and is raised when the road is clear, and a lamp for the night.
pub(super) fn draw_signal(b: &mut Boxes, base: Vec3, fwd: Vec2, clear: bool, night: f32) {
    let yaw = yaw_of(fwd);
    let mut r = Rig::new(b, base, yaw, 1.35);
    let post = [0.22, 0.2, 0.2];
    r.part(Vec3::new(0.0, 1.6, 0.0), Vec3::new(0.07, 1.6, 0.07), post);
    r.part(Vec3::new(0.0, 3.27, 0.0), Vec3::new(0.1, 0.07, 0.1), post);
    // the arm points out over the track (to the driver's right)
    let red = [0.82, 0.16, 0.14];
    let white = [0.95, 0.94, 0.9];
    if clear {
        for k in 0..4 {
            let t = k as f32;
            r.part(Vec3::new(0.18 + t * 0.17, 2.85 + t * 0.17, -0.09), Vec3::new(0.1, 0.07, 0.025), if k == 2 { white } else { red });
        }
    } else {
        r.part(Vec3::new(0.45, 2.85, -0.09), Vec3::new(0.4, 0.07, 0.025), red);
        r.part(Vec3::new(0.62, 2.85, -0.1), Vec3::new(0.05, 0.072, 0.025), white);
    }
    let lamp = if clear { [0.3, 1.0, 0.45] } else { [1.0, 0.3, 0.2] };
    let mat = if night > 0.3 { MAT_GLOW } else { 0.0 };
    r.part_mat(Vec3::new(0.0, 2.55, -0.1), Vec3::new(0.06, 0.06, 0.03), lamp, mat);
}

/// One set of points: the switch blade lying against the route they are set
/// for, and the lever stand beside them with its handle thrown the same way
/// (leaning away from the track for straight on, over towards it for the
/// branch) under a white or red board.
pub(super) fn draw_points(b: &mut Boxes, gen: &Gen, c: &Chord, end: u8, branch: bool) {
    let len = c.length();
    let (p, branch_dir, main_dir) = if end == 0 {
        let dir = if c.cross_a >= c.a_s { 1.0 } else { -1.0 };
        let h = Vec2::from(c.a.point(c.a_s + dir * 3.0, 0.0)) - Vec2::from(c.a.point(c.a_s, 0.0));
        (c.at(0.0), c.tangent(0.0), h.normalize_or_zero())
    } else {
        let dir = if c.cross_b >= c.b_s { 1.0 } else { -1.0 };
        let h = Vec2::from(c.b.point(c.b_s + dir * 3.0, 0.0)) - Vec2::from(c.b.point(c.b_s, 0.0));
        (c.at(len), -c.tangent(len), h.normalize_or_zero())
    };
    let y = gen.rail_level(p.x, p.y) + 1.0;
    // the blade, starting on the rail nearest the branch
    let lateral = Vec2::new(-main_dir.y, main_dir.x);
    let off = c.at(if end == 0 { 8.0 } else { len - 8.0 }) - p;
    let side = if lateral.dot(off) >= 0.0 { 1.0 } else { -1.0 };
    let dir = if branch { branch_dir } else { main_dir };
    let base = p + lateral * side * 0.65;
    let mid = base + dir * 1.4;
    let steel = [0.82, 0.84, 0.9];
    b.push(Vec3::new(mid.x, y + 0.31, mid.y), Vec3::new(0.08, 0.05, 1.4), yaw_of(dir), steel, 0.0);
    // the tip, white for straight on, red for the branch
    let tip = base + dir * 2.7;
    let col = if branch { [0.85, 0.15, 0.12] } else { [0.95, 0.94, 0.9] };
    b.push(Vec3::new(tip.x, y + 0.33, tip.y), Vec3::new(0.09, 0.06, 0.14), yaw_of(dir), col, 0.0);
    // the lever stand, on the outside of the curve a little way along it
    let u = if end == 0 { 3.0 } else { len - 3.0 };
    let mid_c = c.at(len * 0.5);
    let q = c.at(u);
    let n = Vec2::new(-c.tangent(u).y, c.tangent(u).x);
    let out = if n.dot(q - mid_c) > 0.0 { n } else { -n };
    let s = q + out * 2.4;
    let sy = gen.rail_level(s.x, s.y).floor() + 1.0;
    let iron = [0.23, 0.22, 0.24];
    b.push(Vec3::new(s.x, sy + 0.15, s.y), Vec3::new(0.22, 0.15, 0.22), 0.0, iron, 0.0);
    // the handle leans towards the track when set for the branch
    let lean = if branch { -out * 0.3 } else { out * 0.3 };
    for k in 0..4 {
        let t = k as f32 / 3.0;
        let at = s + lean * t;
        let col = if k < 2 { [0.93, 0.93, 0.9] } else { [0.78, 0.2, 0.17] };
        b.push(Vec3::new(at.x, sy + 0.4 + k as f32 * 0.2, at.y), Vec3::new(0.05, 0.11, 0.05), 0.0, col, 0.0);
    }
    let top = s + lean;
    b.push(Vec3::new(top.x, sy + 1.2, top.y), Vec3::new(0.09, 0.05, 0.09), 0.0, iron, 0.0);
    // the board on a short post beside it: face on to the track when set for the branch
    let board = if branch { [0.85, 0.15, 0.12] } else { [0.95, 0.94, 0.9] };
    let post = s + Vec2::new(-out.y, out.x) * 0.35;
    b.push(Vec3::new(post.x, sy + 0.6, post.y), Vec3::new(0.03, 0.6, 0.03), 0.0, iron, 0.0);
    b.push(Vec3::new(post.x, sy + 1.35, post.y), Vec3::new(0.2, 0.17, 0.02), yaw_of(out), board, 0.0);
}

impl Traffic {
    pub fn draw(&self, gen: &Gen, b: &mut Boxes, view: Vec2, view_r: f32, night: f32) {
        for t in &self.trains {
            for (k, car) in t.cars.iter().enumerate() {
                let yaw = t.car_yaw(k);
                let p = t.car_pos(gen, k);
                if Vec2::new(p.x, p.z).distance(view) > view_r + 10.0 {
                    continue;
                }
                draw_car(b, *car, p, yaw, night);
            }
        }
        for c in &self.carts {
            if c.pos.distance(view) > view_r {
                continue;
            }
            if self.era >= 2 {
                draw_motor(b, c, night);
            } else {
                draw_cart(b, c);
            }
        }
        // the points at each junction and the levers that throw them
        let r = view_r as i32 + 10;
        for c in gen.chords_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r) {
            for end in 0..2u8 {
                let at = if end == 0 { c.at(0.0) } else { c.at(c.length()) };
                if at.distance(view) > view_r + 6.0 {
                    continue;
                }
                let branch = self.points.get(&(c.id, end)).copied().unwrap_or(false);
                draw_points(b, gen, &c, end, branch);
            }
        }
        // semaphore signals guarding each crossing, one per approach
        for (a, bl, c) in gen.crossings_near(view.x as i32 - r, view.y as i32 - r, view.x as i32 + r, view.y as i32 + r) {
            for side in 0..4u8 {
                let line = if side < 2 { a } else { bl };
                let dir = if side % 2 == 0 { 1.0 } else { -1.0 };
                let base = signal_pos_at(gen, line, c, side);
                if Vec2::new(base.x, base.z).distance(view) > view_r {
                    continue;
                }
                let off = self.clear.contains(&((a, bl), side));
                let (tx, tz) = line.tangent(line.along(c.x, c.y));
                let fwd = Vec2::new(tx, tz) * dir;
                draw_signal(b, base, fwd, off, night);
            }
        }
    }
}
