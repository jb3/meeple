use super::*;

/// Where segment a-b passes through the axis-aligned square (centre c,
/// half-size h): the entry and exit fractions along the segment, if it does.
pub(super) fn seg_square_span(a: Vec2, b: Vec2, c: Vec2, h: f32) -> Option<(f32, f32)> {
    let d = b - a;
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for (p, dp, lo, hi) in [(a.x, d.x, c.x - h, c.x + h), (a.y, d.y, c.y - h, c.y + h)] {
        if dp.abs() < 1e-6 {
            if p < lo || p > hi {
                return None;
            }
            continue;
        }
        let (mut ta, mut tb) = ((lo - p) / dp, (hi - p) / dp);
        if ta > tb {
            std::mem::swap(&mut ta, &mut tb);
        }
        t0 = t0.max(ta);
        t1 = t1.min(tb);
        if t0 > t1 {
            return None;
        }
    }
    Some((t0, t1))
}

/// Does segment a-b pass through the axis-aligned square (centre c, half-size h)?
pub(super) fn seg_hits_square(a: Vec2, b: Vec2, c: Vec2, h: f32) -> bool {
    seg_square_span(a, b, c, h).is_some()
}

/// Corners of the detour square (centre c, half-size r) to pass, in order,
/// going the short way round from a to b.
pub(super) fn corners_between(a: Vec2, b: Vec2, c: Vec2, r: f32) -> Vec<Vec2> {
    use std::f32::consts::{FRAC_PI_4, PI, TAU};
    let ang = |v: Vec2| (v.y - c.y).atan2(v.x - c.x).rem_euclid(TAU);
    let (aa, ab) = (ang(a), ang(b));
    let sweep = (ab - aa).rem_euclid(TAU);
    let ccw = sweep <= PI;
    let span = if ccw { sweep } else { TAU - sweep };
    let mut out: Vec<(f32, Vec2)> = (0..4)
        .filter_map(|k| {
            let phi = FRAC_PI_4 + k as f32 * (PI / 2.0);
            let off = if ccw { (phi - aa).rem_euclid(TAU) } else { (aa - phi).rem_euclid(TAU) };
            (off > 0.0 && off < span).then(|| (off, c + Vec2::new(phi.cos(), phi.sin()) * r * std::f32::consts::SQRT_2))
        })
        .collect();
    out.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    out.into_iter().map(|(_, p)| p).collect()
}

/// Rewrite a walk starting at `start` so no leg crosses the square obstacle
/// (centre c, half-size `clear`), detouring via corners at half-size `around`.
pub(super) fn avoid_square(start: Vec2, pts: &[Vec2], c: Vec2, clear: f32, around: f32) -> Vec<Vec2> {
    let mut out = Vec::with_capacity(pts.len() + 4);
    let inside = |p: Vec2| (p - c).abs().max_element() < clear;
    // the nearest point just outside the square, keeping to the same side
    let step_out = |p: Vec2| {
        let d = p - c;
        if d.x.abs() >= d.y.abs() {
            Vec2::new(c.x + if d.x < 0.0 { -around } else { around }, p.y)
        } else {
            Vec2::new(p.x, c.y + if d.y < 0.0 { -around } else { around })
        }
    };
    let mut a = start;
    // someone starting at the very edge of the basin steps out of it first
    if inside(a) {
        a = step_out(a);
        out.push(a);
    }
    for (k, &b0) in pts.iter().enumerate() {
        let mut b = b0;
        if inside(b) {
            // a waypoint in the basin (the street's crossing at the plaza's
            // heart) is dropped, and the way on goes round; a destination in
            // it is moved to the edge
            if k + 1 < pts.len() {
                continue;
            }
            b = step_out(b);
        }
        if seg_hits_square(a, b, c, clear) {
            // leave the line where it reaches the detour square, go round,
            // and rejoin the same line on the far side, so walkers stay on
            // the street instead of cutting across to the next waypoint
            let (t0, t1) = seg_square_span(a, b, c, around).unwrap_or((0.0, 1.0));
            let (enter, exit) = (a.lerp(b, t0), a.lerp(b, t1));
            if t0 > 0.0 {
                out.push(enter);
            }
            out.extend(corners_between(enter, exit, c, around));
            if t1 < 1.0 {
                out.push(exit);
            }
        }
        out.push(b);
        a = b;
    }
    out
}

pub(super) fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut d = (b - a) % std::f32::consts::TAU;
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    } else if d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    a + d * t
}
