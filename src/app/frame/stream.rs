use super::super::*;

impl App {
    /// Ask the world for the chunks the view needs (within a time budget), hand
    /// the freshly meshed ones to the renderer and drop those far outside.
    pub(super) fn stream_world(&mut self, here: Vec2, now: &dyn Fn() -> f64) {
        let (lo, hi) = self.cam.ground_rect();
        let pad = 16.0;
        let (c0, c1) = (
            World::chunk_of(lo.x - pad, lo.y - pad),
            World::chunk_of(hi.x + pad, hi.y + pad),
        );
        let center = World::chunk_of(here.x, here.y);
        let mut wanted = Vec::new();
        for cz in c0.1..=c1.1 {
            for cx in c0.0..=c1.0 {
                wanted.push((cx, cz));
            }
        }
        wanted.sort_by_key(|k| (k.0 - center.0).pow(2) + (k.1 - center.1).pow(2));
        let budget = if self.r.has_chunk(center) { self.quality.mesh_budget() } else { 40.0 };
        let built = self.world.stream(&wanted, budget, now);
        if built.is_empty() && !self.world.chunks.is_empty() {
            self.settled = true;
        }
        for b in built {
            self.r.add_chunk(b.key, &b.solid, &b.water);
        }
        let keep = |k: (i32, i32)| k.0 >= c0.0 - 2 && k.0 <= c1.0 + 2 && k.1 >= c0.1 - 2 && k.1 <= c1.1 + 2;
        for k in self.world.evict(&keep) {
            self.r.remove_chunk(k);
        }
    }
}
