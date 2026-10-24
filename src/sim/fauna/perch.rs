use super::*;

impl Fauna {
    /// An owl sits on its treetop, and now and then flies to another.
    /// How high an owl sits on perch `i`: on the crown while the tree's in
    /// leaf, lower down on the bare trunk's end as the leaves fall.
    pub(super) fn perch_y(g: &Group, i: usize, bare: f32) -> f32 {
        let t = (bare / 0.85).clamp(0.0, 1.0);
        let hi = g.perches[i].y;
        hi + (g.perch_low[i] - hi) * t
    }

    pub(super) fn update_owl(g: &mut Group, dt: f32, rng: &mut Rng, bare: f32) {
        let on = g.on_perch;
        let rest_y = if on < g.perches.len() { Some(Self::perch_y(g, on, bare)) } else { None };
        let Some(a) = g.members.first_mut() else { return };
        a.phase += dt;
        a.timer -= dt;
        if a.flee <= 0.0 && a.timer <= 0.0 && g.perches.len() > 1 {
            let i = rng.int(0, g.perches.len() as i32) as usize;
            let next = g.perches[i];
            g.on_perch = i;
            a.target = Vec2::new(next.x, next.z);
            a.graze = next.y + (g.perch_low[i] - next.y) * (bare / 0.85).clamp(0.0, 1.0);
            a.flee = 1.0;
            a.timer = rng.range(20.0, 45.0);
        }
        // sitting still, the owl follows the leaves down as they fall
        if a.flee <= 0.0 {
            if let Some(y) = rest_y {
                a.y += (y - a.y) * approach(1.0, dt);
            }
        }
        if a.flee > 0.0 {
            let d = a.target - a.pos;
            let dist = d.length();
            if dist < 0.2 {
                a.pos = a.target;
                a.y = a.graze;
                a.flee = 0.0;
            } else {
                a.pos += d / dist * (3.0 * dt).min(dist);
                a.yaw = yaw_of(d);
                // a gentle swoop between the trees
                a.y += (a.graze + (dist.min(6.0) * 0.4) - a.y) * approach(3.0, dt);
            }
        }
    }
}
