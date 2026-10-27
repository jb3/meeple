//! Ties world, simulation, camera director, renderer, HUD and audio together.

use std::collections::{HashMap, HashSet, VecDeque};

use glam::{Mat4, Vec2, Vec3};
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, WebGl2RenderingContext};

use crate::camera::Camera;
use crate::quality::{tier_name, Quality, MAX_TIER};
use crate::render::boxes::Boxes;
use crate::render::{Frame, Renderer};
use crate::sim::clock::Clock;
use crate::sim::stations::Stations;
use crate::sim::fauna::Fauna;
use crate::sim::fx::{Fx, FxEnv};
use crate::sim::people::{forecast, Env, Interest, Occasion, Sound, TownLife};
use crate::sim::skies::{self, Skies};
use crate::sim::wonders::Wonders;
use crate::sim::traffic::{Emit, Traffic};
use crate::sim::travellers::Travellers;
use crate::sim::weather::Weather;
use crate::util::*;
use crate::world::chunk::CS;
use crate::world::gen::{Biome, SiteKind, Style, SEA};
use crate::world::World;

mod input;
mod hud;
mod director;
mod story;
use self::story::Story;
mod frame;
mod skyshow;
mod persist;
use self::persist::Viewer;
use self::skyshow::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = meepleAudio, js_name = update, catch)]
    fn audio_update(params: &[f32], events: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace = meepleUi, js_name = snap, catch)]
    fn save_picture() -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace = meepleUi, js_name = record, catch)]
    fn toggle_recording() -> Result<bool, JsValue>;
    #[wasm_bindgen(js_namespace = meepleUi, js_name = recording, catch)]
    fn is_recording() -> Result<bool, JsValue>;
}

/// Places hand-picked as good starting worlds (by the seed hunt in the tests).
const GALLERY: [u32; 8] = [535, 192, 148, 279, 316, 89, 162, 7];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    Travel,
    Linger,
    FollowTrain,
    FollowPerson,
    /// Drifting alongside a narrowboat (canal in follow_canal).
    FollowBoat,
    /// Walking with travellers on the road (party in follow_party).
    FollowTraveller,
    WatchChat,
    /// Committed to watching one evening gathering (town id in follow_town).
    Gathering,
    /// Something happening in a town: a fire, a fair, a christening...
    Event,
}

/// How the camera treats a place it has arrived at.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Shot {
    /// Circle slowly.
    Orbit,
    /// Start wide and push in.
    Push,
    /// Hold a low, steady angle.
    Hold,
}

/// A place the autopilot might head for, and why it might.
struct Cand {
    pos: Vec2,
    id: u32,
    kind: &'static str,
    weight: f32,
    why: String,
    /// The hour what it's going for begins, if we'd arrive before then.
    wait: Option<f32>,
    /// What the town promises if we come (see `Promise`).
    promise: Option<Promise>,
}

/// The autopilot has told a town it's coming for an occasion; the town makes
/// sure it happens and holds its start until the camera is there.
#[derive(Clone, Copy)]
struct Promise {
    town: u32,
    occasion: Occasion,
    day: i64,
    /// The hour the town stops holding the occasion back, and when it ends.
    until: f32,
    to: f32,
}

struct Director {
    mode: Mode,
    dest: Vec2,
    center: Vec2,
    timer: f32,
    vel: Vec2,
    /// Where the autopilot is turning the view to, and when it next picks.
    yaw_goal: f32,
    yaw_t: f32,
    zoom_target: f32,
    auto: bool,
    manual_t: f32,
    heading: Vec2,
    visited: VecDeque<u32>,
    follow_town: u32,
    follow_idx: usize,
    /// Id of the train being followed.
    follow_train: Option<u32>,
    train_cd: f32,
    boat_cd: f32,
    follow_canal: i32,
    follow_party: u32,
    party_cd: f32,
    /// The multi-day story the autopilot is following, if any.
    story: Option<story::Story>,
    person_cd: f32,
    /// Already looked in on someone since setting off from the last place.
    checked_in: bool,
    /// Set off or arrived this recently: nothing minor will turn the camera.
    commit_t: f32,
    chat_b: usize,
    chat_cd: f32,
    /// What the autopilot's heading for, and why.
    reason: String,
    await_hour: Option<f32>,
    /// The sorts of place visited lately.
    kinds: VecDeque<&'static str>,
    /// Events already watched: (town, sort, day).
    watched: Vec<(u32, &'static str, i64)>,
    event_key: &'static str,
    /// The sort of event watched last, so the autopilot never goes from one
    /// fair straight to another.
    last_key: &'static str,
    event_weight: f32,
    /// Villagers it's grown fond of: (town, person seed).
    faves: VecDeque<(u32, u32)>,
    shot: Shot,
    /// Trips since the last long one, and whether this one's a long one.
    legs: u32,
    expedition: bool,
    linger_total: f32,
    pitch_target: f32,
    promise: Option<Promise>,
    /// Seconds left holding the view while a wedding party is wound up.
    wrap_t: f32,
    /// The train being followed was clicked on: follow it for good (until let go).
    follow_pinned: bool,
    /// The rider turned the view with Q or E while on a train: keep it so.
    yaw_locked: bool,
    /// How far the view still has to pan, and how much of that pan it may make
    /// yet (0 until it has zoomed out enough to make it).
    pan_dist: f32,
    pan_gate: f32,
    /// Waiting on a wedding by staying with the couple (for the see-through).
    wed_wait: bool,
    /// Where the ghost is, while the camera's on it (for the see-through).
    ghost_focus: Option<Vec3>,
    /// The ruin the best ghost angle was worked out for, and that angle.
    ghost_view: Option<((i32, i32), f32)>,
    rng: Rng,
}

#[derive(Default)]
pub struct Input {
    pub drag: Option<(f64, f64)>,
    /// Where the mouse went down, to tell a click from a drag.
    pub down: Option<(f64, f64)>,
    pub keys: HashSet<String>,
    pub pan: Vec2,
    pub wheel: f32,
    pub rotate: f32,
    pub moved_t: f32,
}

pub struct App {
    r: Renderer,
    world: World,
    cam: Camera,
    clock: Clock,
    stations: Stations,
    weather: Weather,
    towns: HashMap<u32, TownLife>,
    traffic: Traffic,
    travellers: Travellers,
    fauna: Fauna,
    skies: Skies,
    wonders: Wonders,
    fx: Fx,
    solid: Boxes,
    fxb: Boxes,
    time: f32,
    dir: Director,
    pub input: Input,
    doc: Document,
    hud_t: f32,
    hud_hidden: bool,
    quality: Quality,
    /// The tier last announced on screen, and seconds left on the badge.
    quality_shown: u32,
    badge_t: f32,
    hud_follow: String,
    css: (f64, f64),
    last_hour: i32,
    /// Set once the first view has finished loading.
    settled: bool,
    sky: SkyShow,
    /// Gatherings (town id, day) the autopilot has already watched.
    seen_gatherings: Vec<(u32, i64)>,
    pub seed: u32,
    /// The villager whose card is showing (town id, index), followed until
    /// let go.
    card: Option<(u32, usize)>,
    news_on: bool,
    news: Option<(String, Vec<String>)>,
    map_on: bool,
    map_t: f32,
    /// A postcard moment: the camera holds while a caption shows.
    postcard_t: f32,
    postcard_cd: f32,
    toast_t: f32,
    overlay_t: f32,
    /// What's worth watching in the loaded towns, this frame.
    interests: Vec<(u32, Interest)>,
    /// How swollen the rivers are after long heavy rain, 0..1.
    flood: f32,
    /// How dry the valley is after a long spell without rain, 0..1.
    drought: f32,
    prev_flash: f32,
    /// What's happened in the valley, for the chronicle.
    chronicle: Vec<(i64, String)>,
    log_seen: HashMap<u32, usize>,
    noted: HashSet<(u32, i64)>,
    chron_on: bool,
    firework_t: f32,
    last_era: Option<u8>,
    /// Photograph mode: no HUD, time stopped, a tilt-shift blur.
    photo: bool,
    photo_prev: (bool, f64),
    snap: bool,
    /// 0..1: how far the buildings between the camera and the people being
    /// watched have faded to see-through.
    see_through: f32,
    sail_angle: f32,
    sail_rate: f32,
    /// The half-hour the church clock last struck.
    chime_slot: i32,
    /// Where the rainbow stands in the world while there is one: a point on
    /// the ground and the horizontal direction its plane faces.
    bow: Option<(Vec3, Vec2, f32)>,
    viewer: Viewer,
    /// Looking in on houses: close in and the roof comes off the nearest one with someone home (I turns it off).
    peek_on: bool,
    /// `?prof`: where the frame's CPU time goes, logged every few seconds.
    prof: Option<Prof>,
}

/// Running totals of the milliseconds each part of a frame has taken.
#[derive(Default)]
pub struct Prof {
    last: f64,
    totals: Vec<(&'static str, f64)>,
    frames: u32,
    logged: f64,
}

impl Prof {
    pub fn start(&mut self, now: f64) {
        self.last = now;
        if self.logged == 0.0 {
            self.logged = now;
        }
    }

    pub fn mark(&mut self, name: &'static str, now: f64) {
        let d = now - self.last;
        self.last = now;
        match self.totals.iter_mut().find(|t| t.0 == name) {
            Some(t) => t.1 += d,
            None => self.totals.push((name, d)),
        }
    }

    /// Every five seconds: the average milliseconds per frame in each part.
    pub fn report(&mut self, now: f64) {
        self.frames += 1;
        if now - self.logged < 5000.0 {
            return;
        }
        let n = self.frames.max(1) as f64;
        let line: Vec<String> = self.totals.iter().map(|(k, v)| format!("{k} {:.2}", v / n)).collect();
        web_sys::console::log_1(&format!("meeple prof (ms/frame over {} frames): {}", self.frames, line.join(", ")).into());
        self.totals.clear();
        self.frames = 0;
        self.logged = now;
    }
}

/// How long a postcard moment holds, and the least time between them.
const POSTCARD_HOLD: f32 = 10.0;
const POSTCARD_GAP: f32 = 150.0;

/// Camera zoom while villagers make their way to a gathering, and once
/// the dancing's under way.
/// The autopilot frames everything this much closer than the zoom targets
/// below say, so the director's numbers stay as written.
const TIGHTER: f32 = 0.85;
/// A postcard looks out at this angle above the horizon (radians), against 0.4-0.7 for ordinary shots.
const POSTCARD_PITCH: f32 = 0.24;
/// The tower is 17 voxels tall: aim this far up it, and keep at least this much in frame.
const LIGHTHOUSE_LIFT: f32 = 8.0;
const LIGHTHOUSE_ZOOM: f32 = 24.0;
/// The furthest out the camera will go (the autopilot's own shots stay well inside).
const MAX_ZOOM: f32 = 130.0;
const GATHER_WIDE: f32 = 19.0;
const GATHER_CLOSE: f32 = 12.5;

fn seasonal(s: f32, keys: [Vec3; 4]) -> Vec3 {
    let t = (s - 0.5).rem_euclid(4.0);
    let i = t.floor() as usize;
    let f = smoothstep(0.0, 1.0, t.fract());
    keys[i % 4].lerp(keys[(i + 1) % 4], f)
}

impl App {
    pub fn new(canvas: HtmlCanvasElement, gl: WebGl2RenderingContext, doc: Document, opts: &Opts) -> Result<App, String> {
        crate::sim::people::happenings::BELLA.store(opts.bella, std::sync::atomic::Ordering::Relaxed);
        let mut r = Renderer::new(gl, canvas)?;
        r.pixel = opts.pixel;
        let world = World::new(opts.seed);
        let seed = opts.seed;

        // start next to the town nearest the origin
        let mut start = Vec2::ZERO;
        let mut first_town = 0;
        'search: for ring in 0i32..8 {
            for rx in -ring..=ring {
                for rz in -ring..=ring {
                    if rx.abs() != ring && rz.abs() != ring {
                        continue;
                    }
                    if let Some(t) = world.gen.town(rx, rz) {
                        start = Vec2::new(t.cx as f32, t.cz as f32);
                        first_town = t.id;
                        break 'search;
                    }
                }
            }
        }
        if let Some((x, z)) = opts.at {
            start = Vec2::new(x, z);
        }
        let ground = world.gen.raw_height(start.x, start.y).max(SEA as f32);
        let cam = Camera {
            target: Vec3::new(start.x, ground, start.y),
            yaw: opts.view.map_or(std::f32::consts::FRAC_PI_4, |v| v.0),
            pitch: opts.pitch.unwrap_or(0.62),
            zoom: opts.view.map_or(22.0, |v| v.1),
            aspect: 1.6,
        };
        let season_start = opts.season.unwrap_or_else(|| (hash_u32(seed) % 4) as f64);
        let mut viewer = Viewer::default();
        let mut days = (season_start * crate::sim::clock::DAYS_PER_SEASON).floor() + opts.hour / 24.0;
        if let Some(t) = opts.t {
            days = t;
        } else if opts.resume {
            if let Some((d, note)) = persist::resume(seed, opts.day_len) {
                days = d;
                viewer.welcome = note;
            }
        }
        let clock = Clock {
            days,
            day_len: opts.day_len,
            speed: 1.0,
        };
        world.gen.first_day.set(0);
        world.gen.day.set(clock.day());
        let mut rng = Rng::new(seed as u64 ^ 0xd1ec);
        let a = rng.range(0.0, 6.28);
        let mut visited = VecDeque::new();
        visited.push_back(first_town);
        Ok(App {
            r,
            world,
            cam,
            clock,
            weather: Weather::new(seed),
            towns: HashMap::new(),
            traffic: Traffic::new(seed),
            travellers: Travellers::new(seed),
            fauna: Fauna::new(seed),
            skies: Skies::new(seed),
            wonders: Wonders::default(),
            stations: Stations::default(),
            fx: Fx::new(seed),
            solid: Boxes::default(),
            fxb: Boxes::default(),
            time: 0.0,
            dir: Director {
                mode: Mode::Linger,
                dest: start,
                center: start,
                timer: 50.0,
                vel: Vec2::ZERO,
                yaw_goal: std::f32::consts::FRAC_PI_4,
                yaw_t: 12.0,
                zoom_target: 20.0,
                auto: opts.auto,
                manual_t: 0.0,
                heading: Vec2::new(a.cos(), a.sin()),
                visited,
                follow_town: 0,
                follow_idx: 0,
                follow_train: None,
                train_cd: 30.0,
                boat_cd: 20.0,
                follow_canal: 0,
                follow_party: 0,
                party_cd: 30.0,
                story: None,
                person_cd: 12.0,
                checked_in: false,
                commit_t: 0.0,
                chat_b: 0,
                chat_cd: 5.0,
                reason: String::new(),
                await_hour: None,
                kinds: VecDeque::new(),
                watched: Vec::new(),
                event_key: "",
                last_key: "",
                event_weight: 0.0,
                faves: VecDeque::new(),
                shot: Shot::Orbit,
                legs: 0,
                expedition: false,
                linger_total: 60.0,
                pitch_target: opts.pitch.unwrap_or(0.62),
                promise: None,
                wrap_t: 0.0,
                follow_pinned: false,
                yaw_locked: false,
                pan_dist: 0.0,
                pan_gate: 1.0,
                wed_wait: false,
                ghost_focus: None,
                ghost_view: None,
                rng,
            },
            input: Input::default(),
            doc,
            hud_t: 8.0,
            quality: {
                let mut q = Quality::new(opts.quality.0, opts.quality.1);
                q.off = opts.fx_off.clone();
                q
            },
            quality_shown: opts.quality.0.min(MAX_TIER),
            badge_t: 0.0,
            hud_hidden: false,
            hud_follow: String::new(),
            card: None,
            news_on: false,
            news: None,
            map_on: false,
            map_t: 0.0,
            postcard_t: 0.0,
            postcard_cd: 30.0,
            toast_t: 0.0,
            overlay_t: 0.0,
            interests: Vec::new(),
            flood: 0.0,
            drought: 0.0,
            prev_flash: 0.0,
            chronicle: Vec::new(),
            log_seen: HashMap::new(),
            noted: HashSet::new(),
            chron_on: false,
            firework_t: 0.0,
            last_era: None,
            photo: false,
            photo_prev: (false, 1.0),
            snap: false,
            see_through: 0.0,
            sail_angle: 0.0,
            sail_rate: 0.3,
            chime_slot: i32::MIN,
            bow: None,
            viewer,
            peek_on: true,
            prof: opts.prof.then(Prof::default),
            css: (1280.0, 800.0),
            last_hour: -1,
            settled: false,
            sky: SkyShow { force_aurora: opts.aurora, ..Default::default() },
            seen_gatherings: Vec::new(),
            seed,
        })
    }

    // ---------- input ----------

    // ---------- camera director ----------

    // ---------- frame ----------

    /// Minimum ms between rendered frames, for the main loop's frame cap.
    pub fn min_frame_ms(&self) -> f64 {
        self.quality.min_frame_ms()
    }

    pub fn apply_quality(&mut self) {
        let q = &self.quality;
        self.r.set_shadows(q.shadow_size(), q.shadow_taps());
        self.r.set_hdr(q.hdr());
        self.r.set_fx(q.fx());
        self.r.boost = q.extra_pixel();
        self.r.refit();
        let tier = q.tier;
        web_sys::console::log_1(&format!("meeple: quality tier {tier} ({})", tier_name(tier)).into());
        // announce a change in the corner: which way, and to what
        if tier != self.quality_shown {
            let arrow = if tier > self.quality_shown { "\u{25bc}" } else { "\u{25b2}" };
            self.set_text("quality", &format!("{arrow} Quality: {}", tier_name(tier)));
            self.remove_class("quality", "hidden");
            self.add_class("quality", "on");
            self.badge_t = 5.0;
            self.quality_shown = tier;
        }
    }

    // ---------- on-screen extras ----------

}

impl SkyShow {
}

pub struct Opts {
    pub seed: u32,
    pub pixel: u32,
    /// Starting quality tier, and whether to adapt it to frame times.
    pub quality: (u32, bool),
    /// Effects to leave off (`?fxoff=dof,ssao`), for finding out which one is doing what.
    pub fx_off: String,
    pub day_len: f64,
    pub season: Option<f64>,
    pub hour: f64,
    /// Where to start the camera (else beside the town nearest the origin).
    pub at: Option<(f32, f32)>,
    /// `?bella`: an easter egg.
    pub bella: bool,
    /// Absolute in-game days (`t`), from a shared link.
    pub t: Option<f64>,
    /// Camera yaw and zoom (`yaw`, `zoom`), from a shared link.
    pub view: Option<(f32, f32)>,
    /// Carry on from where this world's clock was left (no hour/season/t given).
    pub resume: bool,
    /// `?prof`: log where the frame time goes.
    pub prof: bool,
    /// `?aurora` (or `?aurora=0.5`): northern lights at that strength whatever the weather and place.
    pub aurora: Option<f32>,
    /// Whether the autopilot starts on.
    pub auto: bool,
    /// `?pitch=0.3`: start looking this far down (lower sees more sky; the autopilot takes over again from there).
    pub pitch: Option<f32>,
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
