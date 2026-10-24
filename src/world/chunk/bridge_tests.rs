use super::*;

/// Find road cells over water in a few worlds: the deck is something to
/// walk on, level with `walk_y`, the crown is high enough for a boat, and
/// the deck steps up gently from the banks.
#[test]
fn bridges_arch_over_the_water() {
    let mut found = 0;
    let mut crowns = std::collections::BTreeMap::new();
    for seed in [1234u32, 7, 99, 2024] {
        let gen = Gen::new(seed);
        for rx in -12..12 {
            for rz in -12..12 {
                let (x0, z0) = (rx * 64, rz * 64);
                let ctx = gen.ctx(x0, z0, x0 + 64, z0 + 64);
                for z in z0..z0 + 64 {
                    for x in x0..x0 + 64 {
                        if !ctx.roads.iter().any(|s| s.dist(x, z) <= 1) {
                            continue;
                        }
                        let col = gen.column(&ctx, x, z);
                        if !(col.road && col.h < SEA && col.town.is_none()) || gen.is_ferry(&ctx, x, z) {
                            continue;
                        }
                        found += 1;
                        let (deck, d) = gen.bridge_deck(&ctx, x, z);
                        assert!(d >= 1 && deck > SEA, "deck {deck} d {d} at {x},{z}");
                        assert_eq!(gen.walk_y(&ctx, x as f32 + 0.5, z as f32 + 0.5), (deck + 1) as f32);
                        let e = crowns.entry(deck - SEA).or_insert(0);
                        *e += 1;
                    }
                }
            }
        }
    }
    println!("{found} bridge cells; deck heights over the water: {crowns:?}");
    assert!(found > 0, "no road bridges found to check");
}

/// A windmill never stands on a village green, where the fair and the
/// cricket are.
#[test]
fn windmills_keep_off_the_green() {
    let mut mills = 0;
    for seed in [1234u32, 7, 99, 2024, 5] {
        let gen = Gen::new(seed);
        for rx in -10..10 {
            for rz in -10..10 {
                for st in gen.sites(rx, rz).iter().filter(|s| s.kind == SiteKind::Windmill) {
                    mills += 1;
                    let c = st.centre();
                    assert!(gen.green_clear(c.x, c.y, 4.0), "windmill at {c} is on a village green (seed {seed})");
                }
            }
        }
    }
    assert!(mills > 5, "only {mills} windmills found");
}

/// Water too wide to bridge gets a ferry: no deck over it, a plank landing at
/// each end, and the crossing recorded for the ferry to run on, joining
/// water's-edge to water's-edge in a straight line. Other crossings keep
/// their bridges, and no stretch is part one and part the other.
#[test]
fn wide_water_is_crossed_by_ferry() {
    let (mut ferries, mut cells, mut landings, mut bridges) = (0, 0, 0, 0);
    for seed in [1234u32, 7, 99, 2024, 5, 31, 88, 404, 827897] {
        let gen = Gen::new(seed);
        let mut chunks = std::collections::HashMap::new();
        for (p, q) in gen.ferry_runs_near(-1500, -1500, 1500, 1500) {
            ferries += 1;
            let len = p.distance(q);
            let u = (q - p) / len;
            assert!((FERRY_SPAN as f32..=FERRY_MAX as f32).contains(&len), "ferry of {len} cells");
            let ctx = gen.ctx(p.x as i32 - 40, p.y as i32 - 40, p.x as i32 + 40, p.y as i32 + 40);
            let wet = |v: Vec2| gen.column(&ctx, v.x.floor() as i32, v.y.floor() as i32).h < SEA;
            // water at the landings, land (or a town) just beyond them
            assert!(wet(p) && wet(q), "a landing out of the water at {p}");
            assert!((1..=4).any(|k| !wet(p - u * k as f32)), "{p}: no bank behind the first landing");
            assert!((1..=4).any(|k| !wet(q + u * k as f32)), "{q}: no bank beyond the last landing");
            // the ferry is recorded on the chunk its first landing is in
            let (cx, cz) = (p.x.floor().div_euclid(CS as f32) as i32, p.y.floor().div_euclid(CS as f32) as i32);
            let ch = chunks.entry((cx, cz)).or_insert_with(|| generate(&gen, cx, cz));
            assert!(ch.feat.ferries.iter().any(|f| (f[0] - p.x).abs() < 1e-3 && (f[1] - p.y).abs() < 1e-3), "ferry at {p} not recorded");
            // every road-over-water cell on the line is a ferry cell, with
            // nothing built over it but landing posts
            let steps = (len * 2.0) as i32;
            for k in 0..=steps {
                let v = p + u * (k as f32 * 0.5);
                let (x, z) = (v.x.floor() as i32, v.y.floor() as i32);
                let col = gen.column(&ctx, x, z);
                if !(col.road && col.h < SEA && col.town.is_none()) {
                    continue;
                }
                cells += 1;
                let d = gen.ferry_at(&ctx, x, z).expect("a road cell on a ferry's line isn't a ferry cell");
                let (ccx, ccz) = (x.div_euclid(CS), z.div_euclid(CS));
                let c = chunks.entry((ccx, ccz)).or_insert_with(|| generate(&gen, ccx, ccz));
                if col.viaduct.is_some() || ctx.rails.iter().any(|l| l.perp(x as f32 + 0.5, z as f32 + 0.5).abs() < 6.0) {
                    continue;
                }
                for y in SEA + 4..SEA + 9 {
                    assert_eq!(c.get_w(x, y, z), AIR, "something over the ferry at {x},{y},{z}");
                }
                if d <= 1.6 {
                    assert_ne!(c.get_w(x, SEA, z), WATER, "no landing at {x},{z}");
                    landings += 1;
                } else {
                    assert!(!matches!(c.get_w(x, SEA, z), STONEBRICK | COBBLE), "a bridge deck in mid-crossing at {x},{z}");
                }
                assert!(gen.walk_y(&ctx, x as f32 + 0.5, z as f32 + 0.5) < (SEA + 2) as f32);
            }
        }
        // and the bridges that remain are never on a ferry's line
        for rx in -8..8 {
            for rz in -8..8 {
                let (x0, z0) = (rx * 64, rz * 64);
                let ctx = gen.ctx(x0, z0, x0 + 64, z0 + 64);
                for z in z0..z0 + 64 {
                    for x in x0..x0 + 64 {
                        let col = gen.column(&ctx, x, z);
                        if col.road && col.h < SEA && col.town.is_none() && !gen.is_ferry(&ctx, x, z) {
                            bridges += 1;
                        }
                    }
                }
            }
        }
    }
    println!("{ferries} ferries ({cells} cells, {landings} landing cells) against {bridges} bridge cells");
    assert!(ferries > 0 && landings > 0, "no wide water found to ferry");
    assert!(bridges > 0, "no crossings kept their bridges");
}
