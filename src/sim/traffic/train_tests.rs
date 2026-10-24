use super::*;

fn a_junction(gen: &Gen) -> Rc<Chord> {
    gen.chords_near(-4000, -4000, 4000, 4000).into_iter().find(|c| c.a != c.b).unwrap()
}

/// Distance from a point to the nearest bit of track it could be on.
fn off_track(c: &Chord, p: Vec2) -> f32 {
    let line = |l: RailLine| l.perp(p.x, p.y).abs();
    c.dist(p).min(line(c.a)).min(line(c.b))
}

/// Run a train at a junction from 70 blocks out; returns where it ended up.
fn run(gen: &Gen, c: &Rc<Chord>, want_turn: bool) -> Train {
    let mut traffic = Traffic::new(1);
    let dir = if c.cross_a > c.a_s { 1.0 } else { -1.0 };
    let id = (1..).find(|&id| {
        let probe = traffic.make_train(gen, c.a, 0.0, dir, id);
        probe.takes(c) == want_turn
    }).unwrap();
    let mut t = traffic.make_train(gen, c.a, c.a_s - dir * 70.0, dir, id);
    for _ in 0..800 {
        t.advance(gen, 0.25, &[]);
        for k in 0..t.cars.len() {
            let p = t.car_pos(gen, k);
            let p = Vec2::new(p.x, p.z);
            assert!(off_track(c, p) < 0.4, "car {k} left the track at {p}");
            if k > 0 {
                let q = t.car_pos(gen, k - 1);
                let gap = p.distance(Vec2::new(q.x, q.z));
                assert!((4.3..=4.75).contains(&gap), "cars {k} and {} are {gap} apart", k - 1);
            }
        }
    }
    t
}

#[test]
fn a_train_can_take_a_junction_onto_the_other_line() {
    let gen = Gen::new(1234);
    let c = a_junction(&gen);
    let t = run(&gen, &c, true);
    match t.front {
        Front::Line { line, s, dir } => {
            assert_eq!(line, c.b, "should have turned onto the other line");
            assert!((s - c.cross_b) * dir > 0.0, "should be heading away from the crossing");
        }
        Front::Chord { .. } => panic!("still on the junction"),
    }
}

/// Send two trains at each crossing so they'd meet in the middle: the
/// signals hold one back and they never touch.
#[test]
fn signals_keep_trains_apart_at_crossings() {
    let gen = Gen::new(1234);
    let (mut runs, mut held) = (0, 0);
    for (a, b, c) in gen.crossings_near(-1500, -1500, 1500, 1500).into_iter().take(4) {
        for (da, db) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
            runs += 1;
            let mut traffic = Traffic::new(7);
            let sa = a.along(c.x, c.y) - da * 110.0;
            let sb = b.along(c.x, c.y) - db * 110.0;
            let ta = traffic.make_train(&gen, a, sa, da, 1);
            let tb = traffic.make_train(&gen, b, sb, db, 2);
            traffic.trains = vec![ta, tb];
            // no other trains for now
            for l in [a, b] {
                traffic.line_t.insert(l, f32::MAX);
            }
            let mut sounds = Vec::new();
            let mut emit = Emit { smoke: Vec::new() };
            let mut stopped = false;
            for _ in 0..1000 {
                traffic.update(&gen, 0.1, c, 400.0, &[], &mut sounds, &mut emit);
                let bodies: Vec<Vec<Vec2>> = traffic.trains.iter().map(|t| t.body()).collect();
                if bodies.len() == 2 {
                    for p in &bodies[0] {
                        for q in &bodies[1] {
                            assert!(p.distance(*q) > 2.5, "trains collided at {p} on the {}/{} crossing", a.coord, b.coord);
                        }
                    }
                }
                stopped |= traffic.trains.iter().any(|t| t.speed < 0.05 && t.stop_t <= 0.0);
            }
            held += stopped as usize;
            // and the one held back gets the road once the other's through
            for t in traffic.trains.iter().filter(|t| t.id <= 2) {
                assert!(t.head().distance(c) > BLOCK_R && (t.speed > 1.0 || t.stop_t > 0.0), "train {} never got through the {}/{} crossing", t.id, a.coord, b.coord);
            }
        }
    }
    println!("{runs} meetings, {held} with a train held at a signal");
    assert!(held * 2 >= runs, "signals hardly ever held a train");
}

#[test]
fn a_train_can_go_straight_past_a_junction() {
    let gen = Gen::new(1234);
    let c = a_junction(&gen);
    let t = run(&gen, &c, false);
    assert!(matches!(t.front, Front::Line { line, .. } if line == c.a), "should have stayed on its line");
}

/// With trains following each other along the same track, none ever runs into another.
#[test]
fn busy_lines_never_see_trains_touch() {
    let gen = Gen::new(1234);
    let (a, _, c) = gen.crossings_near(-1500, -1500, 1500, 1500).into_iter().next().unwrap();
    let _ = a;
    let mut traffic = Traffic::new(3);
    let (mut sounds, mut emit) = (Vec::new(), Emit { smoke: Vec::new() });
    let mut most = 0;
    for step in 0..20000 {
        traffic.update(&gen, 0.1, c, 160.0, &[], &mut sounds, &mut emit);
        sounds.clear();
        most = most.max(traffic.trains.len());
        if step % 5 == 0 {
            let bodies: Vec<Vec<Vec2>> = traffic.trains.iter().map(|t| t.body()).collect();
            for i in 0..bodies.len() {
                for j in i + 1..bodies.len() {
                    for p in &bodies[i] {
                        for q in &bodies[j] {
                            assert!(p.distance(*q) > 2.0, "trains {} and {} touched at {p} (step {step})", traffic.trains[i].id, traffic.trains[j].id);
                        }
                    }
                }
            }
        }
    }
    println!("most trains at once: {most}");
    assert!(most >= 3, "only {most} trains at once");
}
