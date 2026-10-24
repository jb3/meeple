use super::*;

impl App {
    /// Briefly show the HUD so a background viewer sees what the autopilot
    /// just started looking at (still respects the H toggle).
    pub(crate) fn announce(&mut self) {
        self.hud_t = self.hud_t.max(6.0);
    }

    /// Zoom in on two villagers chatting near the middle of the view; the
    /// person being followed always gets watched if they stop to talk.
    pub(crate) fn watch_chat(&mut self, here: Vec2) {
        let reach = self.cam.zoom * 0.9;
        let followed = (self.dir.mode == Mode::FollowPerson).then_some((self.dir.follow_town, self.dir.follow_idx));
        let mut found = None;
        for (id, t) in &self.towns {
            for (i, p) in t.people.iter().enumerate() {
                let Some(j) = p.chatting_with() else { continue };
                if followed == Some((*id, i)) {
                    found = Some((*id, i, j, true));
                    break;
                }
                // skip chats that are about to wrap up
                if i < j && found.is_none() && p.pos.distance(here) < reach && p.chat_left() > 4.0 {
                    found = Some((*id, i, j, false));
                }
            }
        }
        let Some((id, a, b, theirs)) = found else { return };
        if !theirs && !self.dir.rng.chance(0.25) {
            return;
        }
        let t = &self.towns[&id];
        let names = format!("{} and {}", t.people[a].name, t.people[b].name);
        let d = &mut self.dir;
        d.mode = Mode::WatchChat;
        d.follow_town = id;
        d.follow_idx = a;
        d.chat_b = b;
        d.zoom_target = 9.5;
        self.hud_follow = format!("Listening in on {names}");
        self.announce();
    }

    /// Where the camera goes for a wedding and how it looks: with the bride and
    /// groom as they gather, walk to church and come out, seen from the front of
    /// the church, close in, easing out a little as the party fills the square.
    pub(crate) fn wedding_shot(&mut self, id: u32) -> Option<Vec2> {
        let t = self.towns.get(&id)?;
        let couple = t.couple_outdoors();
        let front = t.church_front();
        let (vowed, gathered) = (t.vowed(), t.gathered());
        let crowd = t.festival().map_or(0.0, |(_, band, choir)| band + choir);
        let plaza = Vec2::new(t.town.cx as f32 + 0.5, t.town.cz as f32 + 0.5);
        let mid = (!couple.is_empty()).then(|| couple.iter().fold(Vec2::ZERO, |a, p| a + Vec2::new(p.x, p.z)) / couple.len() as f32);
        // on the couple; at the church door when they're inside
        let centre = mid.or(front.map(|f| f.2)).unwrap_or(plaza);
        if let Some((_, n, _)) = front {
            // the camera sits on the door's side, a little off square so the
            // front and one flank both show
            let square = n.x.atan2(n.y);
            let tau = std::f32::consts::TAU;
            let turn = |a: f32| (a - self.cam.yaw + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI;
            let (l, r) = (turn(square - 0.6), turn(square + 0.6));
            self.dir.yaw_goal = self.cam.yaw + if l.abs() < r.abs() { l } else { r };
            self.dir.yaw_t = self.dir.yaw_t.max(5.0);
        }
        self.dir.zoom_target = if !vowed {
            8.5
        } else if crowd >= 4.0 {
            9.5
        } else {
            7.5 + 3.0 * gathered
        };
        self.dir.pitch_target = 0.5;
        Some(centre)
    }

    /// Circles (x, y, z, radius) round the people the autopilot is watching,
    /// which the see-through effect keeps clear of buildings and trees.
    pub(crate) fn see_through_foci(&self) -> Vec<f32> {
        use crate::sim::people::happenings::Kind as Happening;
        let Some(t) = self.towns.get(&self.dir.follow_town) else { return Vec::new() };
        let mut out: Vec<glam::Vec4> = Vec::new();
        match self.dir.mode {
            Mode::FollowPerson => out.extend(t.outdoor_pos(self.dir.follow_idx).map(|p| p.extend(1.4))),
            Mode::WatchChat => {
                for i in [self.dir.follow_idx, self.dir.chat_b] {
                    out.extend(t.outdoor_pos(i).map(|p| p.extend(1.3)));
                }
            }
            Mode::Event => {
                let kind = if self.dir.event_key == "christening" { Happening::Christening } else { Happening::Closing };
                out.extend(t.party_points(kind).into_iter().map(|p| p.extend(1.3)));
            }
            Mode::Linger if self.dir.wed_wait => {
                out.extend(t.couple_outdoors().into_iter().map(|p| p.extend(1.4)));
            }
            Mode::Linger => {
                if let Some(g) = self.dir.ghost_focus {
                    out.push(g.extend(1.6));
                }
            }
            Mode::Gathering => {
                // the bride and groom themselves, then the whole party
                out.extend(t.couple_outdoors().into_iter().map(|p| p.extend(1.4)));
                let party = t.wedding_party();
                if !party.is_empty() {
                    let c = party.iter().fold(Vec3::ZERO, |a, p| a + *p) / party.len() as f32;
                    let r = party.iter().map(|p| Vec2::new(p.x - c.x, p.z - c.z).length()).fold(0.0, f32::max);
                    out.push(c.extend((r + 1.0).clamp(2.5, 9.0)));
                }
            }
            _ => {}
        }
        // only once the camera has got there and settled: not while it's still
        // panning over to them
        let aim = Vec2::new(self.cam.target.x, self.cam.target.z);
        out.retain(|f| Vec2::new(f.x, f.z).distance(aim) < f.w + 4.0);
        if self.dir.vel.length() > 1.5 {
            out.clear();
        }
        out.truncate(8);
        out.iter().flat_map(|v| v.to_array()).collect()
    }

    /// A wedding, a christening or the inn closing: the autopilot stays with
    /// these to the end rather than wandering off to something else.
    /// Has the autopilot told a town it's coming for an occasion (a wedding, a
    /// christening...) that hasn't happened yet? Then it keeps its mind on that:
    /// no dog chases, boats, trains, chats or wanderers draw it away, and it
    /// doesn't move on while it waits for the thing to begin.
    pub(crate) fn committed(&self) -> bool {
        self.dir.promise.is_some() && matches!(self.dir.mode, Mode::Travel | Mode::Linger)
    }

    pub(crate) fn locked(&self) -> bool {
        match self.dir.mode {
            Mode::Event => matches!(self.dir.event_key, "christening" | "closing"),
            Mode::FollowTrain => self.dir.follow_pinned,
            Mode::Gathering => self.towns.get(&self.dir.follow_town).is_some_and(|t| t.couple().is_some()),
            _ => false,
        }
    }

    /// Head for a gathering that's started nearby and say so on the HUD
    /// (once per gathering).
    /// When a gathering is under way nearby, commit to it: pick the nearest
    /// one not yet watched today, swing over and stay with it for a while.
    /// Gatherings outrank following people, chats and trains.
    pub(crate) fn watch_gatherings(&mut self, here: Vec2, view_r: f32) {
        if self.dir.mode == Mode::Gathering || self.locked() || !self.dir.auto || self.dir.manual_t > 0.0 {
            return;
        }
        let day = self.clock.day();
        let best = self
            .towns
            .values()
            // while committed to one town's occasion, no other gathering draws us off
            .filter(|t| self.dir.promise.map_or(true, |p| p.town == t.town.id))
            // a gathering is only called when enough villagers are coming, so
            // head over straight away and watch them arrive
            .filter(|t| t.gathering.is_some())
            .filter(|t| !self.seen_gatherings.contains(&(t.town.id, day)))
            .map(|t| (t, Vec2::new(t.town.cx as f32 + 0.5, t.town.cz as f32 + 0.5)))
            .filter(|(_, c)| c.distance(here) < view_r + 120.0)
            .min_by(|a, b| a.1.distance(here).partial_cmp(&b.1.distance(here)).unwrap());
        let Some((t, _)) = best else { return };
        let coming = t.gathering.as_ref().map_or(0, |g| g.expected);
        let what = t.gathering.as_ref().map_or("Evening dance", |g| g.festival.name());
        let (id, name) = (t.town.id, t.town.name.clone());
        self.seen_gatherings.push((id, day));
        let d = &mut self.dir;
        d.mode = Mode::Gathering;
        d.follow_town = id;
        d.follow_train = None;
        d.timer = if t.couple().is_some() { 48.0 } else { d.rng.range(100.0, 130.0) };
        d.zoom_target = GATHER_WIDE;
        d.last_key = "";
        self.hud_follow = match t.couple() {
            Some((a, b)) => format!("{a} and {b} are married in {name} · {coming} guests"),
            None => format!("{what} in {name} · {coming} coming"),
        };
        self.announce();
    }

    /// Something worth watching has started in a loaded town: a fire, a
    /// fair, a christening. Go and see, once per sort of event per day; the
    /// weightier ones win over what's being filmed now.
    pub(crate) fn watch_events(&mut self, here: Vec2) {
        if !self.dir.auto || self.dir.manual_t > 0.0 || self.card.is_some() || self.dir.mode == Mode::Gathering || self.locked() {
            return;
        }
        // something already being watched gives way only to something much bigger
        let bar = if self.dir.mode == Mode::Event { self.dir.event_weight + 0.8 } else { 0.0 };
        let day = self.clock.day();
        let best = self
            .interests
            .iter()
            .filter(|(id, i)| i.weight >= 1.0 && i.weight > bar && !self.dir.watched.contains(&(*id, i.key, day)))
            // committed to an occasion in one town: only that town's own occasion counts
            // (a promised christening or fire), never a dog chase or the like
            .filter(|(id, i)| match self.dir.promise {
                Some(p) if matches!(self.dir.mode, Mode::Travel | Mode::Linger) => {
                    p.town == *id && matches!((p.occasion, i.key), (Occasion::Christening, "christening") | (Occasion::Fire, "fire"))
                }
                _ => true,
            })
            // never straight from one fair to another
            .filter(|(_, i)| !(i.key == "fair" && self.dir.last_key == "fair"))
            // nor one inn closing straight after another
            .filter(|(_, i)| !(i.key == "closing" && self.dir.last_key == "closing"))
            .filter(|(_, i)| i.at.distance(here) < 500.0)
            // a minor thing doesn't pull the camera off a place it just reached
            .filter(|(_, i)| i.weight >= 3.0 || (self.dir.commit_t <= 0.0 && matches!(self.dir.mode, Mode::Linger)) || (i.weight >= 2.0 && self.dir.mode == Mode::Event))
            .max_by(|a, b| a.1.weight.partial_cmp(&b.1.weight).unwrap());
        let Some((id, i)) = best else { return };
        let (id, key, caption, zoom, secs, weight) = (*id, i.key, i.caption.clone(), i.zoom, i.secs, i.weight);
        self.dir.watched.push((id, key, day));
        if self.dir.watched.len() > 40 {
            self.dir.watched.remove(0);
        }
        let d = &mut self.dir;
        d.mode = Mode::Event;
        d.follow_town = id;
        d.event_key = key;
        d.last_key = key;
        d.event_weight = weight;
        d.follow_train = None;
        let hold = matches!(key, "christening" | "closing");
        d.timer = if hold { secs } else { secs * d.rng.range(0.8, 1.2) };
        d.zoom_target = if hold { zoom } else { zoom * 1.3 };
        // looking down steeply, so close in the roofs don't hide the people
        d.pitch_target = if hold { d.rng.range(0.85, 0.95) } else { d.rng.range(0.55, 0.68) };
        self.hud_follow = caption;
        self.announce();
    }

    /// A new day: houses being built move on a stage, so the chunks they
    /// stand in are rebuilt.
    pub(crate) fn new_day(&mut self) {
        let gen = &self.world.gen;
        let (was, day) = (gen.day.get(), self.clock.day());
        if day == was {
            return;
        }
        gen.day.set(day);
        let first = gen.first_day.get();
        let mut keys = Vec::new();
        for life in self.towns.values() {
            for lot in gen.lots(&life.town).iter() {
                if lot.stage(first, was) != lot.stage(first, day) {
                    let (a, b) = (World::chunk_of(lot.x0 as f32 - 2.0, lot.z0 as f32 - 2.0), World::chunk_of(lot.x1 as f32 + 2.0, lot.z1 as f32 + 2.0));
                    for cz in a.1..=b.1 {
                        for cx in a.0..=b.0 {
                            if !keys.contains(&(cx, cz)) {
                                keys.push((cx, cz));
                            }
                        }
                    }
                }
            }
        }
        for b in self.world.rebuild(&keys) {
            self.r.add_chunk(b.key, &b.solid, &b.water);
        }
    }
}
