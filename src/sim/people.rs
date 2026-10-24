//! Villagers: homes, jobs, daily schedules, walking the street grid,
//! chatting with each other, plus their dogs and the farm chickens.

use std::rc::Rc;

use glam::{Vec2, Vec3};

use crate::render::boxes::{rgb, yaw_of, Boxes, Rig, MAT_GLOW, MAT_UNLIT};
use crate::util::*;
use crate::world::gen::{Ctx, Gen, Lot, LotKind, Stage, Town, STREET};

pub mod happenings;
pub use happenings::{forecast, Interest};
#[cfg(test)]
pub use happenings::Kind as HappeningKind;

mod setup;
mod goals;
mod routing;
mod behaviour;
mod wedding;
mod gathering;
mod visit;
pub use self::visit::{Occasion, Visit};
mod animals;
mod describe;
mod interior;
mod family;
mod queries;
mod draw;
mod draw_person;
use self::draw_person::*;
mod geometry;
use self::geometry::*;
#[cfg(test)]
mod tests;

pub struct Env<'a> {
    pub gen: &'a Gen,
    pub dt: f32,
    pub time: f32,
    pub hour: f32,
    /// Changes every in-game half hour; seeds people's choices.
    pub slot: u32,
    pub rain: f32,
    pub night: f32,
    /// In-game day number, so each evening gets its own chance of a gathering.
    pub day: i64,
    /// Fine weather for being out of doors: dry, calm, not winter.
    pub nice: bool,
    pub wind: Vec2,
    /// 0..4: spring, summer, autumn, winter.
    pub season: f32,
}

pub struct Sound {
    pub name: &'static str,
    pub pos: Vec3,
    pub vol: f32,
    /// Voice pitch for babble (1 = an average adult).
    pub pitch: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Farmer,
    Baker,
    Barkeep,
    Smith,
    Shopkeeper,
    Priest,
    Teacher,
    Librarian,
    Postie,
    Villager,
    Child,
    Elder,
}

impl Role {
    pub fn label(&self) -> &'static str {
        match self {
            Role::Farmer => "farmer",
            Role::Baker => "baker",
            Role::Barkeep => "innkeeper",
            Role::Smith => "blacksmith",
            Role::Shopkeeper => "shopkeeper",
            Role::Priest => "vicar",
            Role::Teacher => "teacher",
            Role::Librarian => "librarian",
            Role::Postie => "postie",
            Role::Villager => "villager",
            Role::Child => "child",
            Role::Elder => "elder",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Goal {
    Home,
    Work,
    Tavern,
    Errand,
    Visit,
    Plaza,
    Play,
    Bench,
    Gather,
    /// Minding a stall on market day.
    Market,
    /// Browsing the market.
    Shop,
    /// Taking the bread round to the inn.
    Deliver,
    /// Off to church for a wedding.
    Wedding,
    /// Bella's off to lift weights in the square (the `?bella` easter egg).
    Gym,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Dest {
    Lot(usize),
    Field(usize),
    Anvil(usize),
    Plaza,
    Bench(usize),
    Gather,
    Stall(usize),
    /// Where Bella lifts weights.
    Gym,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Place {
    Lot(usize),
    Plaza,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Inside,
    Walking,
    Idle,
    Working,
    Sitting,
    Chatting(usize),
    Dancing,
    Singing,
    Playing,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Emote {
    Talk,
    Heart,
    Note,
    Laugh,
}

#[derive(Clone, Copy, PartialEq)]
enum Hat {
    None,
    Straw,
    Chef,
    Cap,
    Bun,
}

#[derive(Clone, Copy)]
struct Look {
    shirt: [f32; 3],
    pants: [f32; 3],
    skin: [f32; 3],
    hair: [f32; 3],
    hat: Hat,
    hat_col: [f32; 3],
    umbrella: [f32; 3],
    scale: f32,
}

pub struct Person {
    pub name: &'static str,
    pub role: Role,
    home: usize,
    work: Option<usize>,
    pub pos: Vec2,
    pub y: f32,
    yaw: f32,
    path: Vec<Vec2>,
    pi: usize,
    speed: f32,
    phase: f32,
    /// The easter egg's weightlifter.
    pub bella: bool,
    /// Where this walker was a moment ago, and how long they've barely moved;
    /// held up that long, they push through for a while.
    jam_pos: Vec2,
    jam_t: f32,
    ghost_t: f32,
    state: State,
    resume: State,
    goal: Goal,
    dest: Dest,
    at: Place,
    timer: f32,
    chat_cd: f32,
    emote: Emote,
    chat_start: f32,
    first_speaker: bool,
    talk_slot: i32,
    friend: Option<usize>,
    look: Look,
    jitter: f32,
    lane: f32,
    pub visible: bool,
    /// 0..1, eases towards `visible` so people dissolve in and out of doors.
    fade: f32,
    /// Part played at an evening gathering, and where they are round the ring.
    gather_role: GatherRole,
    dance_a: f32,
    /// Distance from the plaza centre while dancing; eases onto the ring.
    dance_r: f32,
    /// Closest this walker has got to their next waypoint, and how long
    /// they've made no headway, so they don't strain forever at a crowd.
    stall_best: f32,
    stall_t: f32,
    /// Smoothed walking direction, so steering round people doesn't flick.
    heading: Vec2,
    /// Said they'd come to tonight's gathering.
    coming: bool,
    /// What they're carrying.
    carry: Carry,
    /// The day the baker last took bread to the inn.
    delivered: i64,
    /// Part in today's wedding, if any.
    wed: Wed,
    married: bool,
    /// The day they were born, for anyone born since the town was founded.
    born: Option<i64>,
    /// Parents of a baby under a week old.
    new_baby: bool,
    work_t: f32,
    seed: u32,
}

impl Person {
    pub fn pos3(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.y, self.pos.y)
    }
    /// Which trade this is, 1 to 12, for giving their voice a character.
    pub fn role_code(&self) -> f32 {
        match self.role {
            Role::Farmer => 1.0,
            Role::Baker => 2.0,
            Role::Barkeep => 3.0,
            Role::Smith => 4.0,
            Role::Shopkeeper => 5.0,
            Role::Priest => 6.0,
            Role::Teacher => 7.0,
            Role::Librarian => 8.0,
            Role::Postie => 9.0,
            Role::Villager => 10.0,
            Role::Child => 11.0,
            Role::Elder => 12.0,
        }
    }

    /// This person's voice pitch: children high, elders low, everyone a
    /// little different.
    pub fn voice(&self) -> f32 {
        let k = unit(hash_u32(self.seed ^ 0x701c));
        match self.role {
            Role::Child => 1.45 + k * 0.3,
            Role::Elder => 0.72 + k * 0.15,
            _ => 0.82 + k * 0.4,
        }
    }

    /// The day a child is grown up.
    fn grown_day(&self, first_day: i64) -> i64 {
        if let Some(b) = self.born {
            return b + family::BABY_DAYS + 6 + (hash_u32(self.seed ^ 0x6a0) % 14) as i64;
        }
        first_day + 4 + (hash_u32(self.seed ^ 0x6a0) % 40) as i64
    }

    /// Some parents push a pram when they're out and about.
    fn pram(&self) -> bool {
        self.role == Role::Villager && (self.new_baby || unit(hash_u32(self.seed ^ 0xba6)) < 0.08)
    }

    /// Seconds left in this person's current conversation (0 if not chatting).
    pub fn chat_left(&self) -> f32 {
        if self.chatting_with().is_some() {
            self.timer.max(0.0)
        } else {
            0.0
        }
    }
    /// Who this person is chatting with, if anyone.
    pub fn chatting_with(&self) -> Option<usize> {
        match self.state {
            State::Chatting(o) if self.visible => Some(o),
            _ => None,
        }
    }
}

struct Dog {
    owner: usize,
    pos: Vec2,
    yaw: f32,
    phase: f32,
    col: [f32; 3],
    target: Vec2,
    home: Vec2,
    bark_t: f32,
    moving: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum CatMode {
    Walk,
    Sit,
    Sleep,
}

/// A town cat: keeps to the streets near its home, naps in the sun, goes
/// out prowling at night and sits out the rain by the door.
struct Cat {
    pos: Vec2,
    yaw: f32,
    phase: f32,
    col: [f32; 3],
    /// 0 plain, 1 tuxedo (white muzzle and paws), 2 tabby stripes.
    pattern: u8,
    target: Vec2,
    home: Vec2,
    mode: CatMode,
    timer: f32,
    meow_t: f32,
}

struct Chicken {
    pos: Vec2,
    yaw: f32,
    target: Vec2,
    lot: usize,
    timer: f32,
    peck: f32,
    cluck_t: f32,
}

pub struct TownLife {
    pub town: Rc<Town>,
    lots: Rc<Vec<Lot>>,
    pub people: Vec<Person>,
    /// Where a hammer came down on an anvil this frame, for the sparks.
    pub strikes: Vec<Vec3>,
    dogs: Vec<Dog>,
    cats: Vec<Cat>,
    chickens: Vec<Chicken>,
    ctx: Ctx,
    y: f32,
    rng: Rng,
    tavern: Option<usize>,
    shops: Vec<usize>,
    homes: Vec<usize>,
    church: Option<usize>,
    last_hour: i32,
    /// Tonight's gathering in the plaza, if there is one.
    pub gathering: Option<Gathering>,
    today: i64,
    crier_t: f32,
    /// The children's game in the plaza, if any.
    game: Option<Game>,
    gather_day: i64,
    /// What the camera has said it's coming to see here.
    pub visit: Option<Visit>,
    pub wedding: Option<Wedding>,
    wedding_day: i64,
    /// Couples married here since we've been watching.
    married_today: Vec<(usize, usize)>,
    /// Market stalls stand in city plazas, so dancers there dance on the spot.
    stalls: bool,
    /// The house the camera has lifted the roof off, to look in on whoever's at home.
    peek: Option<usize>,
    /// How the harvest has gone: 1 is ordinary, more is plenty, less is lean.
    plenty: f32,
    /// Fires, fairs and other goings-on.
    pub happen: happenings::Scene,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GameKind {
    Tag,
    Ball,
    Skipping,
}

/// Children's game in the plaza; a new one each half hour.
struct Game {
    kind: GameKind,
    slot: u32,
    /// Tag: who's "it".
    it: usize,
    /// Ball: thrown from, to, and how far through the throw.
    ball: (usize, usize, f32),
    /// Skipping: the rope's turn.
    rope: f32,
    retarget: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Carry {
    Nothing,
    /// Shopping from the market.
    Basket,
    /// Loaves for the inn.
    Bread,
    /// Out with the baby.
    Pram,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Wed {
    None,
    Bride,
    Groom,
    Guest,
}

/// The ceremony is at three; the party set off for church at one (an hour
/// is only 25 seconds, and it's a walk across town).
const WEDDING_IN: f32 = 13.0;
const WEDDING_VOWS: f32 = 14.2;
const WEDDING_END: f32 = 16.4;
/// A wedding the camera has been promised can still be called this late: the guests
/// go straight to church and the vows follow, leaving the party its evening.
pub(crate) const WEDDING_LATEST: f32 = 15.2;
const WEDDING_CHANCE: f32 = 0.14;
/// The couple dance their own small circle inside everyone else's.
const COUPLE_RING: f32 = 4.7;

/// A wedding: the couple and guests in church, then bells, petals and a
/// dance in the plaza.
pub struct Wedding {
    pub couple: (usize, usize),
    day: i64,
    /// The party's been told to set off for church.
    called: bool,
    /// When they came out married.
    pub vows: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GatherRole {
    None,
    Dancer,
    Singer,
    Musician,
}

/// An evening gathering in the plaza: a circle dance, singing and a musician.
pub struct Gathering {
    musician: Option<usize>,
    joined: u32,
    /// How many villagers said they'd come.
    pub expected: usize,
    /// Which way the circle dance turns.
    spin: f32,
    pub festival: Festival,
}

/// Each season's gathering has its own character.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Festival {
    /// Spring: dancing round a maypole, ribbons streaming.
    Maypole,
    /// Summer: bunting and a jig.
    Midsummer,
    /// Autumn: hay bales, pumpkins and autumn-coloured bunting.
    Harvest,
    /// Winter: a bonfire, with lanterns strung over the square.
    Bonfire,
    /// A wedding party, in the afternoon.
    Wedding,
}

impl Festival {
    pub fn for_season(season: f32) -> Festival {
        [Festival::Maypole, Festival::Midsummer, Festival::Harvest, Festival::Bonfire][season as usize % 4]
    }
    pub fn name(self) -> &'static str {
        match self {
            Festival::Maypole => "Maypole dance",
            Festival::Midsummer => "Midsummer dance",
            Festival::Harvest => "Harvest festival",
            Festival::Bonfire => "Winter bonfire",
            Festival::Wedding => "Wedding",
        }
    }
}

/// A gathering is only called if at least this many villagers will come.
pub const MIN_TURNOUT: usize = 5;

/// Chance that a town holds a gathering on a fine evening.
const GATHER_CHANCE: f32 = 0.4;
/// Gatherings run between these hours.
const GATHER_HOURS: std::ops::Range<f32> = 19.0..22.4;
const DANCE_RING: f32 = 7.0;
/// Room each circle dancer keeps round the ring.
const DANCE_SPACING: f32 = 1.1;
/// Walkers look this far ahead to steer round people...
const LOOK_AHEAD: f32 = 2.0;
/// ...who are within this distance either side of their path.
const PASS_WIDTH: f32 = 0.9;
/// A walker this close to a waypoint who's made no headway for this long
/// counts as having got there.
const STALL_RADIUS: f32 = 1.5;
const STALL_TIME: f32 = 1.2;
/// How far apart two people stand to chat.
const CHAT_DISTANCE: f32 = 1.0;
const SING_RING: f32 = 8.4;

const SHIRTS: [[u8; 3]; 12] = [
    [200, 70, 60], [70, 110, 170], [90, 150, 90], [220, 180, 70], [150, 90, 160], [240, 240, 230],
    [210, 120, 60], [60, 140, 150], [180, 60, 100], [120, 100, 80], [100, 160, 200], [230, 150, 160],
];
const PANTS: [[u8; 3]; 6] = [[60, 60, 90], [90, 70, 50], [50, 50, 55], [120, 100, 70], [70, 90, 120], [110, 60, 60]];
const SKIN: [[u8; 3]; 6] = [[248, 214, 180], [234, 190, 150], [200, 150, 110], [160, 110, 75], [120, 80, 55], [250, 220, 200]];
const HAIR: [[u8; 3]; 7] = [[60, 40, 30], [30, 25, 25], [160, 100, 50], [220, 190, 120], [190, 80, 40], [100, 70, 50], [80, 80, 90]];
const UMBRELLA: [[u8; 3]; 5] = [[220, 60, 70], [70, 120, 200], [240, 200, 60], [90, 170, 100], [160, 90, 180]];

fn col(c: [u8; 3]) -> [f32; 3] {
    rgb(c[0], c[1], c[2])
}

/// How fast people dissolve into and out of doorways (full fade in 0.6 s).
const FADE_RATE: f32 = 1.7;

/// Half-size of the square walkers keep out of around the plaza fountain
/// (the basin is 5x5 voxels, plus room for a body).
const FOUNTAIN_CLEAR: f32 = 3.0;
/// Half-size of the square whose corners detours go through.
const FOUNTAIN_DETOUR: f32 = 3.5;

impl TownLife {
    // ---------- schedule ----------

    // ---------- routing on the street grid ----------

    // ---------- per-frame update ----------

    // ---------- drawing ----------

}

impl TownLife {
}
