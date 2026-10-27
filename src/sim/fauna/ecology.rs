//! Population cycles and the hunt. Each region of the map has its own boom and
//! bust of rabbits, deer and ducks; foxes and wolves follow the prey a little
//! behind, so a lean year for prey is followed by a lean year for predators and
//! the prey recover. Salmon run strongly or poorly from one year to the next.
//! Everything is a pure function of the seed, place and date, so a world looks
//! the same whenever it is loaded. On top of that, hungry predators chase prey
//! they can see, and every kill thins the quarry for a while.

use super::*;
use std::f32::consts::TAU;

/// In-game days for prey numbers to rise and fall once.
pub(super) const CYCLE_DAYS: f64 = 9.0;
/// How far behind the prey a predator's numbers peak, in radians.
const PREDATOR_LAG: f32 = 1.4;
/// Side of the square regions that share a cycle.
const REGION: f32 = 320.0;
/// Days in a year, for the salmon runs.
const YEAR_DAYS: f64 = 12.0;

/// How many of a group's animals are about (0..1) at `pos` on day `days`.
/// Kinds without a cycle are always at full strength.
pub(super) fn abundance(seed: u32, kind: Kind, pos: Vec2, days: f64) -> f32 {
    let phase = unit(hash2(seed ^ 0xec01, (pos.x / REGION).floor() as i32, (pos.y / REGION).floor() as i32)) * TAU;
    let wave = |lag: f32| 0.5 + 0.5 * ((days / CYCLE_DAYS).fract() as f32 * TAU + phase - lag).sin();
    let season = ((days / 3.0) % 4.0) as f32;
    // winter is lean for everything that grazes or forages
    let winter = smoothstep(2.9, 3.3, season) * (1.0 - smoothstep(3.8, 4.0, season));
    let v = match kind {
        Kind::Rabbit | Kind::Deer | Kind::Duck => wave(0.0) * (1.0 - 0.3 * winter),
        Kind::Fox | Kind::Wolf => wave(PREDATOR_LAG),
        Kind::Salmon => 0.15 + 0.85 * unit(hash_u32(hash2(seed ^ 0x5a1, (days / YEAR_DAYS).floor() as i32, 0x77))),
        _ => return 1.0,
    };
    (0.1 + 0.9 * v).clamp(0.1, 1.0)
}

fn word(a: f32) -> &'static str {
    if a < 0.35 {
        "scarce"
    } else if a < 0.65 {
        "fair"
    } else {
        "plentiful"
    }
}

impl Fauna {
    /// The wildlife line of a village newsletter.
    pub fn wildlife_report(&self, pos: Vec2) -> String {
        let a = |k| abundance(self.seed, k, pos, self.days);
        let mut s = format!("Wildlife: rabbits {}, foxes {}, deer {}", word(a(Kind::Rabbit)), word(a(Kind::Fox)), word(a(Kind::Deer)));
        if (2.0..3.0).contains(&((self.days / 3.0) % 4.0)) {
            s += &format!(", salmon run {}", if a(Kind::Salmon) < 0.5 { "poor" } else { "strong" });
        }
        s + "."
    }

    /// Settle each group's numbers towards the cycle and run the hunt.
    pub(super) fn tick_ecology(&mut self, dt: f32) {
        for g in self.groups.values_mut() {
            let target = abundance(self.seed, g.kind, g.home, self.days);
            g.density += (target - g.density) * approach(0.2, dt);
            g.loss = (g.loss - 0.03 * dt).max(0.0);
            g.fed = (g.fed - dt).max(0.0);
        }
        // (hunter, quarry, how near the quarry bolts, how far the hunter sees)
        for (pred, prey, scare, reach) in [(Kind::Fox, Kind::Rabbit, 8.0, 14.0), (Kind::Wolf, Kind::Deer, 0.0, 22.0)] {
            let about = |groups: &HashMap<(i32, i32), Group>, kind: Kind| -> Vec<((i32, i32), usize, Vec2)> {
                groups
                    .iter()
                    .filter(|(_, g)| g.kind == kind && !g.hidden && g.active > 0.5)
                    .flat_map(|(k, g)| g.members.iter().enumerate().take(g.shown()).map(move |(i, a)| (*k, i, a.pos)))
                    .collect()
            };
            let hunters = about(&self.groups, pred);
            let quarry = about(&self.groups, prey);
            if hunters.is_empty() || quarry.is_empty() {
                continue;
            }
            let mut kills = Vec::new();
            for &(k, i, hp) in &hunters {
                let nearest = quarry.iter().filter(|q| q.2.distance(hp) < reach).min_by(|a, b| a.2.distance(hp).total_cmp(&b.2.distance(hp)));
                let (Some(&(qk, _, qp)), Some(g)) = (nearest, self.groups.get_mut(&k)) else { continue };
                if g.fed > 0.0 {
                    continue;
                }
                let a = &mut g.members[i];
                a.target = qp;
                a.flee = 0.5;
                a.graze = 0.0;
                if qp.distance(hp) < 0.9 {
                    g.fed = 30.0;
                    kills.push(qk);
                }
            }
            for k in kills {
                if let Some(g) = self.groups.get_mut(&k) {
                    g.loss += 1.0;
                }
            }
            if scare > 0.0 {
                for g in self.groups.values_mut().filter(|g| g.kind == prey) {
                    for a in &mut g.members {
                        if let (true, Some(h)) = (a.flee <= 0.0, hunters.iter().find(|h| h.2.distance(a.pos) < scare)) {
                            a.target = a.pos + (a.pos - h.2).normalize_or(Vec2::X) * 10.0;
                            a.flee = 2.0;
                            a.graze = 0.0;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_cycle_and_predators_trail_prey() {
        let p = Vec2::new(100.0, -40.0);
        let days: Vec<f64> = (0..360).map(|i| i as f64 * 0.1).collect();
        let prey: Vec<f32> = days.iter().map(|&d| abundance(5, Kind::Rabbit, p, d)).collect();
        let pred: Vec<f32> = days.iter().map(|&d| abundance(5, Kind::Fox, p, d)).collect();
        let (lo, hi) = prey.iter().fold((1.0f32, 0.0f32), |(l, h), &v| (l.min(v), h.max(v)));
        assert!(lo < 0.35 && hi > 0.75, "prey never booms or busts: {lo}..{hi}");
        // the first peak of the predators comes after the first peak of the prey
        let peak = |v: &[f32]| v[..90].iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
        let (a, b) = (peak(&prey), peak(&pred));
        assert!(b != a, "predators peak with the prey");
        assert!(prey.iter().chain(&pred).all(|v| (0.1..=1.0).contains(v)));
        assert_eq!(abundance(5, Kind::Sheep, p, 3.0), 1.0);
    }

    #[test]
    fn salmon_runs_vary_by_year() {
        let runs: Vec<f32> = (0..8).map(|y| abundance(9, Kind::Salmon, Vec2::ZERO, y as f64 * YEAR_DAYS + 7.0)).collect();
        assert!(runs.iter().any(|r| *r < 0.5) && runs.iter().any(|r| *r > 0.5), "{runs:?}");
    }

    #[test]
    fn a_fox_catches_a_rabbit() {
        let world = World::new(3);
        let group = |kind, at: Vec2| Group {
            kind,
            home: at,
            roam: 8.0,
            members: vec![Animal { pos: at, y: 0.0, yaw: 0.0, target: at, timer: 0.0, graze: 0.0, phase: 0.0, tint: [1.0; 3], flee: 0.0 }],
            ctx: world.gen.ctx(0, 0, 40, 40),
            call_t: 5.0,
            hidden: false,
            moor: None,
            active: 1.0,
            perches: Vec::new(),
            perch_low: Vec::new(),
            on_perch: 0,
            density: 1.0,
            loss: 0.0,
            fed: 0.0,
        };
        let mut f = Fauna::new(3);
        f.groups.insert((0, 0), group(Kind::Fox, Vec2::new(10.0, 10.0)));
        f.groups.insert((1, 0), group(Kind::Rabbit, Vec2::new(10.5, 10.0)));
        f.tick_ecology(0.1);
        assert!(f.groups[&(1, 0)].loss >= 1.0, "rabbit wasn't caught");
        assert!(f.groups[&(0, 0)].fed > 0.0);
        // the loss thins the group until it heals
        assert_eq!(f.groups[&(1, 0)].shown(), 0);
    }
}
