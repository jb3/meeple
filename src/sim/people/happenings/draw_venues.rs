use super::*;

impl TownLife {
    pub(super) fn draw_fair(&self, b: &mut Boxes, fx: &mut Boxes, env: &Env, g: Vec2) {
        let time = env.time;
        let night = env.night > 0.35;
        let at = |dx: f32, dz: f32| {
            let p = g + Vec2::new(dx, dz);
            Vec3::new(p.x, self.ground(env, p), p.y)
        };
        // the carousel: a spinning platform of painted horses under a canopy
        let c = at(-5.0, -3.0);
        let spin = time * 0.9;
        let stripes = [[0.86, 0.22, 0.24], [0.96, 0.95, 0.9]];
        b.push(c + Vec3::Y * 0.25, Vec3::new(2.3, 0.15, 2.3), 0.0, [0.55, 0.4, 0.26], MAT_LIT);
        b.push(c + Vec3::Y * 1.6, Vec3::new(0.12, 1.4, 0.12), 0.0, [0.9, 0.8, 0.4], MAT_LIT);
        for k in 0..8 {
            let a = spin + k as f32 * TAU / 8.0;
            b.push(c + Vec3::new(a.cos() * 2.0, 3.05, a.sin() * 2.0), Vec3::new(0.7, 0.18, 0.7), -a, stripes[k % 2], MAT_LIT);
        }
        b.push(c + Vec3::Y * 3.5, Vec3::new(1.4, 0.2, 1.4), -spin, stripes[0], MAT_LIT);
        b.push(c + Vec3::Y * 3.85, Vec3::new(0.5, 0.2, 0.5), -spin, stripes[1], MAT_LIT);
        if night {
            b.push(c + Vec3::Y * 4.15, Vec3::splat(0.14), 0.0, [1.0, 0.9, 0.5], MAT_GLOW);
        }
        for k in 0..6 {
            let a = spin + k as f32 * TAU / 6.0;
            let bob = (time * 2.2 + k as f32 * 1.7).sin() * 0.25;
            let p = c + Vec3::new(a.cos() * 1.6, 1.0 + bob, a.sin() * 1.6);
            let coat = [[0.95, 0.95, 0.9], [0.55, 0.35, 0.2], [0.2, 0.2, 0.22]][k % 3];
            b.push(p + Vec3::Y * 0.1, Vec3::new(0.2, 0.2, 0.42), -a, coat, MAT_LIT);
            b.push(p + Vec3::new(-a.sin() * 0.4, 0.4, a.cos() * 0.4), Vec3::new(0.1, 0.18, 0.12), -a, coat, MAT_LIT);
            b.push(c + Vec3::new(a.cos() * 1.6, 1.75, a.sin() * 1.6), Vec3::new(0.02, 0.75, 0.02), 0.0, [0.9, 0.8, 0.4], MAT_LIT);
        }
        // the ferris wheel, turning slowly in a vertical plane
        let w = at(5.0, 2.0);
        let hub = w + Vec3::Y * 5.6;
        let rot = time * 0.25;
        // rim and spokes are strings of overlapping blocks, since a box can
        // only turn about the vertical
        for k in 0..110 {
            let a = rot + k as f32 * TAU / 110.0;
            b.push(hub + Vec3::new(a.cos() * 4.8, a.sin() * 4.8, 0.0), Vec3::new(0.13, 0.13, 0.13), 0.0, [0.85, 0.85, 0.9], MAT_LIT);
            b.push(hub + Vec3::new(a.cos() * 3.2, a.sin() * 3.2, 0.0), Vec3::new(0.08, 0.08, 0.08), 0.0, [0.75, 0.75, 0.82], MAT_LIT);
        }
        for k in 0..8 {
            let a = rot + k as f32 * TAU / 8.0;
            let mut r = 0.1;
            while r < 4.8 {
                b.push(hub + Vec3::new(a.cos() * r, a.sin() * r, 0.0), Vec3::new(0.085, 0.085, 0.085), 0.0, [0.8, 0.8, 0.85], MAT_LIT);
                r += 0.13;
            }
            let gp = hub + Vec3::new(a.cos() * 4.8, a.sin() * 4.8 - 0.55, 0.0);
            let col = [[0.9, 0.3, 0.3], [0.3, 0.55, 0.9], [0.95, 0.8, 0.3], [0.4, 0.8, 0.45]][k % 4];
            b.push(gp, Vec3::new(0.4, 0.3, 0.45), 0.0, col, MAT_LIT);
            b.push(gp + Vec3::Y * 0.45, Vec3::new(0.02, 0.2, 0.02), 0.0, [0.8, 0.8, 0.85], MAT_LIT);
            if night {
                b.push(gp + Vec3::Y * 0.4, Vec3::splat(0.1), 0.0, [1.0, 0.9, 0.5], MAT_GLOW);
            }
        }
        b.push(hub, Vec3::new(0.3, 0.3, 0.3), 0.0, [0.5, 0.5, 0.55], MAT_LIT);
        for dz in [-0.9, 0.9] {
            b.push(w + Vec3::new(-1.6, 2.7, dz), Vec3::new(0.12, 2.9, 0.1), 0.35, [0.7, 0.2, 0.2], MAT_LIT);
            b.push(w + Vec3::new(1.6, 2.7, dz), Vec3::new(0.12, 2.9, 0.1), -0.35, [0.7, 0.2, 0.2], MAT_LIT);
        }
        // the strongman's tower and bell
        let s = at(0.0, 7.0);
        b.push(s + Vec3::Y * 2.0, Vec3::new(0.18, 2.0, 0.1), 0.0, [0.9, 0.9, 0.85], MAT_LIT);
        let puck = ((time / 7.0).fract() * 3.0).min(1.0);
        let up = if hash_u32(((time / 7.0) as u32) ^ self.town.id) % 3 == 0 { puck * 3.4 } else { puck.min(0.3) * 2.0 };
        b.push(s + Vec3::new(0.0, 0.5 + up, 0.12), Vec3::new(0.12, 0.1, 0.06), 0.0, [0.9, 0.2, 0.2], MAT_LIT);
        b.push(s + Vec3::Y * 4.2, Vec3::new(0.2, 0.2, 0.2), 0.0, [0.9, 0.75, 0.2], MAT_LIT);
        let hammer = Garb { shirt: [0.8, 0.2, 0.2], ..garb(self.town.id, None, None) };
        figure(b, s + Vec3::new(0.9, 0.0, 0.1), -PI * 0.5, 0.0, &hammer, Pose::Dig, Prop::None, time, 1.0);
        // stalls round the edge
        let produce = [[0.85, 0.2, 0.2], [0.95, 0.75, 0.3], [0.95, 0.55, 0.65], [0.35, 0.65, 0.9]];
        for (k, (dx, dz, yaw)) in [(-9.0, 4.0, PI * 0.5), (-2.0, 9.0, 0.0), (8.0, 8.0, 0.4), (9.5, -4.5, -PI * 0.5), (2.0, -9.0, PI)].iter().enumerate() {
            let p = at(*dx, *dz);
            let mut r = Rig::new(b, p, *yaw, 1.0);
            let wood = [0.55, 0.4, 0.26];
            r.part(Vec3::new(0.0, 0.8, 0.0), Vec3::new(0.85, 0.05, 0.42), wood);
            for (x, z) in [(-0.8, -0.38), (0.8, -0.38), (-0.8, 0.38), (0.8, 0.38)] {
                r.part(Vec3::new(x, 0.95, z), Vec3::new(0.04, 0.95, 0.04), wood);
            }
            for i in 0..4 {
                let sc = if k % 2 == 0 { [[0.26, 0.5, 0.88], WHITE] } else { [[0.9, 0.7, 0.2], [0.9, 0.3, 0.3]] };
                r.part(Vec3::new(-0.66 + i as f32 * 0.44, 1.92, 0.05), Vec3::new(0.22, 0.04, 0.6), sc[i % 2]);
            }
            for i in 0..3 {
                r.part(Vec3::new(-0.5 + i as f32 * 0.5, 0.95, 0.05), Vec3::new(0.18, 0.1, 0.16), produce[(k + i) % 4]);
            }
            if night {
                r.part_mat(Vec3::new(0.0, 2.1, 0.0), Vec3::splat(0.08), [1.0, 0.85, 0.5], MAT_GLOW);
            }
        }
        // bunting between two poles
        let (p0, p1) = (at(-8.0, -8.0), at(8.0, -8.0));
        for k in 0..16 {
            let f = k as f32 / 15.0;
            let q = p0.lerp(p1, f) + Vec3::Y * (3.6 - (f * PI).sin() * 0.5);
            let cols = [[0.9, 0.25, 0.3], [0.25, 0.5, 0.9], [0.98, 0.85, 0.3], [0.4, 0.8, 0.45]];
            fx.push(q, Vec3::new(0.1, 0.12, 0.02), 0.0, cols[k % 4], MAT_UNLIT);
        }
        for p in [p0, p1] {
            b.push(p + Vec3::Y * 1.8, Vec3::new(0.06, 1.8, 0.06), 0.0, [0.55, 0.4, 0.26], MAT_LIT);
        }
        // visitors strolling and standing
        for k in 0..14u32 {
            let hk = hash_u32(self.town.id ^ k.wrapping_mul(0x2545));
            let ang = unit(hk) * TAU + time * (0.02 + unit(hk >> 5) * 0.03) * if k % 2 == 0 { 1.0 } else { -1.0 };
            let rr = 5.5 + unit(hk >> 9) * 7.0;
            let p = g + Vec2::new(ang.cos() * rr, ang.sin() * rr * 0.8);
            let tang = Vec2::new(-ang.sin(), ang.cos()) * if k % 2 == 0 { 1.0 } else { -1.0 };
            let y = self.ground(env, p);
            let mut gb = garb(hk, None, None);
            if k % 4 == 0 {
                gb = gb.kid();
            }
            let watching = k % 5 == 0;
            figure(b, Vec3::new(p.x, y, p.y), yaw_of(tang), time * 3.5 + k as f32, &gb, if watching { Pose::Cheer } else { Pose::Walk }, if k % 3 == 0 { Prop::Mug } else { Prop::None }, time, 1.0);
        }
    }

    pub(super) fn draw_cricket(&self, b: &mut Boxes, env: &Env, g: Vec2) {
        let time = env.time;
        let at = |dx: f32, dz: f32| {
            let p = g + Vec2::new(dx, dz);
            Vec3::new(p.x, self.ground(env, p), p.y)
        };
        // the pitch is laid cell by cell on the ground as it is, so it never
        // hangs over a drop
        for i in -5..=5 {
            let p = at(i as f32, 0.0);
            b.push(p + Vec3::new(0.0, 0.04, 0.0), Vec3::new(0.52, 0.04, 0.9), 0.0, [0.78, 0.7, 0.5], MAT_LIT);
        }
        for sx in [-4.6, 4.6] {
            let p = at(sx, 0.0);
            for dz in [-0.12, 0.0, 0.12] {
                b.push(p + Vec3::new(0.0, 0.4, dz), Vec3::new(0.02, 0.38, 0.02), 0.0, [0.95, 0.9, 0.7], MAT_LIT);
            }
        }
        // each ball is ten seconds: the bowler runs up with it in hand and
        // bowls; the striker swings; a fielder chases it down and throws it
        // back (or the keeper takes it); the bowler walks back to his mark
        let cyc = (time / 10.0).fract();
        let n = (time / 10.0) as u32;
        let hit = hash_u32(n ^ self.town.id) % 3 != 0;
        let white = |k: u32| garb(self.town.id ^ k, Some(WHITE), Some(WHITE));
        let hand = Vec3::new(0.15, 1.0, -0.3);
        // where the ball ends up after a stroke
        let a = n as f32 * 1.7;
        let (land, stop) = (Vec2::new(4.4 + a.cos() * 12.0, a.sin() * 12.0), Vec2::new(4.4 + a.cos() * 14.5, a.sin() * 14.5));
        let keeper = Vec2::new(7.2, 0.2);
        let spot = if hit { stop } else { keeper };

        // the bowler: run up, deliver, walk back
        let bx = if cyc < 0.32 {
            -9.0 + ease(cyc / 0.32) * 3.5
        } else if cyc < 0.5 {
            -5.5
        } else if cyc < 0.9 {
            -5.5 - ease((cyc - 0.5) / 0.4) * 3.5
        } else {
            -9.0
        };
        let bowler = at(bx, 0.0);
        let bpose = if (0.32..0.42).contains(&cyc) {
            Pose::Cheer
        } else if cyc < 0.32 || (0.5..0.9).contains(&cyc) {
            Pose::Walk
        } else {
            Pose::Stand
        };
        figure(b, bowler, PI * 0.5, time * 7.0, &white(1), bpose, Prop::None, time, 1.0);

        // the striker takes a stroke, whether or not it connects
        let stroke = if cyc < 0.38 {
            0.0
        } else if cyc < 0.48 {
            ease((cyc - 0.38) / 0.1)
        } else if cyc < 0.62 {
            1.0
        } else if cyc < 0.74 {
            1.0 - ease((cyc - 0.62) / 0.12)
        } else {
            0.0
        };
        figure(b, at(4.6, -0.5), -PI * 0.5, stroke, &white(2), Pose::Swing, Prop::None, time, 1.0);
        figure(b, at(-4.6, 0.7), PI * 0.5, 0.0, &white(3), Pose::Stand, Prop::Bat, time, 1.0);
        let kpose = if (0.25..0.6).contains(&cyc) && !hit { Pose::Reach } else { Pose::Stand };
        figure(b, at(keeper.x, keeper.y), -PI * 0.5, 0.0, &white(4), kpose, Prop::None, time, 1.0);

        // the fielders; the one nearest where the ball ends up goes for it
        let homes: Vec<Vec2> = (0..9u32)
            .map(|k| {
                let ang = k as f32 * TAU / 9.0 + 0.4;
                let rr = 8.5 + unit(hash_u32(k)) * 2.0;
                Vec2::new(ang.cos() * rr, ang.sin() * rr * 0.9)
            })
            .collect();
        let chaser = if hit {
            homes.iter().enumerate().min_by(|a, b| a.1.distance(stop).partial_cmp(&b.1.distance(stop)).unwrap()).map(|(k, _)| k)
        } else {
            None
        };
        for (k, home) in homes.iter().enumerate() {
            let k = k as u32;
            if Some(k as usize) == chaser {
                // off at the stroke, on the ball by 0.7, home again after the throw
                let out = ease(((cyc - 0.4) / 0.3).clamp(0.0, 1.0));
                let back = ease(((cyc - 0.76) / 0.2).clamp(0.0, 1.0));
                let pos = home.lerp(stop, out).lerp(*home, back);
                let moving = (0.4..0.7).contains(&cyc) || (0.76..0.96).contains(&cyc);
                let face = if cyc < 0.74 { stop - *home } else { *home - stop };
                figure(b, at(pos.x, pos.y), yaw_of(face), time * 7.0, &white(10 + k), if moving { Pose::Walk } else if (0.7..0.76).contains(&cyc) { Pose::Reach } else { Pose::Stand }, Prop::None, time, 1.0);
            } else {
                figure(b, at(home.x, home.y), yaw_of(-*home), 0.0, &white(10 + k), Pose::Stand, Prop::None, time, 1.0);
            }
        }

        // the ball
        let ground_at = |p: Vec2| at(p.x, p.y).y;
        let ball = if cyc < 0.32 || cyc >= 0.96 {
            bowler + hand
        } else if cyc < 0.4 {
            // bowled: a bounce on the way down the pitch
            let f = (cyc - 0.32) / 0.08;
            let x = lerp(-5.3, 4.4, f);
            let y = if f < 0.55 { 0.12 + 1.7 * (1.0 - f / 0.55).powi(2) } else { 0.12 + 0.7 * ((f - 0.55) / 0.45 * PI * 0.5).sin() };
            Vec3::new(g.x + x, at(x, 0.0).y + y, g.y)
        } else if hit && cyc < 0.6 {
            let f = (cyc - 0.4) / 0.2;
            let p = Vec2::new(4.4, 0.0).lerp(land, f);
            Vec3::new(g.x + p.x, ground_at(p) + lerp(0.8, 0.12, f) + (f * PI).sin() * 3.2, g.y + p.y)
        } else if hit && cyc < 0.68 {
            let f = ease((cyc - 0.6) / 0.08);
            let p = land.lerp(stop, f);
            Vec3::new(g.x + p.x, ground_at(p) + 0.12, g.y + p.y)
        } else if !hit && cyc < 0.46 {
            let f = (cyc - 0.4) / 0.06;
            let p = Vec2::new(4.4, 0.0).lerp(keeper, f);
            Vec3::new(g.x + p.x, ground_at(p) + lerp(0.8, 0.95, f), g.y + p.y)
        } else if cyc < 0.74 {
            // waiting in the fielder's or keeper's hand
            Vec3::new(g.x + spot.x, ground_at(spot) + 1.0, g.y + spot.y)
        } else {
            // thrown back to the bowler, who's waiting at the end of his mark
            let f = (cyc - 0.74) / 0.22;
            let to = Vec2::new(-9.0, 0.0);
            let p = spot.lerp(to, f);
            let from_y = ground_at(spot) + 1.0;
            let to_y = ground_at(to) + 1.0;
            Vec3::new(g.x + p.x, lerp(from_y, to_y, f) + (f * PI).sin() * 3.0, g.y + p.y)
        };
        b.push(ball, Vec3::splat(0.09), 0.0, [0.8, 0.12, 0.1], MAT_LIT);

        // spectators on the boundary rope, and a tea tent
        let tent = at(-11.0, 7.0);
        b.push(tent + Vec3::Y * 0.8, Vec3::new(1.6, 0.8, 1.2), 0.0, WHITE, MAT_LIT);
        b.push(tent + Vec3::Y * 1.8, Vec3::new(1.8, 0.2, 1.4), 0.0, [0.3, 0.5, 0.85], MAT_LIT);
        b.push(tent + Vec3::new(0.0, 0.9, 1.25), Vec3::new(1.2, 0.35, 0.05), 0.0, [0.5, 0.4, 0.3], MAT_LIT);
        for k in 0..7u32 {
            let p = at(-13.0 + k as f32 * 1.1, 10.5);
            let cheer = cyc > 0.42 && hit && cyc < 0.65;
            figure(b, p, PI * -0.1, k as f32, &garb(self.town.id ^ (40 + k), None, None), if cheer { Pose::Cheer } else { Pose::Stand }, Prop::None, time, 1.0);
        }
    }

    /// The easter egg: Bella, working out in the square in front of a mirror,
    /// admiring himself.
    pub(super) fn draw_bella(&self, b: &mut Boxes, env: &Env, cam_yaw: f32) {
        let time = env.time;
        const GLASS: [f32; 3] = [0.62, 0.74, 0.84];
        // Bella faces -z, into a mirror a couple of steps along, so the glass
        // faces +z and the usual camera sees his back and his face in the mirror.
        // From the other side you see the mirror's back, and nothing of what's in
        // the glass
        let front = cam_yaw.cos() > 0.1;
        let p0 = self.bella_spot();
        let at = |dx: f32, dz: f32| {
            let q = p0 + Vec2::new(dx, dz);
            Vec3::new(q.x, self.ground(env, q), q.y)
        };
        let base = at(0.0, 0.0);
        // a rubber mat, and a rack of spare weights beside it
        b.push(base + Vec3::new(0.0, 0.03, -1.4), Vec3::new(1.7, 0.03, 2.3), 0.0, [0.16, 0.16, 0.19], MAT_LIT);
        let rack = at(1.9, 0.6);
        for dx in [-0.5, 0.5] {
            b.push(rack + Vec3::new(dx, 0.35, 0.0), Vec3::new(0.05, 0.35, 0.05), 0.0, [0.3, 0.3, 0.34], MAT_LIT);
        }
        b.push(rack + Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.55, 0.03, 0.05), 0.0, [0.3, 0.3, 0.34], MAT_LIT);
        for (dx, h) in [(-0.35, 0.1), (0.0, 0.14), (0.35, 0.1)] {
            b.push(rack + Vec3::new(dx, 0.58 + h, 0.0), Vec3::new(0.1, h, 0.1), 0.0, [0.15, 0.15, 0.18], MAT_LIT);
        }
        // the mirror: a wooden frame on two feet, a pane of pale glass down to
        // the ground
        let m = at(0.0, -2.0);
        let wood = [0.45, 0.32, 0.2];
        for dx in [-0.95, 0.95] {
            b.push(m + Vec3::new(dx, 1.2, 0.0), Vec3::new(0.07, 1.2, 0.06), 0.0, wood, MAT_LIT);
            b.push(m + Vec3::new(dx * 1.1, 0.05, 0.0), Vec3::new(0.12, 0.05, 0.25), 0.0, wood, MAT_LIT);
        }
        b.push(m + Vec3::new(0.0, 2.38, 0.0), Vec3::new(1.02, 0.07, 0.06), 0.0, wood, MAT_LIT);
        b.push(m + Vec3::new(0.0, 0.1, 0.0), Vec3::new(1.02, 0.07, 0.06), 0.0, wood, MAT_LIT);
        if front {
            // solid, like a real mirror: nothing behind it shows through
            b.push(m + Vec3::new(0.0, 1.24, 0.0), Vec3::new(0.9, 1.15, 0.02), 0.0, GLASS, MAT_LIT);
        } else {
            // the back: a plain board
            b.push(m + Vec3::new(0.0, 1.24, -0.05), Vec3::new(0.9, 1.15, 0.03), 0.0, [0.34, 0.25, 0.18], MAT_LIT);
        }
        // Bella, big and in a vest, curling dumbbells and watching himself. (He's
        // a villager like any other: he's here from morning, and goes home to bed
        // at night.)
        let lifting = self.people.iter().find(|q| q.bella && q.state == State::Working && q.dest == Dest::Gym && q.fade > 0.0);
        if let Some(q) = lifting {
            // dressed exactly as he is when walking, so it's plainly the same man
            let g = Garb { shirt: q.look.shirt, pants: q.look.pants, skin: q.look.skin, hair: q.look.hair, hat: q.look.hat, hat_col: q.look.hat_col, scale: q.look.scale };
            let beat = time * 2.1;
            figure(b, base, yaw_of(Vec2::new(0.0, -1.0)), beat, &g, Pose::Lift, Prop::None, time, q.fade);
            // his reflection is a flat image on the glass: the same figure pressed
            // flat against the pane, tinted a little towards its blue, with the arms
            // swapped over as a reflection has them. It can't poke out of the frame
            // or through to the front, and it's only drawn when the camera's on the
            // mirror's side
            let wash = |c: [f32; 3]| [c[0] * 0.78 + GLASS[0] * 0.22, c[1] * 0.78 + GLASS[1] * 0.22, c[2] * 0.78 + GLASS[2] * 0.22];
            let mut gr = g;
            gr.shirt = wash(g.shirt);
            gr.pants = wash(g.pants);
            gr.skin = wash(g.skin);
            gr.hair = wash(g.hair);
            if front {
                FLATTEN.with(|f| f.set(0.03));
                figure(b, m + Vec3::new(0.0, 0.0, 0.05), yaw_of(Vec2::new(0.0, 1.0)), beat + PI, &gr, Pose::Lift, Prop::None, time, q.fade);
                FLATTEN.with(|f| f.set(1.0));
            }
        }
        // his name, in block letters on a board above the mirror, readable from
        // both sides
        const GLYPHS: [[u8; 5]; 4] = [[0b110, 0b101, 0b110, 0b101, 0b110], [0b111, 0b100, 0b110, 0b100, 0b111], [0b100, 0b100, 0b100, 0b100, 0b111], [0b010, 0b101, 0b111, 0b101, 0b101]];
        let name = [0usize, 1, 2, 2, 3];
        let px = 0.14;
        let width = (name.len() as f32 * 4.0 - 1.0) * px;
        b.push(m + Vec3::new(0.0, 3.2, 0.0), Vec3::new(width * 0.5 + 0.15, 0.5, 0.05), 0.0, [0.25, 0.2, 0.17], MAT_LIT);
        for dx in [-1.0f32, 1.0] {
            b.push(m + Vec3::new(dx * (width * 0.5 + 0.1), 2.7, 0.0), Vec3::new(0.05, 0.3, 0.05), 0.0, wood, MAT_LIT);
        }
        for side in [-1.0f32, 1.0] {
            // seen from +z the word runs towards +x; from -z towards -x
            let dir = side;
            for (li, &gi) in name.iter().enumerate() {
                for (row, bits) in GLYPHS[gi].iter().enumerate() {
                    for col in 0..3 {
                        if bits & (0b100 >> col) != 0 {
                            let u = (li as f32 * 4.0 + col as f32) * px - width * 0.5 + px * 0.5;
                            b.push(
                                m + Vec3::new(dir * u, 3.2 + 0.28 - row as f32 * px, side * 0.08),
                                Vec3::new(px * 0.5, px * 0.5, 0.025),
                                0.0,
                                [0.98, 0.9, 0.6],
                                MAT_LIT,
                            );
                        }
                    }
                }
            }
        }
    }

    pub(super) fn draw_rink(&self, b: &mut Boxes, fx: &mut Boxes, env: &Env, g: Vec2) {
        let time = env.time;
        // the green is levelled, so the whole rink sits at one height
        let y0 = self.ground(env, g);
        let at = |dx: f32, dz: f32| Vec3::new(g.x + dx, y0, g.y + dz);
        let (rx, rz) = (6.2f32, 4.4f32);
        let ice_top = y0 + 0.11;
        // the ice: a close mat of pale blue-white tiles with soft patches of
        // sheen, so it reads as one frozen sheet
        for i in -13i32..=13 {
            for j in -9i32..=9 {
                let (x, z) = (i as f32 * 0.5, j as f32 * 0.5);
                if (x / (rx - 0.2)).powi(2) + (z / (rz - 0.2)).powi(2) > 1.0 {
                    continue;
                }
                let n = |a: i32, c: i32| unit(hash_u32(self.town.id ^ ((a + 40) as u32).wrapping_mul(131) ^ ((c + 40) as u32).wrapping_mul(977)));
                let patch = n(i.div_euclid(3), j.div_euclid(3)) * 0.05;
                let fleck = n(i, j) * 0.02;
                let col = [0.68 + patch + fleck, 0.80 + patch + fleck, 0.89 + patch * 0.6];
                b.push(at(x, z) + Vec3::new(0.0, 0.06, 0.0), Vec3::new(0.26, 0.05, 0.26), 0.0, col, MAT_LIT);
            }
        }
        // the ice runs on out to the rail all the way round: a ring of it laid
        // over the ragged edge of the tiles
        for k in 0..96 {
            let f0 = k as f32 / 96.0 * TAU;
            let f1 = (k + 1) as f32 / 96.0 * TAU;
            let ring = |f: f32| Vec2::new(f.cos() * (rx - 0.1), f.sin() * (rz - 0.1));
            let (a, c) = (ring(f0), ring(f1));
            let (mid, d) = ((a + c) * 0.5, c - a);
            let n = unit(hash_u32(self.town.id ^ (k as u32).wrapping_mul(0x9e37)));
            b.push(at(mid.x, mid.y) + Vec3::new(0.0, 0.06, 0.0), Vec3::new(0.62, 0.05, d.length() * 0.58), d.x.atan2(d.y), [0.70 + n * 0.03, 0.82 + n * 0.03, 0.90], MAT_LIT);
        }
        // a continuous plank rail round the edge and snow banked against it,
        // built from overlapping lengths that follow the curve
        let seg = 96;
        let ring = |f: f32, grow: f32| Vec2::new(f.cos() * (rx + grow), f.sin() * (rz + grow));
        for k in 0..seg {
            let f0 = k as f32 / seg as f32 * TAU;
            let f1 = (k + 1) as f32 / seg as f32 * TAU;
            for (grow, half_h, half_w, col, rise) in [
                (0.4f32, 0.16f32, 0.09f32, [0.45, 0.32, 0.21], 0.0f32),
                (0.4, 0.04, 0.12, [0.52, 0.38, 0.26], 0.32),
                (1.05, 0.15, 0.4, [0.95, 0.97, 1.0], 0.0),
            ] {
                let (a, c) = (ring(f0, grow), ring(f1, grow));
                let mid = (a + c) * 0.5;
                let d = c - a;
                let wob = if half_w > 0.3 { (k % 3) as f32 * 0.03 } else { 0.0 };
                b.push(at(mid.x, mid.y) + Vec3::new(0.0, half_h + wob + rise, 0.0), Vec3::new(half_w, half_h + wob, d.length() * 0.56), 0.0 + (d.x).atan2(d.y), col, MAT_LIT);
            }
        }
        for k in 0..8 {
            let q = ring(k as f32 / 8.0 * TAU, 0.4);
            b.push(at(q.x, q.y) + Vec3::new(0.0, 0.62, 0.0), Vec3::new(0.07, 0.62, 0.07), 0.0, [0.38, 0.27, 0.18], MAT_LIT);
            if env.night > 0.4 && k % 2 == 0 {
                b.push(at(q.x, q.y) + Vec3::new(0.0, 1.3, 0.0), Vec3::splat(0.14), 0.0, [1.0, 0.8, 0.4], MAT_GLOW);
            }
        }
        // the skaters, each looping about the ice on its own course, with the
        // marks of their blades fading behind them
        let skater = |k: u32, t: f32| -> Vec2 {
            let hk = hash_u32(self.town.id ^ k.wrapping_mul(0x3c3));
            let ph = |n: u32| unit(hash_u32(hk ^ n.wrapping_mul(0x51ed))) * TAU;
            let (ax, az) = (rx - 1.0, rz - 0.9);
            // a rink is never all one pattern: some go round the edge, one
            // practises circles, a couple meander, one crosses and recrosses,
            // and the last tags along behind that one
            let wander = |n: u32, w: [f32; 4]| {
                Vec2::new(
                    ax * 0.8 * ((w[0] * t + ph(n)).sin() * 0.7 + (w[1] * t + ph(n + 1)).sin() * 0.3),
                    az * 0.8 * ((w[2] * t + ph(n + 2)).sin() * 0.7 + (w[3] * t + ph(n + 3)).sin() * 0.3),
                )
            };
            let crossing = |t: f32| {
                let (a, b) = (Vec2::new(-ax * 0.8, az * 0.5 * (ph(9).sin())), Vec2::new(ax * 0.8, -az * 0.5 * (ph(9).sin())));
                let u = 0.5 + 0.5 * (t * 0.33 + ph(10)).sin();
                let d = (b - a).normalize_or_zero();
                a.lerp(b, u) + Vec2::new(-d.y, d.x) * (t * 1.1).sin() * 0.3
            };
            match k {
                0 => Vec2::new((t * 0.42 + ph(1)).cos() * ax * 0.85, (t * 0.42 + ph(1)).sin() * az * 0.85),
                1 => Vec2::new((-t * 0.5 + ph(2)).cos() * ax * 0.55, (-t * 0.5 + ph(2)).sin() * az * 0.55),
                2 => Vec2::new(rx * 0.3, -rz * 0.2) + Vec2::new((t * 0.95 + ph(3)).cos(), (t * 0.95 + ph(3)).sin()) * 1.3,
                3 => wander(4, [0.23, 0.57, 0.31, 0.71]),
                4 => wander(8, [0.29, 0.61, 0.19, 0.47]),
                5 => crossing(t),
                _ => {
                    // behind the crossing skater, a step off to the side
                    let lead = crossing(t - 0.9);
                    let ahead = crossing(t - 0.8);
                    let d = (ahead - lead).normalize_or_zero();
                    lead + Vec2::new(-d.y, d.x) * 0.55
                }
            }
        };
        const TRAIL: u32 = 30;
        const GAP: f32 = 0.12;
        for k in 0..7u32 {
            let mut prev = skater(k, time);
            for j in 1..=TRAIL {
                let q = skater(k, time - j as f32 * GAP);
                let d = prev - q;
                if d.length() > 0.02 {
                    let fade = 1.0 - j as f32 / TRAIL as f32;
                    let mid = (prev + q) * 0.5;
                    fx.push_alpha(
                        Vec3::new(g.x + mid.x, ice_top + 0.004, g.y + mid.y),
                        Vec3::new(0.05, 0.003, d.length() * 0.56),
                        d.x.atan2(d.y),
                        [0.4, 0.53, 0.68],
                        MAT_LIT,
                        0.9 * fade.powf(1.3),
                    );
                }
                prev = q;
            }
            let p = skater(k, time);
            let p1 = skater(k, time + 0.1);
            let v = p1 - p;
            let hk = hash_u32(self.town.id ^ k.wrapping_mul(0x3c3));
            let mut gb = garb(hk, None, None);
            if k % 3 == 0 {
                gb = gb.kid();
            }
            figure(b, Vec3::new(g.x + p.x, ice_top - 0.01, g.y + p.y), yaw_of(v), time * 1.5 + k as f32 * 2.0, &gb, Pose::Skate, Prop::None, time, 1.0);
        }
    }
}
