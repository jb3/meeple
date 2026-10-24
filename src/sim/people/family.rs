//! Families: who is whose parent, babies born to households, children
//! growing up and taking on a parent's trade. A household's births depend
//! only on its house and the day, so a town that has been out of view finds
//! the same families, one year older, when the camera comes back.

use super::*;

/// Days a newborn is carried about in a pram before being a toddler on foot.
pub(super) const BABY_DAYS: i64 = 7;
/// Births per thousand days for a household with two grown-ups.
const BIRTH_PER_THOUSAND: u32 = 14;
/// Fewest days between babies in one house.
const BIRTH_GAP: i64 = 20;
/// Most people one house takes in through births.
const MAX_HOUSEHOLD: usize = 6;
/// How far back a town looks for births when it's loaded (days).
const HISTORY: i64 = 400;

/// The days babies were born in the house with this seed, up to `upto`.
pub(super) fn birth_days(lot_seed: u32, first: i64, upto: i64) -> Vec<i64> {
    let mut out: Vec<i64> = Vec::new();
    let mut last = i64::MIN / 2;
    for d in first.max(upto - HISTORY)..=upto {
        if d - last >= BIRTH_GAP && hash3(lot_seed, d as i32, 0xb1, 3) % 1000 < BIRTH_PER_THOUSAND {
            out.push(d);
            last = d;
        }
    }
    out
}

/// Trades a grown child may take over from a parent. (Bakers, vicars and
/// the like are one to a town, and the schedules assume it.)
fn inheritable(role: Role) -> bool {
    matches!(role, Role::Farmer | Role::Smith | Role::Shopkeeper)
}

impl TownLife {
    /// The household's grown-ups who are in their prime: whoever counts as
    /// the parents.
    fn parents_of(&self, home: usize) -> Vec<usize> {
        (0..self.people.len())
            .filter(|&i| self.people[i].home == home && !matches!(self.people[i].role, Role::Child | Role::Elder))
            .take(2)
            .collect()
    }

    pub fn surname(&self, i: usize) -> &'static str {
        crate::names::surname(self.lots[self.people[i].home].seed)
    }

    pub fn full_name(&self, i: usize) -> String {
        format!("{} {}", self.people[i].name, self.surname(i))
    }

    /// When the town is first loaded: the babies born in its houses over the
    /// past while, as children or grown folk by now.
    pub(super) fn settle_families(&mut self, env: &Env) {
        let first = env.gen.first_day.get();
        for h in self.homes.clone() {
            let parents = self.parents_of(h);
            if parents.len() < 2 {
                continue;
            }
            let lot_seed = self.lots[h].seed;
            for b in birth_days(lot_seed, 0, env.day) {
                let size = self.people.iter().filter(|p| p.home == h).count();
                if size >= MAX_HOUSEHOLD {
                    break;
                }
                if b + BABY_DAYS <= env.day {
                    self.add_baby(h, b);
                } else {
                    self.people[parents[0]].new_baby = true;
                }
            }
        }
        for i in 0..self.people.len() {
            if self.people[i].role == Role::Child && env.day >= self.people[i].grown_day(first) {
                self.grow_up(i);
            }
        }
    }

    /// The child a birth on `day` has become, a week on.
    fn add_baby(&mut self, home: usize, day: i64) {
        let seed = hash3(self.lots[home].seed, day as i32, 0xc41d, 7);
        let school = self.lots.iter().position(|l| l.kind == LotKind::School);
        let mut p = self.make_person(seed, Role::Child, home, school);
        p.born = Some(day);
        self.people.push(p);
    }

    /// Overnight: babies arrive, and last week's babies become children.
    pub(super) fn family_day(&mut self, env: &Env) {
        let since = self.today;
        for h in self.homes.clone() {
            let parents = self.parents_of(h);
            if parents.len() < 2 {
                continue;
            }
            let size = self.people.iter().filter(|p| p.home == h).count();
            for b in birth_days(self.lots[h].seed, 0, env.day) {
                if b > since && b <= env.day && size < MAX_HOUSEHOLD {
                    self.people[parents[0]].new_baby = true;
                    let (a, c) = (self.full_name(parents[0]), self.people[parents[1]].name);
                    let note = format!("A baby was born to {a} and {c} in {}.", self.town.name);
                    self.happen.log.push((env.day, note));
                    if self.happen.log.len() > 40 {
                        self.happen.log.remove(0);
                    }
                }
                if b + BABY_DAYS > since && b + BABY_DAYS <= env.day && size < MAX_HOUSEHOLD {
                    self.people[parents[0]].new_baby = false;
                    self.add_baby(h, b);
                }
            }
        }
    }

    /// A child comes of age: they're a grown villager, and perhaps take over
    /// a parent's trade.
    pub(super) fn come_of_age(&mut self, i: usize, env: &Env) {
        self.grow_up(i);
        let home = self.people[i].home;
        let heir = self
            .parents_of(home)
            .into_iter()
            .find(|&k| inheritable(self.people[k].role) && self.people[k].work.is_some());
        let mut note = format!("{} came of age in {}", self.full_name(i), self.town.name);
        if let Some(k) = heir.filter(|_| hash_u32(self.people[i].seed ^ 0x7ade) % 10 < 6) {
            let (role, work) = (self.people[k].role, self.people[k].work);
            self.people[i].role = role;
            self.people[i].work = work;
            note.push_str(&format!(", and joined {} at {} as a {}", self.people[k].name, self.place_name(work.unwrap()), role.label()));
        }
        note.push('.');
        if self.people[i].born.is_some() {
            self.happen.log.push((env.day, note));
            if self.happen.log.len() > 40 {
                self.happen.log.remove(0);
            }
        }
    }

    fn grow_up(&mut self, i: usize) {
        let p = &mut self.people[i];
        p.role = Role::Villager;
        p.work = None;
        p.look.scale = 1.14 + unit(hash_u32(p.seed ^ 0x6a0)) * 0.12;
        p.speed = 1.3;
    }

    /// Lines about someone's kin for their card.
    pub(super) fn family_lines(&self, i: usize) -> Vec<String> {
        let p = &self.people[i];
        let kin: Vec<usize> = (0..self.people.len()).filter(|&k| k != i && self.people[k].home == p.home).collect();
        let parents = self.parents_of(p.home);
        let names = |v: &[usize]| -> String {
            let n: Vec<&str> = v.iter().map(|&k| self.people[k].name).collect();
            match n.len() {
                0 => String::new(),
                1 => n[0].to_string(),
                _ => format!("{} and {}", n[..n.len() - 1].join(", "), n[n.len() - 1]),
            }
        };
        let mut out = Vec::new();
        let is_parent = parents.contains(&i);
        let children: Vec<usize> = kin.iter().copied().filter(|&k| self.people[k].role == Role::Child || self.people[k].born.is_some()).collect();
        let elders: Vec<usize> = kin.iter().copied().filter(|&k| self.people[k].role == Role::Elder).collect();
        if p.role == Role::Child || (p.born.is_some() && !is_parent && p.role != Role::Elder) {
            let ps: Vec<usize> = parents.iter().copied().filter(|&k| k != i).collect();
            if !ps.is_empty() {
                out.push(format!("Child of {}", names(&ps)));
            }
            let sibs: Vec<usize> = kin.iter().copied().filter(|&k| !parents.contains(&k) && self.people[k].role != Role::Elder).collect();
            if !sibs.is_empty() {
                out.push(format!("Sibling of {}", names(&sibs)));
            }
            if !elders.is_empty() {
                out.push(format!("Grandchild of {}", names(&elders)));
            }
        } else if is_parent {
            let kids: Vec<usize> = children.iter().copied().filter(|k| !parents.contains(k)).collect();
            if !kids.is_empty() {
                out.push(format!("Parent of {}", names(&kids)));
            }
        } else if p.role == Role::Elder && !children.is_empty() {
            out.push(format!("Grandparent of {}", names(&children)));
        }
        if p.new_baby {
            out.push("Has a newborn at home".into());
        }
        out
    }
}
