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
