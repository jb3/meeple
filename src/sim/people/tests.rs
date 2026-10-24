use super::*;

fn clear_of_basin(start: Vec2, path: &[Vec2]) -> bool {
    let c = Vec2::ZERO;
    let mut a = start;
    for &b in path {
        // the basin itself plus a little body room must never be entered
        if seg_hits_square(a, b, c, 2.9) {
            return false;
        }
        a = b;
    }
    true
}

#[test]
fn straight_through_the_middle_is_rerouted() {
    let start = Vec2::new(-11.0, 0.3);
    let pts = [Vec2::new(11.0, 0.3)];
    let out = avoid_square(start, &pts, Vec2::ZERO, FOUNTAIN_CLEAR, FOUNTAIN_DETOUR);
    assert!(out.len() > 1);
    assert!(clear_of_basin(start, &out));
    assert_eq!(*out.last().unwrap(), pts[0]);
}

#[test]
fn every_crossing_direction_is_rerouted() {
    for i in 0..72 {
        let a = i as f32 * std::f32::consts::TAU / 72.0;
        for j in 1..36 {
            let b = a + std::f32::consts::PI + (j as f32 - 18.0) * 0.05;
            let start = Vec2::new(a.cos(), a.sin()) * 8.0;
            let end = Vec2::new(b.cos(), b.sin()) * 8.0;
            let out = avoid_square(start, &[end], Vec2::ZERO, FOUNTAIN_CLEAR, FOUNTAIN_DETOUR);
            assert!(clear_of_basin(start, &out), "{start} -> {end}: {out:?}");
        }
    }
}

#[test]
fn paths_that_miss_are_untouched() {
    let start = Vec2::new(-8.0, 5.0);
    let pts = [Vec2::new(8.0, 5.0), Vec2::new(8.0, -8.0)];
    assert_eq!(avoid_square(start, &pts, Vec2::ZERO, FOUNTAIN_CLEAR, FOUNTAIN_DETOUR), pts.to_vec());
}

use crate::world::gen::Gen;

/// Run a few towns through a whole day and watch every villager: nobody
/// may ever be inside a building's walls, and no bench may hold two.
#[test]
fn a_day_in_town_keeps_people_out_of_walls_and_benches_single() {
    let gen = Gen::new(1234);
    let towns: Vec<_> = (-3..=3).flat_map(|rx| (-3..=3).map(move |rz| (rx, rz)))
        .filter_map(|(rx, rz)| gen.town(rx, rz))
        .take(8)
        .collect();
    assert!(towns.len() >= 6);
    // the real app's frame rate, where crowd jostling shows up
    let dt = 1.0 / 60.0;
    let day_len = 600.0;
    let (mut seen, mut sat, mut danced) = (0usize, 0usize, 0usize);
    let mut gatherings = 0;
    // crowd behaviour: overlapping pairs, walkers making no headway, and
    // frame-to-frame direction reversals (vibration)
    let (mut overlaps, mut stuck, mut reversals, mut moving) = (0usize, 0usize, 0usize, 0usize);
    let mut kinds = std::collections::HashMap::new();
    // village life: stallholders, shoppers with baskets, games, post rounds
    let (mut minding, mut baskets, mut games, mut post) = (0usize, 0usize, 0usize, 0usize);
    for t in towns {
        let mut hour = 6.0f32;
        let env = |hour: f32, time: f32| Env {
            gen: &gen,
            dt,
            time,
            hour,
            slot: (hour * 2.0) as u32,
            rain: 0.0,
            night: if (7.0..19.0).contains(&hour) { 0.0 } else { 1.0 },
            day: 3,
            nice: true,
            wind: Vec2::new(0.3, 0.1),
            season: 1.5,
        };
        let mut life = TownLife::new(&gen, t.clone(), &env(hour, 0.0));
        // a cart standing in the street just off the plaza for half a minute, as one does on arriving
        let carts = {
            let (c, f) = (life.center() + Vec2::new(9.5, 0.5), Vec2::X);
            [(c, f), (c + f * 1.3, f), (c + f * 2.6, f)]
        };
        let mut time = 0.0;
        let mut sounds = Vec::new();
        let mut peak = 0usize;
        let mut expected = 0usize;
        let n = life.people.len();
        let mut prev_pos: Vec<Vec2> = life.people.iter().map(|p| p.pos).collect();
        let mut prev_step = vec![Vec2::ZERO; n];
        let mut anchor: Vec<(Vec2, f32)> = life.people.iter().map(|p| (p.pos, 0.0)).collect();
        while hour < 23.0 {
            life.update(&env(hour, time), if (30.0..60.0).contains(&time) { &carts[..] } else { &[] }, &mut sounds);
            sounds.clear();
            // nobody wades into the fountain's basin
            {
                let c = life.center();
                for q in &life.people {
                    if q.visible && q.state != State::Inside {
                        assert!(
                            (q.pos.x - c.x).abs() > 2.2 || (q.pos.y - c.y).abs() > 2.2,
                            "{} ({:?}, {:?} to {:?}) is in the fountain of {} at {} (centre {c})",
                            q.name, q.role, q.goal, q.dest, t.name, q.pos
                        );
                    }
                }
            }
            let out: Vec<usize> = (0..n)
                .filter(|&k| life.people[k].visible && life.people[k].state != State::Inside)
                .collect();
            for (x, &a) in out.iter().enumerate() {
                for &b in &out[x + 1..] {
                    if life.people[a].pos.distance(life.people[b].pos) < 0.3 {
                        overlaps += 1;
                        let key = format!("{:?}+{:?}", life.people[a].state, life.people[b].state).replace("Chatting", "Chat");
                        *kinds.entry(key.chars().filter(|c| !c.is_ascii_digit() && *c != '(' && *c != ')').collect::<String>()).or_insert(0usize) += 1;
                    }
                }
            }
            for k in 0..n {
                let p = &life.people[k];
                let step = p.pos - prev_pos[k];
                if p.visible && matches!(p.state, State::Walking | State::Idle) && step.length() > 1e-4 {
                    moving += 1;
                    if step.dot(prev_step[k]) < 0.0 {
                        reversals += 1;
                    }
                }
                if p.visible && p.state == State::Walking {
                    if p.pos.distance(anchor[k].0) > 0.5 {
                        anchor[k] = (p.pos, time);
                    } else if time - anchor[k].1 > 3.0 {
                        stuck += 1;
                        anchor[k] = (p.pos, time);
                    }
                } else {
                    anchor[k] = (p.pos, time);
                }
                prev_step[k] = step;
                prev_pos[k] = p.pos;
            }
            if life.game.is_some() {
                games += 1;
            }
            if let Some(g) = &life.gathering {
                expected = g.expected;
                let here = life
                    .people
                    .iter()
                    .filter(|p| p.visible && matches!(p.state, State::Dancing | State::Singing | State::Playing))
                    .count();
                peak = peak.max(here);
            }
            for (k, p) in life.people.iter().enumerate() {
                if p.visible {
                    seen += 1;
                    if p.state == State::Working && matches!(p.dest, Dest::Stall(_)) {
                        minding += 1;
                    }
                    if p.carry == Carry::Basket {
                        baskets += 1;
                    }
                    if p.role == Role::Postie && p.goal == Goal::Work && p.state == State::Idle {
                        post += 1;
                    }
                    if matches!(p.state, State::Dancing | State::Singing | State::Playing) {
                        danced += 1;
                    }
                    if life.inside_building(p.pos) {
                        let hit: Vec<_> = life.lots.iter().enumerate()
                            .filter(|(_, l)| { let (x0, z0, x1, z1) = l.footprint(); p.pos.x > x0 as f32 && p.pos.x < x1 as f32 + 1.0 && p.pos.y > z0 as f32 && p.pos.y < z1 as f32 + 1.0 })
                            .map(|(i, l)| format!("lot {i} {:?} {:?} face {}", l.kind, l.footprint(), l.face))
                            .collect();
                        panic!(
                            "{} ({:?}, {:?}, going to {:?}) is inside a building at {} at {hour:.2}h in {} (centre {}, {})\n  path {:?} (next {})\n  inside {:?}",
                            p.name, p.role, p.state, p.dest, p.pos, t.name, t.cx, t.cz, p.path, p.pi, hit
                        );
                    }
                }
                if let (State::Sitting, Dest::Bench(b)) = (p.state, p.dest) {
                    sat += 1;
                    let others = life.people.iter().enumerate()
                        .filter(|(j, q)| *j != k && q.state == State::Sitting && q.dest == Dest::Bench(b))
                        .count();
                    assert_eq!(others, 0, "bench {b} in {} has two sitters at {hour:.2}h", t.name);
                }
            }
            time += dt;
            hour += dt * 24.0 / day_len;
        }
        // a gathering that's called is one that people actually come to
        if expected > 0 {
            gatherings += 1;
            println!("{}: {expected} said they'd come, {peak} there at once", t.name);
            assert!(expected >= MIN_TURNOUT, "{} called a gathering for {expected}", t.name);
            assert!(peak >= MIN_TURNOUT, "{} expected {expected} but only {peak} turned up", t.name);
        }
    }
    assert!(gatherings > 0, "no gatherings happened");
    println!("{seen} person-steps outdoors, {sat} on benches, {danced} at gatherings");
    println!("overlaps by state: {kinds:?}");
    println!("village life: {minding} stall-minding, {baskets} basket-carrying, {games} game, {post} doorstep post steps");
    assert!(minding > 0 && baskets > 0 && games > 0 && post > 0, "some village life never happened");
    assert_eq!(stuck, 0, "walkers got stuck in crowds");
    assert!(overlaps < 600, "{overlaps} overlapping pair-steps (was 3311 before crowd steering)");
    assert!(reversals * 1000 < moving * 3, "too much jitter: {reversals} reversals in {moving} steps");
    println!(
        "crowds: {overlaps} overlapping pair-steps, {stuck} stuck walkers, {reversals} reversals in {moving} moving steps ({:.2}%)",
        reversals as f32 * 100.0 / moving.max(1) as f32
    );
    assert!(danced > 1000, "only {danced} person-steps at gatherings");
    // make sure the day actually exercised walking and sitting
    assert!(seen > 50_000, "only {seen} person-steps outdoors");
    assert!(sat > 500, "only {sat} person-steps on benches");
}


/// A wedding day runs its course: everyone in church before the vows, the
/// couple dancing afterwards, and sharing a home at the end.
#[test]
fn a_wedding_goes_from_church_to_dance_to_a_shared_home() {
    let gen = Gen::new(1234);
    // only towns with a church hold weddings
    let towns: Vec<_> = (-8..=8).flat_map(|rx| (-8..=8).map(move |rz| (rx, rz)))
        .filter_map(|(rx, rz)| gen.town(rx, rz))
        .filter(|t| gen.lots(t).iter().any(|l| l.kind == LotKind::Church))
        .collect();
    let dt = 1.0 / 30.0;
    let mut weddings = 0;
    for t in towns.iter().take(4) {
        let Some(day) = (0..200).find(|&d| unit(hash3(t.id, d, 3, 0xbe11)) < WEDDING_CHANCE) else { continue };
        let env = |hour: f32, time: f32| Env {
            gen: &gen, dt, time, hour, slot: (hour * 2.0) as u32, rain: 0.0, night: 0.0,
            day: day as i64, nice: true, wind: Vec2::ZERO, season: 1.5,
        };
        let mut hour = 11.5f32;
        let mut life = TownLife::new(&gen, t.clone(), &env(hour, 0.0));
        if life.church.is_none() {
            continue;
        }
        let (mut time, mut sounds) = (0.0, Vec::new());
        let (mut couple, mut in_church, mut danced, mut bells) = (None, false, false, false);
        while hour < 18.5 {
            life.update(&env(hour, time), &[], &mut sounds);
            bells |= sounds.iter().any(|s| s.name == "bell") && (14.0..15.3).contains(&hour);
            sounds.clear();
            if let Some(w) = &life.wedding {
                couple = Some(w.couple);
            }
            if let Some((a, b)) = couple {
                let church = Place::Lot(life.church.unwrap());
                let inside = |i: usize| life.people[i].state == State::Inside && life.people[i].at == church;
                let vowed = life.wedding.as_ref().is_some_and(|w| w.vows.is_some());
                if !vowed && inside(a) && inside(b) {
                    in_church = true;
                }
                for p in life.people.iter().filter(|p| p.visible && p.wed != Wed::None) {
                    assert!(!life.inside_building(p.pos), "{} walked into a wall at {hour:.2}h in {}", p.name, t.name);
                }
                danced |= life.people[a].state == State::Dancing && life.people[b].state == State::Dancing;
            }
            time += dt;
            hour += dt * 24.0 / 600.0;
        }
        let Some((a, b)) = couple else { continue };
        weddings += 1;
        println!("{}: {} weds {}", t.name, life.people[a].name, life.people[b].name);
        assert!(in_church, "the couple weren't in church before three in {}", t.name);
        assert!(bells, "no bells at three in {}", t.name);
        assert!(danced, "the couple never danced in {}", t.name);
        assert_eq!(life.people[a].home, life.people[b].home, "the newlyweds live apart in {}", t.name);
        assert!(life.wedding.is_none() && life.people.iter().all(|p| p.wed == Wed::None));
    }
    assert!(weddings >= 2, "only {weddings} weddings happened");
}

/// Fires, christenings and the rest walk only the streets: never
/// through a building or the fountain, and the fire brigade stops well
/// short of the burning house.
#[test]
fn event_routes_keep_to_the_streets() {
    let gen = Gen::new(1234);
    let env = |time: f32| Env {
        gen: &gen,
        dt: 0.1,
        time,
        hour: 12.0,
        slot: 24,
        rain: 0.0,
        night: 0.0,
        day: 3,
        nice: true,
        wind: Vec2::new(0.3, 0.1),
        season: 1.5,
    };
    let (mut routes, mut cities) = (0, 0);
    for rx in -5..=5 {
        for rz in -5..=5 {
            let Some(t) = gen.town(rx, rz) else { continue };
            cities += t.city as usize;
            let mut life = TownLife::new(&gen, t.clone(), &env(0.0));
            for (n, kind) in [HappeningKind::Fire, HappeningKind::Christening, HappeningKind::Doctor, HappeningKind::Closing].into_iter().enumerate() {
                life.kindle(kind);
                let mut sounds = Vec::new();
                life.update(&env(n as f32), &[], &mut sounds);
            }
            let c = life.center();
            for (kind, path) in life.event_paths() {
                let total: f32 = path.windows(2).map(|w| w[0].distance(w[1])).sum();
                let mut d = 0.0;
                while d < total {
                    let mut left = d;
                    let mut p = path[0];
                    for w in path.windows(2) {
                        let l = w[0].distance(w[1]);
                        if left <= l {
                            p = w[0].lerp(w[1], left / l.max(1e-4));
                            break;
                        }
                        left -= l;
                    }
                    assert!(!life.inside_building(p), "{kind:?} walks through a building at {p} in {}", t.name);
                    assert!((p - c).abs().max_element() > 2.6 || p.distance(c) > 2.6, "{kind:?} walks through the fountain in {} at {p} (centre {c}) path {path:?}", t.name);
                    d += 0.5;
                }
                routes += 1;
            }
        }
    }
    assert!(routes > 20 && cities > 0, "only {routes} routes");
}

/// Bella lifts weights in the square through the day, is always "PUMPING
/// IRON!", and goes home to bed at night.
#[test]
fn bella_lifts_by_day_and_sleeps_at_night() {
    let gen = Gen::new(1234);
    let t = (-3..=3).flat_map(|rx| (-3..=3).map(move |rz| (rx, rz))).filter_map(|(rx, rz)| gen.town(rx, rz)).next().unwrap();
    let dt = 1.0 / 30.0;
    let env = |hour: f32, time: f32| Env {
        gen: &gen, dt, time, hour, slot: (hour * 2.0) as u32, rain: 0.0, night: if (7.0..19.0).contains(&hour) { 0.0 } else { 1.0 },
        day: 3, nice: true, wind: Vec2::ZERO, season: 1.5,
    };
    let mut life = TownLife::new(&gen, t, &env(6.0, 0.0));
    life.add_bella();
    let me = life.people.iter().position(|p| p.bella).expect("no Bella");
    assert_eq!(life.doing(me), "PUMPING IRON!");
    let (mut hour, mut time) = (6.0f32, 0.0f32);
    let mut sounds = Vec::new();
    let (mut at_noon, mut at_night) = (false, false);
    while hour < 23.8 {
        life.update(&env(hour, time), &[], &mut sounds);
        sounds.clear();
        assert_eq!(life.doing(me), "PUMPING IRON!");
        let p = &life.people[me];
        if (12.0..12.1).contains(&hour) {
            at_noon = p.state == State::Working && p.dest == Dest::Gym && p.pos.distance(life.bella_spot()) < 0.6;
        }
        if (23.5..23.8).contains(&hour) {
            at_night = p.state == State::Inside;
        }
        time += dt;
        hour += dt * 24.0 / 1200.0;
    }
    assert!(at_noon, "Bella wasn't at the weights at noon");
    assert!(at_night, "Bella wasn't home in bed at night");
}

/// A world that starts at midday finds Bella already out in the square,
/// and he's soon on his mat.
#[test]
fn bella_is_at_the_weights_when_the_world_starts_at_noon() {
    let gen = Gen::new(1234);
    let t = (-3..=3).flat_map(|rx| (-3..=3).map(move |rz| (rx, rz))).filter_map(|(rx, rz)| gen.town(rx, rz)).next().unwrap();
    let dt = 1.0 / 30.0;
    let env = |time: f32| Env {
        gen: &gen, dt, time, hour: 12.0, slot: 24, rain: 0.0, night: 0.0,
        day: 3, nice: true, wind: Vec2::ZERO, season: 1.5,
    };
    let mut life = TownLife::new(&gen, t, &env(0.0));
    life.add_bella();
    let me = life.people.iter().position(|p| p.bella).unwrap();
    // placed the way the world's start places everyone: out in the square
    life.place_now(me, &env(0.0));
    let mut sounds = Vec::new();
    let mut there = false;
    for k in 0..(120.0 / dt) as i32 {
        life.update(&env(k as f32 * dt), &[], &mut sounds);
        sounds.clear();
        let p = &life.people[me];
        there |= p.state == State::Working && p.dest == Dest::Gym && p.pos.distance(life.bella_spot()) < 0.6;
    }
    assert!(there, "Bella never reached his mat");
}

/// Babies are born to a house on days that depend only on the house, a week
/// apart at the very least, and a town reloaded later finds the same children.
#[test]
fn households_have_the_same_babies_whenever_the_town_loads() {
    let gen = Gen::new(1234);
    let t = (-8..=8).flat_map(|rx| (-8..=8).map(move |rz| (rx, rz))).find_map(|(rx, rz)| gen.town(rx, rz)).unwrap();
    let env = |day: i64| Env {
        gen: &gen, dt: 1.0 / 30.0, time: 0.0, hour: 12.0, slot: 24, rain: 0.0, night: 0.0,
        day, nice: true, wind: Vec2::ZERO, season: 1.5,
    };
    let lots = gen.lots(&t);
    let mut births = 0;
    for l in lots.iter().filter(|l| matches!(l.kind, LotKind::House | LotKind::Apartment)) {
        let days = super::family::birth_days(l.seed, 0, 1000);
        assert!(days.windows(2).all(|w| w[1] - w[0] >= 20), "babies too close together");
        births += days.len();
    }
    assert!(births > 0, "no house had a baby in a thousand days");
    // live one day at a time, or loaded straight at the end: the same people
    let mut a = TownLife::new(&gen, t.clone(), &env(300));
    let mut sounds = Vec::new();
    for d in 301..=340 {
        a.update(&env(d), &[], &mut sounds);
    }
    let b = TownLife::new(&gen, t.clone(), &env(340));
    let born = |l: &TownLife| {
        let mut v: Vec<(u32, Option<i64>)> = l.people.iter().filter(|p| p.born.is_some()).map(|p| (p.seed, p.born)).collect();
        v.sort();
        v
    };
    assert_eq!(born(&a), born(&b));
    // everyone has a surname and the cards mention kin
    let kin = (0..b.people.len()).any(|i| b.card(i).1.iter().any(|l| l.starts_with("Parent of") || l.starts_with("Child of")));
    assert!(kin || b.people.iter().all(|p| p.role != Role::Child), "children but no family lines");
}

/// On a winter afternoon the green has snowmen and a snowball fight; not in
/// foul weather, and not in summer.
#[test]
fn winter_afternoons_bring_snowmen_and_snowballs_to_the_green() {
    let mut checked = 0;
    for seed in [1234u32, 7, 99, 2024, 5, 31] {
        let gen = Gen::new(seed);
        for rx in -6..=6 {
            for rz in -6..=6 {
                let Some(t) = gen.town(rx, rz) else { continue };
                if gen.green(&t).is_none() || checked >= 3 {
                    continue;
                }
                let env = |season: f32, hour: f32, rain: f32| Env {
                    gen: &gen,
                    dt: 1.0 / 30.0,
                    time: 50.0,
                    hour,
                    slot: (hour * 2.0) as u32,
                    rain,
                    night: 0.0,
                    day: 11,
                    nice: rain < 0.05,
                    wind: Vec2::new(0.2, 0.1),
                    season,
                };
                let mut life = TownLife::new(&gen, t.clone(), &env(3.3, 14.0, 0.0));
                let mut sounds = Vec::new();
                life.update(&env(3.3, 14.0, 0.0), &[], &mut sounds);
                let c = life.center();
                let boxes = |season: f32, hour: f32, rain: f32| {
                    let (mut solid, mut fx) = (Boxes::default(), Boxes::default());
                    life.draw(&mut solid, &mut fx, &env(season, hour, rain), 0.0, c, 80.0);
                    solid.len() + fx.len()
                };
                let play = boxes(3.3, 14.0, 0.0);
                assert!(play > boxes(3.3, 14.0, 0.9) + 20, "{}: no snow play on the green ({play})", t.name);
                assert!(life.snowball_fight_on(&env(3.3, 14.0, 0.0)));
                assert!(!life.snowball_fight_on(&env(1.5, 14.0, 0.0)), "snowballs in summer");
                assert!(!life.snowball_fight_on(&env(3.3, 8.0, 0.0)), "snowballs at breakfast");
                // by the evening the snowmen are still standing, but the fight is over
                assert!(boxes(3.3, 19.0, 0.0) < play, "{}: the fight goes on after dark", t.name);
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "no town with a green found");
}
