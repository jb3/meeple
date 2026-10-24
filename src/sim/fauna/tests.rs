use super::*;

/// Load the world around `at` and run the animals there at `hour`.
fn run(world: &mut World, at: Vec2, hour: f32, night: f32, threats: &[Vec2], secs: f32, fauna: &mut Fauna) {
    let (c0, c1) = (World::chunk_of(at.x - 160.0, at.y - 160.0), World::chunk_of(at.x + 160.0, at.y + 160.0));
    let keys: Vec<_> = (c0.1..=c1.1).flat_map(|z| (c0.0..=c1.0).map(move |x| (x, z))).collect();
    world.stream(&keys, f64::MAX, &|| 0.0);
    let mut sounds = Vec::new();
    let mut t = 0.0;
    while t < secs {
        fauna.update(world, 0.1, at, 150.0, night, hour, 0.0, 1.5, threats, &|_| false, true, &mut sounds);
        t += 0.1;
    }
}

#[test]
fn wildlife_turns_up_and_keeps_its_hours() {
    let mut world = World::new(1234);
    let mut fauna = Fauna::new(1234);
    let mut seen = std::collections::HashSet::new();
    for (x, z) in [(0.0, 0.0), (600.0, 0.0), (-600.0, 300.0), (0.0, -700.0), (-850.0, 300.0), (700.0, 700.0)] {
        run(&mut world, Vec2::new(x, z), 19.0, 0.7, &[], 20.0, &mut fauna);
        for g in fauna.groups.values() {
            if g.active > 0.5 && !g.members.is_empty() {
                seen.insert(format!("{:?}", g.kind as u8));
            }
        }
    }
    let names = [Kind::Deer, Kind::Fox, Kind::Rabbit, Kind::Swan, Kind::Heron, Kind::Owl];
    let found = names.iter().filter(|k| seen.contains(&format!("{:?}", **k as u8))).count();
    println!("wild kinds out at dusk: {found} of 6");
    assert!(found >= 4, "too few kinds of wildlife");

    // at midday the shy creatures and owls fade back into cover
    let at = Vec2::new(0.0, 0.0);
    run(&mut world, at, 12.0, 0.0, &[], 25.0, &mut fauna);
    for g in fauna.groups.values() {
        if g.kind.wild() || g.kind == Kind::Owl {
            assert!(g.active < 0.1, "{:?} still out at noon", g.kind as u8);
        }
    }
}

#[test]
fn deer_bolt_from_people() {
    let mut world = World::new(1234);
    let mut fauna = Fauna::new(1234);
    for (x, z) in [(0.0, 0.0), (600.0, 0.0), (-600.0, 300.0), (700.0, 700.0), (0.0, -700.0)] {
        run(&mut world, Vec2::new(x, z), 19.0, 0.7, &[], 5.0, &mut fauna);
        let deer = fauna.groups.values().find(|g| g.kind == Kind::Deer && g.active > 0.5).map(|g| g.members[0].pos);
        if let Some(d) = deer {
            let walker = d + Vec2::new(3.0, 0.0);
            run(&mut world, Vec2::new(x, z), 19.0, 0.7, &[walker], 2.0, &mut fauna);
            let now = fauna.groups.values().find(|g| g.kind == Kind::Deer).unwrap().members[0].pos;
            assert!(now.distance(walker) > d.distance(walker) + 3.0, "deer didn't run: {d} -> {now}");
            return;
        }
    }
    panic!("no deer found to scare");
}

/// Look at a canal for a while and a narrowboat comes along it, and it
/// stays on the water.
#[test]
fn narrowboats_work_the_canals() {
    let mut world = World::new(1234);
    let mut fauna = Fauna::new(1234);
    // the middle of a canal at least 120 long
    let gen = &world.gen;
    let at = gen
        .canals_near(-2000, 2000)
        .into_iter()
        .find_map(|cz| {
            (-2000..2000).step_by(10).find(|&x| (0..12).all(|k| gen.canal_open((x + k * 10) as f32, cz))).map(|x| Vec2::new(x as f32 + 60.0, cz as f32 + 0.5))
        })
        .expect("no long canal");
    let (c0, c1) = (World::chunk_of(at.x - 160.0, at.y - 160.0), World::chunk_of(at.x + 160.0, at.y + 160.0));
    let keys: Vec<_> = (c0.1..=c1.1).flat_map(|z| (c0.0..=c1.0).map(move |x| (x, z))).collect();
    world.stream(&keys, f64::MAX, &|| 0.0);
    let mut sounds = Vec::new();
    let mut seen = false;
    for _ in 0..600 {
        // out of sight to begin with, as in the real thing
        fauna.update(&world, 0.1, at, 60.0, 0.0, 12.0, 0.0, 1.5, &[], &|p: Vec3| Vec2::new(p.x, p.z).distance(at) < 45.0, false, &mut sounds);
        for b in &fauna.narrowboats {
            assert!(world.gen.canal_open(b.x, b.canal), "narrowboat off the canal at x={}", b.x);
            seen |= (b.x - at.x).abs() < 60.0;
        }
    }
    assert!(seen, "no narrowboat came by in a minute");
}

/// In autumn a herd of deer crosses the country heading south, keeping to dry land.
#[test]
fn deer_migrate_south_in_autumn() {
    let mut world = World::new(1234);
    let mut fauna = Fauna::new(1234);
    let at = Vec2::new(0.0, 0.0);
    let (c0, c1) = (World::chunk_of(at.x - 400.0, at.y - 400.0), World::chunk_of(at.x + 400.0, at.y + 400.0));
    let keys: Vec<_> = (c0.1..=c1.1).flat_map(|z| (c0.0..=c1.0).map(move |x| (x, z))).collect();
    world.stream(&keys, f64::MAX, &|| 0.0);
    let mut sounds = Vec::new();
    let (mut seen, mut start_z, mut end_z) = (false, None, 0.0);
    for _ in 0..4000 {
        fauna.update(&world, 0.1, at, 150.0, 0.0, 12.0, 0.0, 2.5, &[], &|_| false, true, &mut sounds);
        if let Some(h) = fauna.herds.first() {
            seen = true;
            let z = h.members.iter().map(|a| a.pos.y).sum::<f32>() / h.members.len() as f32;
            start_z.get_or_insert(z);
            end_z = z;
            for a in &h.members {
                let c = world.gen.column(&h.ctx, a.pos.x.floor() as i32, a.pos.y.floor() as i32);
                assert!(c.h >= SEA - 1, "a migrating deer waded into deep water");
            }
        }
    }
    assert!(seen, "no herd crossed the country in autumn");
    assert!(end_z > start_z.unwrap(), "the herd didn't head south");
    // none in summer
    let mut summer = Fauna::new(1234);
    for _ in 0..4000 {
        summer.update(&world, 0.1, at, 150.0, 0.0, 12.0, 0.0, 1.5, &[], &|_| false, true, &mut sounds);
        assert!(summer.herds.is_empty(), "a herd migrating in summer");
    }
}

/// There's usually something to look at near the camera: the director asks for
/// sights when what it was watching ends, and they're all genuinely nearby.
#[test]
fn there_is_something_nice_to_look_at() {
    let mut world = World::new(1234);
    let mut fauna = Fauna::new(1234);
    let mut with_sights = 0;
    for (x, z) in [(0.0, 0.0), (600.0, 0.0), (-600.0, 300.0), (0.0, -700.0), (700.0, 700.0)] {
        let at = Vec2::new(x, z);
        run(&mut world, at, 19.0, 0.7, &[], 20.0, &mut fauna);
        let sights = fauna.sights(at, 150.0);
        assert!(sights.iter().all(|(p, _)| p.distance(at) < 150.0), "a sight beyond reach");
        with_sights += !sights.is_empty() as usize;
    }
    assert!(with_sights >= 3, "something to look at near only {with_sights} of 5 places");
}
