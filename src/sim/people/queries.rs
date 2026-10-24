use super::*;

impl TownLife {
    /// Where this villager is, if they're out in the open.
    pub fn outdoor_pos(&self, i: usize) -> Option<Vec3> {
        self.people.get(i).filter(|p| p.visible && p.state != State::Inside).map(|p| p.pos3())
    }

    /// How many children are out playing within `r` of `c`.
    pub fn children_near(&self, c: Vec2, r: f32) -> usize {
        self.people.iter().filter(|p| p.role == Role::Child && p.visible && p.state != State::Inside && p.pos.distance(c) < r).count()
    }

    /// The inn's front door, if the town has one.
    pub fn tavern_front(&self) -> Option<Vec2> {
        self.tavern.map(|t| Vec2::from(self.lots[t].front))
    }

    /// Is this villager out in the open (not indoors, not faded away)?
    pub fn outdoors(&self, i: usize) -> bool {
        self.people.get(i).is_some_and(|p| p.visible && p.state != State::Inside)
    }

    /// Is this point inside the walls of a building (not its yard)?
    pub fn inside_building(&self, p: Vec2) -> bool {
        self.lots.iter().any(|l| {
            if matches!(l.kind, LotKind::Farm | LotKind::Garden) {
                return false;
            }
            let (x0, z0, x1, z1) = l.footprint();
            p.x > x0 as f32 + 0.05 && p.x < x1 as f32 + 0.95 && p.y > z0 as f32 + 0.05 && p.y < z1 as f32 + 0.95
        })
    }

    /// Is bench `b` claimed by anyone other than person `me` (walking to it
    /// or sitting on it)?
    pub(super) fn bench_taken(&self, b: usize, me: usize) -> bool {
        self.people.iter().enumerate().any(|(k, q)| {
            k != me && q.dest == Dest::Bench(b) && matches!(q.state, State::Walking | State::Sitting | State::Chatting(_))
        })
    }

    /// Where people are standing or walking, for traffic to give way to.
    pub fn walkers(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.people.iter().filter(|p| p.visible).map(|p| p.pos)
    }

    pub fn visible_count_near(&self, p: Vec2, r: f32) -> usize {
        self.people.iter().filter(|q| q.visible && q.pos.distance(p) < r).count()
    }
}
