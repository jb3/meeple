//! CPU-side builder for instanced boxes. Everything that moves (people,
//! trains, animals, particles) is drawn as a handful of coloured boxes.

use glam::{Vec2, Vec3};

/// Instance floats: center xyz, yaw, half-size xyz, material, rgb, alpha.
pub const FLOATS: usize = 12;

pub const MAT_LIT: f32 = 0.0;
pub const MAT_GLOW: f32 = 10.0;
pub const MAT_UNLIT: f32 = 12.0;
pub const MAT_NIGHT_WINDOW: f32 = 13.0;

#[derive(Default)]
pub struct Boxes {
    pub data: Vec<f32>,
}

impl Boxes {
    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len() / FLOATS
    }

    #[inline]
    pub fn push(&mut self, c: Vec3, half: Vec3, yaw: f32, rgb: [f32; 3], mat: f32) {
        self.push_alpha(c, half, yaw, rgb, mat, 1.0);
    }

    /// `alpha` below 1 dissolves the box with a dither pattern.
    #[inline]
    pub fn push_alpha(&mut self, c: Vec3, half: Vec3, yaw: f32, rgb: [f32; 3], mat: f32, alpha: f32) {
        self.data.extend_from_slice(&[
            c.x, c.y, c.z, yaw, half.x, half.y, half.z, mat, rgb[0], rgb[1], rgb[2], alpha,
        ]);
    }
}

/// Places parts relative to an object's origin and facing.
pub struct Rig<'a> {
    pub b: &'a mut Boxes,
    pub pos: Vec3,
    pub yaw: f32,
    pub scale: f32,
    pub alpha: f32,
    /// Depth (along the figure's facing) is multiplied by this: 1 normally, near
    /// 0 for a flat image such as a reflection lying on a mirror.
    pub squash: f32,
    cos: f32,
    sin: f32,
}

impl<'a> Rig<'a> {
    pub fn new(b: &'a mut Boxes, pos: Vec3, yaw: f32, scale: f32) -> Self {
        Rig { b, pos, yaw, scale, alpha: 1.0, squash: 1.0, cos: yaw.cos(), sin: yaw.sin() }
    }

    /// `local`: x = right, y = up, z = forward.
    pub fn part(&mut self, local: Vec3, half: Vec3, rgb: [f32; 3]) {
        self.part_mat(local, half, rgb, 0.0);
    }

    pub fn part_mat(&mut self, local: Vec3, half: Vec3, rgb: [f32; 3], mat: f32) {
        let mut l = local * self.scale;
        let mut half = half;
        if self.squash < 1.0 {
            l.z *= self.squash;
            half.z = (half.z * self.squash).max(0.012 / self.scale);
        }
        let w = Vec3::new(l.x * self.cos + l.z * self.sin, l.y, -l.x * self.sin + l.z * self.cos);
        self.b.push_alpha(self.pos + w, half * self.scale, self.yaw, rgb, mat, self.alpha);
    }
}

pub fn yaw_of(d: Vec2) -> f32 {
    d.x.atan2(d.y)
}

pub fn rgb(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}
