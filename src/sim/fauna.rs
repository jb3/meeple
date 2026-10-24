//! Wildlife and livestock outside towns: grazing herds, ducks and swans,
//! herons, deer, foxes and rabbits at dawn and dusk, owls at night,
//! sailboats, narrowboats, fishing boats and flocks of birds.

use std::collections::HashMap;

use glam::{Vec2, Vec3};

use super::people::Sound;
use crate::render::boxes::{rgb, yaw_of, Boxes, Rig, MAT_GLOW};
use crate::util::*;
use crate::world::chunk::CS;
use crate::world::gen::{Ctx, Gen, SEA};
use crate::world::World;

mod boats;
mod draw;
mod flocks;
mod herds;
mod perch;
mod spawn;
#[cfg(test)]
mod tests;

/// How far downstream of a weir's middle the salmon wait, and so how far they leap to clear it.
pub(crate) const WEIR_FOOT: f32 = 1.9;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Sheep,
    Cow,
    Duck,
    Boat,
    Swan,
    Heron,
    Deer,
    Fox,
    Rabbit,
    Owl,
    /// Leaps up the rivers in autumn.
    Salmon,
    /// Hunts at dusk and through the night in the cold north; deer give it a wide berth.
    Wolf,
}

impl Kind {
    /// Shy creatures that keep to dawn and dusk and bolt from people.
    fn wild(self) -> bool {
        matches!(self, Kind::Deer | Kind::Fox | Kind::Rabbit | Kind::Wolf)
    }
    fn on_water(self) -> bool {
        matches!(self, Kind::Duck | Kind::Boat | Kind::Swan | Kind::Heron | Kind::Salmon)
    }
}

struct Animal {
    pos: Vec2,
    y: f32,
    yaw: f32,
    target: Vec2,
    timer: f32,
    graze: f32,
    phase: f32,
    tint: [f32; 3],
    /// Seconds left bolting (or, for an owl, flying to another tree).
    flee: f32,
}

struct Group {
    kind: Kind,
    home: Vec2,
    roam: f32,
    members: Vec<Animal>,
    ctx: Ctx,
    call_t: f32,
    /// Created while its spot was on screen: kept out of the world until
    /// the camera looks away, so animals never pop into view.
    hidden: bool,
    /// Fishing boats tie up here (a harbour jetty's end) from dusk.
    moor: Option<Vec2>,
    /// How present the group is (0..1): wild things come out at dawn and
    /// dusk, owls at night.
    active: f32,
    /// Treetops an owl can fly between, and the height each is once its leaves are down.
    perches: Vec<Vec3>,
    perch_low: Vec<f32>,
    /// The perch the owl is on.
    on_perch: usize,
}

/// A narrowboat working along a canal.
struct Narrowboat {
    canal: i32,
    x: f32,
    dir: f32,
    wait: f32,
    hull: [f32; 3],
    trim: [f32; 3],
}

/// Deer on the move between summer and winter ground: a long loose line
/// crossing the country, north in spring and south in autumn.
struct Herd {
    members: Vec<Animal>,
    dir: Vec2,
    travelled: f32,
    ctx: Ctx,
    ctx_at: Vec2,
}

struct Bird {
    off: Vec2,
    phase: f32,
}

struct Flock {
    pos: Vec3,
    dir: Vec2,
    travelled: f32,
    birds: Vec<Bird>,
    dark: bool,
}

pub struct Fauna {
    groups: HashMap<(i32, i32), Group>,
    flocks: Vec<Flock>,
    flock_t: f32,
    herds: Vec<Herd>,
    herd_t: f32,
    /// How dark it is (0..1), for putting the gulls to bed.
    night: f32,
    narrowboats: Vec<Narrowboat>,
    boat_t: f32,
    rng: Rng,
}

impl Fauna {
    /// Narrowboats about: (the canal they're on, where they are).
    pub fn narrowboats(&self) -> Vec<(i32, Vec2)> {
        self.narrowboats.iter().map(|b| (b.canal, Vec2::new(b.x, b.canal as f32 + 0.5))).collect()
    }

    /// Things out in the country worth a look near `view`: where, and what to
    /// call them. Herds and flocks, wildlife out at their hours, boats.
    pub fn sights(&self, view: Vec2, reach: f32) -> Vec<(Vec2, &'static str)> {
        let mut out = Vec::new();
        let mean = |ps: &mut dyn Iterator<Item = Vec2>| {
            let (mut sum, mut n) = (Vec2::ZERO, 0.0);
            for p in ps {
                sum += p;
                n += 1.0;
            }
            (n > 0.0).then(|| sum / n)
        };
        for g in self.groups.values() {
            if g.hidden || g.active < 0.35 {
                continue;
            }
            let Some(c) = mean(&mut g.members.iter().map(|a| a.pos)) else { continue };
            let what = match g.kind {
                Kind::Sheep => "Sheep grazing",
                Kind::Cow => "Cattle grazing",
                Kind::Duck => "Ducks on the water",
                Kind::Boat => "Fishing boats",
                Kind::Swan => "Swans on the water",
                Kind::Heron => "A heron at the water's edge",
                Kind::Deer => "Deer in the open",
                Kind::Fox => "A fox about",
                Kind::Rabbit => "Rabbits out",
                Kind::Owl => "An owl in the trees",
                Kind::Salmon => "Salmon leaping",
                Kind::Wolf => "Wolves on the hunt",
            };
            out.push((c, what));
        }
        for h in &self.herds {
            if let Some(c) = mean(&mut h.members.iter().map(|a| a.pos)) {
                out.push((c, "A herd of deer on the move"));
            }
        }
        for f in &self.flocks {
            out.push((Vec2::new(f.pos.x, f.pos.z), "Birds on the wing"));
        }
        for b in &self.narrowboats {
            out.push((Vec2::new(b.x, b.canal as f32 + 0.5), "A narrowboat on the canal"));
        }
        out.retain(|(p, _)| p.distance(view) < reach);
        out
    }

    pub fn new(seed: u32) -> Self {
        Fauna { groups: HashMap::new(), flocks: Vec::new(), flock_t: 5.0, herds: Vec::new(), herd_t: 20.0, night: 0.0, narrowboats: Vec::new(), boat_t: 3.0, rng: Rng::new(seed as u64 ^ 0xfa11a) }
    }

    fn is_water(gen: &Gen, ctx: &Ctx, p: Vec2) -> bool {
        let c = gen.column(ctx, p.x.floor() as i32, p.y.floor() as i32);
        c.h < SEA - 1 && !c.road
    }

    /// `on_screen` says whether a point is visible right now; during `warmup`
    /// (the world first appearing) new animals may show up in view.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        world: &World,
        dt: f32,
        view: Vec2,
        view_r: f32,
        night: f32,
        hour: f32,
        bare: f32,
        season: f32,
        threats: &[Vec2],
        on_screen: &dyn Fn(Vec3) -> bool,
        warmup: bool,
        sounds: &mut Vec<Sound>,
    ) {
        let gen = &world.gen;
        self.night = night;
        self.spawn_groups(world, view, view_r, warmup, on_screen);
        for g in self.groups.values_mut().filter(|g| g.hidden) {
            g.hidden = g.members.iter().any(|a| on_screen(Vec3::new(a.pos.x, a.y, a.pos.y)));
        }
        self.groups.retain(|k, _| {
            world.chunks.contains_key(k) && {
                let c = Vec2::new((k.0 * CS) as f32, (k.1 * CS) as f32);
                c.distance(view) < view_r + 160.0
            }
        });

        let dawn_or_dusk = (5.0..8.5).contains(&hour) || (17.0..21.0).contains(&hour);
        for g in self.groups.values_mut().filter(|g| !g.hidden) {
            let water = g.kind.on_water();
            let speed = match g.kind {
                Kind::Sheep => 0.5,
                Kind::Cow => 0.4,
                Kind::Duck => 0.45,
                Kind::Boat => 1.0,
                Kind::Swan => 0.3,
                Kind::Heron | Kind::Owl => 0.0,
                Kind::Deer => 0.7,
                Kind::Fox => 1.1,
                Kind::Rabbit => 0.9,
                Kind::Salmon => 0.8,
                Kind::Wolf => 1.0,
            };
            // wild things come out at dawn and dusk, owls at night
            let present = match g.kind {
                Kind::Wolf => night > 0.3 || dawn_or_dusk,
                Kind::Salmon => (2.0..3.0).contains(&(season % 4.0)),
                k if k.wild() => dawn_or_dusk,
                Kind::Owl => night > 0.6,
                _ => true,
            };
            g.active += (if present { 1.0 } else { 0.0 } - g.active) * approach(0.3, dt);
            if g.active < 0.02 {
                continue;
            }
            if g.kind == Kind::Owl {
                Self::update_owl(g, dt, &mut self.rng, bare);
                continue;
            }
            for (slot, a) in g.members.iter_mut().enumerate() {
                a.timer -= dt;
                a.phase += dt;
                a.flee = (a.flee - dt).max(0.0);
                // shy creatures bolt from anyone coming near
                if g.kind.wild() && a.flee <= 0.0 {
                    if let Some(t) = threats.iter().find(|t| t.distance(a.pos) < 7.0) {
                        let away = (a.pos - *t).normalize_or(Vec2::X);
                        a.target = a.pos + away * 10.0;
                        a.flee = 2.5;
                        a.graze = 0.0;
                    }
                }
                if g.kind == Kind::Salmon {
                    // they hold their place below the weir; the drawing does the leaping
                    continue;
                }
                if g.kind == Kind::Heron {
                    // stand still, stabbing at fish now and then
                    if a.timer <= 0.0 {
                        a.timer = self.rng.range(4.0, 12.0);
                        a.graze = 0.6;
                    }
                    a.graze = (a.graze - dt).max(0.0);
                    continue;
                }
                // fishing boats come home at dusk and stay tied up till morning
                if a.flee > 0.0 {
                    // keep running
                } else if let (Some(m), true) = (g.moor, night > 0.25) {
                    // each boat keeps its own berth (picking by the clock flipped
                    // the berth every few frames and the boats thrashed about)
                    a.target = m + Vec2::new((slot % 2) as f32 * 2.2 - 1.1, 0.0);
                    a.timer = 5.0;
                } else if a.timer <= 0.0 {
                    a.timer = self.rng.range(3.0, 9.0);
                    if !water && self.rng.chance(0.45) {
                        a.graze = a.timer;
                    } else {
                        let ang = self.rng.range(0.0, 6.28);
                        let t = g.home + Vec2::new(ang.cos(), ang.sin()) * self.rng.range(0.0, g.roam);
                        if !water || Self::is_water(gen, &g.ctx, t) {
                            a.target = t;
                        }
                    }
                }
                a.graze = (a.graze - dt).max(0.0);
                let d = a.target - a.pos;
                // a boat that would run aground stops and picks another way
                let aground = water && !(g.moor.is_some() && night > 0.25) && d.length() > 0.1 && !Self::is_water(gen, &g.ctx, a.pos + d.normalize() * 0.8);
                if aground {
                    a.timer = 0.0;
                    a.target = a.pos;
                }
                if d.length() > 0.1 && a.graze <= 0.0 && !aground {
                    let run = if a.flee > 0.0 { 3.5 } else { 1.0 };
                    a.pos += d.normalize() * (speed * run * dt).min(d.length());
                    let want = yaw_of(d);
                    let mut turn = (want - a.yaw).rem_euclid(std::f32::consts::TAU);
                    if turn > std::f32::consts::PI {
                        turn -= std::f32::consts::TAU;
                    }
                    a.yaw += turn * approach(3.0, dt);
                    if !water {
                        a.y += (gen.walk_y(&g.ctx, a.pos.x, a.pos.y) - a.y) * approach(10.0, dt);
                    }
                }
            }
            g.call_t -= dt;
            if g.call_t <= 0.0 {
                g.call_t = self.rng.range(8.0, 30.0);
                if let Some(a) = g.members.first() {
                    let name = match g.kind {
                        Kind::Sheep if self.rng.chance(0.5) => "sheepbell",
                        Kind::Sheep => "baa",
                        Kind::Cow => "moo",
                        Kind::Duck | Kind::Swan => "quack",
                        Kind::Boat if g.moor.is_some() && self.rng.chance(0.6) => "gull",
                        Kind::Boat => "splash",
                        Kind::Owl if night > 0.6 => "hoot",
                        Kind::Wolf if night > 0.5 && self.rng.chance(0.35) => "howl",
                        Kind::Salmon if self.rng.chance(0.4) => "splash",
                        _ => "",
                    };
                    // owls call to each other across the dark every ten seconds or so
                    if g.kind == Kind::Owl {
                        g.call_t = self.rng.range(8.0, 22.0);
                    }
                    // cows low more as the light goes
                    if g.kind == Kind::Cow && (0.2..0.8).contains(&night) {
                        g.call_t = self.rng.range(5.0, 12.0);
                    }
                    if !name.is_empty() && (night < 0.7 || matches!(g.kind, Kind::Boat | Kind::Owl | Kind::Wolf)) {
                        let vol = if g.kind == Kind::Owl { 0.9 } else { 0.5 };
                        sounds.push(Sound { name, pos: Vec3::new(a.pos.x, a.y, a.pos.y), vol, pitch: 1.0 });
                    }
                }
            }
        }

        // wolves about: deer in the same country scatter before them
        let wolves: Vec<Vec2> = self.groups.values().filter(|g| g.kind == Kind::Wolf && g.active > 0.3).flat_map(|g| g.members.iter().map(|a| a.pos)).collect();
        if !wolves.is_empty() {
            for g in self.groups.values_mut().filter(|g| g.kind == Kind::Deer) {
                for a in &mut g.members {
                    if let Some(w) = wolves.iter().find(|w| w.distance(a.pos) < 16.0) {
                        if a.flee <= 0.0 {
                            let away = (a.pos - *w).normalize_or(Vec2::X);
                            a.target = a.pos + away * 14.0;
                            a.flee = 3.0;
                            a.graze = 0.0;
                        }
                    }
                }
            }
        }
        // migrating deer: a herd crosses the country in spring and autumn
        self.update_herds(world, dt, view, view_r, night, season, on_screen, warmup);

        self.update_narrowboats(gen, dt, view, view_r, warmup, on_screen);

        self.update_flocks(dt, view, view_r, night, season);
    }
}
