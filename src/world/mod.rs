pub mod blocks;
pub mod chunk;
pub mod gen;
pub mod mesh;

use std::collections::HashMap;

use chunk::{Features, CS};
use gen::Gen;
use mesh::MeshData;

pub struct Built {
    pub key: (i32, i32),
    pub solid: MeshData,
    pub water: MeshData,
}

/// Streams chunks in and out around the camera. Voxel data is discarded
/// after meshing; only the features (lamps, chimneys...) are kept.
pub struct World {
    pub gen: Gen,
    pub chunks: HashMap<(i32, i32), Features>,
}

impl World {
    pub fn new(seed: u32) -> Self {
        World { gen: Gen::new(seed), chunks: HashMap::new() }
    }

    pub fn chunk_of(x: f32, z: f32) -> (i32, i32) {
        ((x / CS as f32).floor() as i32, (z / CS as f32).floor() as i32)
    }

    /// Generate missing chunks from `wanted` (nearest first) until the time budget runs out.
    pub fn stream(&mut self, wanted: &[(i32, i32)], budget_ms: f64, now: &dyn Fn() -> f64) -> Vec<Built> {
        let start = now();
        let mut out = Vec::new();
        for &key in wanted {
            if self.chunks.contains_key(&key) {
                continue;
            }
            let ch = chunk::generate(&self.gen, key.0, key.1);
            let (solid, water) = mesh::build(&ch);
            self.chunks.insert(key, ch.feat);
            out.push(Built { key, solid, water });
            if now() - start > budget_ms {
                break;
            }
        }
        out
    }

    /// Regenerate chunks already loaded (something in them has changed).
    pub fn rebuild(&mut self, keys: &[(i32, i32)]) -> Vec<Built> {
        let keys: Vec<_> = keys.iter().copied().filter(|k| self.chunks.contains_key(k)).collect();
        keys.into_iter()
            .map(|key| {
                let ch = chunk::generate(&self.gen, key.0, key.1);
                let (solid, water) = mesh::build(&ch);
                self.chunks.insert(key, ch.feat);
                Built { key, solid, water }
            })
            .collect()
    }

    /// Forget chunks outside the keep set; returns their keys so GPU buffers can be freed.
    pub fn evict(&mut self, keep: &dyn Fn((i32, i32)) -> bool) -> Vec<(i32, i32)> {
        let gone: Vec<_> = self.chunks.keys().copied().filter(|k| !keep(*k)).collect();
        for k in &gone {
            self.chunks.remove(k);
        }
        gone
    }

    pub fn features_near(&self, x: f32, z: f32, radius: f32) -> impl Iterator<Item = (&(i32, i32), &Features)> {
        let r = radius / CS as f32 + 1.0;
        let (cx, cz) = Self::chunk_of(x, z);
        self.chunks.iter().filter(move |(k, _)| {
            ((k.0 - cx) as f32).abs() <= r && ((k.1 - cz) as f32).abs() <= r
        })
    }
}
