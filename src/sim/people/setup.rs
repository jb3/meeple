use super::*;

impl TownLife {
    pub fn new(gen: &Gen, town: Rc<Town>, env: &Env) -> Self {
        let lots = gen.lots(&town);
        let r = town.radius + 30;
        let ctx = gen.ctx(town.cx - r, town.cz - r, town.cx + r, town.cz + r);
        let mut rng = Rng::new(town.id as u64);
        let find = |k: LotKind| lots.iter().position(|l| l.kind == k);
        let tavern = find(LotKind::Tavern);
        let church = find(LotKind::Church);
        let shops: Vec<usize> = lots
            .iter()
            .enumerate()
            .filter(|(_, l)| matches!(l.kind, LotKind::Shop | LotKind::Bakery | LotKind::Smithy))
            .map(|(i, _)| i)
            .collect();
        // only finished houses have anyone living in them yet
        let homes: Vec<usize> = lots
            .iter()
            .enumerate()
            .filter(|(_, l)| matches!(l.kind, LotKind::House | LotKind::Apartment) && gen.stage(l) == Stage::Done)
            .map(|(i, _)| i)
            .collect();
        let mut jobs: Vec<(usize, Role)> = lots
            .iter()
            .enumerate()
            .filter_map(|(i, l)| {
                let role = match l.kind {
                    LotKind::Tavern => Role::Barkeep,
                    LotKind::Bakery => Role::Baker,
                    LotKind::Smithy => Role::Smith,
                    LotKind::Shop => Role::Shopkeeper,
                    LotKind::Church => Role::Priest,
                    LotKind::School => Role::Teacher,
                    LotKind::Library => Role::Librarian,
                    LotKind::Farm => Role::Farmer,
                    _ => return None,
                };
                Some((i, role))
            })
            .collect();
        jobs.reverse();

        let mut life = TownLife {
            y: town.level as f32 + 1.0,
            town,
            lots: lots.clone(),
            people: Vec::new(),
            dogs: Vec::new(),
            cats: Vec::new(),
            chickens: Vec::new(),
            ctx,
            rng: rng.clone(),
            tavern,
            shops,
            homes: homes.clone(),
            church,
            last_hour: env.hour as i32,
            gathering: None,
            today: env.day,
            crier_t: 0.0,
            game: None,
            gather_day: i64::MIN,
            visit: None,
            wedding: None,
            wedding_day: i64::MIN,
            married_today: Vec::new(),
            stalls: false,
            peek: None,
            plenty: 1.0,
            happen: happenings::Scene::default(),
            strikes: Vec::new(),
        };

        life.stalls = life.town.city;
        let school = lots.iter().position(|l| l.kind == LotKind::School);
        let cap = if life.town.city { 70 } else { 34 };
        'outer: for &h in &homes {
            let lot = &lots[h];
            for k in 0..lot.residents() {
                if life.people.len() >= cap {
                    break 'outer;
                }
                let seed = hash3(lot.seed, k as i32, 3, 7);
                let r = unit(seed);
                let role = if life.people.len() == 3 && homes.len() >= 6 {
                    Role::Postie
                } else if k > 0 && r < 0.22 {
                    Role::Child
                } else if r > 0.85 {
                    Role::Elder
                } else if let Some((_, role)) = jobs.last() {
                    *role
                } else {
                    Role::Villager
                };
                let work = if !matches!(role, Role::Child | Role::Elder | Role::Villager) {
                    jobs.pop().map(|j| j.0)
                } else {
                    None
                };
                // children go to the school, if the town has one
                let work = if role == Role::Child { school } else { work };
                let p = life.make_person(seed, role, h, work);
                life.people.push(p);
            }
        }
        life.settle_families(env);
        // the easter egg: Bella, who lifts weights in the square
        if happenings::BELLA.load(std::sync::atomic::Ordering::Relaxed) {
            life.add_bella();
        }
        // a few pets
        for i in 0..life.people.len() {
            let p = &life.people[i];
            if p.role != Role::Child && unit(hash_u32(p.seed ^ 0xd06)) < 0.12 {
                let home = Vec2::from(lots[p.home].front);
                let c = [[150, 100, 60], [60, 50, 45], [230, 220, 200], [200, 150, 80]][p.seed as usize % 4];
                life.dogs.push(Dog {
                    owner: i,
                    pos: home,
                    yaw: 0.0,
                    phase: 0.0,
                    col: col(c),
                    target: home,
                    home,
                    bark_t: rng.range(5.0, 30.0),
                    moving: false,
                });
            }
        }
        // and cats, who keep their own counsel
        for i in 0..life.people.len() {
            let p = &life.people[i];
            if p.role != Role::Child && unit(hash_u32(p.seed ^ 0xca7)) < 0.15 {
                let home = Vec2::from(lots[p.home].front);
                let (c, pattern) = match hash_u32(p.seed ^ 0xc01) % 6 {
                    0 => ([0.82, 0.48, 0.2], 2),
                    1 => ([0.5, 0.5, 0.53], 2),
                    2 => ([0.1, 0.1, 0.11], 0),
                    3 => ([0.93, 0.92, 0.88], 0),
                    4 => ([0.12, 0.12, 0.14], 1),
                    _ => ([0.78, 0.6, 0.36], 0),
                };
                life.cats.push(Cat {
                    pos: home,
                    yaw: rng.range(0.0, 6.28),
                    phase: 0.0,
                    col: c,
                    pattern,
                    target: home,
                    home,
                    mode: CatMode::Sit,
                    timer: rng.range(1.0, 10.0),
                    meow_t: rng.range(10.0, 60.0),
                });
            }
        }
        for (i, l) in lots.iter().enumerate() {
            if l.kind == LotKind::Farm {
                for _ in 0..rng.int(2, 5) {
                    let p = Vec2::new(rng.range(l.x0 as f32 + 1.5, l.x1 as f32), rng.range(l.z0 as f32 + 1.5, l.z1 as f32));
                    life.chickens.push(Chicken {
                        pos: p,
                        yaw: rng.range(0.0, 6.28),
                        target: p,
                        lot: i,
                        timer: rng.range(0.0, 3.0),
                        peck: 0.0,
                        cluck_t: rng.range(4.0, 20.0),
                    });
                }
            }
        }
        // drop everyone into the place their schedule says they should be
        for i in 0..life.people.len() {
            life.place_now(i, env);
            let p = &mut life.people[i];
            p.fade = if p.visible { 1.0 } else { 0.0 };
        }
        life
    }

    pub(super) fn make_person(&mut self, seed: u32, role: Role, home: usize, work: Option<usize>) -> Person {
        let h = |k: u32| hash_u32(seed ^ k) as usize;
        let elder = role == Role::Elder;
        let hat = match role {
            Role::Farmer => Hat::Straw,
            Role::Baker => Hat::Chef,
            Role::Smith | Role::Barkeep | Role::Postie => Hat::Cap,
            _ if h(5) % 5 == 0 => Hat::Bun,
            _ if h(5) % 7 == 0 => Hat::Cap,
            _ => Hat::None,
        };
        let look = Look {
            shirt: col(SHIRTS[h(1) % SHIRTS.len()]),
            pants: col(PANTS[h(2) % PANTS.len()]),
            skin: col(SKIN[h(3) % SKIN.len()]),
            hair: if elder { col([215, 215, 220]) } else { col(HAIR[h(4) % HAIR.len()]) },
            hat,
            hat_col: match hat {
                Hat::Straw => col([228, 196, 110]),
                Hat::Chef => col([250, 250, 250]),
                _ if role == Role::Postie => col([60, 80, 150]),
                _ => col(SHIRTS[h(6) % SHIRTS.len()]),
            },
            umbrella: col(UMBRELLA[h(7) % UMBRELLA.len()]),
            scale: match role {
                Role::Child => 0.8,
                _ => 1.14 + unit(h(8) as u32) * 0.12,
            },
        };
        let lot = &self.lots[home];
        Person {
            name: crate::names::person_name(seed),
            role,
            home,
            work,
            pos: Vec2::from(lot.front),
            y: self.y,
            yaw: 0.0,
            path: Vec::new(),
            pi: 0,
            speed: match role {
                Role::Elder => 0.9,
                Role::Child => 1.9,
                _ => 1.25 + unit(h(9) as u32) * 0.35,
            },
            phase: unit(h(10) as u32) * 6.0,
            bella: false,
            jam_pos: Vec2::ZERO,
            jam_t: 0.0,
            ghost_t: 0.0,
            state: State::Inside,
            resume: State::Idle,
            goal: Goal::Home,
            dest: Dest::Lot(home),
            at: Place::Lot(home),
            timer: 0.0,
            chat_cd: 5.0,
            emote: Emote::Talk,
            chat_start: 0.0,
            first_speaker: false,
            talk_slot: -1,
            friend: None,
            look,
            jitter: (unit(h(11) as u32) - 0.5) * 1.4,
            lane: (unit(h(12) as u32) - 0.5) * 1.3,
            visible: false,
            fade: 0.0,
            gather_role: GatherRole::None,
            dance_a: 0.0,
            dance_r: DANCE_RING,
            stall_best: f32::MAX,
            stall_t: 0.0,
            heading: Vec2::ZERO,
            coming: false,
            carry: Carry::Nothing,
            delivered: i64::MIN,
            wed: Wed::None,
            married: false,
            born: None,
            new_baby: false,
            work_t: 0.0,
            seed,
        }
    }

    /// The easter egg's weightlifter: a villager like the rest, who lifts
    /// weights in the square all day and goes home to one of the houses at night.
    pub fn add_bella(&mut self) {
        if self.homes.is_empty() {
            return;
        }
        let tid = self.town.id;
        let home = self.homes[hash_u32(tid ^ 0xbe11a) as usize % self.homes.len()];
        let mut p = self.make_person(hash_u32(tid ^ 0xbe11a5), Role::Villager, home, None);
        p.name = "Bella";
        p.bella = true;
        // always the same man: an orange vest, dark trousers, and his own skin and hair
        p.look.shirt = [0.95, 0.5, 0.15];
        p.look.pants = [0.12, 0.12, 0.16];
        p.look.hat = Hat::None;
        p.look.scale = 1.12;
        self.people.push(p);
    }

    /// Where Bella's mat and mirror are, in the square.
    pub fn bella_spot(&self) -> Vec2 {
        self.center() + Vec2::new(8.4, -2.8)
    }
}
