use super::super::*;

impl App {
    /// What the camera was watching has ended (a dog caught, a chat over, a crowd gone home):
    /// rather than stay on the bare ground, pull back and find something nice to look at
    /// nearby, wildlife or a boat. It isn't a new journey, just somewhere pleasant to rest
    /// the eye for a while before the autopilot decides where next.
    pub(in crate::app) fn look_around(&mut self, here: Vec2) {
        // animals first: boats only do when nothing living is in sight
        let is_boat = |what: &str| what.contains("boat");
        let mut sights = self.fauna.sights(here, 150.0);
        if sights.iter().all(|s| is_boat(s.1)) {
            // none close by: go and find some, the nearest herd or flock within a long walk
            let mut far = self.fauna.sights(here, 500.0);
            far.retain(|s| !is_boat(s.1));
            far.sort_by(|a, b| a.0.distance(here).partial_cmp(&b.0.distance(here)).unwrap());
            if let Some(&(at, what)) = far.first() {
                let d = &mut self.dir;
                d.dest = at;
                d.promise = None;
                d.reason = what.to_string();
                d.await_hour = None;
                d.heading = (at - here).normalize_or_zero();
                d.checked_in = false;
                d.commit_t = 10.0;
                d.mode = Mode::Travel;
                d.expedition = false;
                d.zoom_target = d.rng.range(24.0, 34.0);
                d.pitch_target = d.rng.range(0.55, 0.7);
                self.hud_follow = format!("Heading for: {what}");
                self.announce();
                return;
            }
        } else {
            sights.retain(|s| !is_boat(s.1));
        }
        sights.sort_by(|a, b| a.0.distance(here).partial_cmp(&b.0.distance(here)).unwrap());
        sights.truncate(4);
        let d = &mut self.dir;
        d.mode = Mode::Linger;
        d.shot = Shot::Orbit;
        d.commit_t = 6.0;
        d.person_cd = d.rng.range(20.0, 40.0);
        let pick = if sights.is_empty() { None } else { Some(sights[d.rng.next_u32() as usize % sights.len()]) };
        match pick {
            Some((at, what)) => {
                d.center = at;
                d.timer = d.rng.range(18.0, 32.0);
                d.zoom_target = d.rng.range(17.0, 24.0);
                d.pitch_target = d.rng.range(0.5, 0.62);
                d.linger_total = d.timer;
                self.hud_follow = what.to_string();
                self.announce();
            }
            None => {
                // nothing living in sight: at least lift the view and take in the lie of the land
                d.center = here;
                d.timer = d.rng.range(12.0, 22.0);
                d.zoom_target = d.rng.range(28.0, 36.0);
                d.pitch_target = d.rng.range(0.5, 0.6);
                d.linger_total = d.timer;
                self.hud_follow.clear();
            }
        }
    }

    pub(in crate::app) fn direct(&mut self, dt: f32) {
        let here = Vec2::new(self.cam.target.x, self.cam.target.z);
        self.dir.wed_wait = false;
        self.dir.ghost_focus = None;
        let d = &mut self.dir;
        d.train_cd -= dt;
        d.boat_cd -= dt;
        d.party_cd -= dt;
        d.person_cd -= dt;
        d.chat_cd -= dt;
        d.commit_t -= dt;
        d.yaw_t -= dt;
        if d.yaw_t <= 0.0 {
            d.yaw_t = d.rng.range(18.0, 45.0);
            // a slow swing of up to a quarter turn either way
            let swing = d.rng.range(0.35, 1.6);
            d.yaw_goal = self.cam.yaw + if d.rng.range(0.0, 1.0) < 0.5 { -swing } else { swing };
        }
        if d.manual_t > 0.0 || !d.auto {
            // the user's hands are on the view: don't swing back afterwards
            d.yaw_goal = self.cam.yaw;
        }
        // a view turned by hand on a train stays as set for as long as the train is followed
        if d.yaw_locked {
            if d.mode == Mode::FollowTrain {
                d.yaw_goal = self.cam.yaw;
                d.yaw_t = d.yaw_t.max(5.0);
            } else {
                d.yaw_locked = false;
            }
        }
        if d.manual_t > 0.0 {
            d.manual_t -= dt;
            if d.manual_t <= 0.0 && d.auto && !d.follow_pinned {
                self.start_travel();
            }
            return;
        }
        if !d.auto {
            return;
        }
        // ease round to the chosen heading (gentler in chunky mode, where
        // rotating pixels shimmers)
        let turn = if self.r.eff_pixel() <= 1 { 0.3 } else { 0.15 };
        self.cam.yaw += (self.dir.yaw_goal - self.cam.yaw) * approach(turn, dt);
        // swing in briskly for a gathering, gently otherwise
        let zoom_rate = if matches!(self.dir.mode, Mode::Gathering | Mode::Event) { 0.9 } else { 0.25 };
        // a postcard looks out towards the horizon rather than down at the ground
        let postcard_view = self.postcard_t > 0.0 && !matches!(self.dir.mode, Mode::FollowPerson | Mode::FollowTrain | Mode::FollowBoat);
        let pt = if postcard_view { self.dir.pitch_target.min(POSTCARD_PITCH) } else { self.dir.pitch_target };
        self.cam.pitch += (pt - self.cam.pitch) * approach(0.2, dt);
        // a lighthouse is a tall thing: stand back far enough to fit it whole
        let tower = matches!(self.dir.mode, Mode::Travel | Mode::Linger) && self.dir.kinds.back() == Some(&"lighthouse");
        let mut zoom_goal = if tower { self.dir.zoom_target.max(LIGHTHOUSE_ZOOM) } else { self.dir.zoom_target };
        // a long pan across the land is made from further out: zoom out first,
        // then pan, then zoom back in on arrival, not rush across close up
        self.dir.pan_gate = 1.0;
        if self.dir.pan_dist > 20.0 && !matches!(self.dir.mode, Mode::Travel) && self.postcard_t <= 0.0 {
            let lift = (self.dir.pan_dist * 0.5).clamp(0.0, 50.0);
            let (zg, zl) = (zoom_goal * TIGHTER, lift * TIGHTER);
            if zl > zg {
                zoom_goal = zoom_goal.max(lift);
                self.dir.pan_gate = smoothstep(0.55, 0.9, (self.cam.zoom - zg) / (zl - zg));
            }
        }
        self.cam.zoom += (zoom_goal * TIGHTER - self.cam.zoom) * approach(if self.dir.pan_gate < 1.0 { 0.6 } else { zoom_rate }, dt);

        let mut goal: Option<Vec2> = None;
        match self.dir.mode {
            Mode::Travel => {
                let d = &mut self.dir;
                let to = d.dest - here;
                let dist = to.length();
                let speed = (dist * 0.1 + 1.2).min(if d.expedition { 26.0 } else { 15.0 });
                let want = to.normalize_or_zero() * speed;
                d.vel += (want - d.vel) * approach(1.0, dt);
                if dist < 5.0 {
                    self.arrive();
                }
            }
            Mode::Linger => {
                let d = &mut self.dir;
                d.timer -= dt;
                let t = self.time * 0.04;
                let prog = (1.0 - d.timer / d.linger_total.max(1.0)).clamp(0.0, 1.0);
                goal = Some(match d.shot {
                    Shot::Orbit => d.center + Vec2::new(t.cos(), t.sin()) * 7.0,
                    Shot::Push => {
                        d.zoom_target = lerp(28.0, 12.0, smoothstep(0.0, 1.0, prog));
                        d.center + Vec2::new(t.cos(), t.sin()) * 3.0
                    }
                    Shot::Hold => d.center + Vec2::new(t.cos(), t.sin() * 0.5) * 1.5,
                });
                // waiting on a wedding: be with the couple as they gather and go to
                // church, not staring at the empty square
                self.dir.wed_wait = false;
                if let Some(pr) = self.dir.promise.filter(|p| p.occasion == Occasion::Wedding) {
                    if self.towns.get(&pr.town).is_some_and(|t| t.couple().is_some() && !t.vowed()) {
                        if let Some(g) = self.wedding_shot(pr.town) {
                            goal = Some(g);
                            self.dir.follow_town = pr.town;
                            self.dir.wed_wait = true;
                        }
                    }
                }
                let d = &mut self.dir;
                // at a haunted ruin in the dark: stay with the ghost while it hovers,
                // from the angle that shows its face, and hold the view there
                if d.kinds.back() == Some(&"ruin") {
                    let dark = (self.clock.light().night + self.weather.fog * 0.5).min(1.0);
                    let (c, dest, time) = (d.center, d.dest, self.time);
                    if dark >= 0.55 {
                        let site = self
                            .world
                            .gen
                            .sites_near(c.x as i32, c.y as i32, c.x as i32, c.y as i32)
                            .into_iter()
                            .find(|s| s.kind == SiteKind::Ruin && (s.centre().distance(c) < 4.0 || s.centre().distance(dest) < 4.0));
                        if let Some(site) = site.filter(|s| crate::sim::wonders::haunted(s)) {
                            // the best-seen angle, worked out once per ruin: in front of
                            // the figure, but turned aside if a tree or wall is in the way
                            let key = (site.x, site.z);
                            let want = match self.dir.ghost_view {
                                Some((k, y)) if k == key => y,
                                _ => {
                                    let y = self.best_ghost_yaw(&site);
                                    self.dir.ghost_view = Some((key, y));
                                    y
                                }
                            };
                            let tau = std::f32::consts::TAU;
                            let diff = (want - self.cam.yaw + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI;
                            let d = &mut self.dir;
                            d.yaw_goal = self.cam.yaw + diff;
                            // no autorotation while it's the best angle
                            d.yaw_t = d.yaw_t.max(5.0);
                            if let Some((p, _)) = crate::sim::wonders::ghost_at(&site, time) {
                                goal = Some(Vec2::new(p.x, p.z));
                                d.zoom_target = 9.0;
                                d.pitch_target = 0.42;
                                // whatever's still in the way dissolves
                                d.ghost_focus = Some(p + Vec3::Y * 0.4);
                            }
                        }
                    }
                }
                let d = &mut self.dir;
                let (expired, person_due, committed) = (d.timer <= 0.0, d.person_cd <= 0.0, self.dir.promise.is_some());
                if expired && committed {
                    // waiting for what we came for: stay till it starts
                    self.dir.timer = 8.0;
                } else if expired {
                    self.start_travel();
                } else if person_due && !committed {
                    self.dir.person_cd = 60.0;
                    // only a quick look in on someone, and only in towns: the
                    // camera's here to explore
                    let town = matches!(self.dir.kinds.back(), Some(&"city") | Some(&"village"));
                    if town && !self.dir.checked_in && self.dir.rng.chance(0.3) {
                        self.dir.checked_in = true;
                        self.pick_person(here);
                    }
                }
            }
            Mode::FollowPerson => {
                self.dir.timer -= dt;
                // someone you clicked on is waited for even while indoors
                let picked = self.card == Some((self.dir.follow_town, self.dir.follow_idx));
                let p = self
                    .towns
                    .get(&self.dir.follow_town)
                    .and_then(|t| t.people.get(self.dir.follow_idx))
                    .filter(|p| p.visible || picked)
                    .map(|p| p.pos);
                match p {
                    Some(p) if self.dir.timer > 0.0 => goal = Some(p),
                    _ => self.look_around(here),
                }
            }
            Mode::Gathering => {
                self.dir.timer -= dt;
                let id = self.dir.follow_town;
                let wedding = self.towns.get(&id).is_some_and(|t| t.couple().is_some());
                if self.dir.wrap_t > 0.0 {
                    // the party's been wound up: stay for the last chord, then move on
                    self.dir.wrap_t -= dt;
                    goal = Some(self.dir.center);
                    self.dir.zoom_target = 12.5;
                    if self.dir.wrap_t <= 0.0 {
                        self.dir.wrap_t = 0.0;
                        self.start_travel();
                    }
                } else if wedding {
                    if self.dir.timer <= 0.0 {
                        // seen enough: the party breaks up and the music wraps up
                        if let Some(t) = self.towns.get_mut(&id) {
                            t.finish_wedding();
                        }
                        self.dir.wrap_t = 8.0;
                        goal = Some(self.dir.center);
                    } else {
                        goal = self.wedding_shot(id);
                        if let Some(g) = goal {
                            self.dir.center = g;
                        }
                    }
                } else {
                    let on = self.towns.get(&id).filter(|t| t.gathering.is_some()).map(|t| {
                        let crowd = t.festival().map_or(0.0, |(_, band, choir)| band + choir);
                        (Vec2::new(t.town.cx as f32 + 0.5, t.town.cz as f32 + 0.5), crowd)
                    });
                    match on {
                        Some((c, crowd)) if self.dir.timer > 0.0 => {
                            // wide enough to see folk leave their houses, then in
                            // close once the dancing's under way
                            self.dir.zoom_target = if crowd >= 4.0 { GATHER_CLOSE } else { GATHER_WIDE };
                            // a slow drift round the dancing, never wandering off
                            goal = Some(c + Vec2::new((self.time * 0.05).cos(), (self.time * 0.05).sin()) * 2.5);
                        }
                        _ => self.look_around(here),
                    }
                }
            }
            Mode::Event => {
                self.dir.timer -= dt;
                let (id, key) = (self.dir.follow_town, self.dir.event_key);
                match self.interests.iter().find(|(t, i)| *t == id && i.key == key) {
                    Some((_, i)) if self.dir.timer > 0.0 => {
                        // aim a little up the screen from the ground, at what's raised above it
                        let up = self.cam.ground_axes().1 * (i.lift / self.cam.pitch.tan().max(0.3));
                        goal = Some(i.at + up);
                        self.dir.zoom_target = i.zoom;
                    }
                    _ => self.look_around(here),
                }
            }
            Mode::WatchChat => {
                let (a, b) = (self.dir.follow_idx, self.dir.chat_b);
                let pair = self.towns.get(&self.dir.follow_town).and_then(|t| {
                    let (pa, pb) = (t.people.get(a)?, t.people.get(b)?);
                    (pa.chatting_with() == Some(b)).then(|| (pa.pos + pb.pos) * 0.5)
                });
                match pair {
                    Some(mid) => goal = Some(mid),
                    None => {
                        // they've said their goodbyes: ease back out
                        self.dir.chat_cd = self.dir.rng.range(15.0, 30.0);
                        self.look_around(here);
                    }
                }
            }
            Mode::FollowBoat => {
                self.dir.timer -= dt;
                let canal = self.dir.follow_canal;
                match self.fauna.narrowboats().into_iter().find(|b| b.0 == canal) {
                    Some((_, p)) if self.dir.timer > 0.0 => goal = Some(p),
                    _ => self.start_travel(),
                }
            }
            Mode::FollowTraveller => {
                self.dir.timer -= dt;
                match self.travellers.find(self.dir.follow_party) {
                    Some(p) if self.dir.timer > 0.0 => goal = Some(p),
                    _ => self.start_travel(),
                }
            }
            Mode::FollowTrain => {
                self.dir.timer -= dt;
                let key = self.dir.follow_train;
                let t = self.traffic.trains.iter().find(|t| Some(t.id) == key);
                match t {
                    Some(t) if self.dir.timer > 0.0 => {
                        let m = t.middle(&self.world.gen);
                        goal = Some(Vec2::new(m.x, m.z));
                    }
                    _ => {
                        // the train has gone off the line; wander on
                        self.dir.follow_train = None;
                        self.dir.follow_pinned = false;
                        self.hud_follow.clear();
                        self.start_travel();
                    }
                }
            }
        }

        // a conversation nearby is worth leaning in for
        if matches!(self.dir.mode, Mode::Linger | Mode::FollowPerson) && self.dir.commit_t <= 0.0 && self.dir.chat_cd <= 0.0 && !self.committed() {
            self.dir.chat_cd = 3.0;
            self.watch_chat(here);
        }

        // so is a narrowboat on the canal
        if self.dir.mode == Mode::Linger && self.dir.commit_t <= 0.0 && self.dir.boat_cd <= 0.0 && !self.committed() {
            self.dir.boat_cd = 4.0;
            let reach = self.cam.zoom * 1.6;
            if let Some((canal, _)) = self.fauna.narrowboats().into_iter().find(|b| b.1.distance(here) < reach) {
                self.dir.boat_cd = 120.0;
                if self.dir.rng.chance(0.7) {
                    self.dir.mode = Mode::FollowBoat;
                    self.dir.follow_canal = canal;
                    self.dir.timer = self.dir.rng.range(25.0, 40.0);
                    self.dir.zoom_target = 14.0;
                    self.hud_follow = String::from("A narrowboat on the canal");
                    self.announce();
                }
            }
        }

        // so are travellers on the road
        if self.dir.mode == Mode::Linger && self.dir.commit_t <= 0.0 && self.dir.party_cd <= 0.0 && !self.committed() {
            self.dir.party_cd = 4.0;
            let reach = self.cam.zoom * 1.6;
            if let Some((id, _, kind)) = self.travellers.about().into_iter().find(|(_, p, _)| p.distance(here) < reach) {
                self.dir.party_cd = 100.0;
                if self.dir.rng.chance(0.65) {
                    self.dir.mode = Mode::FollowTraveller;
                    self.dir.follow_party = id;
                    self.dir.timer = self.dir.rng.range(25.0, 40.0);
                    self.dir.zoom_target = 12.0;
                    self.hud_follow = kind.describe().to_string();
                    self.announce();
                }
            }
        }

        // trains passing close by are worth a look
        if self.dir.mode == Mode::Linger && self.dir.commit_t <= 0.0 && self.dir.train_cd <= 0.0 && !self.committed() {
            let gen = &self.world.gen;
            let near = self
                .traffic
                .trains
                .iter()
                .filter(|t| t.speed > 3.0)
                .map(|t| (t, t.middle(gen)))
                .find(|(_, m)| Vec2::new(m.x, m.z).distance(here) < self.cam.zoom * 1.2);
            if let Some((t, _)) = near {
                self.dir.train_cd = 90.0;
                if self.dir.rng.chance(0.55) {
                    self.dir.follow_train = Some(t.id);
                    self.dir.mode = Mode::FollowTrain;
                    self.dir.timer = self.dir.rng.range(20.0, 35.0);
                    self.dir.zoom_target = 20.0;
                    self.hud_follow = String::from("Following the train");
                    self.announce();
                }
            }
        }

        let d = &mut self.dir;
        // a postcard moment holds the view still (unless following someone)
        let goal = if self.postcard_t > 0.0 && !matches!(d.mode, Mode::FollowPerson | Mode::FollowTrain | Mode::FollowBoat) {
            d.vel *= 1.0 - approach(1.2, dt);
            None
        } else {
            goal
        };
        d.pan_dist = goal.map_or(0.0, |g| g.distance(here));
        match goal {
            Some(g) => {
                let to = g - here;
                // gatherings get a brisk, decisive swing; others a gentle chase
                let (gain, rate, top) = if d.mode == Mode::Event { (2.6, 3.2, 34.0) } else if d.mode == Mode::Gathering { (1.6, 2.5, 14.0) } else { (1.2, 1.5, f32::MAX) };
                let want = (to * gain).clamp_length_max(top) * d.pan_gate;
                d.vel += (want - d.vel) * approach(rate, dt);
            }
            None if d.mode != Mode::Travel || self.postcard_t > 0.0 => d.vel *= 1.0 - approach(1.0, dt),
            None => {}
        }
        self.cam.target.x += d.vel.x * dt;
        self.cam.target.z += d.vel.y * dt;
    }
}
