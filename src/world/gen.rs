//! The world planner: a pure function of the seed that answers "what is at
//! (x, z)?". Terrain, biomes, rivers, towns, roads and railways are all
//! derived here, so chunks can be generated in any order and still agree.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use glam::Vec2;

use super::blocks::*;
use crate::names;
use crate::util::*;

mod rail;
pub use self::rail::*;
mod canals;
mod sites;
pub use self::sites::*;
mod weirs;
pub use self::weirs::*;
mod towns;
pub use self::towns::*;
mod terrain;
pub use self::terrain::*;

pub const SEA: i32 = 12;
/// How far from a road cell over water we look for the banks.
pub const BANK_CAP: i32 = 12;
/// A road crossing water this many cells wide or more is a ferry rather than a bridge.
pub const FERRY_SPAN: i32 = 10;
/// The longest a ferry crossing can be, landing to landing; water beyond that is left to a bridge.
pub const FERRY_MAX: i32 = 30;
/// The road may stray no further than this from the line between a ferry's
/// landings, else it's winding along a river and keeps its bridge.
pub const FERRY_STRAY: f32 = 1.5;
/// How far from a ferry's line a cell of its road can be: the road's wander
/// (`FERRY_STRAY`) plus its half-width and a little for the diagonal.
pub const FERRY_ROAD_HALF: f32 = FERRY_STRAY + 1.9;
/// A ski resort's premade run: a graded corridor `SKI_HALF_LEN` either way
/// along the slope from the site's centre and `SKI_HALF_WIDTH` either way
/// across it, laid over the land and blended into it for `SKI_BLEND` more.
pub const SKI_HALF_LEN: f32 = 42.0;
pub const SKI_HALF_WIDTH: f32 = 17.0;
pub const SKI_BLEND: f32 = 9.0;
pub const REGION: i32 = 192;
/// How wide a river is, in terms of the river noise.
const RIVER_WIDTH: f32 = 0.022;
/// Share of regions with a river through them that have a mill on it.
const MILL_CHANCE: f32 = 0.6;
pub const STREET: i32 = 11;
pub const MAX_H: i32 = 54;
/// How many directions railways run in, and how far apart parallel ones lie.
const RAIL_FAMILIES: u32 = 5;
const RAIL_SPACING: i32 = 1800;
/// How far a line's course is nudged off its even spacing.
const RAIL_JITTER: i32 = 260;
const RAIL_NODE: i32 = 96;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
    Tundra,
    Mountain,
    /// Hot and wet: tall broadleaf trees, palms and thick undergrowth.
    Jungle,
}

/// Railways pass at least this far from a town's centre, clear of its plaza.
const PLAZA_RAIL_CLEAR: f32 = 18.0;

/// How far a line can wander either side of its straight course.
pub const RAIL_BEND_MAX: f32 = 60.0;


/// A railway this far above the ground crosses it on a viaduct...
const VIADUCT_RISE: f32 = 4.5;
/// ...and this far below it goes through a tunnel.
const TUNNEL_DEPTH: f32 = 7.5;

/// Canals run east-west, dug level through low-lying land.
const CANAL_SPACING: i32 = 250;
/// Canals only exist where the land is no higher than this above the sea.
const CANAL_MAX_RISE: f32 = 8.0;
/// Canals are judged in samples this far apart, cached in tiles this wide.
const CANAL_STEP: i32 = 1;
const CANAL_TILE: i32 = 64;
/// A canal joins two stretches of open water through low land, between
/// these lengths; anything else isn't dug.
const CANAL_MIN_RUN: i32 = 40;
const CANAL_MAX_RUN: i32 = 2400;

/// Chance that a crossing of two lines gets a junction chord between them.
const JUNCTION_CHANCE: f32 = 0.75;
/// How far before and after a crossing a chord leaves and joins the lines.
pub const JUNCTION_REACH: f32 = 40.0;
/// Away from junctions a line may have a passing loop: one per this much
/// line, with this chance, this long, and this far off the main track.
const LOOP_PERIOD: i32 = 420;
const LOOP_CHANCE: f32 = 0.8;
const LOOP_LEN: f32 = 170.0;
const LOOP_OFFSET: f32 = 6.5;







pub struct Gen {
    pub seed: u32,
    towns: RefCell<HashMap<(i32, i32), Option<Rc<Town>>>>,
    lots: RefCell<HashMap<u32, Rc<Vec<Lot>>>>,
    rail_nodes: RefCell<HashMap<(i32, i32), f32>>,
    chords: RefCell<HashMap<((u16, i32), (u16, i32)), Option<Rc<Chord>>>>,
    loops: RefCell<HashMap<(RailLine, i32), Option<Rc<Chord>>>>,
    greens: RefCell<HashMap<u32, Option<(i32, i32)>>>,
    sites: RefCell<HashMap<(i32, i32), Rc<Vec<Rc<Site>>>>>,
    weirs: RefCell<HashMap<(i32, i32), Rc<Vec<Weir>>>>,
    /// Per canal and tile along it: the lie of the land at each sample, and
    /// whether the canal is dug there.
    canal_land: RefCell<HashMap<(i32, i32), Rc<Vec<u8>>>>,
    canal_dug: RefCell<HashMap<(i32, i32), Rc<Vec<bool>>>>,
    /// Road bridge cells: (deck level, distance to the nearest bank).
    bridges: RefCell<HashMap<(i32, i32), (i32, i32)>>,
    /// Ferry crossings (landing to landing) on each town-to-town road, and
    /// which road-over-water cells lie on one.
    ferry_runs: RefCell<HashMap<(u32, u32), Rc<Vec<(Vec2, Vec2)>>>>,
    ferry_cells: RefCell<HashMap<(i32, i32), Option<f32>>>,
    /// Today, and the day the world was first seen: a few houses are still
    /// going up then, and are finished over the days that follow.
    pub day: Cell<i64>,
    pub first_day: Cell<i64>,
}

/// How far along a new house is.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stage {
    /// Nothing yet: bare ground where a house will go.
    Vacant,
    /// Pegged out, with materials piled ready.
    Plot,
    /// Walls going up inside scaffolding, 0..1.
    Frame(f32),
    Done,
    /// Abandoned and crumbling, ivy taking the walls.
    Ruin,
}

/// A house takes this many days from plot to moving in.
pub const BUILD_DAYS: i64 = 3;
/// Share of houses still being built when the world is first seen.
const NEW_BUILD_CHANCE: u32 = 22;
/// Share of houses that are abandoned some day.
const DECAY_CHANCE: u32 = 6;
/// New houses go up over this many days.
const GROWTH_DAYS: i64 = 70;

pub fn div_floor(a: i32, b: i32) -> i32 {
    a.div_euclid(b)
}

impl Gen {
    pub fn new(seed: u32) -> Self {
        Gen {
            seed,
            towns: RefCell::new(HashMap::new()),
            lots: RefCell::new(HashMap::new()),
            rail_nodes: RefCell::new(HashMap::new()),
            chords: RefCell::new(HashMap::new()),
            loops: RefCell::new(HashMap::new()),
            greens: RefCell::new(HashMap::new()),
            sites: RefCell::new(HashMap::new()),
            weirs: RefCell::new(HashMap::new()),
            canal_land: RefCell::new(HashMap::new()),
            canal_dug: RefCell::new(HashMap::new()),
            bridges: RefCell::new(HashMap::new()),
            ferry_runs: RefCell::new(HashMap::new()),
            ferry_cells: RefCell::new(HashMap::new()),
            day: Cell::new(0),
            first_day: Cell::new(0),
        }
    }

    /// How far along a lot's building is today.
    pub fn stage(&self, lot: &Lot) -> Stage {
        lot.stage(self.first_day.get(), self.day.get())
    }

    // ---------- terrain ----------

    // ---------- railways ----------

    // ---------- canals ----------

    // ---------- water near towns ----------

    // ---------- countryside sites ----------

    // ---------- junctions ----------

    // ---------- towns ----------

    /// How far round a town's green the ground is kept clear of trees and
    /// countryside sites.
    pub const GREEN_R: f32 = 14.0;

    // ---------- columns ----------

    // ---------- lots ----------

}

#[cfg(test)]
mod biome_tests {
    use super::*;

    #[test]
    fn jungles_and_warm_styles_turn_up() {
        let (mut jungle, mut med, mut tropical) = (0, 0, 0);
        for seed in 1..=12u32 {
            let gen = Gen::new(seed);
            for k in 0..900 {
                let (x, z) = ((k % 30 - 15) as f32 * 120.0, (k / 30 - 15) as f32 * 120.0);
                let h = gen.raw_height(x, z);
                if h > SEA as f32 && gen.biome(x, z, h) == Biome::Jungle {
                    jungle += 1;
                }
            }
            for rx in -6..=6 {
                for rz in -6..=6 {
                    if let Some(t) = gen.town(rx, rz) {
                        match t.style {
                            Style::Mediterranean => med += 1,
                            Style::Tropical => tropical += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        println!("jungle samples {jungle}, mediterranean towns {med}, tropical towns {tropical}");
        assert!(jungle > 50 && med > 0 && tropical > 0);
    }
}
