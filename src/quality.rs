//! Adaptive quality: watches frame times and steps the renderer down a tier
//! when it can't keep up (TVs, old phones). It never steps back up, so it
//! can't flap.

/// Highest (cheapest) tier.
pub const MAX_TIER: u32 = 5;

/// Seconds of start-up (shader compiles, first chunks) to ignore.
const WARMUP: f32 = 6.0;
/// Seconds to let things settle after a tier change.
const SETTLE: f32 = 2.0;
/// Length of one measuring window.
const WINDOW: f32 = 2.5;
/// Consecutive slow windows before stepping down.
const STRIKES: u32 = 2;

/// Names of the tiers, best first, for the on-screen badge.
pub fn tier_name(tier: u32) -> &'static str {
    ["Ultra", "High", "Medium", "Low", "Very low", "Minimal"][tier.min(MAX_TIER) as usize]
}

/// Which of the extra rendering effects a tier pays for. The renderer compiles
/// its shaders against this (so what's off costs nothing) and the app uses it
/// to decide what to feed them.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Fx {
    /// A second, tighter shadow map for the area right around the camera.
    pub cascades: bool,
    /// Screen-space ambient occlusion: 0 off, 1 = 8 taps, 2 = 14.
    pub ssao: u8,
    /// Light shafts marched through the shadow map: 0 off, 1 = 8 steps, 2 = 14.
    pub rays: u8,
    /// Depth of field in the ordinary view.
    pub dof: bool,
    /// Temporal anti-aliasing (only used at pixel scale 1).
    pub taa: bool,
    /// Colour grading through a 3D lookup table.
    pub grade: bool,
    /// 0 flat colour, 1 gradient sky with sun and moon, 2 with stars and clouds.
    pub sky: u8,
    /// 0 as before, 1 sky reflection, 2 depth tint and shore foam, 3 caustics
    /// and reflections of the banks.
    pub water: u8,
    /// Chromatic aberration at the edges of the picture.
    pub cab: bool,
    /// Fog that glows towards the sun.
    pub fog_scatter: bool,
    /// Sharper cloud shadows that shift with the sun's angle.
    pub clouds: bool,
    /// Per-voxel colour variation, bevelled edges, wet sheen, snow glitter.
    pub materials: bool,
    /// Ground colour bounced up into the shade.
    pub bounce: bool,
    /// Most rain, snow and leaf particles drawn on the GPU (0 = on the CPU as before).
    pub particles: u32,
    /// Distant sounds lose their treble.
    pub air: bool,
}

impl Fx {
    pub const NONE: Fx = Fx {
        cascades: false,
        ssao: 0,
        rays: 0,
        dof: false,
        taa: false,
        grade: false,
        sky: 0,
        water: 0,
        cab: false,
        fog_scatter: false,
        clouds: false,
        materials: false,
        bounce: false,
        particles: 0,
        air: false,
    };
}

pub struct Quality {
    pub tier: u32,
    auto: bool,
    /// Seconds before measuring starts.
    hold: f32,
    sum: f32,
    n: u32,
    strikes: u32,
    /// Effects switched off by name (`?fxoff=dof,ssao`), whatever the tier says.
    pub off: String,
}

impl Quality {
    pub fn new(tier: u32, auto: bool) -> Quality {
        Quality { tier: tier.min(MAX_TIER), auto, hold: WARMUP, sum: 0.0, n: 0, strikes: 0, off: String::new() }
    }

    /// Shadow map edge in texels; 0 means no shadows at all.
    pub fn shadow_size(&self) -> i32 {
        match self.tier {
            0 => 2048,
            1..=4 => 1024,
            _ => 0,
        }
    }

    /// Shadow lookups per pixel.
    pub fn shadow_taps(&self) -> f32 {
        match self.tier {
            0 => 4.0,
            1..=4 => 1.0,
            _ => 0.0,
        }
    }

    /// A floating-point scene buffer with bloom and tone mapping: the top tier
    /// only, since it costs memory bandwidth.
    pub fn hdr(&self) -> bool {
        self.tier == 0
    }

    /// The extra effects this tier can afford.
    pub fn fx(&self) -> Fx {
        let mut fx = self.tier_fx();
        for name in self.off.split(',') {
            match name.trim() {
                "cascades" => fx.cascades = false,
                "ssao" => fx.ssao = 0,
                "rays" => fx.rays = 0,
                "dof" => fx.dof = false,
                "taa" => fx.taa = false,
                "grade" => fx.grade = false,
                "sky" => fx.sky = 0,
                "water" => fx.water = 0,
                "cab" => fx.cab = false,
                "fog" => fx.fog_scatter = false,
                "clouds" => fx.clouds = false,
                "materials" => fx.materials = false,
                "bounce" => fx.bounce = false,
                "particles" => fx.particles = 0,
                "air" => fx.air = false,
                _ => {}
            }
        }
        fx
    }

    fn tier_fx(&self) -> Fx {
        let n = Fx::NONE;
        match self.tier {
            0 => Fx {
                cascades: true,
                ssao: 2,
                rays: 2,
                dof: true,
                taa: true,
                grade: true,
                sky: 2,
                water: 3,
                cab: true,
                fog_scatter: true,
                clouds: true,
                materials: true,
                bounce: true,
                particles: 60_000,
                air: true,
            },
            1 => Fx {
                ssao: 1,
                rays: 1,
                grade: true,
                sky: 2,
                water: 2,
                cab: true,
                fog_scatter: true,
                clouds: true,
                materials: true,
                bounce: true,
                particles: 30_000,
                air: true,
                ..n
            },
            2 => Fx {
                grade: true,
                sky: 1,
                water: 1,
                fog_scatter: true,
                clouds: true,
                materials: true,
                bounce: true,
                particles: 15_000,
                air: true,
                ..n
            },
            3 => Fx { grade: true, sky: 1, clouds: true, materials: true, bounce: true, ..n },
            4 => Fx { grade: true, bounce: true, ..n },
            _ => n,
        }
    }

    /// Extra chunkiness added to the pixel scale.
    pub fn extra_pixel(&self) -> u32 {
        self.tier.saturating_sub(2).min(2)
    }

    /// Minimum milliseconds between rendered frames (0 = every display frame).
    pub fn min_frame_ms(&self) -> f64 {
        if self.tier >= 2 {
            1000.0 / 30.0
        } else {
            0.0
        }
    }

    /// Per-frame chunk meshing budget in ms once the view is populated.
    pub fn mesh_budget(&self) -> f64 {
        if self.tier >= 4 {
            3.0
        } else {
            7.0
        }
    }

    /// Feed one frame's wall-clock interval (seconds); true when the tier
    /// just dropped and the renderer needs reconfiguring.
    pub fn observe(&mut self, dt: f32) -> bool {
        if !self.auto || self.tier >= MAX_TIER {
            return false;
        }
        // a hidden tab or a stall isn't evidence about rendering speed
        if dt > 0.5 {
            self.sum = 0.0;
            self.n = 0;
            return false;
        }
        if self.hold > 0.0 {
            self.hold -= dt;
            return false;
        }
        self.sum += dt;
        self.n += 1;
        if self.sum < WINDOW {
            return false;
        }
        let avg_ms = self.sum / self.n as f32 * 1000.0;
        self.sum = 0.0;
        self.n = 0;
        // under a 30 fps cap frames land on 33 ms or 50 ms, so allow for that
        let limit = if self.min_frame_ms() > 0.0 { 45.0 } else { 24.0 };
        if avg_ms <= limit {
            self.strikes = 0;
            return false;
        }
        self.strikes += 1;
        if self.strikes < STRIKES {
            return false;
        }
        self.strikes = 0;
        self.tier += 1;
        self.hold = SETTLE;
        true
    }
}
