//! Time of day, seasons and the resulting sun/moon lighting.

use glam::Vec3;
use std::f32::consts::{PI, TAU};

use crate::util::smoothstep;

pub const DAYS_PER_SEASON: f64 = 3.0;
pub const SEASONS: [&str; 4] = ["Spring", "Summer", "Autumn", "Winter"];

pub struct Clock {
    /// Elapsed in-game days (fractional).
    pub days: f64,
    /// Real seconds per in-game day.
    pub day_len: f64,
    pub speed: f64,
}

pub struct Light {
    pub dir: Vec3,
    pub color: Vec3,
    pub amb_sky: Vec3,
    pub amb_ground: Vec3,
    pub sky: Vec3,
    /// 0 = full day, 1 = deep night
    pub night: f32,
}

impl Clock {
    pub fn tick(&mut self, dt: f64) {
        self.days += dt * self.speed / self.day_len;
    }

    pub fn hour(&self) -> f32 {
        (self.days.fract() * 24.0) as f32
    }

    pub fn day(&self) -> i64 {
        self.days.floor() as i64
    }

    /// 0..4: spring, summer, autumn, winter (fractional).
    pub fn season(&self) -> f32 {
        ((self.days / DAYS_PER_SEASON) % 4.0) as f32
    }

    pub fn season_name(&self) -> &'static str {
        SEASONS[self.season() as usize % 4]
    }

    fn summer(&self) -> f32 {
        0.5 + 0.5 * (TAU * (self.season() - 1.5) / 4.0).cos()
    }

    /// How full the moon is, 0 (new) to 1 (full), over an eight-day cycle.
    pub fn moon_full(&self) -> f32 {
        0.5 - 0.5 * (TAU * (self.days / 8.0).fract() as f32).cos()
    }

    pub fn sun_times(&self) -> (f32, f32) {
        let s = self.summer();
        (6.5 - s * 1.2, 18.0 + s * 1.8)
    }

    pub fn light(&self) -> Light {
        let h = self.hour();
        let summer = self.summer();
        let (rise, set) = self.sun_times();
        let emax = 0.55 + summer * 0.6;
        let orbit = |phase: f32, e: f32| {
            let az = PI * phase;
            Vec3::new(az.cos() * e.cos(), e.sin(), 0.55 * e.cos() + 0.15).normalize()
        };
        let (dir, color, day, elev);
        if h >= rise && h <= set {
            let phase = (h - rise) / (set - rise);
            let e = (PI * phase).sin() * emax;
            elev = e;
            dir = orbit(phase, e.max(0.06));
            let k = smoothstep(0.0, 0.2, e);
            let warm = Vec3::new(1.0, 0.56, 0.32);
            let white = Vec3::new(1.0, 0.95, 0.86);
            color = warm.lerp(white, smoothstep(0.08, 0.55, e)) * 1.08 * k;
            day = smoothstep(-0.02, 0.3, e);
        } else {
            let nlen = 24.0 - (set - rise);
            let phase = (h - set).rem_euclid(24.0) / nlen;
            let e = (PI * phase).sin() * 0.95;
            elev = -e;
            dir = orbit(phase, e.max(0.06));
            // a full moon lights the night far more than a sliver
            color = Vec3::new(0.36, 0.44, 0.66) * 0.6 * smoothstep(0.0, 0.25, e) * (0.55 + 0.45 * self.moon_full());
            day = 0.0;
        }
        let twilight = 1.0 - smoothstep(0.0, 0.35, elev.abs());
        let night_amb = Vec3::new(0.25, 0.30, 0.48);
        let day_amb = Vec3::new(0.56, 0.6, 0.7);
        let warm = Vec3::new(0.28, 0.13, 0.1) * twilight;
        let amb_sky = night_amb.lerp(day_amb, day) + warm;
        let amb_ground = amb_sky * Vec3::new(0.6, 0.55, 0.5);
        let sky = Vec3::new(0.09, 0.12, 0.23).lerp(Vec3::new(0.64, 0.8, 0.92), day)
            + Vec3::new(0.4, 0.2, 0.1) * twilight;
        Light { dir, color, amb_sky, amb_ground, sky, night: 1.0 - day }
    }
}
