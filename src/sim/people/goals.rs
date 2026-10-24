use super::*;

impl TownLife {
    pub(super) fn want(&self, p: &Person, env: &Env) -> Goal {
        let h = (env.hour + p.jitter).rem_euclid(24.0);
        let wet = env.rain > 0.45;
        let r = unit(hash_u32(p.seed ^ env.slot.wrapping_mul(2654435761)));
        if !(6.3..22.3).contains(&h) {
            return Goal::Home;
        }
        // Bella lifts weights from morning till night, whatever else is on
        if p.bella {
            return Goal::Gym;
        }
        // the wedding party go to church, then on to the dance
        if p.wed != Wed::None && (env.hour >= WEDDING_IN || self.wedding.as_ref().is_some_and(|w| w.called)) && !self.vows_said() {
            return Goal::Wedding;
        }
        // those who said they'd come to tonight's gathering go (children
        // head home at nine)
        if p.coming && self.gather_open(env) && !(p.role == Role::Child && env.hour > 21.0) {
            return Goal::Gather;
        }
        // market mornings: stallholders mind their stalls, folk come to browse
        if self.market_on(env) {
            if self.vendor_stall(p).is_some() {
                return Goal::Market;
            }
            if matches!(p.role, Role::Villager | Role::Elder) && p.carry != Carry::Basket && r < 0.55 {
                return Goal::Shop;
            }
        }
        // the post goes round in the morning; the baker takes bread to the inn
        match p.role {
            Role::Postie if (8.0..12.0).contains(&h) => return Goal::Work,
            Role::Baker if (9.0..9.7).contains(&h) && self.tavern.is_some() && p.delivered != env.day => {
                return Goal::Deliver
            }
            _ => {}
        }
        let evening = |r: f32| {
            if r < 0.38 && self.tavern.is_some() {
                Goal::Tavern
            } else if r < 0.6 && !wet {
                Goal::Plaza
            } else if r < 0.7 {
                Goal::Visit
            } else {
                Goal::Home
            }
        };
        match p.role {
            // school on weekday mornings
            Role::Child if p.work.is_some() && env.day % 7 < 5 && (9.0..14.5).contains(&h) => Goal::Work,
            Role::Child => {
                if h < 8.0 || h > 19.5 || wet || r > 0.8 {
                    Goal::Home
                } else {
                    Goal::Play
                }
            }
            Role::Elder => {
                if h < 8.5 || h > 19.0 || wet {
                    Goal::Home
                } else if r < 0.45 {
                    Goal::Bench
                } else if r < 0.65 {
                    Goal::Plaza
                } else if r < 0.8 {
                    Goal::Errand
                } else {
                    Goal::Home
                }
            }
            Role::Villager => {
                if h < 7.5 {
                    Goal::Home
                } else if h < 17.5 {
                    if r < 0.28 {
                        Goal::Errand
                    } else if r < 0.48 && !wet {
                        Goal::Plaza
                    } else if r < 0.6 {
                        Goal::Visit
                    } else if r < 0.68 && !wet {
                        Goal::Bench
                    } else {
                        Goal::Home
                    }
                } else {
                    evening(r)
                }
            }
            Role::Barkeep => {
                if h >= 10.5 {
                    Goal::Work
                } else if h > 8.0 && r < 0.4 {
                    Goal::Errand
                } else {
                    Goal::Home
                }
            }
            _ => {
                if (8.0..12.0).contains(&h) || (13.0..17.5).contains(&h) {
                    Goal::Work
                } else if h < 8.0 {
                    Goal::Home
                } else if h < 13.0 {
                    if r < 0.4 && self.tavern.is_some() {
                        Goal::Tavern
                    } else if r < 0.7 && !wet {
                        Goal::Plaza
                    } else {
                        Goal::Home
                    }
                } else {
                    evening(r)
                }
            }
        }
    }

    pub(super) fn dest_for(&self, p: &Person, g: Goal, env: &Env) -> Dest {
        let r = hash_u32(p.seed ^ env.slot ^ 0x77) as usize;
        match g {
            Goal::Home => Dest::Lot(p.home),
            Goal::Work => match (p.role, p.work) {
                // the postie's next door: somewhere new each time
                (Role::Postie, _) if !self.homes.is_empty() => {
                    Dest::Lot(self.homes[(r ^ (env.time * 7.0) as usize) % self.homes.len()])
                }
                (Role::Farmer, Some(w)) => Dest::Field(w),
                (Role::Smith, Some(w)) => Dest::Anvil(w),
                (_, Some(w)) => Dest::Lot(w),
                _ => Dest::Lot(p.home),
            },
            Goal::Tavern => self.tavern.map(Dest::Lot).unwrap_or(Dest::Plaza),
            Goal::Errand => {
                if self.shops.is_empty() {
                    Dest::Plaza
                } else {
                    Dest::Lot(self.shops[r % self.shops.len()])
                }
            }
            Goal::Visit => match (p.friend, self.homes.is_empty()) {
                (Some(f), _) => Dest::Lot(self.people[f].home),
                (None, false) => Dest::Lot(self.homes[r % self.homes.len()]),
                _ => Dest::Plaza,
            },
            Goal::Plaza | Goal::Play => Dest::Plaza,
            Goal::Bench => Dest::Bench(r % 4),
            Goal::Gather => Dest::Gather,
            Goal::Market => Dest::Stall(self.vendor_stall(p).unwrap_or(0)),
            Goal::Shop => match self.stall_spots().len() {
                0 => Dest::Plaza,
                n => Dest::Stall(r % n),
            },
            Goal::Deliver => self.tavern.map(Dest::Lot).unwrap_or(Dest::Plaza),
            Goal::Wedding => self.church.map(Dest::Lot).unwrap_or(Dest::Plaza),
            Goal::Gym => Dest::Gym,
        }
    }

    /// Every third day each town holds a morning market (not in a downpour).
    pub(super) fn market_on(&self, env: &Env) -> bool {
        (env.day + (self.town.id % 3) as i64).rem_euclid(3) == 0 && (8.0..12.5).contains(&env.hour) && env.rain < 0.45
    }

    /// Market stalls round the plaza: (centre, outward direction). City
    /// plazas skip spots their permanent stalls already take.
    pub(super) fn stall_spots(&self) -> Vec<(Vec2, Vec2)> {
        let c = self.center();
        [80.0f32, 200.0, 320.0]
            .iter()
            .filter_map(|deg| {
                let a = deg.to_radians();
                let out = Vec2::new(a.cos(), a.sin());
                let p = c + out * 7.2;
                (!self.near_stall(p)).then_some((p, out))
            })
            .collect()
    }

    /// Which stall this person minds on market day, if they're a stallholder.
    pub(super) fn vendor_stall(&self, p: &Person) -> Option<usize> {
        let k = self
            .people
            .iter()
            .filter(|q| matches!(q.role, Role::Shopkeeper | Role::Baker | Role::Farmer))
            .position(|q| q.seed == p.seed)?;
        (k < self.stall_spots().len()).then_some(k)
    }

    pub(super) fn place_now(&mut self, i: usize, env: &Env) {
        let g = self.want(&self.people[i], env);
        let d = self.dest_for(&self.people[i], g, env);
        let pt = self.plaza_point();
        let lots = self.lots.clone();
        let p = &mut self.people[i];
        p.goal = g;
        p.dest = d;
        let j = unit(hash_u32(p.seed ^ 0x3131));
        match d {
            Dest::Lot(l) => {
                p.state = State::Inside;
                p.visible = false;
                p.at = Place::Lot(l);
                p.pos = Vec2::from(lots[l].front);
                p.timer = 5.0 + j * 60.0;
            }
            Dest::Plaza | Dest::Bench(_) | Dest::Gather | Dest::Stall(_) | Dest::Gym => {
                p.state = State::Idle;
                p.visible = true;
                p.at = Place::Plaza;
                p.pos = pt;
                p.timer = 1.0 + j * 8.0;
            }
            Dest::Field(l) | Dest::Anvil(l) => {
                p.state = State::Inside;
                p.visible = false;
                p.at = Place::Lot(l);
                p.pos = Vec2::from(lots[l].front);
                p.timer = j * 10.0;
                p.goal = Goal::Home;
            }
        }
    }

    /// Is a point on one of a city plaza's market stalls?
    pub(super) fn near_stall(&self, p: Vec2) -> bool {
        let (cx, cz) = (self.town.cx as f32, self.town.cz as f32);
        [(-7.0, 3.0), (4.0, -8.0)].iter().any(|&(sx, sz)| {
            let (x0, z0) = (cx + sx - 0.8, cz + sz - 0.8);
            p.x > x0 && p.x < x0 + 5.6 && p.y > z0 && p.y < z0 + 4.6
        })
    }
}
