//! Procedural weather: a slow random walk between weather kinds with
//! smooth transitions, seasonal odds, lightning and snow cover.

use glam::Vec2;

use crate::util::{approach, Rng};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Fog,
    Snow,
}

const ALL: [Kind; 6] = [Kind::Clear, Kind::Cloudy, Kind::Rain, Kind::Storm, Kind::Fog, Kind::Snow];

pub struct Weather {
    pub kind: Kind,
    timer: f32,
    pub cloud: f32,
    pub precip: f32,
    pub storm: f32,
    pub fog: f32,
    pub wind: Vec2,
    wind_target: Vec2,
    pub flash: f32,
    lightning_t: f32,
    pub snowing: bool,
    pub ground_snow: f32,
    pub wet: f32,
    thunder: Vec<(f32, f32)>,
    rng: Rng,
}

impl Weather {
    pub fn new(seed: u32) -> Self {
        Weather {
            kind: Kind::Clear,
            timer: 120.0,
            cloud: 0.15,
            precip: 0.0,
            storm: 0.0,
            fog: 0.0,
            wind: Vec2::new(0.3, 0.1),
            wind_target: Vec2::new(0.3, 0.1),
            flash: 0.0,
            lightning_t: 10.0,
            snowing: false,
            ground_snow: 0.0,
            wet: 0.0,
            thunder: Vec::new(),
            rng: Rng::new(seed as u64 ^ 0xfeed),
        }
    }

    fn targets(&self) -> (f32, f32, f32, f32) {
        match self.kind {
            Kind::Clear => (0.12, 0.0, 0.0, 0.0),
            Kind::Cloudy => (0.62, 0.0, 0.0, 0.06),
            Kind::Rain => (0.85, 0.7, 0.0, 0.18),
            Kind::Storm => (1.0, 1.0, 1.0, 0.12),
            Kind::Fog => (0.45, 0.0, 0.0, 0.85),
            Kind::Snow => (0.8, 0.65, 0.0, 0.3),
        }
    }

    fn pick(&mut self, season: f32) -> Kind {
        let w: [f32; 6] = match season as usize % 4 {
            0 => [4.0, 3.0, 3.0, 1.0, 1.0, 0.0],
            1 => [6.0, 2.0, 1.0, 1.5, 0.3, 0.0],
            2 => [3.0, 3.0, 3.0, 1.0, 2.0, 0.0],
            _ => [3.0, 3.0, 0.5, 0.0, 2.0, 3.0],
        };
        let total: f32 = w.iter().sum();
        let mut r = self.rng.f32() * total;
        for (i, wi) in w.iter().enumerate() {
            if r < *wi {
                return ALL[i];
            }
            r -= wi;
        }
        Kind::Clear
    }

    pub fn set(&mut self, k: Kind) {
        self.kind = k;
        self.timer = self.rng.range(100.0, 260.0);
        let mag = match k {
            Kind::Storm => 1.0,
            Kind::Rain | Kind::Snow => 0.6,
            Kind::Cloudy => 0.4,
            _ => 0.18,
        };
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        self.wind_target = Vec2::new(a.cos(), a.sin()) * mag;
    }

    pub fn cycle(&mut self) {
        let i = ALL.iter().position(|k| *k == self.kind).unwrap_or(0);
        self.set(ALL[(i + 1) % ALL.len()]);
    }

    /// Advance; returns thunder volumes that should be heard this frame.
    pub fn update(&mut self, dt: f32, season: f32, cold: f32) -> Vec<f32> {
        self.timer -= dt;
        if self.timer <= 0.0 {
            let k = self.pick(season);
            self.set(k);
        }
        let winter = (2.9..3.95).contains(&season);
        if self.kind == Kind::Snow && !winter && cold < 0.5 {
            self.kind = Kind::Rain;
        }
        if self.kind == Kind::Rain && (winter || cold > 0.6) {
            self.kind = Kind::Snow;
        }
        let (c, p, s, f) = self.targets();
        let k = approach(0.05, dt);
        self.cloud += (c - self.cloud) * k;
        self.precip += (p - self.precip) * k;
        self.storm += (s - self.storm) * k;
        self.fog += (f - self.fog) * k;
        self.wind += (self.wind_target - self.wind) * approach(0.08, dt);
        self.snowing = self.kind == Kind::Snow || (self.precip > 0.05 && (winter || cold > 0.6));

        let snow_target = if winter { 0.6 + 0.4 * self.snowing as u8 as f32 } else { cold * 0.7 };
        let rate = if snow_target > self.ground_snow { 1.0 / 90.0 } else { 1.0 / 160.0 };
        self.ground_snow += (snow_target - self.ground_snow) * approach(rate * 4.0, dt);
        let wet_target = if self.snowing { 0.2 } else { self.precip };
        self.wet += (wet_target - self.wet) * approach(if wet_target > self.wet { 0.15 } else { 0.02 }, dt);

        // lightning
        self.flash *= (-dt * 7.0).exp();
        if self.storm > 0.6 {
            self.lightning_t -= dt;
            if self.lightning_t <= 0.0 {
                self.flash = 1.0;
                let d = self.rng.range(0.3, 3.0);
                self.thunder.push((d, 1.2 - d * 0.25));
                if self.rng.chance(0.4) {
                    self.thunder.push((d + 0.15, 0.0));
                    self.flash = 1.4;
                }
                self.lightning_t = self.rng.range(5.0, 16.0);
            }
        }
        let mut out = Vec::new();
        self.thunder.retain_mut(|(t, v)| {
            *t -= dt;
            if *t <= 0.0 {
                if *v > 0.0 {
                    out.push(*v);
                }
                false
            } else {
                true
            }
        });
        out
    }

    pub fn label(&self) -> &'static str {
        let heavy = self.precip > 0.55;
        if self.storm > 0.5 {
            "Thunderstorm"
        } else if self.fog > 0.5 {
            "Misty"
        } else if self.snowing && self.precip > 0.1 {
            if heavy {
                "Snowing"
            } else {
                "Light snow"
            }
        } else if self.precip > 0.1 {
            if heavy {
                "Raining"
            } else {
                "Drizzle"
            }
        } else if self.cloud > 0.5 {
            "Overcast"
        } else if self.cloud > 0.3 {
            "Partly cloudy"
        } else {
            "Clear skies"
        }
    }
}
