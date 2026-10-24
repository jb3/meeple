use super::super::*;

impl App {
    /// Advance the weather and everything that follows from it: floods and
    /// drought, the harvest, the chronicle's harvest note, lightning strikes.
    /// Returns the thunder claps to sound this frame.
    pub(super) fn update_climate(&mut self, dt: f32, here: Vec2, season: f32) -> Vec<f32> {
        let cold = self.world.gen.coldness(here.x, here.y);
        let thunder = self.weather.update(dt, season, cold);
        // long heavy rain swells the rivers; they drain away when it stops
        self.flood = (self.flood + if self.weather.precip > 0.55 && !self.weather.snowing { dt * 0.012 } else { -dt * 0.008 }).clamp(0.0, 1.0);
        // weeks of fine weather in the warm months run the rivers low, and dry
        // thatch burns easily; rain brings them back
        let ddays = (dt as f64 * self.clock.speed / self.clock.day_len) as f32;
        let warm = (0.8..2.3).contains(&season);
        let dry_now = warm && self.weather.precip < 0.08 && self.weather.cloud < 0.7;
        let drying = if dry_now { ddays * 0.25 } else if self.weather.precip > 0.25 { -ddays } else { -ddays * 0.03 };
        self.drought = (self.drought + drying).clamp(0.0, 1.0);
        // how the harvest is going: luck of the year, helped by autumn's
        // gathering in and spoilt by drought or flood
        let year = (self.clock.day() / 12) as i32;
        let luck = 0.75 + 0.5 * unit(hash2(self.seed, year, 0xba7));
        let plenty = (luck + if (2.0..3.0).contains(&(season % 4.0)) { 0.35 } else { 0.0 } - self.drought * 0.8 - self.flood * 0.5).clamp(0.2, 1.6);
        self.traffic.set_plenty(plenty);
        for t in self.towns.values_mut() {
            t.set_plenty(plenty);
        }
        if (2.0..2.15).contains(&(season % 4.0)) && self.noted.insert((0xba7, self.clock.day() / 12)) {
            let day = self.clock.day();
            let note = if plenty > 1.15 {
                "A bumper harvest: carts rolled in heavy and the market stalls were heaped high."
            } else if plenty < 0.55 {
                "A poor harvest: lean pickings, and prices rose at market."
            } else {
                "The harvest was gathered in."
            };
            self.chronicle.push((day, note.into()));
        }
        // a bolt in a storm now and then sets a house alight
        let strike = self.weather.flash > 0.9 && self.prev_flash <= 0.9 && self.weather.storm > 0.5;
        self.prev_flash = self.weather.flash;
        let risk = 1.0 + self.drought * 3.0 + self.weather.storm * 1.5;
        let mut struck_one = strike;
        for t in self.towns.values_mut() {
            t.set_fire_risk(risk);
            if struck_one && self.dir.rng.chance(0.2) {
                t.lightning();
                struck_one = false;
            }
        }
        thunder
    }
}
