//! Goings-on in a town beyond the villagers' own routines: the occasional
//! fire and the brigade that comes to it, christenings, a dog
//! chased across the square, the doctor's rounds and closing time at the
//! inn; the travelling fair, cricket, skating and the farming year's
//! chores out on the green; the lamplighter at dusk.
//!
//! The people here are "extras": drawn from where the event says they are,
//! with no minds of their own, so none of this can disturb the villagers.

use std::f32::consts::{PI, TAU};
use std::sync::atomic::{AtomicBool, Ordering};

use super::*;
use crate::render::boxes::MAT_LIT;

pub(super) mod figures;
pub use self::figures::*;
mod logic;
mod draw_events;
mod draw_venues;
mod draw_life;

/// `?bella` in the URL: a man called Bella lifts weights in front of a mirror in every town square.
pub static BELLA: AtomicBool = AtomicBool::new(false);

const FIRE_CHANCE: f32 = 0.07;
const CHRISTENING_CHANCE: f32 = 0.14;
const CHASE_CHANCE: f32 = 0.12;
const DOCTOR_CHANCE: f32 = 0.22;
const CLOSING_CHANCE: f32 = 0.7;
/// A fair stays this many days, and comes round this often.
const FAIR_DAYS: i64 = 2;
const FAIR_EVERY: i64 = 36;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Fire,
    Christening,
    Chase,
    Doctor,
    Closing,
}

impl Kind {
    const ALL: [Kind; 5] = [Kind::Fire, Kind::Christening, Kind::Chase, Kind::Doctor, Kind::Closing];
    fn slot(self) -> usize {
        Kind::ALL.iter().position(|k| *k == self).unwrap()
    }
}

/// One thing going on now: when it began (on the real clock, so it plays at
/// its own pace whatever the speed of the days), how long it lasts, and the
/// route the people in it take.
pub struct Active {
    pub kind: Kind,
    t0: f32,
    dur: f32,
    lot: usize,
    path: Vec<Vec2>,
    seed: u32,
}

/// Something worth pointing the camera at.
pub struct Interest {
    pub weight: f32,
    /// Which sort of thing, so the same sort isn't watched twice in a row.
    pub key: &'static str,
    pub caption: String,
    pub at: Vec2,
    pub zoom: f32,
    pub secs: f32,
    /// How high above the ground the interesting part is, so the camera can
    /// frame it rather than the ground beneath.
    pub lift: f32,
}

/// What a town will be up to on a given day, worked out in advance from the
/// same dice the town itself rolls: a window of hours, how well worth a
/// visit, and why.
pub struct Forecast {
    pub weight: f32,
    pub from: f32,
    pub to: f32,
    pub why: String,
    /// Only happens in fine weather (see `Env::nice`): weddings, christenings,
    /// the evening gathering, cricket.
    pub fine_only: bool,
    /// What the town promises the camera if it comes for this.
    pub occasion: Option<Occasion>,
}

pub fn forecast(t: &Town, day: i64, season: f32) -> Vec<Forecast> {
    let mut out = Vec::new();
    let name = &t.name;
    let mut add = |weight: f32, from: f32, to: f32, why: String, fine_only: bool, occasion: Option<Occasion>| out.push(Forecast { weight, from, to, why, fine_only, occasion });
    if (day + (t.id % 3) as i64).rem_euclid(3) == 0 {
        add(1.4 + t.city as u8 as f32 * 0.3, 8.0, 12.5, format!("Market day in {name}"), false, None);
    }
    let off = (hash_u32(t.id ^ 0xfa1) % FAIR_EVERY as u32) as i64;
    if !(2.95..4.0).contains(&season) && (day + off).rem_euclid(FAIR_EVERY) < FAIR_DAYS {
        add(1.7, 10.0, 23.0, format!("The fair is at {name}"), false, None);
    }
    if t.city {
        let roll = |k: i32, hr: i32| unit(hash3(t.id, day as i32, hr, 0x4a9 + k));
        if unit(hash3(t.id, day as i32, 3, 0xbe11)) < WEDDING_CHANCE {
            add(3.0, 13.0, WEDDING_LATEST, format!("A wedding at {name}'s church"), true, Some(Occasion::Wedding));
        }
        if roll(Kind::Christening.slot() as i32, 0) < CHRISTENING_CHANCE {
            add(3.2, 10.0, 13.0, format!("A christening at {name}"), true, Some(Occasion::Christening));
        }
    }
    if unit(hash3(t.id, day as i32, 0, 0x4a9 + Kind::Fire.slot() as i32)) < FIRE_CHANCE {
        add(2.6, 13.0, 18.0, format!("Trouble brewing in {name}"), false, Some(Occasion::Fire));
    }
    if unit(hash3(t.id, day as i32, 7, 77)) < 0.4 {
        let f = Festival::for_season(season);
        add(2.2, 19.0, 22.4, format!("{} in {name}", f.name()), true, Some(Occasion::Gathering));
    }
    if (1.0..2.0).contains(&season) && !(2.95..4.0).contains(&season) {
        add(0.5, 13.0, 18.0, format!("Cricket at {name}"), true, None);
    }
    if (2.95..3.95).contains(&season) {
        add(1.1, 12.0, 16.0, format!("A snowball fight at {name}"), false, None);
    }
    out
}

#[derive(Default)]
pub struct Scene {
    /// The day each kind of event was last rolled for (hourly, for some).
    rolled: [i64; 6],
    rolled_hour: [i32; 6],
    pub active: Vec<Active>,
    /// Houses scorched by a fire, and the day.
    soot: Vec<(usize, i64)>,
    /// What's happened here, for the chronicle.
    pub log: Vec<(i64, String)>,
    green: Option<Option<Vec2>>,
    bell_t: f32,
    music_t: f32,
    note: usize,
    cheer_t: f32,
    forced: Option<Kind>,
    /// How likely fires are just now (1 = usual): dry spells and storms.
    risk: f32,
    /// The next fire is a lightning strike.
    struck: bool,
    /// Where the extras in the current events were drawn this frame, for the
    /// camera to keep clear of buildings.
    party: std::cell::RefCell<Vec<(Kind, Vec3)>>,
}

#[derive(Clone, Copy)]
struct Garb {
    shirt: [f32; 3],
    pants: [f32; 3],
    skin: [f32; 3],
    hair: [f32; 3],
    hat: Hat,
    hat_col: [f32; 3],
    scale: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Pose {
    Walk,
    Stand,
    Cheer,
    Dig,
    Pass,
    Play,
    Skate,
    Swing,
    Kneel,
    Reach,
    /// Staggering home from the inn.
    Drunk,
    /// Curling dumbbells.
    Lift,
}

#[derive(Clone, Copy, PartialEq)]
enum Prop {
    None,
    Bag,
    Pole,
    Bat,
    Baby,
    Posy,
    Mug,
}

const BLACK: [f32; 3] = [0.1, 0.1, 0.12];
const WHITE: [f32; 3] = [0.95, 0.95, 0.92];

impl Garb {
}

thread_local! {
    /// While not 1, figures are drawn pressed flat along their facing (a reflection).
    static FLATTEN: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}

impl TownLife {
    // ---------- scheduling ----------

    // ---------- drawing ----------

    // ----- fire -----

    // ----- christening -----

    // ----- small scenes -----

    // ----- the green -----

    // ----- the year's chores, and evenings -----

}
