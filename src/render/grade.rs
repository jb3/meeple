//! Colour grading: the look of the time of day and the season, baked into a
//! small 3D lookup table the post pass reads once per pixel.

use glam::Vec3;

/// Texels along each edge of the table.
pub const SIZE: usize = 16;

/// What the grade depends on: golden-hour warmth, night, winter, overcast
/// (each 0..1). The renderer rebuilds the table when these move far enough.
pub type Params = [f32; 4];

/// Rounds the parameters to the steps the table is rebuilt at.
pub fn quantise(p: Params) -> [u8; 4] {
    p.map(|v| (v.clamp(0.0, 1.0) * 24.0).round() as u8)
}

fn luma(c: Vec3) -> f32 {
    c.dot(Vec3::new(0.3, 0.59, 0.11))
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The graded colour for one input colour.
pub fn grade(mut c: Vec3, p: Params) -> Vec3 {
    let [warm, night, winter, over] = p;
    // dim screens crush the darks at night: lift them
    c = c.max(Vec3::ZERO).powf(1.0 - 0.16 * night);
    let l = luma(c);
    let shadows = 1.0 - smooth(0.0, 0.5, l);
    let lights = smooth(0.45, 1.0, l);
    // a little teal in the shadows by day, amber in the highlights and plum in
    // the shadows at golden hour, blue all over at night
    c += Vec3::new(-0.012, 0.0, 0.02) * shadows * (1.0 - night * 0.5) * (1.0 - warm * 0.6);
    c += Vec3::new(0.05, 0.012, -0.04) * lights * warm;
    c += Vec3::new(0.014, -0.006, 0.012) * shadows * warm;
    c = c.lerp(c * Vec3::new(0.9, 0.97, 1.1), night * 0.6);
    // winter, rain and darkness drain the colour
    let sat = 1.05 - 0.14 * winter - 0.1 * over - 0.1 * night;
    let l2 = luma(c);
    c = Vec3::splat(l2) + (c - Vec3::splat(l2)) * sat;
    // a gentle S-curve
    let s = c.clamp(Vec3::ZERO, Vec3::ONE);
    s.lerp(s * s * (Vec3::splat(3.0) - 2.0 * s), 0.18)
}

/// The table as RGBA8, red varying fastest, then green, then blue.
pub fn build(p: Params) -> Vec<u8> {
    let mut out = Vec::with_capacity(SIZE * SIZE * SIZE * 4);
    let k = (SIZE - 1) as f32;
    for b in 0..SIZE {
        for g in 0..SIZE {
            for r in 0..SIZE {
                let c = grade(Vec3::new(r as f32, g as f32, b as f32) / k, p);
                out.extend_from_slice(&[(c.x * 255.0 + 0.5) as u8, (c.y * 255.0 + 0.5) as u8, (c.z * 255.0 + 0.5) as u8, 255]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_grade_keeps_black_and_white_close() {
        let p = [0.0; 4];
        assert!(grade(Vec3::ZERO, p).length() < 0.05);
        assert!((grade(Vec3::ONE, p) - Vec3::ONE).length() < 0.08);
    }

    #[test]
    fn table_has_every_texel() {
        assert_eq!(build([0.5; 4]).len(), SIZE * SIZE * SIZE * 4);
    }
}
