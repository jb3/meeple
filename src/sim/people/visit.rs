use super::*;

/// Something the camera can be promised a sight of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Occasion {
    Gathering,
    Wedding,
    Christening,
    Fire,
}

/// The autopilot has said it's coming to see an occasion here today. The town
/// makes sure it happens (a little bad weather or a thin turnout won't stop
/// it) and holds the start until the camera is near, or `until` passes.
#[derive(Clone, Copy, Debug)]
pub struct Visit {
    pub occasion: Occasion,
    pub day: i64,
    pub until: f32,
    pub arrived: bool,
}

impl TownLife {
    /// Tell the town what the camera is coming for (None: nothing), and
    /// whether it's close. Once it has arrived it stays arrived for the day.
    pub fn expect_visitor(&mut self, want: Option<(Occasion, i64, f32)>, near: bool) {
        self.visit = match (want, self.visit) {
            (None, _) => None,
            (Some((o, d, until)), Some(v)) if v.occasion == o && v.day == d => Some(Visit { until, arrived: v.arrived || near, ..v }),
            (Some((occasion, day, until)), _) => Some(Visit { occasion, day, until, arrived: near }),
        };
    }

    /// Has the camera been promised this occasion today?
    pub(super) fn promised(&self, o: Occasion, day: i64) -> bool {
        self.visit.is_some_and(|v| v.occasion == o && v.day == day)
    }

    /// Is this occasion waiting for the camera to arrive?
    pub(super) fn held(&self, o: Occasion, env: &Env) -> bool {
        self.visit.is_some_and(|v| v.occasion == o && v.day == env.day && !v.arrived && env.hour < v.until)
    }

    /// Weather good enough for an occasion the camera has been promised.
    pub(super) fn fine_for(&self, o: Occasion, env: &Env) -> bool {
        env.nice || (self.promised(o, env.day) && env.rain < 0.35)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::gen::Gen;

    /// A town the camera has been promised a festival in holds it back until
    /// the camera arrives, then holds it even in weather the town would
    /// usually stay in for; a town nobody is coming to doesn't.
    #[test]
    fn promised_festival_waits_for_the_camera_then_happens() {
        let gen = Gen::new(1234);
        let towns: Vec<_> = (-3..=3).flat_map(|rx| (-3..=3).map(move |rz| (rx, rz))).filter_map(|(rx, rz)| gen.town(rx, rz)).take(6).collect();
        assert!(towns.len() >= 4);
        let dt = 1.0 / 30.0;
        let env = |hour: f32, time: f32| Env {
            gen: &gen,
            dt,
            time,
            hour,
            slot: (hour * 2.0) as u32,
            rain: 0.1,
            night: if (7.0..19.0).contains(&hour) { 0.0 } else { 1.0 },
            day: 3,
            // drizzly: ordinarily no gathering tonight
            nice: false,
            wind: Vec2::new(0.3, 0.1),
            season: 1.5,
        };
        let (mut held, mut happened) = (0, 0);
        for t in towns {
            let mut life = TownLife::new(&gen, t, &env(18.0, 0.0));
            let mut sounds = Vec::new();
            life.expect_visitor(Some((Occasion::Gathering, 3, 21.0)), false);
            let (mut hour, mut time) = (18.0f32, 0.0f32);
            let mut step = |life: &mut TownLife, until: f32, hour: &mut f32, time: &mut f32| {
                while *hour < until {
                    *hour += dt / 6.0;
                    *time += dt;
                    life.update(&env(*hour, *time), &[], &mut sounds);
                }
            };
            // the camera is still far off: nothing starts
            step(&mut life, 19.8, &mut hour, &mut time);
            assert!(life.gathering.is_none(), "{} started before the camera came", life.town.name);
            held += 1;
            // it arrives
            life.expect_visitor(Some((Occasion::Gathering, 3, 21.0)), true);
            step(&mut life, 20.2, &mut hour, &mut time);
            if life.gathering.is_some() {
                happened += 1;
            }
        }
        assert!(held >= 4);
        assert!(happened >= held - 1, "only {happened} of {held} promised festivals happened");
    }

    /// A wedding the camera has been promised happens even in drizzle, however
    /// late the camera comes, and the vows wait for it.
    #[test]
    fn promised_wedding_waits_for_the_camera_and_then_happens() {
        let gen = Gen::new(1234);
        let cities: Vec<_> = (-6..=6).flat_map(|rx| (-6..=6).map(move |rz| (rx, rz))).filter_map(|(rx, rz)| gen.town(rx, rz)).filter(|t| t.city).take(4).collect();
        assert!(cities.len() >= 3);
        let dt = 1.0 / 30.0;
        let mut happened = 0;
        for t in cities {
            // a day this town's own dice call a wedding on
            let day = (0..400i64).find(|&d| unit(hash3(t.id, d as i32, 3, 0xbe11)) < WEDDING_CHANCE).unwrap();
            let env = |hour: f32, time: f32| Env { gen: &gen, dt, time, hour, slot: (hour * 2.0) as u32, rain: 0.1, night: 0.0, day, nice: false, wind: Vec2::new(0.3, 0.1), season: 1.5 };
            let mut life = TownLife::new(&gen, t, &env(11.0, 0.0));
            let mut sounds = Vec::new();
            let want = Some((Occasion::Wedding, day, 15.0));
            life.expect_visitor(want, false);
            let (mut hour, mut time) = (11.0f32, 0.0f32);
            let mut step = |life: &mut TownLife, until: f32, hour: &mut f32, time: &mut f32| {
                while *hour < until {
                    *hour += dt / 6.0;
                    *time += dt;
                    life.update(&env(*hour, *time), &[], &mut sounds);
                }
            };
            // the camera is late and far: the bells don't peal for an empty church
            step(&mut life, 14.7, &mut hour, &mut time);
            assert!(!life.vows_said(), "{}: married before the camera came", life.town.name);
            // it arrives: the vows follow
            life.expect_visitor(want, true);
            step(&mut life, 15.6, &mut hour, &mut time);
            if life.vows_said() {
                happened += 1;
            }
        }
        assert!(happened >= 3, "only {happened} of 4 promised weddings happened");
    }
}
