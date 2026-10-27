use super::*;

/// Not a check: prints the most varied seeds to start a gallery from.
/// Run with `cargo test rank_seeds -- --ignored --nocapture`.
#[test]
#[ignore]
fn rank_seeds() {
    let mut scored: Vec<(f32, u32)> = Vec::new();
    for seed in 1..=600u32 {
        let gen = Gen::new(seed);
        let r = 800;
        let sites = gen.sites_near(-r, -r, r, r);
        let kinds: std::collections::HashSet<String> = sites.iter().map(|s| format!("{:?}", s.kind)).collect();
        let towns = gen.towns_near(-r, -r, r, r, 0);
        let cities = towns.iter().filter(|t| t.city).count();
        let stations = towns.iter().filter(|t| t.rail.is_some()).count();
        let crossings = gen.crossings_near(-r, -r, r, r).len();
        let loops = gen.chords_near(-r, -r, r, r).iter().filter(|c| c.a == c.b).count();
        let canals = gen.canals_near(-r, r).len();
        let score = kinds.len() as f32 * 3.0 + towns.len().min(10) as f32 + cities.min(3) as f32 * 2.0 + stations.min(4) as f32 + crossings.min(4) as f32 + loops.min(3) as f32 + canals.min(2) as f32 * 2.0;
        scored.push((score, seed));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (s, seed) in scored.iter().take(14) {
        println!("seed {seed}: {s}");
    }
}

/// Not a check: finds a place for each thing the feature list promises, for
/// docs/seeds.md. Prints `seed|feature|x|z|distance from the origin` lines.
/// Run with `cargo test find_feature_spots -- --ignored --nocapture`.
#[test]
#[ignore]
fn find_feature_spots() {
    use std::collections::BTreeMap;
    const R: i32 = 1100;
    let lo = std::env::var("HUNT_LO").ok().and_then(|s| s.parse().ok()).unwrap_or(1u32);
    let hi = std::env::var("HUNT_HI").ok().and_then(|s| s.parse().ok()).unwrap_or(120u32);
    let mut all: BTreeMap<u32, Vec<(String, f32, f32)>> = BTreeMap::new();
    for seed in lo..=hi {
        let gen = Gen::new(seed);
        let mut out: Vec<(String, f32, f32)> = Vec::new();
        // biomes, on dry land (and open water)
        let mut x = -R;
        while x <= R {
            let mut z = -R;
            while z <= R {
                let (fx, fz) = (x as f32, z as f32);
                let h = gen.raw_height(fx, fz);
                if h < SEA as f32 - 4.0 {
                    out.push(("biome:water".into(), fx, fz));
                } else if h > SEA as f32 + 2.0 {
                    out.push((format!("biome:{:?}", gen.biome(fx, fz, h)), fx, fz));
                }
                z += 48;
            }
            x += 48;
        }
        for t in gen.towns_near(-R, -R, R, R, 0) {
            let p = (t.cx as f32, t.cz as f32);
            out.push((format!("style:{:?}", t.style), p.0, p.1));
            out.push((if t.city { "town:city".into() } else { "town:village".into() }, p.0, p.1));
            if t.rail.is_some() {
                out.push(("town:station".into(), p.0, p.1));
            }
            if gen.green(&t).is_some() {
                out.push(("town:green".into(), p.0, p.1));
            }
        }
        for s in gen.sites_near(-R, -R, R, R) {
            let c = s.centre();
            out.push((format!("site:{:?}", s.kind), c.x, c.y));
        }
        for w in gen.weirs_near(-R, -R, R, R) {
            out.push(("weir".into(), w.c.x, w.c.y));
        }
        for (p, q) in gen.ferry_runs_near(-R, -R, R, R) {
            let m = (p + q) * 0.5;
            out.push(("ferry".into(), m.x, m.y));
        }
        for (_, _, p) in gen.crossings_near(-R, -R, R, R) {
            out.push(("rail:crossing".into(), p.x, p.y));
        }
        for c in gen.chords_near(-R, -R, R, R) {
            let m = c.at(c.length() * 0.5);
            out.push((if c.a == c.b { "rail:loop".into() } else { "rail:junction".into() }, m.x, m.y));
        }
        for cz in gen.canals_near(-R, R) {
            let mut x = -R;
            while x <= R {
                if gen.canal_open(x as f32, cz) {
                    out.push(("canal".into(), x as f32, cz as f32));
                }
                x += 24;
            }
        }
        all.insert(seed, out);
    }
    // the bridges, viaducts and tunnels need columns: look at the seeds with the most variety
    let mut ranked: Vec<(usize, u32)> = all.iter().map(|(s, v)| (v.iter().map(|f| f.0.clone()).collect::<std::collections::HashSet<_>>().len(), *s)).collect();
    ranked.sort_by(|a, b| b.cmp(a));
    for &(_, seed) in ranked.iter().take(20) {
        let gen = Gen::new(seed);
        let out = all.get_mut(&seed).unwrap();
        let mut seen = std::collections::HashSet::new();
        for rx in -9..9 {
            for rz in -9..9 {
                let (x0, z0) = (rx * 64, rz * 64);
                let ctx = gen.ctx(x0, z0, x0 + 64, z0 + 64);
                for z in z0..z0 + 64 {
                    for x in x0..x0 + 64 {
                        let near_road = ctx.roads.iter().any(|s| s.dist(x, z) <= 1);
                        let near_rail = ctx.rails.iter().any(|l| l.perp(x as f32, z as f32).abs() < 2.0);
                        if !near_road && !near_rail {
                            continue;
                        }
                        let col = gen.column(&ctx, x, z);
                        let name = if col.road && col.h < SEA && col.town.is_none() && !gen.is_ferry(&ctx, x, z) {
                            "bridge"
                        } else if col.viaduct.is_some() && col.h < SEA {
                            "rail:viaduct-over-water"
                        } else if col.viaduct.is_some() {
                            "rail:viaduct"
                        } else if col.tunnel.is_some() {
                            "rail:tunnel"
                        } else {
                            continue;
                        };
                        // one spot per 96-cell square is plenty
                        if seen.insert((name, x.div_euclid(96), z.div_euclid(96))) {
                            out.push((name.into(), x as f32, z as f32));
                        }
                    }
                }
            }
        }
    }
    for (seed, v) in &all {
        let mut best: BTreeMap<&str, (f32, f32, f32)> = BTreeMap::new();
        for (name, x, z) in v {
            let d = (x * x + z * z).sqrt();
            let e = best.entry(name.as_str()).or_insert((d, *x, *z));
            if d < e.0 {
                *e = (d, *x, *z);
            }
        }
        for (name, (d, x, z)) in best {
            println!("HUNT|{seed}|{name}|{}|{}|{}", x.round(), z.round(), d.round());
        }
    }
    // how many things a single view of 400 cells holds, for the showcase seed
    for (seed, v) in &all {
        let mut top = (0usize, 0.0f32, 0.0f32);
        for (_, cx, cz) in v.iter().filter(|f| f.0.starts_with("town:") || f.0.starts_with("site:")) {
            let names: std::collections::HashSet<&str> = v.iter().filter(|(_, x, z)| (x - cx).abs() < 350.0 && (z - cz).abs() < 350.0).map(|f| f.0.as_str()).collect();
            if names.len() > top.0 {
                top = (names.len(), *cx, *cz);
            }
        }
        println!("CLUSTER|{seed}|{}|{}|{}", top.0, top.1.round(), top.2.round());
    }
}
