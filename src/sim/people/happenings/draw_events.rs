use super::*;

impl TownLife {
    pub fn draw_happenings(&self, solid: &mut Boxes, fx: &mut Boxes, env: &Env, cam_yaw: f32, view: Vec2, view_r: f32) {
        let c = self.center();
        if c.distance(view) > view_r + self.town.radius as f32 + 30.0 {
            return;
        }
        self.happen.party.borrow_mut().clear();
        if BELLA.load(Ordering::Relaxed) {
            self.draw_bella(solid, env, cam_yaw);
        }
        for a in &self.happen.active {
            let t = env.time - a.t0;
            match a.kind {
                Kind::Fire => self.draw_fire(solid, fx, env, a, t),
                Kind::Christening => self.draw_christening(solid, fx, env, a, t),
                Kind::Chase => self.draw_chase(solid, env, a, t),
                Kind::Doctor => self.draw_doctor(solid, env, a, t),
                Kind::Closing => self.draw_closing(solid, env, a, t),
            }
        }
        for &(lot, _) in &self.happen.soot {
            let burning = self.happen.active.iter().any(|a| a.kind == Kind::Fire && a.lot == lot && env.time - a.t0 < 55.0);
            if !burning {
                self.draw_soot(solid, lot);
            }
        }
        if let Some(g) = self.happen.green.flatten() {
            if g.distance(view) < view_r + 20.0 {
                if self.fair_on(env) {
                    self.draw_fair(solid, fx, env, g);
                } else if self.cricket_on(env) {
                    self.draw_cricket(solid, env, g);
                } else if (2.95..3.95).contains(&env.season) && (10.0..20.0).contains(&env.hour) && env.rain < 0.4 {
                    self.draw_rink(solid, fx, env, g);
                }
            }
        }
        if let Some(g) = self.happen.green.flatten() {
            let winter = (2.95..3.95).contains(&env.season) && env.rain < 0.45 && (10.0..24.0).contains(&env.hour);
            if winter && g.distance(view) < view_r + 20.0 && !self.fair_on(env) {
                self.draw_snow_play(solid, fx, env, g);
            }
        }
        self.draw_chores(solid, env, view, view_r);
        self.draw_evening(solid, fx, env, cam_yaw);
    }

    pub(super) fn draw_soot(&self, b: &mut Boxes, lot: usize) {
        // a scorched stretch of ridge where the fire took hold
        let (c, along, half, apex) = self.roof_of(lot);
        let across = Vec2::new(along.y, along.x);
        for (f, lat, w) in [(-0.4f32, 0.0f32, 1.4f32), (0.3, 0.0, 1.0), (0.05, 1.0, 0.8), (-0.1, -1.0, 0.8)] {
            let p = c + along * f * half + across * lat;
            b.push(Vec3::new(p.x, apex - lat.abs() + 0.03, p.y), Vec3::new(w * 0.5, 0.04, w * 0.5), 0.0, [0.16, 0.14, 0.13], MAT_LIT);
        }
    }

    /// Where a house's ridge runs, matching how the roof is built: (ridge
    /// centre, unit vector along the ridge, half its length, height of the
    /// topmost tiles).
    pub(super) fn roof_of(&self, lot: usize) -> (Vec2, Vec2, f32, f32) {
        let l = &self.lots[lot];
        let (x0, z0, x1, z1) = l.footprint();
        let ridge_x = l.face % 2 == 0;
        // the roof overhangs the walls by a block all round
        let (ax0, ax1, pz0, pz1) = if ridge_x { (x0 - 1, x1 + 1, z0 - 1, z1 + 1) } else { (z0 - 1, z1 + 1, x0 - 1, x1 + 1) };
        let layers = (pz1 - pz0 + 2) / 2;
        let mid = (pz0 + pz1 + 1) as f32 * 0.5;
        let along_mid = (ax0 + ax1 + 1) as f32 * 0.5;
        let (c, along) = if ridge_x { (Vec2::new(along_mid, mid), Vec2::X) } else { (Vec2::new(mid, along_mid), Vec2::Y) };
        let apex = self.y + l.floors as f32 * 3.0 + layers as f32;
        (c, along, (ax1 - ax0 + 1) as f32 * 0.5 - 0.6, apex)
    }

    pub(super) fn draw_fire(&self, b: &mut Boxes, fx: &mut Boxes, env: &Env, a: &Active, t: f32) {
        let l = &self.lots[a.lot];
        let (x0, z0, x1, z1) = l.footprint();
        let (ridge, along, half, apex) = self.roof_of(a.lot);
        let k = fire_level(t);
        let time = env.time;
        // a flame is a column of small glowing cubes that narrow and shift
        // colour as they climb, each wobbling in its own time
        let flame = |fx: &mut Boxes, base: Vec3, width: f32, height: f32, ph: f32| {
            let n = ((height / 0.2) as usize).max(2);
            for j in 0..n {
                let f = j as f32 / (n - 1) as f32;
                let wob = (time * 7.0 + ph + f * 5.0).sin();
                let sway = (time * 4.0 + ph * 1.3 + f * 3.0).sin() * 0.16 * f;
                // the tip breaks up into flickers
                if f > 0.7 && (time * 11.0 + ph + f * 9.0).sin() < -0.2 {
                    continue;
                }
                let sz = width * (0.24 - f * 0.15) + wob * 0.02;
                let col = if f < 0.3 { [0.95, 0.22, 0.06] } else if f < 0.62 { [1.0, 0.55, 0.12] } else { [1.0, 0.86, 0.38] };
                let y = base.y + f * height * (0.9 + 0.1 * wob);
                fx.push_alpha(Vec3::new(base.x + sway, y, base.z + sway * 0.6), Vec3::splat(sz.max(0.05)), ph + f * 1.7, col, MAT_GLOW, 1.0 - f * f * 0.45);
            }
        };
        // tongues along the ridge, and lower down both slopes (the roof falls
        // a block for every block out from the ridge)
        let across = Vec2::new(along.y, along.x);
        let n = (half * 2.0 / 1.2).ceil() as usize + 1;
        for (row, lat) in [(0usize, 0.0f32), (1, 1.5), (2, -1.5)] {
            for i in 0..n {
                if (i as f32 + 0.5) / n as f32 > k.max(0.15) || (row > 0 && (i + row) % 2 == 1) {
                    continue;
                }
                let f = if n > 1 { i as f32 / (n - 1) as f32 * 2.0 - 1.0 } else { 0.0 };
                let p = ridge + along * f * half + across * lat;
                let ph = i as f32 * 1.9 + row as f32 * 2.3 + (a.seed % 1000) as f32 * 0.01;
                let tall = (1.7 + (i % 3) as f32 * 0.7) * k.max(0.3) * if row == 0 { 1.0 } else { 0.6 };
                flame(fx, Vec3::new(p.x, apex - lat.abs() + 0.05, p.y), 0.8 + 0.2 * k, tall, ph);
            }
        }
        // flames out of the windows, the wall scorched above each
        let sides: [(Vec2, Vec2, f32, f32); 4] = [
            (Vec2::new(x0 as f32 - 0.35, 0.0), Vec2::Y, z0 as f32, (z1 + 1) as f32),
            (Vec2::new((x1 + 1) as f32 + 0.35, 0.0), Vec2::Y, z0 as f32, (z1 + 1) as f32),
            (Vec2::new(0.0, z0 as f32 - 0.35), Vec2::X, x0 as f32, (x1 + 1) as f32),
            (Vec2::new(0.0, (z1 + 1) as f32 + 0.35), Vec2::X, x0 as f32, (x1 + 1) as f32),
        ];
        for (si, (off, dir, lo, hi)) in sides.iter().enumerate() {
            let mut u = lo + 1.5;
            let mut wi = 0u32;
            while u < hi - 0.5 {
                for floor in 0..l.floors {
                    let hh = hash_u32(a.seed ^ (si as u32 * 31 + wi * 7 + floor as u32));
                    if unit(hh) > k * 0.9 {
                        continue;
                    }
                    let pos = Vec3::new(off.x + dir.x * u, self.y + floor as f32 * 3.0 + 1.2, off.y + dir.y * u);
                    flame(fx, pos, 0.55, 1.1, wi as f32 * 1.3 + si as f32 + floor as f32);
                }
                u += 2.0;
                wi += 1;
            }
        }
        // embers drifting up off the roof
        for j in 0..14u32 {
            let hj = hash_u32(a.seed ^ j.wrapping_mul(0x9e37));
            let f = (time * 0.35 + unit(hj)).fract();
            let p = ridge + Vec2::new(unit(hj >> 8) - 0.5, unit(hj >> 16) - 0.5) * 3.0 + env.wind * f * 6.0;
            fx.push(Vec3::new(p.x, apex + 0.5 + f * 5.0, p.y), Vec3::splat(0.06), 0.0, [1.0, 0.6, 0.2], MAT_GLOW);
        }
        // Everything arrives along the street route to the house and stops well
        // short of it: the pump and its horse, then a chain of villagers
        // passing pails from the pump towards the roof, then (beside them) a
        // fireman at the hose.
        let total = path_len(&a.path);
        let gap = 1.15;
        let door_stop = 4.5;
        let n_fit = (((total - door_stop - 5.0) / gap) as usize + 1).min(10);
        let n = n_fit - n_fit % 2;
        let last_s = total - door_stop;
        let first_s = last_s - (n.max(1) - 1) as f32 * gap;
        let horse_s = (first_s - 0.9).max(0.0);
        let at_s = |sv: f32| walk(&a.path, sv);
        let d = (t - 2.0).max(0.0) * 3.2;
        let pump_s = d.min(horse_s);
        let moving = d < horse_s - 0.01;
        if t > 2.0 {
            let (p, pdir, _) = at_s(pump_s);
            let y = self.ground(env, p);
            let yaw = yaw_of(pdir);
            horse(b, Vec3::new(p.x, y, p.y), yaw, time * 9.0, moving, [0.4, 0.28, 0.2]);
            let back = p - pdir * 2.0;
            let mut r = Rig::new(b, Vec3::new(back.x, y, back.y), yaw, 1.0);
            r.part(Vec3::new(0.0, 0.85, 0.0), Vec3::new(0.55, 0.3, 0.95), [0.75, 0.12, 0.1]);
            r.part(Vec3::new(0.0, 1.35, -0.3), Vec3::new(0.3, 0.3, 0.45), [0.78, 0.62, 0.2]);
            r.part(Vec3::new(0.0, 1.0, 0.6), Vec3::new(0.3, 0.06, 0.2), [0.3, 0.2, 0.15]);
            for (xx, zz) in [(-0.6, 0.5), (0.6, 0.5), (-0.6, -0.5), (0.6, -0.5)] {
                r.part(Vec3::new(xx, 0.42, zz), Vec3::new(0.06, 0.42, 0.42), [0.2, 0.15, 0.12]);
            }
        }
        let ready = !moving && t > 8.0 && n >= 2;
        if ready {
            let c = (time * 0.4).fract();
            let ease_in = |lo: f32, hi: f32| smoothstep(lo, hi, c);
            let people: Vec<(Vec2, Vec2, f32)> = (0..n)
                .map(|i| {
                    let (q, dir, _) = at_s(first_s + i as f32 * gap);
                    (q, dir, self.ground(env, q))
                })
                .collect();
            // even ones give while odd ones take, then the other way about
            let swing = (1.0 - ((c - 0.25) * TAU).cos()) * 0.5 * PI;
            let hand = |i: usize, yaw: f32| {
                let (q, _, y) = people[i];
                Vec3::new(q.x + yaw.sin() * 0.5, y + 0.95, q.y + yaw.cos() * 0.5)
            };
            let yaws: Vec<f32> = (0..n).map(|i| yaw_of(people[i].1) + if i % 2 == 0 { swing } else { PI - swing }).collect();
            for i in 0..n {
                let (q, _, y) = people[i];
                let g = garb(a.seed ^ i as u32, None, None).hat(Hat::Cap, [0.75, 0.15, 0.12]);
                figure(b, Vec3::new(q.x, y, q.y), yaws[i], i as f32, &g, Pose::Pass, Prop::None, time, 1.0);
            }
            // one pail for every two people, always in somebody's hands
            for k in 0..n / 2 {
                let g0 = 2 * k;
                let pos = if c < 0.1 {
                    hand(g0, yaws[g0])
                } else if c < 0.4 {
                    let u = ease_in(0.1, 0.4);
                    hand(g0, yaws[g0]).lerp(hand(g0 + 1, yaws[g0 + 1]), u) + Vec3::Y * (u * PI).sin() * 0.12
                } else if c < 0.6 {
                    hand(g0 + 1, yaws[g0 + 1])
                } else if g0 + 2 < n {
                    if c < 0.9 {
                        let u = ease_in(0.6, 0.9);
                        hand(g0 + 1, yaws[g0 + 1]).lerp(hand(g0 + 2, yaws[g0 + 2]), u) + Vec3::Y * (u * PI).sin() * 0.12
                    } else {
                        hand(g0 + 2, yaws[g0 + 2])
                    }
                } else {
                    // the last in the line empties it over the fire
                    let start = hand(g0 + 1, yaws[g0 + 1]);
                    let u = ((c - 0.6) / 0.25).clamp(0.0, 1.0);
                    if c < 0.75 {
                        let target = Vec3::new(ridge.x, self.y + l.floors as f32 * 3.0 - 1.0, ridge.y);
                        for j in 0..10 {
                            let f = (u * 1.4 - j as f32 * 0.05).clamp(0.0, 1.0);
                            fx.push_alpha(start.lerp(target, f) + Vec3::Y * (f * PI).sin() * 0.8, Vec3::splat(0.1), 0.0, [0.7, 0.85, 1.0], MAT_UNLIT, 0.85);
                        }
                        start
                    } else {
                        continue;
                    }
                };
                pail(b, pos, true);
            }
            // a fireman at the hose, off to the side, with the jet over the eaves
            let (hq, hdir, _) = at_s((last_s - 1.2).max(0.0));
            let side = Vec2::new(hdir.y, -hdir.x);
            let post = hq + side * 1.4;
            let gy = self.ground(env, post);
            let to = (ridge - post).normalize_or_zero();
            let g = garb(a.seed ^ 0x77, Some([0.15, 0.2, 0.4]), None).hat(Hat::Cap, [0.8, 0.65, 0.2]);
            figure(b, Vec3::new(post.x, gy, post.y), yaw_of(to), 0.0, &g, Pose::Reach, Prop::None, time, 1.0);
            let (pp, _, _) = at_s(pump_s);
            let from_hose = pp - Vec2::new(0.0, 0.0);
            let hn = (from_hose.distance(post) / 0.4) as usize;
            for sidx in 0..hn {
                let f = sidx as f32 / hn.max(1) as f32;
                let q = from_hose.lerp(post, f) + side * 1.4 * f;
                b.push(Vec3::new(q.x, self.ground(env, q) + 0.07, q.y), Vec3::new(0.1, 0.07, 0.1), 0.0, [0.28, 0.2, 0.14], MAT_LIT);
            }
            let nozzle = Vec3::new(post.x + to.x * 0.4, gy + 1.45, post.y + to.y * 0.4);
            let target = Vec3::new(ridge.x, apex - 1.2, ridge.y);
            for j in 0..36 {
                let u = ((time * 1.6 + j as f32 / 36.0) % 1.0).clamp(0.0, 1.0);
                let q = nozzle.lerp(target, u) + Vec3::Y * (u * PI).sin() * 2.4;
                let sz = 0.14 + 0.05 * (j % 3) as f32 - u * 0.05;
                fx.push_alpha(q, Vec3::splat(sz), 0.0, [0.62, 0.8, 0.98], MAT_UNLIT, 0.85 - u * 0.25);
            }
            for j in 0..8u32 {
                let hj = hash_u32(j * 91 + a.seed);
                let f = (time * 1.4 + unit(hj)).fract();
                let o = Vec3::new(unit(hj >> 8) - 0.5, f * 1.4, unit(hj >> 14) - 0.5) * Vec3::new(1.6, 1.0, 1.6);
                fx.push_alpha(target + o, Vec3::splat(0.1), 0.0, [0.85, 0.92, 1.0], MAT_UNLIT, 0.6 * (1.0 - f));
            }
        }
    }

    pub(super) fn draw_christening(&self, b: &mut Boxes, fx: &mut Boxes, env: &Env, a: &Active, t: f32) {
        // there to the church, a pause at the door in a shower of petals, home
        let there = path_len(&a.path);
        let sp = 1.0;
        let go = there / sp;
        let stay = 14.0;
        let d = if t < go { t * sp } else if t < go + stay { there } else { (there - (t - go - stay) * sp).max(0.0) };
        let moving = t < go || t > go + stay;
        let party: [(f32, Prop, Garb, bool); 6] = [
            (0.0, Prop::Baby, garb(a.seed, Some([0.98, 0.95, 0.9]), None), false),
            (-0.9, Prop::None, garb(a.seed ^ 1, None, None), false),
            (-1.9, Prop::Posy, garb(a.seed ^ 2, Some([0.9, 0.7, 0.8]), None).hat(Hat::Bun, BLACK), false),
            (-2.6, Prop::None, garb(a.seed ^ 3, None, None).hat(Hat::Cap, BLACK), false),
            (-3.2, Prop::None, garb(a.seed ^ 4, None, None).kid(), true),
            (-3.8, Prop::None, garb(a.seed ^ 5, None, None).kid(), true),
        ];
        for (k, (back, prop, g, kid)) in party.iter().enumerate() {
            let dd = if t < go + stay { d + *back } else { d - *back };
            let dd = dd.clamp(0.0, there);
            let (p, dir, _) = walk(&a.path, dd);
            let dir = if t >= go + stay { -dir } else { dir };
            let p = p + Vec2::new(dir.y, -dir.x) * if k % 2 == 0 { 0.25 } else { -0.25 };
            let y = self.ground(env, p);
            let pose = if moving { Pose::Walk } else { Pose::Stand };
            let _ = kid;
            figure(b, Vec3::new(p.x, y, p.y), yaw_of(dir), t * 4.3 + k as f32, g, pose, *prop, env.time, 1.0);
            self.happen.party.borrow_mut().push((Kind::Christening, Vec3::new(p.x, y, p.y)));
        }
        if !moving {
            let (p, _, _) = walk(&a.path, there);
            let y = self.ground(env, p);
            for k in 0..16u32 {
                let hk = hash_u32(a.seed ^ k.wrapping_mul(0x9e37));
                let f = (env.time * 0.5 + unit(hk)).fract();
                let ang = unit(hk >> 8) * TAU + env.time;
                let rr = 0.3 + unit(hk >> 16) * 1.2;
                let c = [[1.0, 0.8, 0.86], [1.0, 1.0, 0.95], [0.98, 0.65, 0.75]][k as usize % 3];
                fx.push(Vec3::new(p.x + ang.cos() * rr, y + 2.8 - f * 2.7, p.y + ang.sin() * rr), Vec3::new(0.045, 0.012, 0.045), ang, c, MAT_UNLIT);
            }
        }
    }

    pub(super) fn draw_chase(&self, b: &mut Boxes, env: &Env, a: &Active, t: f32) {
        // a dog bolts round the fountain with three children after it
        let c = self.center();
        let run = |lag: f32, r: f32| {
            let ang = (t - lag) * 0.75 * (1.0 + 0.1 * (t * 0.7).sin());
            (c + Vec2::new(ang.cos(), ang.sin()) * r, Vec2::new(-ang.sin(), ang.cos()))
        };
        let fade = ease(t / 2.0).min(ease((a.dur - t) / 2.0));
        let (dp, dd) = run(0.0, 6.4);
        let y = self.ground(env, dp);
        {
            let mut r = Rig::new(b, Vec3::new(dp.x, y, dp.y), yaw_of(dd), 1.0);
            r.alpha = fade;
            let sw = (env.time * 18.0).sin() * 0.1;
            let col = [0.55, 0.38, 0.22];
            r.part(Vec3::new(0.0, 0.38, 0.0), Vec3::new(0.13, 0.11, 0.3), col);
            r.part(Vec3::new(0.0, 0.52, 0.33), Vec3::new(0.11, 0.11, 0.12), col);
            r.part(Vec3::new(0.0, 0.5, -0.33), Vec3::new(0.03, 0.03, 0.1), col);
            for (x, z, s) in [(0.08, 0.2, 1.0), (-0.08, 0.2, -1.0), (0.08, -0.2, -1.0), (-0.08, -0.2, 1.0)] {
                r.part(Vec3::new(x, 0.14, z + sw * s * 2.0), Vec3::new(0.04, 0.14, 0.04), col);
            }
        }
        for k in 0..3 {
            let (p, d) = run(1.6 + k as f32 * 0.9, 6.4 + (k as f32 - 1.0) * 0.5);
            let g = garb(a.seed ^ k as u32, None, None).kid();
            let yy = self.ground(env, p);
            figure(b, Vec3::new(p.x, yy, p.y), yaw_of(d), t * 7.0 + k as f32, &g, Pose::Walk, Prop::None, env.time, fade);
        }
    }

    pub(super) fn draw_doctor(&self, b: &mut Boxes, env: &Env, a: &Active, t: f32) {
        let total = path_len(&a.path);
        let (p, dir, moving) = walk(&a.path, t * 1.25);
        let g = garb(a.seed, Some([0.25, 0.25, 0.3]), Some([0.2, 0.2, 0.24])).hat(Hat::Chef, BLACK);
        let alpha = ease(t / 1.2).min(ease((a.dur - t) / 1.2));
        let y = self.ground(env, p);
        let _ = total;
        figure(b, Vec3::new(p.x, y, p.y), yaw_of(dir), t * 5.0, &g, if moving { Pose::Walk } else { Pose::Stand }, Prop::Bag, env.time, alpha);
    }

    pub(super) fn draw_closing(&self, b: &mut Boxes, env: &Env, a: &Active, t: f32) {
        // last orders: a merry fellow lurching home with a friend steering. He
        // goes in fits and starts, with a pause now and then to get his bearings
        let tau = (t - 3.0).max(0.0);
        let d = 0.8 * tau + 0.38 * (1.7 * tau).sin();
        let (p, dir, moving) = walk(&a.path, d.max(0.0));
        // the stride follows the ground covered, so his feet only go when he does
        let stride = d * 3.4;
        // weaving gently across the road; the heading wanders more than the track
        let weave = 0.25 * (tau * 1.1).sin() * if moving { 1.0 } else { 0.0 };
        let side = Vec2::new(dir.y, -dir.x);
        let q = p + side * weave;
        let alpha = ease(t / 1.5).min(ease((a.dur - t) / 1.5));
        let g = garb(a.seed, None, None);
        let g2 = garb(a.seed ^ 5, None, None).hat(Hat::Cap, [0.3, 0.3, 0.35]);
        let y = self.ground(env, q);
        let heading = yaw_of(dir) + 0.45 * (tau * 1.1 + 0.6).sin() * if moving { 1.0 } else { 0.0 };
        figure(b, Vec3::new(q.x, y, q.y), heading, stride, &g, Pose::Drunk, Prop::Mug, env.time, alpha);
        self.happen.party.borrow_mut().push((Kind::Closing, Vec3::new(q.x, y, q.y)));
        // the friend walks at an even pace, a step or so behind, however the
        // drunk lurches ahead and falls back
        let steady = (0.8 * tau - 1.1).max(0.0);
        let (p2, d2, moving2) = walk(&a.path, steady);
        let y2 = self.ground(env, p2);
        figure(b, Vec3::new(p2.x, y2, p2.y), yaw_of(d2), steady * 3.4, &g2, if moving2 && tau > 1.4 { Pose::Walk } else { Pose::Stand }, Prop::None, env.time, alpha);
        self.happen.party.borrow_mut().push((Kind::Closing, Vec3::new(p2.x, y2, p2.y)));
    }
}
