use super::*;

pub(super) fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

impl TownLife {
    pub fn set_plenty(&mut self, plenty: f32) {
        self.plenty = plenty;
    }

    pub fn describe(&self, i: usize) -> String {
        let p = &self.people[i];
        format!("{} the {}", p.name, p.role.label())
    }

    pub(super) fn place_name(&self, l: usize) -> &'static str {
        match self.lots[l].kind {
            LotKind::Tavern => "the inn",
            LotKind::Bakery => "the bakery",
            LotKind::Smithy => "the smithy",
            LotKind::Shop => "the shop",
            LotKind::Church => "the church",
            LotKind::School => "the school",
            LotKind::Library => "the library",
            LotKind::Farm => "the farm",
            LotKind::Manor => "the manor",
            _ => "a house",
        }
    }

    /// How interesting someone is to watch right now.
    pub fn spotlight(&self, i: usize) -> f32 {
        let p = &self.people[i];
        if !p.visible || p.state == State::Inside {
            return 0.0;
        }
        let mut w = match p.state {
            State::Playing => 2.6,
            State::Singing => 1.6,
            State::Dancing => 1.2,
            State::Working if matches!(p.role, Role::Smith | Role::Baker) => 1.3,
            State::Working => 0.7,
            State::Walking => 0.45,
            State::Sitting => 0.5,
            _ => 0.2,
        };
        if p.pram() && p.carry == Carry::Pram {
            w += 0.8;
        }
        if p.carry != Carry::Nothing {
            w += 0.4;
        }
        if matches!(p.wed, Wed::Bride | Wed::Groom) {
            w += 3.0;
        }
        if p.role == Role::Postie && p.goal == Goal::Work {
            w += 0.7;
        }
        if p.goal == Goal::Play {
            w += 0.8;
        }
        w
    }

    /// Stable across visits, unlike a person's position in the list.
    pub fn person_seed(&self, i: usize) -> u32 {
        self.people[i].seed
    }

    /// What someone's up to, in words.
    pub fn doing(&self, i: usize) -> String {
        let p = &self.people[i];
        if p.bella {
            return "PUMPING IRON!".into();
        }
        let name = |k: usize| self.people.get(k).map_or("someone", |q| q.name);
        let to = match p.goal {
            Goal::Home => "home".to_string(),
            Goal::Work => match p.role {
                Role::Child => "to school".into(),
                Role::Postie => "on the post round".into(),
                Role::Farmer => "out to the fields".into(),
                _ => "to work".into(),
            },
            Goal::Tavern => "to the inn".into(),
            Goal::Errand => "on an errand".into(),
            Goal::Visit => match p.friend {
                Some(f) => format!("to visit {}", name(f)),
                None => "visiting".into(),
            },
            Goal::Plaza => "to the square".into(),
            Goal::Play => "out to play".into(),
            Goal::Bench => "for a sit down".into(),
            Goal::Gather => "to the dancing".into(),
            Goal::Market => "to mind the market stall".into(),
            Goal::Shop => "to the market".into(),
            Goal::Deliver => "with bread for the inn".into(),
            Goal::Wedding => "to church for the wedding".into(),
            Goal::Gym => "to the weights".into(),
        };
        match p.state {
            State::Inside => match p.at {
                Place::Lot(l) if l == p.home => "At home".into(),
                Place::Lot(l) => format!("Inside {}", self.place_name(l)),
                Place::Plaza => "In the square".into(),
            },
            State::Walking => format!("On the way {to}"),
            State::Chatting(o) => format!("Chatting with {}", name(o)),
            State::Sitting => "Sitting on a bench".into(),
            State::Working => match p.role {
                Role::Farmer => "Working the fields".into(),
                Role::Smith => "Hammering at the anvil".into(),
                _ => "Minding the market stall".into(),
            },
            State::Dancing => "Dancing".into(),
            State::Singing => "Singing".into(),
            State::Playing => "Playing the lute".into(),
            State::Idle => match p.goal {
                Goal::Play => "Playing in the square".into(),
                Goal::Shop => "Browsing the market".into(),
                Goal::Work if p.role == Role::Postie => "Delivering the post".into(),
                _ => "Taking the air".into(),
            },
        }
    }

    /// The lines of a villager's card: who they are, where they live, who
    /// they know, and what they're doing.
    pub fn card(&self, i: usize) -> (String, Vec<String>) {
        let p = &self.people[i];
        let name = |k: usize| self.people.get(k).map_or("someone", |q| q.name);
        let mut lines = vec![format!("{} of {}", capitalise(p.role.label()), self.town.name)];
        if let Some(w) = p.work.filter(|_| !matches!(p.role, Role::Child | Role::Villager | Role::Elder)) {
            lines.push(format!("Works at {}", self.place_name(w)));
        }
        if p.role == Role::Child && p.work.is_some() {
            lines.push("Goes to the village school".into());
        }
        let household = self.people.iter().filter(|q| q.home == p.home).count();
        lines.push(match household {
            1 => "Lives alone".into(),
            n => format!("Lives with {} other{}", n - 1, if n > 2 { "s" } else { "" }),
        });
        lines.extend(self.family_lines(i));
        if let Some(f) = p.friend {
            lines.push(format!("{} {}", if p.married { "Married to" } else { "Best friend:" }, name(f)));
        }
        match p.wed {
            Wed::Bride | Wed::Groom => lines.push("Getting married today!".into()),
            Wed::Guest => lines.push("A guest at today's wedding".into()),
            Wed::None => {}
        }
        if p.coming && self.gathering.is_some() {
            lines.push("Going to tonight's dancing".into());
        }
        if p.pram() && !p.new_baby {
            lines.push("A new baby at home".into());
        }
        lines.push(format!("Now: {}", self.doing(i)));
        (self.full_name(i), lines)
    }

    /// "Today in …": the day's goings-on, for the village newsletter.
    pub fn news(&self, env: &Env) -> Vec<String> {
        let mut out = Vec::new();
        let h = env.hour;
        // market every third day
        let market_day = (env.day + (self.town.id % 3) as i64).rem_euclid(3) == 0;
        if market_day {
            out.push(if h < 12.5 { "Market day: stalls in the square until half past twelve.".into() } else { "It was market day; the stalls are packed away.".into() });
        } else {
            let next = (0..3).find(|d| (env.day + d + (self.town.id % 3) as i64).rem_euclid(3) == 0).unwrap_or(1);
            out.push(format!("Next market {}.", if next == 1 { "tomorrow".to_string() } else { format!("in {next} days") }));
        }
        if self.plenty > 1.15 {
            out.push("A bumper harvest: the stalls are heaped high.".into());
        } else if self.plenty < 0.55 {
            out.push("A poor season: lean pickings at market, and prices are up.".into());
        }
        if let Some((a, b)) = self.couple() {
            let vowed = self.vows_said();
            out.push(if vowed {
                format!("Wedding bells! {a} and {b} are married, and there's dancing in the square.")
            } else {
                format!("{a} and {b} are to be married at three o'clock at the church.")
            });
        } else if let Some(&(a, b)) = self.married_today.iter().last() {
            out.push(format!("Congratulations to {} and {}, married today.", self.people[a].name, self.people[b].name));
        }
        match &self.gathering {
            Some(g) if g.festival != Festival::Wedding => {
                out.push(format!("{} in the square tonight; {} said they'd come.", g.festival.name(), g.expected))
            }
            _ if h < GATHER_HOURS.start && env.nice => out.push(format!("Fine weather: perhaps a {} this evening.", Festival::for_season(env.season).name().to_lowercase())),
            _ => {}
        }
        let first = env.gen.first_day.get();
        let (mut going_up, mut moved_in) = (0, 0);
        for l in self.lots.iter() {
            match l.stage(first, env.day) {
                Stage::Plot | Stage::Frame(_) => going_up += 1,
                Stage::Done if l.finished_day(first) == Some(env.day) => moved_in += 1,
                _ => {}
            }
        }
        if moved_in > 0 {
            out.push(format!("Welcome to the {} moving into {} today.", if moved_in == 1 { "new family" } else { "new families" }, if moved_in == 1 { "a new house" } else { "new houses" }));
        }
        if going_up > 0 {
            out.push(format!("Builders at work: {going_up} new house{} going up.", if going_up == 1 { "" } else { "s" }));
        }
        let grown = self.people.iter().filter(|p| p.role == Role::Child && p.grown_day(first) == env.day + 1).count();
        if grown > 0 {
            out.push(format!("{grown} of the village children come of age tomorrow."));
        }
        if self.church.is_some() {
            out.push("Church bells ring at eight, noon and six.".into());
        }
        if self.homes.len() >= 6 {
            out.push("The post goes round between eight and noon.".into());
        }
        out.push(format!("{} folk live here.", self.people.len()));
        out
    }
}
