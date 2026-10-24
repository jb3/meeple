use super::*;

impl TownLife {
    pub(super) fn draw_chores(&self, b: &mut Boxes, env: &Env, view: Vec2, view_r: f32) {
        let time = env.time;
        let h = env.hour;
        let c = self.center();
        if c.distance(view) > view_r + 30.0 {
            return;
        }
        let day_ok = env.rain < 0.45;
        // winter: shovelling snow off the streets, piled either side
        if (2.95..3.95).contains(&env.season) && (7.5..15.5).contains(&h) && hash_u32(self.town.id ^ env.day as u32) % 2 == 0 {
            for k in 0..3u32 {
                let side = if k % 2 == 0 { 1.0 } else { -1.0 };
                let p = self.intersection(c + Vec2::new(side * 11.0, (k as f32 - 1.0) * 11.0)) + Vec2::new(2.0, 1.2 * side);
                let y = self.ground(env, p);
                let gb = garb(self.town.id ^ k, Some([0.35, 0.3, 0.25]), None).hat(Hat::Cap, [0.7, 0.3, 0.2]);
                figure(b, Vec3::new(p.x, y, p.y), 0.8 * side, k as f32 * 2.0, &gb, Pose::Dig, Prop::None, time, 1.0);
                for j in 0..5 {
                    let q = p + Vec2::new(1.2 + j as f32 * 0.5, 0.6 * side);
                    b.push(Vec3::new(q.x, y + 0.18, q.y), Vec3::new(0.35, 0.18 + (j % 2) as f32 * 0.05, 0.3), 0.0, [0.95, 0.97, 1.0], MAT_LIT);
                }
            }
        }
        // autumn: apple picking from a ladder against the garden's oak
        if (2.0..2.95).contains(&env.season) && (9.0..16.0).contains(&h) && day_ok {
            if let Some(l) = self.lots.iter().find(|l| l.kind == LotKind::Garden) {
                let (x, z) = l.center();
                let tree = Vec2::new(x, z);
                let y = self.ground(env, tree);
                b.push(Vec3::new(tree.x + 1.2, y + 1.6, tree.y), Vec3::new(0.05, 1.7, 0.05), 0.0, [0.6, 0.45, 0.28], MAT_LIT);
                b.push(Vec3::new(tree.x + 1.5, y + 1.6, tree.y), Vec3::new(0.05, 1.7, 0.05), 0.0, [0.6, 0.45, 0.28], MAT_LIT);
                for r in 0..7 {
                    b.push(Vec3::new(tree.x + 1.35, y + 0.4 + r as f32 * 0.45, tree.y), Vec3::new(0.2, 0.025, 0.04), 0.0, [0.6, 0.45, 0.28], MAT_LIT);
                }
                let gb = garb(l.seed, Some([0.6, 0.35, 0.2]), None).hat(Hat::Straw, [0.85, 0.75, 0.4]);
                figure(b, Vec3::new(tree.x + 1.35, y + 2.0, tree.y - 0.35), -PI * 0.5, 0.0, &gb, Pose::Reach, Prop::None, time, 1.0);
                for k in 0..3 {
                    b.push(Vec3::new(tree.x + 2.1 + k as f32 * 0.35, y + 0.2, tree.y + 0.4), Vec3::new(0.2, 0.15, 0.2), 0.0, [0.62, 0.48, 0.28], MAT_LIT);
                    b.push(Vec3::new(tree.x + 2.1 + k as f32 * 0.35, y + 0.38, tree.y + 0.4), Vec3::new(0.15, 0.08, 0.15), 0.0, [0.85, 0.2, 0.18], MAT_LIT);
                }
                let gb2 = garb(l.seed ^ 3, None, None).kid();
                figure(b, Vec3::new(tree.x + 2.6, y, tree.y + 1.4), 2.4, 0.0, &gb2, Pose::Stand, Prop::Posy, time, 1.0);
            }
        }
        // spring: a sheep sheared in the farmyard, the fleece piled beside
        if (0.2..1.0).contains(&env.season) && (9.0..15.0).contains(&h) && day_ok {
            if let Some(l) = self.lots.iter().find(|l| l.kind == LotKind::Farm) {
                let (x, z) = l.center();
                let y = self.ground(env, Vec2::new(x, z));
                let sheep = Vec3::new(x + 0.4, y + 0.38, z);
                let shorn = ((time * 0.05).fract() * 2.0).min(1.0);
                b.push(sheep, Vec3::new(0.3, 0.2 + 0.1 * (1.0 - shorn), 0.5), 0.0, if shorn > 0.7 { [0.85, 0.78, 0.7] } else { [0.96, 0.96, 0.93] }, MAT_LIT);
                b.push(sheep + Vec3::new(0.0, 0.1, 0.5), Vec3::new(0.13, 0.13, 0.13), 0.0, [0.2, 0.18, 0.18], MAT_LIT);
                for (dx, dz) in [(0.15, 0.25), (-0.15, 0.25), (0.15, -0.25), (-0.15, -0.25)] {
                    b.push(Vec3::new(sheep.x + dx, y + 0.15, sheep.z + dz), Vec3::new(0.04, 0.15, 0.04), 0.0, [0.2, 0.18, 0.18], MAT_LIT);
                }
                for k in 0..4 {
                    b.push(Vec3::new(x - 1.4 + k as f32 * 0.2, y + 0.15 + (k % 2) as f32 * 0.1, z + 0.9), Vec3::new(0.22, 0.14, 0.22), k as f32, [0.97, 0.97, 0.94], MAT_LIT);
                }
                let gb = garb(l.seed, Some([0.4, 0.5, 0.65]), None).hat(Hat::Straw, [0.85, 0.75, 0.4]);
                figure(b, Vec3::new(x - 0.3, y, z), PI * 0.5, 0.0, &gb, Pose::Kneel, Prop::None, time, 1.0);
            }
        }
    }

    /// The lamplighter at dusk and dawn, the fiddler at the inn door.
    pub(super) fn draw_evening(&self, b: &mut Boxes, fx: &mut Boxes, env: &Env, cam_yaw: f32) {
        let time = env.time;
        let h = env.hour;
        let c = self.center();
        let (rise, set) = (6.4, 18.6);
        let dusk = (set - 1.4..set + 1.0).contains(&h);
        let dawn = (rise - 1.2..rise + 0.6).contains(&h);
        if dusk || dawn {
            // round the four lamps' corners of the square, a lap every minute
            let s = STREET as f32;
            let corners = [c + Vec2::new(-s, -s), c + Vec2::new(s, -s), c + Vec2::new(s, s), c + Vec2::new(-s, s), c + Vec2::new(-s, -s)];
            let lap = path_len(&corners);
            let d = (time * 1.3) % lap;
            let (p, dir, _) = walk(&corners, d);
            let side = Vec2::new(dir.y, -dir.x) * 1.5;
            let q = p + side;
            let y = self.ground(env, q);
            let gb = garb(self.town.id, Some([0.28, 0.24, 0.2]), Some([0.2, 0.18, 0.16])).hat(Hat::Cap, [0.2, 0.18, 0.16]);
            figure(b, Vec3::new(q.x, y, q.y), yaw_of(dir), time * 4.0, &gb, Pose::Walk, Prop::Pole, time, 1.0);
        }
        if let Some(t) = self.tavern {
            if (19.2..23.0).contains(&h) && env.rain < 0.5 {
                let l = &self.lots[t];
                let (ox, oz) = l.outward();
                let p = Vec2::from(l.front) + Vec2::new(-oz as f32, ox as f32) * 2.6 + Vec2::new(ox as f32, oz as f32) * 0.8;
                let y = self.ground(env, p);
                let gb = garb(l.seed, Some([0.55, 0.25, 0.2]), None).hat(Hat::Cap, [0.3, 0.2, 0.15]);
                figure(b, Vec3::new(p.x, y, p.y), yaw_of(Vec2::new(ox as f32, oz as f32)), 0.0, &gb, Pose::Play, Prop::None, time, 1.0);
                let note_on = (time * 0.7 + unit(l.seed) * 5.0) % 2.0 < 0.9;
                if note_on {
                    draw_emote(fx, Vec3::new(p.x, y + 2.3, p.y), Emote::Note, cam_yaw);
                }
            }
        }
    }

    /// Winter play on the green outside the rink: children build snowmen
    /// through the morning and, after noon, split into two sides for a
    /// snowball fight.
    pub(super) fn draw_snow_play(&self, b: &mut Boxes, fx: &mut Boxes, env: &Env, g: Vec2) {
        let time = env.time;
        let h = env.hour;
        let y0 = self.ground(env, g);
        let at = |p: Vec2| Vec3::new(p.x, y0, p.y);
        let day = env.day as u32;
        // snowmen: the first goes up at ten and the rest through the day
        // after, left standing overnight until the thaw of the next morning
        let n = 3 + hash_u32(self.town.id ^ day) % 3;
        for k in 0..n {
            let a = unit(hash_u32(self.town.id ^ k.wrapping_mul(0x9e37))) * TAU;
            let r = 9.5 + 2.2 * unit(hash_u32(self.town.id ^ k.wrapping_mul(0x51ed) ^ 7));
            let p = g + Vec2::new(a.cos() * r, a.sin() * r * 0.8);
            // keep out of the rink's end
            if (p - g).x.abs() < 7.4 && (p - g).y.abs() < 5.4 {
                continue;
            }
            let start = 10.2 + k as f32 * 0.9;
            let built = ((h - start) / 0.8).clamp(0.0, 1.0);
            if built <= 0.0 {
                continue;
            }
            let base = at(p);
            let white = [0.97, 0.98, 1.0];
            // three balls, the last two as the building goes on
            b.push(base + Vec3::new(0.0, 0.34, 0.0), Vec3::splat(0.4), 0.0, white, MAT_LIT);
            if built > 0.4 {
                b.push(base + Vec3::new(0.0, 0.92, 0.0), Vec3::splat(0.3), 0.0, white, MAT_LIT);
            }
            if built > 0.75 {
                let face = a + PI;
                let (fx_, fz_) = (face.cos(), face.sin());
                b.push(base + Vec3::new(0.0, 1.36, 0.0), Vec3::splat(0.22), 0.0, white, MAT_LIT);
                // coal eyes, a carrot nose, a scarf and a hat
                for s in [-0.08f32, 0.08] {
                    b.push(base + Vec3::new(fx_ * 0.2 - fz_ * s, 1.42, fz_ * 0.2 + fx_ * s), Vec3::splat(0.03), 0.0, [0.1, 0.1, 0.1], MAT_LIT);
                }
                b.push(base + Vec3::new(fx_ * 0.28, 1.34, fz_ * 0.28), Vec3::splat(0.04), 0.0, [0.95, 0.5, 0.1], MAT_LIT);
                let scarf = [[0.8, 0.15, 0.15], [0.2, 0.4, 0.8], [0.2, 0.6, 0.3]][k as usize % 3];
                b.push(base + Vec3::new(0.0, 1.14, 0.0), Vec3::new(0.26, 0.05, 0.26), 0.0, scarf, MAT_LIT);
                b.push(base + Vec3::new(0.0, 1.62, 0.0), Vec3::new(0.19, 0.12, 0.19), 0.0, [0.12, 0.1, 0.1], MAT_LIT);
                b.push(base + Vec3::new(0.0, 1.52, 0.0), Vec3::new(0.27, 0.025, 0.27), 0.0, [0.12, 0.1, 0.1], MAT_LIT);
                for s in [-1.0f32, 1.0] {
                    b.push(base + Vec3::new(-fz_ * 0.5 * s, 0.98, fx_ * 0.5 * s), Vec3::new(0.3, 0.025, 0.025), face.sin().atan2(face.cos()), [0.4, 0.3, 0.2], MAT_LIT);
                }
            } else {
                // the child still rolling the next ball
                let gb = garb(self.town.id ^ k, Some([0.7, 0.3, 0.3]), None).hat(Hat::Cap, [0.2, 0.4, 0.7]).kid();
                figure(b, Vec3::new(base.x + 0.9, y0, base.z), PI * 0.5, 0.0, &gb, Pose::Kneel, Prop::None, time, 1.0);
            }
        }
        if !self.snowball_fight_on(env) {
            return;
        }
        // two sides facing across the green
        let sides = [(-1.0f32, 0u32), (1.0, 3u32)];
        let hats = [[0.85, 0.2, 0.2], [0.2, 0.4, 0.85]];
        let kids: Vec<(Vec2, f32, usize, u32)> = sides
            .iter()
            .enumerate()
            .flat_map(|(s, (dir, base))| {
                (0..3u32).map(move |j| {
                    let sway = (time * 0.5 + (base + j) as f32).sin() * 0.6;
                    let p = g + Vec2::new(dir * 10.6, (j as f32 - 1.0) * 2.8 + sway);
                    (p, -dir * PI * 0.5, s, base + j)
                })
            })
            .collect();
        for &(p, yaw, s, id) in &kids {
            let gb = garb(self.town.id ^ id.wrapping_mul(131), None, None).hat(Hat::Cap, hats[s]).kid();
            let throwing = ((time * 0.45 + id as f32 * 0.37) % 1.0) < 0.35;
            let pose = if throwing { Pose::Reach } else { Pose::Stand };
            let dodge = if throwing { 0.0 } else { (time * 3.0 + id as f32).sin() * 0.1 };
            figure(b, at(p + Vec2::new(0.0, dodge)), yaw, time * 3.0 + id as f32, &gb, pose, Prop::None, time, 1.0);
        }
        // snowballs in the air: each kid lobs one at somebody on the other side every couple of seconds
        for &(p, _, s, id) in &kids {
            let period = 2.3 + (id % 3) as f32 * 0.4;
            let t = (time + id as f32 * 0.7) / period;
            let f = t.fract();
            let round = t.floor() as u32;
            let foes: Vec<&(Vec2, f32, usize, u32)> = kids.iter().filter(|k| k.2 != s).collect();
            let foe = foes[(hash_u32(id ^ round.wrapping_mul(0x9e37)) as usize) % foes.len()];
            let flight = 0.55;
            if f < flight {
                let u = f / flight;
                let q = p.lerp(foe.0, u);
                b.push(at(q) + Vec3::new(0.0, 1.0 + (u * PI).sin() * 2.2, 0.0), Vec3::splat(0.1), 0.0, [0.98, 0.99, 1.0], MAT_LIT);
            } else if f < flight + 0.12 {
                // the splat
                let u = (f - flight) / 0.12;
                for j in 0..5 {
                    let a = j as f32 * TAU / 5.0;
                    fx.push_alpha(at(foe.0) + Vec3::new(a.cos() * u * 0.5, 0.9 + u * 0.3, a.sin() * u * 0.5), Vec3::splat(0.07), 0.0, [1.0, 1.0, 1.0], MAT_UNLIT, 1.0 - u);
                }
            }
        }
    }
}
