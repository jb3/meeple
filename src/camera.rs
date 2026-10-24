//! Orthographic isometric camera.

use glam::{Mat4, Vec2, Vec3};

pub struct Camera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    /// Half of the visible height in world units.
    pub zoom: f32,
    pub aspect: f32,
}

/// How far back the eye sits (and so the depth range) at the usual zooms; it
/// grows when zoomed far out so nothing is clipped.
const DIST: f32 = 300.0;

impl Camera {
    fn dist(&self) -> f32 {
        DIST.max(self.zoom * 4.5)
    }

    /// World units between the near and far planes: what one whole step of the
    /// depth buffer spans (the projection is orthographic, so it's linear).
    pub fn depth_range(&self) -> f32 {
        self.dist() * 2.4 - 1.0
    }

    /// Unit vector from the target towards the eye.
    pub fn back(&self) -> Vec3 {
        Vec3::new(self.pitch.cos() * self.yaw.sin(), self.pitch.sin(), self.pitch.cos() * self.yaw.cos())
    }

    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let f = -self.back();
        let r = f.cross(Vec3::Y).normalize();
        let u = r.cross(f);
        (f, r, u)
    }

    /// Ground-plane right and forward directions, for panning.
    pub fn ground_axes(&self) -> (Vec2, Vec2) {
        let (_, r, _) = self.basis();
        let right = Vec2::new(r.x, r.z).normalize();
        let fwd = Vec2::new(-self.yaw.sin(), -self.yaw.cos());
        (right, fwd)
    }

    /// View-projection, optionally snapping the target to the pixel grid so
    /// chunky pixels don't crawl. Also returns how far (in texels) the snap
    /// moved the view, so the upscale can shift back by that much and the
    /// glide stays smooth at full display resolution.
    pub fn view_proj(&self, px_h: Option<f32>) -> (Mat4, Vec2) {
        let (_, r, u) = self.basis();
        let mut t = self.target;
        let mut shift = Vec2::ZERO;
        if let Some(h) = px_h {
            let texel = 2.0 * self.zoom / h;
            let a = t.dot(r);
            let b = t.dot(u);
            let da = (a / texel).round() * texel - a;
            let db = (b / texel).round() * texel - b;
            t += r * da + u * db;
            shift = Vec2::new(da, db) / texel;
        }
        let view = Mat4::look_at_rh(t + self.back() * self.dist(), t, Vec3::Y);
        let w = self.zoom * self.aspect;
        let proj = Mat4::orthographic_rh_gl(-w, w, -self.zoom, self.zoom, 1.0, self.dist() * 2.4);
        (proj * view, shift)
    }

    /// Axis-aligned ground rectangle covering everything on screen between
    /// heights 0 and 70.
    pub fn ground_rect(&self) -> (Vec2, Vec2) {
        let (f, r, u) = self.basis();
        let eye = self.target + self.back() * self.dist();
        let w = self.zoom * self.aspect;
        let mut lo = Vec2::splat(f32::MAX);
        let mut hi = Vec2::splat(f32::MIN);
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let o = eye + r * (sx * w) + u * (sy * self.zoom);
            for y in [0.0, 70.0] {
                let t = (y - o.y) / f.y;
                let p = o + f * t;
                lo = lo.min(Vec2::new(p.x, p.z));
                hi = hi.max(Vec2::new(p.x, p.z));
            }
        }
        (lo, hi)
    }

    pub fn view_radius(&self) -> f32 {
        let (lo, hi) = self.ground_rect();
        (hi - lo).length() * 0.5
    }
}
