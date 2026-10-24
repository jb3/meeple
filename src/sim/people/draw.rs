use super::*;

impl TownLife {
    pub fn draw(&self, solid: &mut Boxes, fx: &mut Boxes, env: &Env, cam_yaw: f32, view: Vec2, view_r: f32) {
        let umbrellas = env.rain > 0.3;
        if let Some(g) = &self.gathering {
            if self.center().distance(view) < view_r + 12.0 {
                self.draw_bunting(solid, env.time, g.festival);
                self.draw_festival(solid, env.time, g.festival);
            }
        }
        if env.night < 0.5 && env.rain < 0.2 {
            self.draw_laundry(solid, env, view, view_r);
        }
        self.draw_farm_work(solid, env, view, view_r);
        if self.market_on(env) && self.center().distance(view) < view_r + 10.0 {
            self.draw_market(solid);
        }
        self.draw_game(solid);
        if let Some(l) = self.peek {
            self.draw_interior(solid, fx, env, l);
        }
        self.draw_happenings(solid, fx, env, cam_yaw, view, view_r);
        // petals thrown over the newlyweds as they come out of church
        if let Some(w) = &self.wedding {
            if w.vows.is_some_and(|v| env.hour < v + 0.5) {
                let petal = [[1.0, 0.8, 0.86], [1.0, 1.0, 0.95], [0.98, 0.65, 0.75]];
                for &i in [w.couple.0, w.couple.1].iter() {
                    let p = &self.people[i];
                    if !p.visible || p.pos.distance(view) > view_r {
                        continue;
                    }
                    for k in 0..14u32 {
                        let hk = hash_u32(p.seed ^ k.wrapping_mul(0x9e37));
                        let t = (env.time * 0.45 + unit(hk)).fract();
                        let a = unit(hk >> 8) * std::f32::consts::TAU + env.time * 0.7;
                        let rr = 0.25 + unit(hk >> 16) * 0.9;
                        let q = Vec3::new(p.pos.x + a.cos() * rr, p.y + 2.6 - t * 2.5, p.pos.y + a.sin() * rr);
                        fx.push(q, Vec3::new(0.045, 0.012, 0.045), a, petal[k as usize % 3], MAT_UNLIT);
                    }
                }
            }
        }
        for p in &self.people {
            if p.fade <= 0.0 || p.pos.distance(view) > view_r {
                continue;
            }
            // Bella lifting is drawn with his mirror
            if p.bella && p.state == State::Working && p.dest == Dest::Gym {
                continue;
            }
            draw_person(solid, fx, p, umbrellas, cam_yaw, env.time);
        }
        for d in &self.dogs {
            if d.pos.distance(view) > view_r {
                continue;
            }
            let mut r = Rig::new(solid, Vec3::new(d.pos.x, self.y, d.pos.y), d.yaw, 1.0);
            let sw = if d.moving { d.phase.sin() * 0.08 } else { 0.0 };
            r.part(Vec3::new(0.0, 0.38, 0.0), Vec3::new(0.13, 0.11, 0.3), d.col);
            r.part(Vec3::new(0.0, 0.52, 0.33), Vec3::new(0.11, 0.11, 0.12), d.col);
            r.part(Vec3::new(0.0, 0.48, 0.47), Vec3::new(0.06, 0.05, 0.05), [0.15, 0.1, 0.1]);
            r.part(Vec3::new(0.0, 0.5, -0.33), Vec3::new(0.03, 0.03, 0.1), d.col);
            for (x, z, s) in [(0.08, 0.2, 1.0), (-0.08, 0.2, -1.0), (0.08, -0.2, -1.0), (-0.08, -0.2, 1.0)] {
                r.part(Vec3::new(x, 0.14, z + sw * s), Vec3::new(0.04, 0.14, 0.04), d.col);
            }
        }
        for c in &self.cats {
            if c.pos.distance(view) > view_r {
                continue;
            }
            let mut r = Rig::new(solid, Vec3::new(c.pos.x, self.y, c.pos.y), c.yaw, 1.0);
            let col = c.col;
            let dark = [col[0] * 0.6, col[1] * 0.6, col[2] * 0.6];
            let white = [0.95, 0.94, 0.9];
            let awake_eyes = env.night > 0.5 && c.mode != CatMode::Sleep;
            let ear = |r: &mut Rig, y: f32, z: f32| {
                for x in [0.035, -0.035] {
                    r.part(Vec3::new(x, y, z), Vec3::new(0.015, 0.025, 0.015), col);
                }
            };
            match c.mode {
                CatMode::Sleep => {
                    // a loaf, nose tucked in, tail round the side
                    r.part(Vec3::new(0.0, 0.07, 0.0), Vec3::new(0.09, 0.07, 0.14), col);
                    r.part(Vec3::new(0.0, 0.07, 0.15), Vec3::new(0.055, 0.05, 0.05), col);
                    r.part(Vec3::new(0.09, 0.03, 0.02), Vec3::new(0.02, 0.025, 0.13), if c.pattern == 2 { dark } else { col });
                    ear(&mut r, 0.125, 0.14);
                    if c.pattern == 2 {
                        for z in [-0.06, 0.0, 0.06] {
                            r.part(Vec3::new(0.0, 0.142, z), Vec3::new(0.092, 0.004, 0.012), dark);
                        }
                    }
                }
                CatMode::Sit => {
                    let flick = (env.time * 1.3 + c.home.x).sin() * 0.03;
                    r.part(Vec3::new(0.0, 0.14, -0.03), Vec3::new(0.07, 0.12, 0.09), col);
                    r.part(Vec3::new(0.0, 0.32, 0.03), Vec3::new(0.06, 0.06, 0.06), col);
                    ear(&mut r, 0.4, 0.02);
                    for x in [0.035, -0.035] {
                        r.part(Vec3::new(x, 0.09, 0.07), Vec3::new(0.02, 0.09, 0.02), if c.pattern == 1 { white } else { col });
                    }
                    r.part(Vec3::new(0.05 + flick, 0.025, -0.12), Vec3::new(0.02, 0.02, 0.12), if c.pattern == 2 { dark } else { col });
                    if c.pattern == 1 {
                        r.part(Vec3::new(0.0, 0.15, 0.065), Vec3::new(0.04, 0.09, 0.01), white);
                        r.part(Vec3::new(0.0, 0.3, 0.092), Vec3::new(0.03, 0.03, 0.01), white);
                    }
                    if c.pattern == 2 {
                        for y in [0.1, 0.16, 0.22] {
                            r.part(Vec3::new(0.0, y, -0.122), Vec3::new(0.05, 0.012, 0.004), dark);
                        }
                    }
                    if awake_eyes {
                        for x in [0.025, -0.025] {
                            r.part_mat(Vec3::new(x, 0.335, 0.092), Vec3::splat(0.008), [0.6, 1.0, 0.45], MAT_GLOW);
                        }
                    }
                }
                CatMode::Walk => {
                    let sw = c.phase.sin() * 0.05;
                    let bob = (c.phase * 2.0).sin().abs() * 0.01;
                    r.part(Vec3::new(0.0, 0.2 + bob, 0.0), Vec3::new(0.07, 0.07, 0.17), col);
                    r.part(Vec3::new(0.0, 0.26 + bob, 0.2), Vec3::new(0.06, 0.06, 0.06), col);
                    ear(&mut r, 0.34 + bob, 0.19);
                    // tail held up in a hook
                    let wag = (c.phase * 0.5).sin() * 0.02;
                    r.part(Vec3::new(wag, 0.3 + bob, -0.21), Vec3::new(0.018, 0.1, 0.018), if c.pattern == 2 { dark } else { col });
                    for (x, z, s) in [(0.045, 0.12, 1.0), (-0.045, 0.12, -1.0), (0.045, -0.12, -1.0), (-0.045, -0.12, 1.0)] {
                        r.part(Vec3::new(x, 0.07, z + sw * s), Vec3::new(0.02, 0.07, 0.02), col);
                        if c.pattern == 1 {
                            r.part(Vec3::new(x, 0.015, z + sw * s), Vec3::new(0.021, 0.016, 0.021), white);
                        }
                    }
                    if c.pattern == 1 {
                        r.part(Vec3::new(0.0, 0.245 + bob, 0.256), Vec3::new(0.03, 0.03, 0.008), white);
                    }
                    if c.pattern == 2 {
                        for z in [-0.1, -0.03, 0.04, 0.11] {
                            r.part(Vec3::new(0.0, 0.272 + bob, z), Vec3::new(0.072, 0.004, 0.014), dark);
                        }
                    }
                    if awake_eyes {
                        for x in [0.025, -0.025] {
                            r.part_mat(Vec3::new(x, 0.285 + bob, 0.262), Vec3::splat(0.008), [0.6, 1.0, 0.45], MAT_GLOW);
                        }
                    }
                }
            }
        }
        for c in &self.chickens {
            if c.pos.distance(view) > view_r {
                continue;
            }
            let mut r = Rig::new(solid, Vec3::new(c.pos.x, self.y, c.pos.y), c.yaw, 1.0);
            let pk = c.peck * 0.12;
            r.part(Vec3::new(0.0, 0.22, 0.0), Vec3::new(0.1, 0.09, 0.14), [0.97, 0.95, 0.9]);
            r.part(Vec3::new(0.0, 0.36 - pk, 0.12 + pk * 0.5), Vec3::new(0.06, 0.06, 0.06), [0.97, 0.95, 0.9]);
            r.part(Vec3::new(0.0, 0.44 - pk, 0.12 + pk * 0.5), Vec3::new(0.02, 0.03, 0.04), [0.85, 0.15, 0.1]);
            r.part(Vec3::new(0.0, 0.34 - pk, 0.19 + pk * 0.5), Vec3::new(0.02, 0.015, 0.03), [0.95, 0.65, 0.1]);
            r.part(Vec3::new(0.0, 0.07, 0.0), Vec3::new(0.05, 0.07, 0.02), [0.95, 0.65, 0.1]);
        }
    }

    /// The farming year: a horse at the plough in spring, hay bales stacked
    /// in summer, a laden harvest cart at the gate in autumn.
    pub(super) fn draw_farm_work(&self, b: &mut Boxes, env: &Env, view: Vec2, view_r: f32) {
        let season = env.season;
        // ploughing: a horse just ahead of each farmer at work in spring
        if season < 0.9 {
            for p in self.people.iter().filter(|p| p.role == Role::Farmer && p.state == State::Working && p.visible) {
                if p.pos.distance(view) > view_r {
                    continue;
                }
                let fwd = Vec2::new(p.yaw.sin(), p.yaw.cos());
                let at = p.pos + fwd * 1.6;
                let mut r = Rig::new(b, Vec3::new(at.x, p.y, at.y), p.yaw, 1.0);
                r.alpha = p.fade;
                let col = [0.45, 0.32, 0.2];
                let sw = (env.time * 4.0).sin() * 0.08;
                r.part(Vec3::new(0.0, 0.95, 0.0), Vec3::new(0.22, 0.25, 0.55), col);
                r.part(Vec3::new(0.0, 1.3, 0.55), Vec3::new(0.12, 0.24, 0.13), col);
                r.part(Vec3::new(0.0, 1.48, 0.76), Vec3::new(0.11, 0.11, 0.2), col);
                for (x, z, s) in [(0.13, 0.4, 1.0), (-0.13, 0.4, -1.0), (0.13, -0.4, -1.0), (-0.13, -0.4, 1.0)] {
                    r.part(Vec3::new(x, 0.36, z + sw * s), Vec3::new(0.06, 0.36, 0.06), col);
                }
                // plough
                r.part(Vec3::new(0.0, 0.45, -0.95), Vec3::new(0.04, 0.04, 0.45), [0.4, 0.3, 0.2]);
                r.part(Vec3::new(0.0, 0.18, -1.35), Vec3::new(0.12, 0.12, 0.12), [0.45, 0.45, 0.5]);
            }
        }
        for lot in self.lots.iter().filter(|l| l.kind == LotKind::Farm) {
            let c = Vec2::from(lot.center());
            if c.distance(view) > view_r + 6.0 {
                continue;
            }
            let y = self.y;
            if (1.4..2.1).contains(&season) {
                // hay bales stacked in a corner
                let base = Vec2::new(lot.x1 as f32 - 0.6, lot.z1 as f32 - 0.6);
                for (k, (dx, dz, dy)) in [(0.0, 0.0, 0.0), (-1.0, 0.0, 0.0), (0.0, -1.0, 0.0), (-0.5, -0.5, 0.9)].iter().enumerate() {
                    if k == 3 && hash_u32(lot.seed ^ 0xba1e) % 2 == 0 {
                        continue;
                    }
                    b.push(Vec3::new(base.x + dx, y + 0.45 + dy, base.y + dz), Vec3::new(0.45, 0.45, 0.45), 0.2, [0.86, 0.74, 0.36], 0.0);
                }
            } else if (2.0..2.6).contains(&season) {
                // harvest cart by the gate, piled with the crop
                let (fx, fz) = lot.front;
                let (ox, oz) = lot.outward();
                let along = if lot.face % 2 == 0 { Vec2::new(1.0, 0.0) } else { Vec2::new(0.0, 1.0) };
                let at = Vec2::new(fx + ox as f32 * 1.5, fz + oz as f32 * 1.5) + along * 2.2;
                let mut r = Rig::new(b, Vec3::new(at.x, y, at.y), yaw_of(along), 1.0);
                let wood = [0.55, 0.4, 0.25];
                r.part(Vec3::new(0.0, 0.75, 0.0), Vec3::new(0.6, 0.1, 0.9), wood);
                for x in [-0.6, 0.6] {
                    r.part(Vec3::new(x, 0.95, 0.0), Vec3::new(0.03, 0.2, 0.9), wood);
                    r.part(Vec3::new(x * 1.12, 0.35, 0.0), Vec3::new(0.05, 0.34, 0.34), [0.35, 0.25, 0.18]);
                }
                let pumpkins = hash_u32(lot.seed ^ 0x9f) % 4 == 0;
                for k in 0..5 {
                    let (px, pz) = ((k % 2) as f32 * 0.5 - 0.25, (k / 2) as f32 * 0.5 - 0.5);
                    let (col, half) = if pumpkins { ([0.9, 0.5, 0.15], 0.22) } else { ([0.88, 0.75, 0.32], 0.25) };
                    r.part(Vec3::new(px, 1.05, pz), Vec3::splat(half), col);
                }
            }
        }
    }

    /// Where tonight's bonfire burns, if it's a winter gathering.
    pub fn bonfire(&self) -> Option<Vec3> {
        let g = self.gathering.as_ref()?;
        (g.festival == Festival::Bonfire).then(|| {
            let c = self.center() + Vec2::new(-4.6, 0.0);
            Vec3::new(c.x, self.y, c.y)
        })
    }

    /// The season's centrepiece: a maypole with ribbons to the dancers,
    /// harvest bales and pumpkins, or a bonfire.
    pub(super) fn draw_festival(&self, b: &mut Boxes, time: f32, festival: Festival) {
        let c = self.center();
        match festival {
            Festival::Maypole => {
                let top = Vec3::new(c.x, self.y + 7.0, c.y);
                b.push(Vec3::new(c.x, self.y + 4.5, c.y), Vec3::new(0.09, 2.5, 0.09), 0.0, [0.92, 0.9, 0.85], 0.0);
                b.push(top + Vec3::Y * 0.15, Vec3::new(0.25, 0.12, 0.25), 0.3, [0.4, 0.65, 0.3], 0.0);
                let ribbons = [[0.95, 0.3, 0.4], [0.3, 0.55, 0.95], [0.98, 0.85, 0.3], [0.45, 0.8, 0.45], [0.85, 0.5, 0.95]];
                for (k, p) in self.people.iter().filter(|p| p.state == State::Dancing).enumerate() {
                    let hand = Vec3::new(p.pos.x, p.y + 1.3, p.pos.y);
                    for i in 1..10 {
                        let t = i as f32 / 10.0;
                        let q = top.lerp(hand, t) - Vec3::Y * (std::f32::consts::PI * t).sin() * 0.4;
                        b.push(q, Vec3::splat(0.05), 0.0, ribbons[k % ribbons.len()], 0.0);
                    }
                }
            }
            Festival::Harvest => {
                for k in 0..6 {
                    let a = k as f32 * std::f32::consts::TAU / 6.0 + 0.5;
                    let p = c + Vec2::new(a.cos(), a.sin()) * 8.6;
                    if k % 2 == 0 {
                        b.push(Vec3::new(p.x, self.y + 0.45, p.y), Vec3::splat(0.45), a, [0.86, 0.74, 0.36], 0.0);
                        b.push(Vec3::new(p.x, self.y + 1.15, p.y), Vec3::new(0.22, 0.25, 0.22), a, [0.92, 0.52, 0.14], 0.0);
                    } else {
                        for (dx, dz) in [(0.0, 0.0), (0.45, 0.2), (-0.3, 0.35)] {
                            b.push(Vec3::new(p.x + dx, self.y + 0.22, p.y + dz), Vec3::splat(0.22), dx, [0.92, 0.52, 0.14], 0.0);
                        }
                    }
                }
            }
            Festival::Bonfire => {
                if let Some(f) = self.bonfire() {
                    for (k, a) in [0.0f32, 1.1, 2.2].iter().enumerate() {
                        b.push(f + Vec3::Y * (0.15 + k as f32 * 0.05), Vec3::new(0.1, 0.1, 0.7), *a, [0.4, 0.27, 0.18], 0.0);
                    }
                    for i in 0..7 {
                        let fl = (time * 9.0 + i as f32 * 1.7).sin() * 0.5 + 0.5;
                        let a = i as f32 * 0.9;
                        let off = Vec3::new(a.cos() * 0.2, 0.4 + i as f32 * 0.12 + fl * 0.2, a.sin() * 0.2);
                        let col = if i % 2 == 0 { [1.0, 0.55, 0.15] } else { [1.0, 0.82, 0.35] };
                        b.push(f + off, Vec3::splat(0.22 - i as f32 * 0.025 + fl * 0.05), a, col, MAT_GLOW);
                    }
                }
            }
            Festival::Midsummer | Festival::Wedding => {}
        }
    }

    /// Market stalls: a table of produce under a striped awning.
    pub(super) fn draw_market(&self, b: &mut Boxes) {
        let produce = [[0.85, 0.2, 0.2], [0.35, 0.65, 0.25], [0.82, 0.62, 0.35], [0.95, 0.55, 0.15], [0.75, 0.78, 0.82]];
        let stripes = [[[0.86, 0.22, 0.24], [0.96, 0.95, 0.9]], [[0.25, 0.48, 0.86], [0.96, 0.95, 0.9]], [[0.3, 0.62, 0.35], [0.96, 0.92, 0.6]]];
        for (k, (c, out)) in self.stall_spots().into_iter().enumerate() {
            let mut r = Rig::new(b, Vec3::new(c.x, self.y, c.y), yaw_of(out), 1.0);
            let wood = [0.55, 0.4, 0.26];
            r.part(Vec3::new(0.0, 0.8, 0.0), Vec3::new(0.85, 0.05, 0.42), wood);
            for (x, z) in [(-0.8, -0.38), (0.8, -0.38), (-0.8, 0.38), (0.8, 0.38)] {
                r.part(Vec3::new(x, 0.95, z), Vec3::new(0.04, 0.95, 0.04), wood);
            }
            for i in 0..4 {
                let col = stripes[k % 3][i % 2];
                r.part(Vec3::new(-0.66 + i as f32 * 0.44, 1.92, 0.05), Vec3::new(0.22, 0.04, 0.6), col);
            }
            // the table fills with a good harvest and thins out in a bad one
            let lines = if self.plenty < 0.55 { 1 } else if self.plenty < 1.15 { 3 } else { 3 };
            for i in 0..lines {
                let col = produce[(k * 2 + i) % produce.len()];
                r.part(Vec3::new(-0.5 + i as f32 * 0.5, 0.95, 0.05), Vec3::new(0.18, 0.1, 0.16), col);
                if self.plenty > 1.15 {
                    let col2 = produce[(k * 2 + i + 2) % produce.len()];
                    r.part(Vec3::new(-0.5 + i as f32 * 0.5, 1.15, 0.05), Vec3::new(0.15, 0.1, 0.13), col2);
                    r.part(Vec3::new(-0.5 + i as f32 * 0.5, 0.95, 0.32), Vec3::new(0.16, 0.1, 0.06), col2);
                }
            }
        }
    }

    /// The ball in flight, or the skipping rope turning.
    pub(super) fn draw_game(&self, b: &mut Boxes) {
        let Some(g) = &self.game else { return };
        match g.kind {
            GameKind::Ball => {
                let (a, z, t) = (self.people[g.ball.0].pos, self.people[g.ball.1].pos, g.ball.2);
                let p = a.lerp(z, t);
                let y = self.y + 1.0 + (std::f32::consts::PI * t).sin() * 1.6;
                b.push(Vec3::new(p.x, y, p.y), Vec3::splat(0.11), t * 6.0, [0.9, 0.25, 0.2], 0.0);
            }
            GameKind::Skipping => {
                let holders: Vec<Vec2> = self
                    .people
                    .iter()
                    .filter(|p| p.visible && p.role == Role::Child && p.goal == Goal::Play && p.at == Place::Plaza)
                    .take(2)
                    .map(|p| p.pos)
                    .collect();
                if holders.len() < 2 {
                    return;
                }
                let (a, z) = (holders[0], holders[1]);
                let side = (z - a).perp().normalize_or(Vec2::X);
                for i in 1..12 {
                    let t = i as f32 / 12.0;
                    let p = a.lerp(z, t);
                    let swing = (std::f32::consts::PI * t).sin() * 0.85;
                    let off = side * g.rope.sin() * swing;
                    let y = self.y + 0.75 + g.rope.cos() * swing;
                    b.push(Vec3::new(p.x + off.x, y.max(self.y + 0.03), p.y + off.y), Vec3::splat(0.035), 0.0, [0.95, 0.85, 0.3], 0.0);
                }
            }
            GameKind::Tag => {}
        }
    }

    /// Washing hung out in some cottage gardens, flapping in the wind.
    pub(super) fn draw_laundry(&self, b: &mut Boxes, env: &Env, view: Vec2, view_r: f32) {
        let cloth = [[0.95, 0.95, 0.92], [0.7, 0.8, 0.95], [0.95, 0.75, 0.75], [0.85, 0.9, 0.7]];
        let wl = env.wind.length();
        for lot in self.lots.iter().filter(|l| l.kind == LotKind::House && hash_u32(l.seed ^ 0x1a0d) % 3 == 0) {
            // along the yard on the side away from the door
            let (x0, z0, x1, z1) = (lot.x0 as f32, lot.z0 as f32, lot.x1 as f32 + 1.0, lot.z1 as f32 + 1.0);
            let (a, z) = match lot.face {
                0 => (Vec2::new(x0 + 1.5, z1 - 0.5), Vec2::new(x1 - 1.5, z1 - 0.5)),
                2 => (Vec2::new(x0 + 1.5, z0 + 0.5), Vec2::new(x1 - 1.5, z0 + 0.5)),
                1 => (Vec2::new(x0 + 0.5, z0 + 1.5), Vec2::new(x0 + 0.5, z1 - 1.5)),
                _ => (Vec2::new(x1 - 0.5, z0 + 1.5), Vec2::new(x1 - 0.5, z1 - 1.5)),
            };
            if a.distance(view) > view_r {
                continue;
            }
            let y = self.y;
            for p in [a, z] {
                b.push(Vec3::new(p.x, y + 0.8, p.y), Vec3::new(0.05, 0.8, 0.05), 0.0, [0.45, 0.33, 0.22], 0.0);
            }
            let along = (z - a).normalize_or_zero();
            let yaw = yaw_of(along);
            b.push(Vec3::new((a.x + z.x) * 0.5, y + 1.55, (a.y + z.y) * 0.5), Vec3::new(0.01, 0.01, a.distance(z) * 0.5), yaw, [0.85, 0.85, 0.8], 0.0);
            let side = Vec2::new(along.y, -along.x);
            let lean = env.wind.dot(side).clamp(-1.0, 1.0);
            for k in 0..4 {
                let t = (k as f32 + 0.75) / 4.4;
                let p = a.lerp(z, t);
                let flap = (env.time * (3.0 + wl * 4.0) + k as f32 * 1.7).sin() * (0.05 + wl * 0.2);
                let off = side * (lean * 0.25 + flap);
                b.push(
                    Vec3::new(p.x + off.x, y + 1.3, p.y + off.y),
                    Vec3::new(0.02, 0.24, 0.2),
                    yaw + std::f32::consts::FRAC_PI_2 + lean * 0.4 + flap,
                    cloth[(hash_u32(lot.seed ^ k) as usize) % cloth.len()],
                    0.0,
                );
            }
        }
    }

    /// Strings of flags between the plaza's lamp posts, sagging a little and
    /// stirring in the breeze.
    pub(super) fn draw_bunting(&self, b: &mut Boxes, time: f32, festival: Festival) {
        let c = self.center();
        let top = self.y + 3.55;
        let corner = |sx: f32, sz: f32| Vec2::new(c.x + 4.0 * sx, c.y + 4.0 * sz);
        let posts = [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)];
        let colours: &[[f32; 3]] = match festival {
            Festival::Harvest => &[[0.86, 0.42, 0.12], [0.95, 0.75, 0.2], [0.72, 0.22, 0.12], [0.6, 0.5, 0.2]],
            Festival::Maypole => &[[0.98, 0.75, 0.85], [0.75, 0.9, 0.98], [0.98, 0.95, 0.7], [0.8, 0.95, 0.75]],
            Festival::Wedding => &[[0.98, 0.98, 0.96], [0.98, 0.78, 0.85]],
            _ => &[[0.86, 0.22, 0.24], [0.96, 0.78, 0.2], [0.25, 0.48, 0.86], [0.3, 0.7, 0.36], [0.95, 0.95, 0.92]],
        };
        let lanterns = festival == Festival::Bonfire;
        let mut strings: Vec<(Vec2, Vec2)> = (0..4).map(|k| (posts[k], posts[(k + 1) % 4])).collect();
        strings.push((posts[0], posts[2]));
        strings.push((posts[1], posts[3]));
        for (n, (a, z)) in strings.into_iter().enumerate() {
            let flags = ((a.distance(z) / 0.7) as usize).max(4);
            let yaw = yaw_of(z - a) + std::f32::consts::FRAC_PI_2;
            for k in 1..flags {
                let t = k as f32 / flags as f32;
                let p = a.lerp(z, t);
                let sag = (std::f32::consts::PI * t).sin() * 0.7;
                let flutter = (time * 3.0 + k as f32 * 0.9 + n as f32).sin() * 0.04;
                if lanterns {
                    // warm lanterns, every other spot
                    if k % 2 == 0 {
                        b.push(Vec3::new(p.x, top - sag - 0.2, p.y), Vec3::new(0.09, 0.12, 0.09), yaw, [1.0, 0.7, 0.35], MAT_GLOW);
                    }
                    continue;
                }
                b.push(
                    Vec3::new(p.x, top - sag - 0.14 + flutter, p.y),
                    Vec3::new(0.02, 0.13, 0.12),
                    yaw,
                    colours[(k + n) % colours.len()],
                    0.0,
                );
            }
        }
    }
}
