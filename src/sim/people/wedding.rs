use super::*;

impl TownLife {
    /// Now and then on a fine day two villagers marry: the couple and their
    /// guests go into church at two, come out to bells and petals at three,
    /// and dance in the plaza. Afterwards they share a home.
    pub(super) fn update_wedding(&mut self, env: &Env, sounds: &mut Vec<Sound>) {
        let h = env.hour;
        // a wedding the camera was promised starts as soon as it arrives, at
        // whatever hour and in whatever weather
        let here = self.visit.is_some_and(|v| v.occasion == Occasion::Wedding && v.day == env.day && v.arrived);
        let open = if here { 0.0..WEDDING_LATEST } else { 12.0..if self.promised(Occasion::Wedding, env.day) { WEDDING_LATEST } else { WEDDING_IN } };
        if self.wedding.is_none() && self.wedding_day != env.day && open.contains(&h) {
            self.wedding_day = env.day;
            let roll = unit(hash3(self.town.id, env.day as i32, 3, 0xbe11));
            if let (Some(_), true, true) = (self.church, here || self.fine_for(Occasion::Wedding, env), roll < WEDDING_CHANCE) {
                let free: Vec<usize> = (0..self.people.len())
                    .filter(|&i| {
                        let p = &self.people[i];
                        !p.married && !p.bella && !matches!(p.role, Role::Child | Role::Elder | Role::Priest | Role::Barkeep)
                    })
                    .collect();
                let pick = hash_u32(self.town.id ^ (env.day as u32).wrapping_mul(0x2545_f491)) as usize;
                let couple = free.get(pick % free.len().max(1)).and_then(|&a| {
                    let ha = self.people[a].home;
                    free.iter().copied().cycle().skip(pick % free.len() + 1).take(free.len()).find(|&b| self.people[b].home != ha).map(|b| (a, b))
                });
                let party = |s: &Self, a: usize, b: usize, day: u32| {
                    s.people.iter().enumerate().filter(|(i, p)| {
                        *i == a || *i == b || p.friend == Some(a) || p.friend == Some(b)
                            || (p.role != Role::Barkeep && unit(hash_u32(p.seed ^ day ^ 0x3ed)) < 0.55)
                    }).count()
                };
                // a wedding needs a few guests to make a dance of it
                let couple = couple.filter(|&(a, b)| party(self, a, b, env.day as u32) >= MIN_TURNOUT);
                if let Some((a, b)) = couple {
                    let day = env.day as u32;
                    for (i, p) in self.people.iter_mut().enumerate() {
                        p.wed = if i == a {
                            Wed::Bride
                        } else if i == b {
                            Wed::Groom
                        } else if p.friend == Some(a)
                            || p.friend == Some(b)
                            || (p.role != Role::Barkeep && unit(hash_u32(p.seed ^ day ^ 0x3ed)) < 0.55)
                        {
                            Wed::Guest
                        } else {
                            Wed::None
                        };
                    }
                    self.wedding = Some(Wedding { couple: (a, b), day: env.day, called: false, vows: None });
                }
            }
        }
        let Some(w) = &mut self.wedding else { return };
        if !w.called && (h >= WEDDING_IN || here) {
            w.called = true;
            for p in self.people.iter_mut().filter(|p| p.wed != Wed::None) {
                p.timer = p.timer.min(unit(hash_u32(p.seed ^ 0x5e7)) * 6.0);
            }
        }
        // anyone out on the streets going somewhere else turns round for
        // church at the next corner (not while still leaving a doorway, or
        // turning in at one, where the way on isn't a street). Asked every
        // tick, so someone partway through a long errand when word came
        // turns round once they're on a street
        if self.wedding.as_ref().is_some_and(|w| w.called) && !self.vows_said() {
            if let Some(c) = self.church {
                let (street, front) = (Vec2::from(self.lots[c].street), Vec2::from(self.lots[c].front));
                for i in 0..self.people.len() {
                    let p = &self.people[i];
                    let on_street = p.pi >= 2 && p.pi + 1 < p.path.len();
                    if p.wed != Wed::None && p.state == State::Walking && p.goal != Goal::Wedding && on_street {
                        let next = p.path[p.pi];
                        let mut pts = vec![next];
                        pts.extend(self.route(next, street, false));
                        pts.push(front);
                        self.people[i].goal = Goal::Wedding;
                        self.set_path(i, pts, Dest::Lot(c));
                    }
                }
            }
        }
        // anyone in the party who's indoors elsewhere (they were on their
        // way home when word came, say) comes straight back out
        if self.wedding.as_ref().is_some_and(|w| w.called) {
            let vowed = self.vows_said();
            let church = self.church;
            for p in self.people.iter_mut().filter(|p| p.wed != Wed::None && p.state == State::Inside) {
                let there = !vowed && church.is_some_and(|c| p.at == Place::Lot(c));
                if !there {
                    p.timer = p.timer.min(2.0);
                }
            }
        }
        let Some(w) = &self.wedding else { return };
        if w.day != env.day || h >= WEDDING_END {
            self.finish_wedding();
            return;
        }
        // three o'clock, or as soon after as the couple are both in church:
        // the bells peal and the party sets off for the plaza
        let (a, b) = w.couple;
        let in_church = |i: usize| {
            let p = &self.people[i];
            p.state == State::Inside && self.church.is_some_and(|c| p.at == Place::Lot(c))
        };
        let ready = (in_church(a) && in_church(b)) || h >= WEDDING_VOWS + 1.0;
        if (h >= WEDDING_VOWS || here) && ready && !self.vows_said() && !self.held(Occasion::Wedding, env) {
            if let Some(c) = self.church {
                let (x, z) = self.lots[c].center();
                for pitch in [1.0, 1.26, 0.84] {
                    sounds.push(Sound { name: "bell", pos: Vec3::new(x, self.y + 12.0, z), vol: 1.0, pitch });
                }
            }
            let expected = self.people.iter().filter(|p| p.wed != Wed::None).count();
            if let Some(w) = &mut self.wedding {
                w.vows = Some(h);
            }
            self.gathering = Some(Gathering { musician: None, joined: 0, expected, spin: 1.0, festival: Festival::Wedding });
            for p in &mut self.people {
                p.coming = p.wed != Wed::None;
                if p.coming {
                    p.timer = p.timer.min(unit(hash_u32(p.seed ^ 0x77e)) * 6.0);
                }
            }
            // the couple come out first
            let (a, b) = self.wedding.as_ref().unwrap().couple;
            self.people[a].timer = 0.0;
            self.people[b].timer = 0.3;
        }
    }

    /// The wedding is over, whether the evening ended or the camera has seen
    /// enough: the bride moves in with the groom, the party breaks up and the
    /// dancers drift off.
    pub fn finish_wedding(&mut self) {
        let Some(w) = self.wedding.take() else { return };
        let (a, b) = w.couple;
        self.people[a].home = self.people[b].home;
        self.people[a].married = true;
        self.people[b].married = true;
        self.people[a].friend = Some(b);
        self.people[b].friend = Some(a);
        self.married_today.push((a, b));
        for p in &mut self.people {
            p.wed = Wed::None;
        }
        if self.gathering.as_ref().is_some_and(|g| g.festival == Festival::Wedding) {
            self.gathering = None;
            for p in &mut self.people {
                p.coming = false;
                if matches!(p.state, State::Dancing | State::Singing | State::Playing) {
                    p.timer = p.timer.min(unit(hash_u32(p.seed ^ 0x6e7)) * 4.0);
                }
            }
        }
    }

    /// Have the vows been said, so the couple are out and the party under way?
    pub fn vowed(&self) -> bool {
        self.vows_said()
    }

    /// Where the bride and groom are, those of them in the open.
    pub fn couple_outdoors(&self) -> Vec<Vec3> {
        self.wedding.as_ref().map_or_else(Vec::new, |w| [w.couple.0, w.couple.1].into_iter().filter_map(|i| self.outdoor_pos(i)).collect())
    }

    /// Have today's couple come out of church married?
    pub(super) fn vows_said(&self) -> bool {
        self.gathering.as_ref().is_some_and(|g| g.festival == Festival::Wedding)
    }

    /// Where the wedding party is out in the open: the couple and their guests.
    pub fn wedding_party(&self) -> Vec<Vec3> {
        self.people.iter().filter(|p| p.wed != Wed::None && p.visible && p.state != State::Inside).map(|p| p.pos3()).collect()
    }

    /// The church: its centre, the direction its door faces (as x, z) and the
    /// spot just outside the door, for putting the camera where the front
    /// shows and the couple come out.
    pub fn church_front(&self) -> Option<(Vec2, Vec2, Vec2)> {
        let lot = &self.lots[self.church?];
        let n = match lot.face {
            0 => Vec2::new(0.0, -1.0),
            1 => Vec2::new(1.0, 0.0),
            2 => Vec2::new(0.0, 1.0),
            _ => Vec2::new(-1.0, 0.0),
        };
        Some((Vec2::from(lot.center()), n, Vec2::from(lot.front)))
    }

    /// The couple's names, while a wedding's on.
    pub fn couple(&self) -> Option<(&'static str, &'static str)> {
        self.wedding.as_ref().map(|w| (self.people[w.couple.0].name, self.people[w.couple.1].name))
    }
}
