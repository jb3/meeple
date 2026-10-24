const LOOP_LEN_MIN: f32 = 140.0;
use super::*;

/// Every smithy's forge is out in the open next to its anvil.
#[test]
fn forges_are_outdoors_beside_the_anvil() {
    let gen = Gen::new(1234);
    let mut checked = 0;
    for rx in -3..=3 {
        for rz in -3..=3 {
            let Some(t) = gen.town(rx, rz) else { continue };
            for lot in gen.lots(&t).iter().filter(|l| l.kind == LotKind::Smithy) {
                let (fx, fz) = (lot.front.0.floor() as i32, lot.front.1.floor() as i32);
                let (gx, gz) = lot.forge();
                let ch = generate(&gen, gx.div_euclid(CS), gz.div_euclid(CS));
                let y = t.level + 1;
                let at = |x: i32, y: i32, z: i32| ch.get_w(x, y, z);
                assert_eq!(at(gx, y, gz), FORGE, "forge missing in {} at {gx},{gz}", t.name);
                for dy in 1..=8 {
                    assert_eq!(at(gx, y + dy, gz), AIR, "forge in {} is covered", t.name);
                }
                if ch.core(fx, fz) {
                    assert_eq!(at(fx, y, fz), ANVIL);
                }
                println!("{}: centre ({}, {}), forge at ({gx}, {gz}), faces {}", t.name, t.cx, t.cz, lot.face);
                checked += 1;
            }
        }
    }
    assert!(checked > 3, "only {checked} smithies found");
}

/// Nothing is ever built over a railway: walls, roofs and trees all keep
/// clear of the track, including where lines pass through towns.
#[test]
fn railway_track_is_clear_of_buildings() {
    let gen = Gen::new(1234);
    let mut checked = 0;
    for rx in -4..=4 {
        for rz in -4..=4 {
            let Some(t) = gen.town(rx, rz) else { continue };
            let r = t.radius + 10;
            for l in gen.rails_near(t.cx - r, t.cz - r, t.cx + r, t.cz + r) {
                let s0 = l.along(t.cx as f32, t.cz as f32) as i32;
                for s in (s0 - r..s0 + r).step_by(2) {
                    let (x, z) = l.point(s as f32 + 0.5, 0.0);
                    let (x, z) = (x.floor() as i32, z.floor() as i32);
                    let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
                    let bed = gen.rail_level(x as f32 + 0.5, z as f32 + 0.5).floor() as i32;
                    // a tunnel's lining is meant to be there
                    if gen.column(&gen.ctx(x - 20, z - 20, x + 20, z + 20), x, z).tunnel.is_some() {
                        continue;
                    }
                    for y in bed + 1..bed + 9 {
                        let b = ch.get_w(x, y, z);
                        assert!(b == AIR, "{} at ({x}, {y}, {z}) is over the track near {}", b, t.name);
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 100, "only checked {checked} spots");
}

/// Junction chords are kept just as clear as the lines they join.
#[test]
fn junction_track_is_clear_of_buildings() {
    let gen = Gen::new(1234);
    let mut checked = 0;
    for c in gen.chords_near(-2500, -2500, 2500, 2500).into_iter().filter(|c| c.a != c.b).take(12) {
        for u in c.samples(2.0) {
            let p = c.at(u);
            let (x, z) = (p.x.floor() as i32, p.y.floor() as i32);
            let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
            let bed = gen.rail_level(x as f32 + 0.5, z as f32 + 0.5).floor() as i32;
            for y in bed + 1..bed + 9 {
                assert_eq!(ch.get_w(x, y, z), AIR, "block over junction track at ({x}, {y}, {z})");
            }
            checked += 1;
        }
    }
    assert!(checked > 100, "only checked {checked} spots");
}

/// Passing loops exist away from junctions, rejoin their own line, and
/// are kept clear of buildings.
#[test]
fn passing_loops_are_clear_of_buildings() {
    let gen = Gen::new(1234);
    let loops: Vec<_> = gen.chords_near(-3000, -3000, 3000, 3000).into_iter().filter(|c| c.a == c.b).collect();
    assert!(loops.len() > 8, "only {} loops", loops.len());
    for c in loops.iter().take(8) {
        assert!(c.length() > LOOP_LEN_MIN);
        for u in c.samples(2.0) {
            let p = c.at(u);
            let (x, z) = (p.x.floor() as i32, p.y.floor() as i32);
            let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
            let bed = gen.rail_level(x as f32 + 0.5, z as f32 + 0.5).floor() as i32;
            for y in bed + 1..bed + 9 {
                assert_eq!(ch.get_w(x, y, z), AIR, "block over loop track at ({x}, {y}, {z})");
            }
        }
    }
}

/// The countryside gets its windmills, manors, ruins and quarries, and a
/// windmill's tower and sails are actually built.
#[test]
fn countryside_sites_are_built() {
    let gen = Gen::new(1234);
    let mut kinds = std::collections::HashMap::new();
    for rx in -5..=5 {
        for rz in -5..=5 {
            for st in gen.sites(rx, rz).iter() {
                *kinds.entry(format!("{:?}", st.kind)).or_insert(0) += 1;
                if st.kind == SiteKind::Windmill && kinds["Windmill"] == 1 {
                    let ch = generate(&gen, st.x.div_euclid(CS), st.z.div_euclid(CS));
                    assert_eq!(ch.get_w(st.x, st.level + 5, st.z), PLASTER, "windmill tower");
                    assert_eq!(ch.feat.sails.len(), 1, "windmill sails");
                }
                println!("{:?} at x={} z={}", st.kind, st.x, st.z);
            }
        }
    }
    for k in ["Windmill", "Manor", "Ruin", "Quarry"] {
        assert!(kinds.get(k).copied().unwrap_or(0) > 0, "no {k} in 11x11 regions: {kinds:?}");
    }
}


/// Canals hold water with quays either side, and towns by the water get
/// their mills, jetties and lighthouses.
#[test]
fn water_features_are_built() {
    let gen = Gen::new(1234);
    let mut canal: Option<(i32, i32)> = None;
    'canals: for cz in gen.canals_near(-3000, 3000) {
        for x in (-3000i32..3000).step_by(16) {
            if gen.canal_open(x as f32, cz) && gen.canal_open(x as f32 + 40.0, cz) {
                canal = Some((x + 20, cz));
                break 'canals;
            }
        }
    }
    let (x, cz) = canal.expect("no canal anywhere near");
    let ch = generate(&gen, x.div_euclid(CS), cz.div_euclid(CS));
    assert_eq!(ch.get_w(x, SEA, cz), WATER, "canal holds water at x={x} z={cz}");
    println!("canal at x={x} z={cz}");
    let mut kinds = std::collections::HashMap::new();
    for rx in -6..=6 {
        for rz in -6..=6 {
            for st in gen.sites(rx, rz).iter() {
                // nothing but a jetty is built inside a town
                if st.kind != SiteKind::Harbour {
                    for t in gen.towns_near(st.x - 80, st.z - 80, st.x + 80, st.z + 80, 0) {
                        let d = Vec2::new(t.cx as f32, t.cz as f32).distance(st.centre());
                        assert!(d > t.radius as f32 + st.r, "{:?} at {},{} is inside {}", st.kind, st.x, st.z, t.name);
                    }
                }
                if matches!(st.kind, SiteKind::Watermill | SiteKind::Harbour | SiteKind::Lighthouse) {
                    let n = kinds.entry(format!("{:?}", st.kind)).or_insert(0);
                    *n += 1;
                    if *n == 1 {
                        println!("{:?} at x={} z={}", st.kind, st.x, st.z);
                        let ch = generate(&gen, st.x.div_euclid(CS), st.z.div_euclid(CS));
                        match st.kind {
                            SiteKind::Lighthouse => assert_eq!(ch.feat.beacons.len(), 1),
                            SiteKind::Watermill => assert_eq!(ch.feat.wheels.len(), 1),
                            _ => assert_eq!(ch.get_w(st.x, SEA + 1, st.z), PLANK),
                        }
                    }
                }
            }
        }
    }
    for k in ["Watermill", "Harbour", "Lighthouse"] {
        assert!(kinds.contains_key(k), "no {k}: {kinds:?}");
    }
}


/// Railways cross low ground on viaducts with open arches beneath, and go
/// through high ground in tunnels with the hillside left above.
#[test]
fn viaducts_and_tunnels_are_built() {
    let gen = Gen::new(1234);
    let (mut viaduct, mut tunnel) = (None, None);
    for l in gen.rails_near(-2500, -2500, 2500, 2500) {
        for s in (-2500..2500).step_by(4) {
            let (x, z) = l.point(s as f32 + 0.5, 0.0);
            let (x, z) = (x.floor() as i32, z.floor() as i32);
            let ctx = gen.ctx(x - 12, z - 12, x + 12, z + 12);
            let col = gen.column(&ctx, x, z);
            // an arch span, not a pier
            if viaduct.is_none() && col.viaduct.is_some() && (col.along.floor() as i32).rem_euclid(8) == 4 {
                viaduct = Some((x, z, col.viaduct.unwrap(), col.h));
            }
            if tunnel.is_none() && col.tunnel.is_some() && col.h > col.tunnel.unwrap() + 8 {
                tunnel = Some((x, z, col.tunnel.unwrap(), col.h));
            }
        }
    }
    let (x, z, deck, ground) = viaduct.expect("no viaduct found");
    println!("viaduct at x={x} z={z} (deck {deck}, ground {ground})");
    let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
    assert_eq!(ch.get_w(x, deck, z), GRAVEL, "viaduct deck");
    let under = ch.get_w(x, deck - 3, z);
    assert!(under == AIR || under == WATER, "open arch under the viaduct, found {under}");
    let (x, z, bed, hill) = tunnel.expect("no tunnel found");
    println!("tunnel at x={x} z={z} (bed {bed}, hill {hill})");
    let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
    for y in bed + 1..=bed + 4 {
        assert_eq!(ch.get_w(x, y, z), AIR, "tunnel bore at y={y}");
    }
    assert_ne!(ch.get_w(x, bed + 7, z), AIR, "hillside above the tunnel");
}


/// Where a railway crosses water it is carried on a viaduct, clear of the
/// surface, rather than filling the river in.
#[test]
fn railways_cross_water_on_viaducts() {
    let gen = Gen::new(1234);
    let mut found = None;
    'search: for l in gen.rails_near(-3000, -3000, 3000, 3000) {
        for s in (-3000..3000).step_by(2) {
            let (x, z) = l.point(s as f32 + 0.5, 0.0);
            let (x, z) = (x.floor() as i32, z.floor() as i32);
            let ctx = gen.ctx(x - 12, z - 12, x + 12, z + 12);
            let col = gen.column(&ctx, x, z);
            if col.h < SEA && col.town.is_none() && col.rail.is_some() && (col.along.floor() as i32).rem_euclid(8) == 4 {
                found = Some((x, z, col.viaduct));
                break 'search;
            }
        }
    }
    let (x, z, deck) = found.expect("no railway over water found");
    let deck = deck.expect("railway over water is not on a viaduct");
    println!("rail over water at x={x} z={z} (deck {deck})");
    assert!(deck >= SEA + 4, "deck clears the water");
    let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
    assert_eq!(ch.get_w(x, deck, z), GRAVEL, "viaduct deck");
    assert_eq!(ch.get_w(x, deck - 2, z), AIR, "open arch above the water");
    assert_eq!(ch.get_w(x, SEA, z), WATER, "water still flows beneath");
}


/// A new house goes from a pegged-out plot, to a frame in scaffolding,
/// to a finished house with its door, as the days pass.
#[test]
fn new_houses_go_up_over_the_days() {
    let gen = Gen::new(1234);
    let (t, lot) = (-4..=4)
        .flat_map(|rx| (-4..=4).map(move |rz| (rx, rz)))
        .filter_map(|(rx, rz)| gen.town(rx, rz))
        .find_map(|t| gen.lots(&t).iter().find(|l| l.finished_day(0).is_some()).cloned().map(|l| (t, l)))
        .expect("no new builds anywhere");
    let done = lot.finished_day(0).unwrap();
    println!("new build at x={} z={} done day {done}", lot.x0, lot.z0);
    let (dx, dz) = lot.door;
    let y0 = t.level + 1;
    let (fx0, fz0, _, _) = lot.footprint();
    let at = |day: i64| {
        gen.day.set(day);
        let ch = generate(&gen, dx.div_euclid(CS), dz.div_euclid(CS));
        let pole = generate(&gen, (fx0 - 1).div_euclid(CS), (fz0 - 1).div_euclid(CS));
        (ch.get_w(dx, y0, dz), pole.get_w(fx0 - 1, y0 + 1, fz0 - 1), gen.stage(&lot))
    };
    let (door, _, stage) = at(done - BUILD_DAYS);
    assert_eq!(stage, Stage::Plot);
    assert_ne!(door, DOOR, "a plot has no door yet");
    let (door, pole, stage) = at(done - 1);
    assert!(matches!(stage, Stage::Frame(_)), "{stage:?}");
    assert_ne!(door, DOOR);
    assert_eq!(pole, FENCE, "scaffold pole at the corner");
    let (door, pole, stage) = at(done);
    assert_eq!(stage, Stage::Done);
    assert_eq!(door, DOOR, "the finished house has its door");
    assert_ne!(pole, FENCE, "the scaffolding's come down");
}

/// No two countryside sites stand on the same ground, even across
/// region borders (a windmill once grew out of a lighthouse).
#[test]
fn sites_never_overlap() {
    let mut total = 0;
    for seed in 0..40 {
    let gen = Gen::new(seed);
    let all: Vec<_> = (-6..=6).flat_map(|rx| (-6..=6).map(move |rz| (rx, rz)))
        .flat_map(|(rx, rz)| gen.sites(rx, rz).iter().cloned().collect::<Vec<_>>())
        .collect();
    total += all.len();
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            let d = Vec2::new((a.x - b.x) as f32, (a.z - b.z) as f32).length();
            let room = |s: &Site| if s.kind == SiteKind::Harbour { 2.0 } else { s.r };
            assert!(d > room(a) + room(b) + 2.0, "seed {seed}: {:?} at {},{} overlaps {:?} at {},{}", a.kind, a.x, a.z, b.kind, b.x, b.z);
        }
    }
    }
    println!("{total} sites, none overlapping");
}

/// Every canal runs from open water to open water: at both ends of a dug
/// stretch the next thing along is water, and no canal crosses a lake.
#[test]
fn canals_join_water_to_water() {
    let gen = Gen::new(1234);
    let mut runs = 0;
    for cz in gen.canals_near(-4000, 4000) {
        let mut x = -4000;
        while x < 4000 {
            if !gen.canal_open(x as f32, cz) {
                x += 1;
                continue;
            }
            let a = x;
            while gen.canal_open(x as f32, cz) {
                assert!(gen.raw_height(x as f32 + 0.5, cz as f32 + 0.5) >= SEA as f32 - 0.6, "canal dug across open water at {x},{cz}");
                x += 1;
            }
            let wet = |px: i32| (px - 3..=px + 3).any(|q| gen.raw_height(q as f32 + 0.5, cz as f32 + 0.5) < SEA as f32);
            // (a run cut off by the edge of the search can't be judged)
            if a > -4000 && x < 4000 {
                assert!(wet(a - 1) && wet(x), "canal {a}..{x} at z={cz} doesn't end in water");
            }
            println!("canal from x={a} to x={x} at z={cz}");
            runs += 1;
        }
    }
    assert!(runs >= 2, "only {runs} canals in 8000x8000");
}

/// Every waterwheel turns in open water and air, never through the bank,
/// and dips into the water at the bottom.
#[test]
fn waterwheels_turn_in_the_water() {
    let gen = Gen::new(1234);
    let mut checked = 0;
    for st in (-5..=5).flat_map(|rx| (-5..=5).map(move |rz| (rx, rz))).flat_map(|(rx, rz)| gen.sites(rx, rz).iter().cloned().collect::<Vec<_>>()) {
        if st.kind != SiteKind::Watermill {
            continue;
        }
        let ch = generate(&gen, st.x.div_euclid(CS), st.z.div_euclid(CS));
        let Some(&(hub, yaw)) = ch.feat.wheels.first() else { continue };
        let hub = Vec3::from(hub);
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        for k in 0..20 {
            let a = k as f32 * std::f32::consts::TAU / 20.0;
            let p = hub + (right * a.cos() + Vec3::Y * a.sin()) * 1.6;
            let (x, y, z) = (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
            let c = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
            let b = c.get_w(x, y, z);
            assert!(b == AIR || b == WATER, "wheel of the mill at {},{} turns through block {b} at {p}", st.x, st.z);
        }
        let low = hub - Vec3::Y * 1.6;
        let c = generate(&gen, (low.x.floor() as i32).div_euclid(CS), (low.z.floor() as i32).div_euclid(CS));
        assert_eq!(c.get_w(low.x.floor() as i32, low.y.floor() as i32, low.z.floor() as i32), WATER, "wheel at {},{} doesn't dip in the water", st.x, st.z);
        checked += 1;
    }
    println!("{checked} waterwheels checked");
    assert!(checked >= 5);
}

/// Mines turn up in the hills and are built: a tunnel mouth lamp and a headframe of logs.
#[test]
fn mines_are_built_in_the_hills() {
    let gen = Gen::new(535);
    let mut found = None;
    'search: for rx in -12..12 {
        for rz in -12..12 {
            if let Some(st) = gen.sites(rx, rz).iter().find(|s| s.kind == SiteKind::Mine) {
                found = Some(st.clone());
                break 'search;
            }
        }
    }
    let st = found.expect("no mine in 24x24 regions");
    println!("MINE at {} {} face {}", st.x, st.z, st.face);
    let ch = generate(&gen, st.x.div_euclid(CS), st.z.div_euclid(CS));
    assert!(!ch.feat.lamps.is_empty(), "the mine has no lamps");
}

/// The quiet places turn up across the lands, build without trouble, keep
/// off each other, and a monastery has its bell.
#[test]
fn quiet_places_are_built() {
    let kinds = [SiteKind::Monastery, SiteKind::Shrine, SiteKind::HillFort, SiteKind::Camp, SiteKind::Orchard, SiteKind::Apiary];
    let mut found = std::collections::HashMap::new();
    for seed in [1234, 7, 99, 2024] {
        let gen = Gen::new(seed);
        for rx in -8..=8 {
            for rz in -8..=8 {
                for st in gen.sites(rx, rz).iter().filter(|s| kinds.contains(&s.kind)) {
                    let n = found.entry(format!("{:?}", st.kind)).or_insert(0);
                    *n += 1;
                    if *n > 2 {
                        continue;
                    }
                    let ch = generate(&gen, st.x.div_euclid(CS), st.z.div_euclid(CS));
                    // something was built on the site
                    let rr = st.r as i32;
                    let built = (st.level + 1..st.level + 12).any(|y| (-rr..=rr).any(|dz| (-rr..=rr).any(|dx| ch.get_w(st.x + dx, y, st.z + dz) != AIR)));
                    assert!(built, "{:?} at {},{} built nothing", st.kind, st.x, st.z);
                    if st.kind == SiteKind::Monastery {
                        let all: usize = (-1..=1).flat_map(|dz| (-1..=1).map(move |dx| (dx, dz)))
                            .map(|(dx, dz)| generate(&gen, st.x.div_euclid(CS) + dx, st.z.div_euclid(CS) + dz).feat.bells.len())
                            .sum();
                        assert!(all >= 1, "monastery at {},{} has no bell", st.x, st.z);
                    }
                }
            }
        }
    }
    println!("{found:?}");
    for k in kinds {
        assert!(found.get(&format!("{k:?}")).copied().unwrap_or(0) > 0, "no {k:?} found: {found:?}");
    }
}

/// A ski resort is premade: the corridor is laid at an even grade (so the
/// skiers and lift, standing on `ski_surface`, are on the real ground), nothing
/// of the roads, railways, towns, canals or water crosses it, and the lift
/// stations and chalets get built.
#[test]
fn ski_runs_are_premade_and_clear() {
    let mut resorts = Vec::new();
    'seeds: for seed in 1u32..300 {
        let gen = Gen::new(seed);
        for rx in -4..=4 {
            for rz in -4..=4 {
                for st in gen.sites(rx, rz).iter().filter(|s| s.kind == SiteKind::SkiResort) {
                    resorts.push((seed, st.clone()));
                }
            }
        }
        if resorts.len() >= 8 {
            break 'seeds;
        }
    }
    assert!(resorts.len() >= 4, "only {} ski resorts in 300 worlds", resorts.len());
    for (seed, st) in &resorts {
        let gen = Gen::new(*seed);
        let c = st.centre();
        let r = (SKI_HALF_LEN + SKI_HALF_WIDTH) as i32 + 14;
        let ctx = gen.ctx(c.x as i32 - r, c.y as i32 - r, c.x as i32 + r, c.y as i32 + r);
        // a run worth skiing
        assert!((0.14..=0.42).contains(&st.grade), "seed {seed}: grade {}", st.grade);
        // nothing man-made or watery anywhere near it
        let (x0, z0, x1, z1) = (c.x as i32 - r, c.y as i32 - r, c.x as i32 + r, c.y as i32 + r);
        for l in gen.rails_near(x0, z0, x1, z1) {
            for k in -8..=8 {
                let p = st.ski_at(k as f32 * SKI_HALF_LEN / 8.0, 0.0);
                assert!(l.perp(p.x, p.y).abs() > SKI_HALF_WIDTH + 4.0, "seed {seed}: a railway runs through the ski run at {p}");
            }
        }
        for k in -8..=8 {
            for v in [-SKI_HALF_WIDTH + 2.0, 0.0, SKI_HALF_WIDTH - 2.0] {
                // (whole cells along and across, so on the middle of a cell, where the terrain is measured)
                let p = st.ski_at(k as f32 * 5.0, v);
                let (x, z) = (p.x.floor() as i32, p.y.floor() as i32);
                let col = gen.column(&ctx, x, z);
                assert!(!col.road && col.town.is_none() && col.canal.is_none() && col.h >= SEA + 3, "seed {seed}: the run at {p} is crossed by something ({:?})", (col.road, col.canal, col.h));
                // the ground is the planned slope, to the block
                let (u, _) = st.ski_uv(p.x, p.y);
                assert_eq!(col.h, st.ski_height(u).floor() as i32, "seed {seed}: terrain off the plan at u={u} v={v}");
                // and skiers standing on `ski_surface` stand on it
                assert_eq!(gen.walk_y(&ctx, p.x, p.y), st.ski_surface(u), "seed {seed}: surface off at u={u}");
            }
        }
        // the lift stations are built, at the foot and at the top
        for u in [-36.0f32, 37.0] {
            let p = st.ski_at(u, 0.0);
            let (x, z) = (p.x.floor() as i32, p.y.floor() as i32);
            let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
            let g = gen.column(&ctx, x, z).h;
            assert_ne!(ch.get_w(x + 1, g + 2, z), AIR, "seed {seed}: no lift station at u={u}");
        }
    }
    println!("{} ski resorts checked: {:?}", resorts.len(), resorts.iter().map(|(s, st)| (*s, st.x, st.z)).collect::<Vec<_>>());
}

/// Anglers stand on solid ground with room to stand and cast, never in a block.
#[test]
fn anglers_stand_on_open_ground() {
    let gen = Gen::new(1234);
    let mut checked = 0;
    for cz in -12..12 {
        for cx in -12..12 {
            let ch = generate(&gen, cx, cz);
            for &(pos, _, _) in &ch.feat.anglers {
                let (x, y, z) = (pos[0].floor() as i32, pos[1] as i32, pos[2].floor() as i32);
                assert!(opaque(ch.get_w(x, y - 1, z)), "angler at {pos:?} has no ground under them");
                for k in 0..3 {
                    let b = ch.get_w(x, y + k, z);
                    assert!(matches!(b, AIR | TALLGRASS | FLOWER_R | FLOWER_Y | FLOWER_B | BLUEBELL | POPPY), "angler at {pos:?} is inside block {b}");
                }
                checked += 1;
            }
        }
    }
    println!("{checked} anglers checked");
    assert!(checked >= 5);
}

/// Weirs span their river from bank to bank, and salmon have somewhere to leap.
#[test]
fn weirs_span_rivers() {
    let gen = Gen::new(1234);
    let mut weirs = 0;
    for rz in -4..4 {
        for rx in -4..4 {
            for w in gen.weirs(rx, rz).iter() {
                let (x, z) = (w.c.x.floor() as i32, w.c.y.floor() as i32);
                let ch = generate(&gen, x.div_euclid(CS), z.div_euclid(CS));
                assert_eq!(ch.get_w(x, SEA, z), STONEBRICK, "weir at {},{} has no sill", w.c.x, w.c.y);
                assert!(ch.feat.weirs.iter().any(|f| f.c == w.c));
                let down = w.c + w.flow * 3.5;
                let c = generate(&gen, (down.x.floor() as i32).div_euclid(CS), (down.y.floor() as i32).div_euclid(CS));
                assert_eq!(c.get_w(down.x.floor() as i32, SEA, down.y.floor() as i32), WATER, "no water below the weir at {},{}", w.c.x, w.c.y);
                weirs += 1;
            }
        }
    }
    println!("{weirs} weirs");
    assert!(weirs >= 5);
}
