use super::*;

impl Fauna {
    /// Give each chunk coming into view its animals: pasture herds, birds in
    /// the trees, boats at the piers, fish at the weirs. A group stays hidden
    /// until nothing of it is on screen, unless the world is just loading.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn spawn_groups(&mut self, world: &World, view: Vec2, view_r: f32, warmup: bool, on_screen: &dyn Fn(Vec3) -> bool) {
        let gen = &world.gen;
        // spawn groups for chunks entering view
        let mut spawn = Vec::new();
        for (key, f) in world.features_near(view.x, view.y, view_r + 40.0) {
            if self.groups.contains_key(key) {
                continue;
            }
            spawn.push((*key, f.pasture, f.water, f.piers.first().copied(), f.nature, f.treetops.clone(), f.weirs.first().copied()));
        }
        for (key, pasture, water, pier, nature, treetops, weir) in spawn {
            let h = hash2(gen.seed ^ 0xa11, key.0, key.1);
            let center = Vec2::new((key.0 * CS + CS / 2) as f32, (key.1 * CS + CS / 2) as f32);
            let ctx = gen.ctx(key.0 * CS - 24, key.1 * CS - 24, key.0 * CS + CS + 24, key.1 * CS + CS + 24);
            let mut g = None;
            let mut moor = None;
            if let Some(p) = pier {
                // fishing boats based at a harbour jetty
                moor = Some(Vec2::new(p[0], p[2]));
                g = Some((Kind::Boat, Vec2::new(p[0], p[2]), 2, 30.0));
            } else if let (Some(w), true) = (weir, (h >> 5) % 4 != 0) {
                // salmon gather below a weir, leaping it on their way upstream
                g = Some((Kind::Salmon, w.c + w.flow * WEIR_FOOT, 3 + (h >> 8) % 3, 0.0));
            } else if let Some(p) = pasture {
                let kind = if h % 3 == 0 { Kind::Cow } else { Kind::Sheep };
                let n = if kind == Kind::Cow { 2 + h % 3 } else { 3 + h % 5 };
                g = Some((kind, Vec2::new(p[0], p[2]), n, 8.0));
            } else if water > 0.55 && h % 3 == 0 {
                g = Some((Kind::Boat, center, 1, 22.0));
            } else if water > 0.3 && h % 5 == 1 {
                g = Some((Kind::Swan, center, 2, 12.0));
            } else if water > 0.05 && water < 0.7 && h % 5 == 2 {
                g = Some((Kind::Heron, center, 1, 0.0));
            } else if water > 0.05 && h % 4 == 0 {
                g = Some((Kind::Duck, center, 3 + h % 3, 10.0));
            } else if nature > 0.3 && gen.coldness(center.x, center.y) > 0.4 && h % 3 == 0 {
                g = Some((Kind::Wolf, center, 2 + (h >> 8) % 3, 22.0));
            } else if nature > 0.35 && h % 4 == 3 {
                g = Some(match (h >> 4) % 3 {
                    0 => (Kind::Deer, center, 2 + (h >> 8) % 2, 14.0),
                    1 => (Kind::Fox, center, 1, 18.0),
                    _ => (Kind::Rabbit, center, 3 + (h >> 8) % 3, 8.0),
                });
            } else if treetops.len() > 1 && h % 3 == 0 {
                let t = treetops[0];
                g = Some((Kind::Owl, Vec2::new(t[0], t[2]), 1, 0.0));
            }
            let perches = treetops.iter().map(|t| Vec3::new(t[0], t[1], t[2])).collect();
            let perch_low = treetops.iter().map(|t| t[3]).collect();
            let mut group = Group {
                kind: Kind::Sheep,
                home: center,
                roam: 8.0,
                members: Vec::new(),
                ctx,
                call_t: 5.0,
                hidden: false,
                moor,
                active: 0.0,
                perches,
                perch_low,
                on_perch: 0,
            };
            if let Some((kind, home, n, roam)) = g {
                group.kind = kind;
                group.roam = roam;
                group.home = home;
                if kind.on_water() && group.moor.is_none() && kind != Kind::Salmon {
                    // find open water inside this chunk (a heron wants the shallows by the bank)
                    let mut found = None;
                    for i in 0..48 {
                        let q = Vec2::new(
                            (key.0 * CS) as f32 + unit(hash_u32(h ^ i)) * CS as f32,
                            (key.1 * CS) as f32 + unit(hash_u32(h ^ i ^ 0x55)) * CS as f32,
                        );
                        let bank = || {
                            [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y].iter().any(|d| !Self::is_water(gen, &group.ctx, q + *d * 1.5))
                        };
                        if Self::is_water(gen, &group.ctx, q) && (kind != Kind::Heron || bank()) {
                            found = Some(q);
                            break;
                        }
                    }
                    match found {
                        Some(q) => group.home = q,
                        None => {
                            self.groups.insert(key, group);
                            continue;
                        }
                    }
                }
                for i in 0..n {
                    let a = unit(hash_u32(h ^ (i * 31))) * 6.28;
                    let spread = match kind {
                        Kind::Boat | Kind::Heron | Kind::Owl => 0.0,
                        _ => 1.5,
                    };
                    let mut p = group.home + Vec2::new(a.cos(), a.sin()) * spread;
                    if let (Kind::Salmon, Some(w)) = (kind, weir) {
                        // spread along the foot of the weir, each at its own spot
                        let u = unit(hash_u32(h ^ (i * 31) ^ 0x5a1)) * (w.a + w.b - 3.0).max(0.0) - w.a + 1.5;
                        p = w.c + w.flow * WEIR_FOOT + w.cross * u;
                    }
                    let tint = match kind {
                        Kind::Sheep => {
                            if i % 6 == 5 {
                                rgb(70, 65, 70)
                            } else {
                                rgb(240, 238, 230)
                            }
                        }
                        Kind::Cow => {
                            if i % 2 == 0 {
                                rgb(240, 240, 235)
                            } else {
                                rgb(150, 100, 60)
                            }
                        }
                        Kind::Duck => rgb(120, 90, 60),
                        Kind::Boat => [rgb(240, 235, 220), rgb(220, 90, 70), rgb(90, 140, 200)][(h % 3) as usize],
                        Kind::Swan => rgb(245, 245, 240),
                        Kind::Heron => rgb(150, 156, 170),
                        Kind::Deer => rgb(150, 100, 60),
                        Kind::Fox => rgb(214, 104, 40),
                        Kind::Rabbit => [rgb(140, 120, 100), rgb(110, 95, 80), rgb(190, 180, 170)][i as usize % 3],
                        Kind::Owl => rgb(130, 100, 70),
                        Kind::Salmon => [rgb(205, 95, 80), rgb(170, 180, 190)][i as usize % 2],
                        Kind::Wolf => [rgb(120, 122, 128), rgb(90, 92, 98), rgb(170, 170, 172)][i as usize % 3],
                    };
                    let y = match kind {
                        Kind::Duck | Kind::Boat | Kind::Swan | Kind::Salmon => SEA as f32 + 0.9,
                        Kind::Heron => SEA as f32 + 0.1,
                        Kind::Owl => group.perches[0].y,
                        _ => gen.walk_y(&group.ctx, p.x, p.y),
                    };
                    let (yaw, graze) = match (kind, weir) {
                        // head upstream, over the weir; `graze` is how often it tries
                        (Kind::Salmon, Some(w)) => (yaw_of(-w.flow), 3.5 + unit(hash_u32(h ^ i ^ 0x77)) * 3.0),
                        _ => (a, 0.0),
                    };
                    group.members.push(Animal {
                        pos: p,
                        y,
                        yaw,
                        target: p,
                        timer: unit(hash_u32(h ^ i)) * 4.0,
                        graze,
                        phase: a,
                        tint,
                        flee: 0.0,
                    });
                }
            }
            group.hidden = !warmup && group.members.iter().any(|a| on_screen(Vec3::new(a.pos.x, a.y, a.pos.y)));
            self.groups.insert(key, group);
        }
    }
}
