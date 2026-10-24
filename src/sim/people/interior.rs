//! Looking in at home: with the camera close, a house with someone in has
//! its roof cut away (by the see-through effect) and shows a furnished room.

use super::happenings::figures::resident;
use super::*;

impl TownLife {
    pub fn set_peek(&mut self, lot: Option<usize>) {
        self.peek = lot;
    }

    /// The nearest house within a few steps of `p` with someone indoors. While
    /// a house is on fire the burning house is the thing to see, so no roof
    /// comes off anywhere in town.
    pub fn lit_house_near(&self, p: Vec2) -> Option<usize> {
        if self.happening(super::happenings::Kind::Fire) {
            return None;
        }
        self.homes
            .iter()
            .copied()
            .filter(|&l| self.people.iter().any(|q| q.state == State::Inside && q.at == Place::Lot(l)))
            .map(|l| {
                let (x, z) = self.lots[l].center();
                (l, Vec2::new(x, z).distance(p))
            })
            .filter(|&(_, d)| d < 13.0)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(l, _)| l)
    }

    /// What the see-through effect clears for a house: the room itself laid
    /// open (a negative radius asks the shader for that), and a line of sight
    /// to it, so that whatever stands between the camera and the house goes
    /// too. Two (x, y, z, radius) circles.
    pub fn peek_foci(&self, l: usize) -> [f32; 8] {
        let lot = &self.lots[l];
        let (x, z) = lot.center();
        let half = ((lot.x1 - lot.x0).max(lot.z1 - lot.z0) as f32 + 1.0) * 0.5;
        [x, self.y, z, -(half + 0.2), x, self.y, z, half + 1.2]
    }

    /// The room on the ground floor: a rug and table, a bed against the far
    /// wall, a fire in the hearth, and whoever's home going about it.
    pub(super) fn draw_interior(&self, b: &mut Boxes, fx: &mut Boxes, env: &Env, l: usize) {
        let lot = &self.lots[l];
        let (cx, cz) = lot.center();
        let y = self.y;
        let (dx, dz) = (lot.door.0 as f32 + 0.5 - cx, lot.door.1 as f32 + 0.5 - cz);
        // which way the door lies, so the bed and hearth go on the far side
        let (ax, az) = if dx.abs() > dz.abs() { (-dx.signum(), 0.0) } else { (0.0, -dz.signum()) };
        let (px, pz) = (-az, ax);
        let w = ((lot.x1 - lot.x0) as f32 * 0.5 - 1.0).max(1.0);
        let d = ((lot.z1 - lot.z0) as f32 * 0.5 - 1.0).max(1.0);
        let reach = if ax != 0.0 { w } else { d };
        let wood = [0.5, 0.36, 0.24];
        let at = |u: f32, v: f32| Vec3::new(cx + ax * u + px * v, y, cz + az * u + pz * v);
        let tone = (hash_u32(lot.seed) % 4) as usize;
        let rug = [[0.62, 0.22, 0.2], [0.25, 0.4, 0.62], [0.3, 0.5, 0.32], [0.7, 0.55, 0.25]][tone];
        let p = at(0.0, 0.0);
        b.push(Vec3::new(p.x, y + 0.03, p.z), Vec3::new(1.5, 0.03, 1.1), yaw_of(Vec2::new(ax, az)), rug, 0.0);
        // table and stools
        b.push(Vec3::new(p.x, y + 0.62, p.z), Vec3::new(0.55, 0.05, 0.4), yaw_of(Vec2::new(ax, az)), wood, 0.0);
        for (u, v) in [(-0.42, -0.28), (0.42, -0.28), (-0.42, 0.28), (0.42, 0.28)] {
            let q = at(u, v);
            b.push(Vec3::new(q.x, y + 0.3, q.z), Vec3::new(0.04, 0.3, 0.04), 0.0, wood, 0.0);
        }
        for v in [-0.95f32, 0.95] {
            let q = at(0.0, v);
            b.push(Vec3::new(q.x, y + 0.3, q.z), Vec3::new(0.2, 0.3, 0.2), 0.0, wood, 0.0);
        }
        // a bed in the far corner, with a coloured blanket
        let bed = at(reach - 0.6, -d.min(w) * 0.5);
        let sheet = [[0.9, 0.9, 0.85], [0.85, 0.85, 0.92]][tone % 2];
        b.push(Vec3::new(bed.x, y + 0.32, bed.z), Vec3::new(0.5, 0.18, 0.9), yaw_of(Vec2::new(ax, az)), wood, 0.0);
        b.push(Vec3::new(bed.x, y + 0.55, bed.z), Vec3::new(0.46, 0.08, 0.84), yaw_of(Vec2::new(ax, az)), rug, 0.0);
        b.push(Vec3::new(bed.x, y + 0.66, bed.z - az * 0.6), Vec3::new(0.3, 0.07, 0.2), 0.0, sheet, 0.0);
        // a hearth on the far wall, lit in the cold and the dark
        let hearth = at(reach, d.min(w) * 0.4);
        b.push(Vec3::new(hearth.x, y + 0.5, hearth.z), Vec3::new(0.5, 0.5, 0.5), 0.0, [0.45, 0.43, 0.42], 0.0);
        let fire = env.night > 0.3 || env.season > 2.5 || env.rain > 0.3;
        if fire {
            let flick = 0.8 + 0.2 * (env.time * 9.0 + lot.seed as f32).sin();
            fx.push(Vec3::new(hearth.x - ax * 0.25, y + 0.4, hearth.z - az * 0.25), Vec3::new(0.25, 0.2 * flick, 0.25), 0.0, [1.0, 0.6, 0.2], MAT_GLOW);
        }
        // whoever's at home, round the room
        let mut k = 0;
        for q in self.people.iter().filter(|q| q.state == State::Inside && q.at == Place::Lot(l)) {
            let spots = [(0.0, 1.0), (0.0, -1.0), (1.3, 0.0), (-1.3, 0.6), (0.9, -1.4)];
            let (u, v) = spots[k % spots.len()];
            k += 1;
            let drift = (env.time * 0.3 + q.phase).sin() * 0.12;
            let pos = at(u + drift, v);
            let face = at(0.0, 0.0) - pos;
            resident(b, q, pos, yaw_of(Vec2::new(face.x, face.z)), env.time, false);
        }
    }
}
