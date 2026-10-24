//! Roads between towns: the links, their paths and the segments they lay down.

use super::super::*;

impl Gen {
    /// Neighbouring towns this town builds a road to (east and south).
    pub fn links(&self, t: &Town) -> Vec<Rc<Town>> {
        let mut out = Vec::new();
        for d in 1..=2 {
            if let Some(u) = self.town(t.rx + d, t.rz) {
                out.push(u);
                break;
            }
        }
        for d in 1..=2 {
            if let Some(u) = self.town(t.rx, t.rz + d) {
                out.push(u);
                break;
            }
        }
        out
    }

    /// The road between towns `a` and `b` (as returned by `links`), as
    /// waypoints. It leaves each town along its own street line, then
    /// wanders between them in a long S-bend, so no two roads run alike.
    pub fn road_path(a: &Town, b: &Town) -> Vec<(i32, i32)> {
        const OUT: i32 = 6;
        let east = b.rx != a.rx;
        let (pa, pb) = if east {
            ((a.cx + a.radius + OUT, a.cz), (b.cx - b.radius - OUT, b.cz))
        } else {
            ((a.cx, a.cz + a.radius + OUT), (b.cx, b.cz - b.radius - OUT))
        };
        let (along, across) = if east { (pb.0 - pa.0, pb.1 - pa.1) } else { (pb.1 - pa.1, pb.0 - pa.0) };
        // towns too close together to wander between: the old dogleg
        if along < 40 {
            return if east {
                vec![(a.cx, a.cz), (b.cx, a.cz), (b.cx, b.cz)]
            } else {
                vec![(a.cx, a.cz), (a.cx, b.cz), (b.cx, b.cz)]
            };
        }
        let unit = |h: u32| (h & 0xffff) as f32 / 32767.5 - 1.0;
        let h = hash2(a.id, b.id as i32, 0x70ad);
        let (s1, s2, ph) = (unit(h), unit(h >> 7 ^ h.rotate_left(13)), unit(h.rotate_left(21)));
        let amp = (along as f32 * 0.3).min(44.0);
        let n = (along / 15).clamp(2, 12);
        let mut out = vec![(a.cx, a.cz), pa];
        for i in 1..n {
            let t = i as f32 / n as f32;
            let off = amp * ((std::f32::consts::PI * t).sin() * s1 + 0.55 * (std::f32::consts::TAU * t + ph * 3.0).sin() * (std::f32::consts::TAU * t).sin() * s2);
            let along_pos = along as f32 * t;
            let across_pos = across as f32 * t + off;
            let (x, z) = if east { (pa.0 as f32 + along_pos, pa.1 as f32 + across_pos) } else { (pa.0 as f32 + across_pos, pa.1 as f32 + along_pos) };
            out.push((x.round() as i32, z.round() as i32));
        }
        out.push(pb);
        out.push((b.cx, b.cz));
        out
    }

    /// The voxel runs a road is laid in: the straight legs as whole
    /// segments, the slanting ones as a chain of dots two apart.
    pub(in crate::world::gen) fn road_segs(path: &[(i32, i32)], mut emit: impl FnMut(Seg)) {
        for w in path.windows(2) {
            let (p, q) = (w[0], w[1]);
            if p.0 == q.0 || p.1 == q.1 {
                emit(Seg::new(p, q));
                continue;
            }
            let (dx, dz) = (q.0 - p.0, q.1 - p.1);
            let n = (dx.abs().max(dz.abs()) + 1) / 2;
            for k in 0..=n {
                let f = k as f32 / n as f32;
                let at = ((p.0 as f32 + dx as f32 * f).round() as i32, (p.1 as f32 + dz as f32 * f).round() as i32);
                emit(Seg::new(at, at));
            }
        }
    }

    pub fn roads_near(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<Seg> {
        let mut out = Vec::new();
        // roads bend off their towns' lines, so look a region further each way
        for rx in div_floor(x0, REGION) - 2..=div_floor(x1, REGION) + 1 {
            for rz in div_floor(z0, REGION) - 2..=div_floor(z1, REGION) + 1 {
                let Some(t) = self.town(rx, rz) else { continue };
                for u in self.links(&t) {
                    let p = Self::road_path(&t, &u);
                    Self::road_segs(&p, |s| {
                        if s.hits_rect(x0, z0, x1, z1, 2) {
                            out.push(s);
                        }
                    });
                }
            }
        }
        out
    }
}
