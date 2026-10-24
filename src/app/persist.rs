//! What the browser remembers between visits (the world's clock), and the
//! time-lapse.

use super::*;

/// Fastest the clock runs in the time-lapse: a day in five seconds.
const TIMELAPSE_SPEED: f64 = 240.0;
/// Most days that can pass while the page was closed.
const MAX_AWAY_DAYS: f64 = 3.0;

fn store() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn get(seed: u32, name: &str) -> Option<String> {
    store()?.get_item(&format!("meeple.{name}.{seed}")).ok().flatten()
}

fn put(seed: u32, name: &str, value: &str) {
    if let Some(s) = store() {
        let _ = s.set_item(&format!("meeple.{name}.{seed}"), value);
    }
}

#[derive(Default)]
pub struct Viewer {
    /// The clock speed before the time-lapse began.
    pub lapse: Option<f64>,
    pub save_t: f32,
    /// Greeting for a returning visitor, shown once the first view is up.
    pub welcome: Option<String>,
}

/// Clock to start from when the visit isn't pinned by `hour`/`season`/`t`:
/// the days the world had reached last time, plus some for the time away.
/// Returns the days and, if time passed, a line for the welcome.
pub fn resume(seed: u32, day_len: f64) -> Option<(f64, Option<String>)> {
    let days: f64 = get(seed, "days")?.parse().ok()?;
    let seen: f64 = get(seed, "seen").and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let away = if seen > 0.0 { ((js_sys::Date::now() - seen) / 1000.0 / day_len).clamp(0.0, MAX_AWAY_DAYS) } else { 0.0 };
    let note = (away >= 0.25).then(|| {
        let n = away.round().max(1.0) as i64;
        format!("Welcome back. {n} day{} passed in the valley while you were away.", if n == 1 { "" } else { "s" })
    });
    Some((days + away, note))
}

impl App {
    /// Called every frame: write the clock down now and then, and greet a
    /// returning visitor once the first view is up.
    pub(super) fn persist(&mut self, dt: f32) {
        self.viewer.save_t -= dt;
        if self.viewer.save_t <= 0.0 {
            self.viewer.save_t = 5.0;
            put(self.seed, "days", &format!("{:.4}", self.clock.days));
            put(self.seed, "seen", &format!("{}", js_sys::Date::now()));
        }
        if self.settled {
            if let Some(w) = self.viewer.welcome.take() {
                self.chronicle.push((self.clock.day(), w.clone()));
                self.toast(&w);
            }
        }
    }

    /// Y: run the clock fast to watch seasons sweep over the valley.
    pub(super) fn toggle_timelapse(&mut self) {
        match self.viewer.lapse.take() {
            Some(prev) => {
                self.clock.speed = prev;
                self.toast("Time-lapse ended");
            }
            None => {
                self.viewer.lapse = Some(self.clock.speed.max(1.0));
                self.clock.speed = TIMELAPSE_SPEED;
                self.dir.zoom_target = 70.0;
                self.toast("Time-lapse: Y to return to normal speed");
            }
        }
    }
}
