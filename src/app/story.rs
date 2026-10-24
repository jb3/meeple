//! Story arcs for the autopilot. Rather than only hopping between the best
//! sights of the moment, it sometimes follows one town's occasion across a few
//! days: the eve of a wedding as the lamps come on, the day itself, and the
//! morning after. The occasions are predicted from the towns' own dice (see
//! `forecast`), so the chapters are ones the town will actually deliver.

use glam::Vec2;

use crate::sim::clock::DAYS_PER_SEASON;
use crate::sim::people::{forecast, Occasion};
use crate::util::*;
use crate::world::gen::Gen;

/// One chapter: where and when to be, and what the camera sees.
#[derive(Clone, Debug)]
pub struct Beat {
    pub day: i64,
    pub from: f32,
    pub to: f32,
    pub why: String,
    /// What the town promises the camera, if it's an occasion.
    pub occasion: Option<Occasion>,
}

#[derive(Clone, Debug)]
pub struct Story {
    pub title: String,
    pub town: u32,
    pub pos: Vec2,
    pub beats: Vec<Beat>,
    pub next: usize,
}

/// How far ahead (in game hours) of a chapter's start the camera sets off.
pub const LEAD: f32 = 3.0;

fn season_after(season: f32, days: i64) -> f32 {
    (season + (days as f64 / DAYS_PER_SEASON) as f32).rem_euclid(4.0)
}

impl Story {
    /// Look for an occasion in the next couple of days, near enough to reach,
    /// and plan a story round it.
    pub fn plan(gen: &Gen, here: Vec2, day: i64, hour: f32, season: f32, rng: &mut Rng) -> Option<Story> {
        let r = 800;
        let towns = gen.towns_near(here.x as i32 - r, here.y as i32 - r, here.x as i32 + r, here.y as i32 + r, 0);
        let mut best: Option<(f32, Story)> = None;
        for t in towns {
            let p = Vec2::new(t.cx as f32, t.cz as f32);
            for k in 1..=2i64 {
                for f in forecast(&t, day + k, season_after(season, k)) {
                    let Some(occasion) = f.occasion else { continue };
                    if f.weight < 2.2 {
                        continue;
                    }
                    let name = &t.name;
                    let (title, eve, after) = match occasion {
                        Occasion::Wedding => (format!("A wedding in {name}"), format!("The eve of a wedding: {name} makes ready"), format!("The morning after the wedding in {name}")),
                        Occasion::Christening => (format!("A christening in {name}"), format!("The night before a christening in {name}"), format!("{name} the morning after the christening")),
                        Occasion::Fire => (format!("Trouble in {name}"), format!("A quiet evening in {name}"), format!("{name} the morning after the fire")),
                        Occasion::Gathering => (f.why.clone(), format!("Villagers of {name} make ready for tonight's gathering"), format!("{name} the morning after")),
                    };
                    let mut beats = Vec::new();
                    // the eve, if there's still time to catch it
                    let eve_day = day + k - 1;
                    if eve_day > day || hour < 19.0 {
                        beats.push(Beat { day: eve_day, from: 17.0, to: 21.0, why: eve, occasion: None });
                    }
                    beats.push(Beat { day: day + k, from: f.from, to: f.to, why: f.why.clone(), occasion: Some(occasion) });
                    beats.push(Beat { day: day + k + 1, from: 7.0, to: 10.0, why: after, occasion: None });
                    let near = p.distance(here);
                    let score = f.weight - near / 600.0 + rng.range(0.0, 0.5);
                    if near < 700.0 && best.as_ref().map_or(true, |b| score > b.0) {
                        best = Some((score, Story { title, town: t.id, pos: p, beats, next: 0 }));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    pub fn beat(&self) -> Option<&Beat> {
        self.beats.get(self.next)
    }

    /// Drop chapters whose time has gone by; the story ends when none are left.
    pub fn catch_up(&mut self, day: i64, hour: f32) {
        while let Some(b) = self.beat() {
            if b.day < day || (b.day == day && hour > b.to) {
                self.next += 1;
            } else {
                break;
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.next >= self.beats.len()
    }

    /// Is the next chapter one to head for now, arriving at `eta` (a game
    /// hour today)? That's from a few hours before it starts to before it ends.
    pub fn due(&self, day: i64, eta: f32) -> Option<&Beat> {
        self.beat().filter(|b| b.day == day && eta >= b.from - LEAD && eta <= b.to - 0.3)
    }

    /// "Chapter 2 of 3".
    pub fn chapter(&self) -> String {
        format!("Chapter {} of {}", (self.next + 1).min(self.beats.len()), self.beats.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stories_are_ordered_chapters_about_one_town() {
        let mut found = 0;
        for seed in [1234u32, 7, 99, 2024] {
            let gen = Gen::new(seed);
            let mut rng = Rng::new(seed as u64);
            for day in 0..60 {
                let town = gen.town(0, 0).or_else(|| gen.town(1, 0)).or_else(|| gen.town(0, 1));
                let Some(t) = town else { continue };
                let here = Vec2::new(t.cx as f32, t.cz as f32);
                let Some(s) = Story::plan(&gen, here, day, 9.0, (day as f32 / 3.0) % 4.0, &mut rng) else { continue };
                found += 1;
                assert!(s.beats.len() >= 2, "{} has too few chapters", s.title);
                let main = s.beats.iter().filter(|b| b.occasion.is_some()).count();
                assert_eq!(main, 1, "{}: exactly one chapter is the occasion", s.title);
                for w in s.beats.windows(2) {
                    assert!(w[0].day <= w[1].day, "{}: chapters out of order", s.title);
                }
                assert!(s.beats[0].day >= day, "{} starts in the past", s.title);
                // heading for the occasion: due from a few hours before it
                let b = s.beats.iter().find(|b| b.occasion.is_some()).unwrap();
                let mut st = s.clone();
                st.next = s.beats.iter().position(|x| x.day == b.day && x.occasion.is_some()).unwrap();
                assert!(st.due(b.day, b.from - LEAD - 0.5).is_none(), "{} too early", s.title);
                assert!(st.due(b.day, b.from - 1.0).is_some(), "{} should be due", s.title);
                // and it lapses once its time is past
                st.catch_up(b.day, b.to + 0.1);
                assert!(st.beat().map_or(true, |n| n.day > b.day));
            }
        }
        assert!(found > 3, "only {found} stories planned");
    }
}
