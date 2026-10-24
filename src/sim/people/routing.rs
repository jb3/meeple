use super::*;

impl TownLife {
    pub(super) fn center(&self) -> Vec2 {
        Vec2::new(self.town.cx as f32 + 0.5, self.town.cz as f32 + 0.5)
    }

    pub(super) fn line(&self, v: f32, c: i32) -> f32 {
        let k = ((v - c as f32 - 0.5) / STREET as f32).round();
        c as f32 + 0.5 + k * STREET as f32
    }

    pub(super) fn intersection(&self, p: Vec2) -> Vec2 {
        Vec2::new(self.line(p.x, self.town.cx), self.line(p.y, self.town.cz))
    }

    pub(super) fn route(&self, a: Vec2, b: Vec2, flip: bool) -> Vec<Vec2> {
        let same_v = (a.x - b.x).abs() < 0.3 && (a.x - self.line(a.x, self.town.cx)).abs() < 0.6;
        let same_h = (a.y - b.y).abs() < 0.3 && (a.y - self.line(a.y, self.town.cz)).abs() < 0.6;
        if same_v || same_h {
            return vec![b];
        }
        let ia = self.intersection(a);
        let ib = self.intersection(b);
        let corner = if flip { Vec2::new(ib.x, ia.y) } else { Vec2::new(ia.x, ib.y) };
        let mut out: Vec<Vec2> = Vec::new();
        for p in [ia, corner, ib, b] {
            if out.last().map_or(true, |q| q.distance(p) > 0.3) {
                out.push(p);
            }
        }
        out
    }

    pub(super) fn plaza_point(&mut self) -> Vec2 {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        // far enough out that even diagonal spots clear the fountain, and
        // inside the dance ring while a gathering is on
        let outer = if self.gathering.is_some() { DANCE_RING - 1.4 } else { 8.0 };
        let r = self.rng.range(4.5, outer);
        self.center() + Vec2::new(a.cos(), a.sin()) * r
    }

    pub(super) fn plaza_arm(&self, toward: Vec2) -> Vec2 {
        let c = self.center();
        let d = toward - c;
        if d.x.abs() > d.y.abs() {
            c + Vec2::new(7.0 * d.x.signum(), 0.0)
        } else {
            c + Vec2::new(0.0, 7.0 * d.y.signum())
        }
    }

    pub(super) fn bench(&self, i: usize) -> (Vec2, Vec2) {
        let (dx, dz) = [(-5, -3), (5, 3), (-3, 5), (3, -5)][i % 4];
        let x = (self.town.cx + dx) as f32 + 0.5;
        let z = (self.town.cz + dz) as f32;
        (Vec2::new(x, z + 0.48), Vec2::new(x, z - 0.6))
    }

    pub(super) fn field_point(&mut self, l: usize) -> Vec2 {
        let lot = &self.lots[l];
        Vec2::new(
            self.rng.range(lot.x0 as f32 + 1.5, lot.x1 as f32 - 0.5),
            self.rng.range(lot.z0 as f32 + 1.5, lot.z1 as f32 - 0.5),
        )
    }

    pub(super) fn plan(&mut self, i: usize, dest: Dest) {
        let flip = self.rng.chance(0.5);
        let lots = self.lots.clone();
        let (pos, at, lane) = {
            let p = &self.people[i];
            (p.pos, p.at, p.lane)
        };
        let mut pts: Vec<Vec2> = Vec::new();
        let start = match at {
            Place::Lot(l) => {
                pts.push(Vec2::from(lots[l].front));
                Vec2::from(lots[l].street)
            }
            Place::Plaza => {
                let target_hint = match dest {
                    Dest::Lot(l) | Dest::Field(l) | Dest::Anvil(l) => Vec2::from(lots[l].street),
                    _ => pos,
                };
                let arm = self.plaza_arm(target_hint);
                if matches!(dest, Dest::Plaza | Dest::Bench(_) | Dest::Gather | Dest::Stall(_) | Dest::Gym) {
                    let (seat, front) = self.plaza_target(i, dest);
                    pts.push(front);
                    if seat != front {
                        pts.push(seat);
                    }
                    self.set_path(i, pts, dest);
                    return;
                }
                pts.push(arm);
                arm
            }
        };
        let (goal_street, tail): (Vec2, Vec<Vec2>) = match dest {
            // end on the doorstep: the door itself is a solid block, and
            // anyone fading inside it would just seem to vanish
            Dest::Lot(l) => (Vec2::from(lots[l].street), vec![Vec2::from(lots[l].front)]),
            Dest::Field(l) => {
                let fp = self.field_point(l);
                (Vec2::from(lots[l].street), vec![Vec2::from(lots[l].front), fp])
            }
            Dest::Anvil(l) => {
                let lot = &lots[l];
                let f = Vec2::from(lot.front);
                let side = if lot.face % 2 == 0 { Vec2::new(1.0, 0.0) } else { Vec2::new(0.0, 1.0) };
                (Vec2::from(lot.street), vec![f + side * 1.0])
            }
            Dest::Plaza | Dest::Bench(_) | Dest::Gather | Dest::Stall(_) | Dest::Gym => {
                let arm = self.plaza_arm(start);
                let (seat, front) = self.plaza_target(i, dest);
                let mut tail = Vec::new();
                tail.push(front);
                if seat != front {
                    tail.push(seat);
                }
                (arm, tail)
            }
        };
        pts.push(start);
        let off = Vec2::splat(lane);
        for p in self.route(start, goal_street, flip) {
            pts.push(p + off);
        }
        pts.extend(tail);
        self.set_path(i, pts, dest);
    }

    /// Where in the plaza to head for: (final spot, approach point).
    pub(super) fn plaza_target(&mut self, i: usize, dest: Dest) -> (Vec2, Vec2) {
        match dest {
            Dest::Bench(b) => self.bench(b),
            Dest::Gym => {
                let p = self.bella_spot();
                (p, p)
            }
            Dest::Gather => {
                let p = self.gather_spot(i);
                (p, p)
            }
            Dest::Stall(k) => {
                // stallholders stand behind the table, shoppers in front
                let spots = self.stall_spots();
                let Some(&(c, out)) = spots.get(k) else {
                    let p = self.plaza_point();
                    return (p, p);
                };
                let p = if self.people[i].goal == Goal::Market { c - out * 0.9 } else { c + out * 1.1 };
                (p, p)
            }
            _ => {
                let p = self.plaza_point();
                (p, p)
            }
        }
    }

    /// Give person `i` a part in tonight's gathering and a place to stand:
    /// the first to arrive plays, then most dance and some sing.
    pub(super) fn gather_spot(&mut self, i: usize) -> Vec2 {
        let c = self.center();
        let seed = self.people[i].seed;
        let Some(g) = self.gathering.as_mut() else { return self.plaza_point() };
        let role = if g.musician.is_none() || g.musician == Some(i) {
            g.musician = Some(i);
            GatherRole::Musician
        } else if unit(hash_u32(seed ^ 0x51)) < 0.35 {
            GatherRole::Singer
        } else {
            GatherRole::Dancer
        };
        let k = g.joined as f32;
        g.joined += 1;
        let wed = self.people[i].wed;
        let role = if matches!(wed, Wed::Bride | Wed::Groom) { GatherRole::Dancer } else { role };
        if g.musician == Some(i) && role != GatherRole::Musician {
            g.musician = None;
        }
        self.people[i].gather_role = role;
        let at = |a: f32, r: f32| c + Vec2::new(a.cos(), a.sin()) * r;
        match wed {
            Wed::Bride => return at(3.6, COUPLE_RING),
            Wed::Groom => return at(3.6 + 0.3, COUPLE_RING),
            _ => {}
        }
        match role {
            GatherRole::Musician => at(0.3, 4.3),
            // golden-angle spacing spreads people evenly round the ring
            GatherRole::Singer => at(k * 2.39996 + 0.5, SING_RING),
            _ => {
                let mut a = k * 2.39996;
                if self.stalls {
                    // city plazas have market stalls: find a clear spot
                    for _ in 0..16 {
                        if !self.near_stall(at(a, DANCE_RING)) {
                            break;
                        }
                        a += 0.4;
                    }
                }
                at(a, DANCE_RING)
            }
        }
    }

    pub(super) fn set_path(&mut self, i: usize, pts: Vec<Vec2>, dest: Dest) {
        self.people[i].stall_best = f32::MAX;
        self.people[i].stall_t = 0.0;
        if dest != Dest::Gather {
            self.people[i].gather_role = GatherRole::None;
            if let Some(g) = self.gathering.as_mut() {
                if g.musician == Some(i) {
                    g.musician = None;
                }
            }
        }
        let pts = avoid_square(self.people[i].pos, &pts, self.center(), FOUNTAIN_CLEAR, FOUNTAIN_DETOUR);
        let p = &mut self.people[i];
        p.path = pts;
        p.pi = 0;
        p.dest = dest;
        p.state = State::Walking;
        p.visible = true;
    }
}
