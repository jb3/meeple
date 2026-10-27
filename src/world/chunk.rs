//! Voxel chunk generation. A chunk stores its 32x32 columns plus a one-voxel
//! border copied from its neighbours so meshing can cull faces and compute
//! ambient occlusion without looking anything up.

use std::rc::Rc;

use glam::{Vec2, Vec3};

use super::blocks::*;
use super::gen::*;
use crate::util::*;

mod terrain;
use self::terrain::*;
mod trees;
use self::trees::*;
mod town;
use self::town::*;
mod lots;
use self::lots::*;
mod sites;
use self::sites::*;
mod weirs;
use self::weirs::*;
mod anglers;
use self::anglers::*;
pub(crate) use self::sites::{apiary_hives, orchard_trees};
#[cfg(test)]
mod tests;
#[cfg(test)]
mod seed_hunt;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod bridge_tests;

pub const CS: i32 = 32;
pub const CH: i32 = 64;
pub const P: i32 = CS + 2;

/// Things in a chunk that the simulation and renderer care about after meshing.
#[derive(Default, Clone)]
pub struct Features {
    pub lamps: Vec<[f32; 3]>,
    pub chimneys: Vec<[f32; 3]>,
    pub fountains: Vec<[f32; 3]>,
    pub forges: Vec<[f32; 3]>,
    pub water: f32,
    pub nature: f32,
    pub pasture: Option<[f32; 3]>,
    /// Windmill sail hubs, and the way each mill faces (yaw).
    pub sails: Vec<([f32; 3], f32)>,
    /// Watermill wheel hubs and the way each faces (yaw).
    pub wheels: Vec<([f32; 3], f32)>,
    /// Lighthouse lamps.
    pub beacons: Vec<[f32; 3]>,
    /// Monastery bells.
    pub bells: Vec<[f32; 3]>,
    /// Ferry crossings, landing to landing: x, z of the first, x, z of the last.
    pub ferries: Vec<[f32; 4]>,
    /// The far ends of harbour jetties, where fishing boats moor.
    pub piers: Vec<[f32; 3]>,
    /// Treetops (a few per chunk), where owls perch at night.
    /// Where an owl can perch: x, height of the top of the tree in leaf, z, and
    /// the height it settles to once the leaves are down (the same for
    /// evergreens).
    pub treetops: Vec<[f32; 4]>,
    /// Hot springs that steam, and geysers that now and then erupt.
    pub steams: Vec<[f32; 3]>,
    /// Anglers on river banks: where they stand, the way they face, a seed for their looks.
    /// Weirs across rivers (a weir belongs to the chunk its middle is in).
    pub weirs: Vec<Weir>,
    pub anglers: Vec<([f32; 3], f32, u32)>,
}

/// One step of railway track along a line. Track follows the line's curve
/// and the smooth rail height rather than the voxel grid, so it is meshed as
/// its own ribbon of geometry.
pub struct RailSeg {
    /// Centre of the ballast top at the start and end of the step.
    pub a: Vec3,
    pub b: Vec3,
    /// Unit sideways direction at each end, so neighbouring steps join up.
    pub na: Vec2,
    pub nb: Vec2,
    /// Top of the voxel bed underneath.
    pub bed: f32,
}

pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub blocks: Vec<u8>,
    pub feat: Features,
    pub rails: Vec<RailSeg>,
}

impl Chunk {
    fn new(cx: i32, cz: i32) -> Self {
        Chunk { cx, cz, blocks: vec![AIR; (P * P * CH) as usize], feat: Features::default(), rails: Vec::new() }
    }

    #[inline]
    fn idx(lx: i32, y: i32, lz: i32) -> usize {
        ((y * P + lz + 1) * P + lx + 1) as usize
    }

    /// Local lookup; lx/lz may range over -1..=CS.
    #[inline]
    pub fn get(&self, lx: i32, y: i32, lz: i32) -> u8 {
        if y < 0 {
            return STONE;
        }
        if y >= CH || lx < -1 || lz < -1 || lx > CS || lz > CS {
            return AIR;
        }
        self.blocks[Self::idx(lx, y, lz)]
    }

    #[inline]
    fn local(&self, wx: i32, wz: i32) -> (i32, i32) {
        (wx - self.cx * CS, wz - self.cz * CS)
    }

    fn set(&mut self, wx: i32, y: i32, wz: i32, b: u8) {
        let (lx, lz) = self.local(wx, wz);
        if lx < -1 || lz < -1 || lx > CS || lz > CS || y < 0 || y >= CH {
            return;
        }
        self.blocks[Self::idx(lx, y, lz)] = b;
    }

    pub fn get_w(&self, wx: i32, y: i32, wz: i32) -> u8 {
        let (lx, lz) = self.local(wx, wz);
        self.get(lx, y, lz)
    }

    fn set_air(&mut self, wx: i32, y: i32, wz: i32, b: u8) {
        if self.get_w(wx, y, wz) == AIR {
            self.set(wx, y, wz, b);
        }
    }

    /// Within this chunk or its one-block border.
    fn core_or_pad(&self, wx: i32, wz: i32) -> bool {
        let (lx, lz) = self.local(wx, wz);
        (-1..=CS).contains(&lx) && (-1..=CS).contains(&lz)
    }

    fn core(&self, wx: i32, wz: i32) -> bool {
        let (lx, lz) = self.local(wx, wz);
        (0..CS).contains(&lx) && (0..CS).contains(&lz)
    }

    fn lamp(&mut self, x: i32, y: i32, z: i32) {
        self.set(x, y, z, LAMP);
        if self.core(x, z) {
            self.feat.lamps.push([x as f32 + 0.5, y as f32 + 0.3, z as f32 + 0.5]);
        }
    }

    fn chimney(&mut self, x: i32, y: i32, z: i32) {
        if self.core(x, z) {
            self.feat.chimneys.push([x as f32 + 0.5, y as f32 + 0.2, z as f32 + 0.5]);
        }
    }

    fn fill_box(&mut self, x0: i32, y0: i32, z0: i32, x1: i32, y1: i32, z1: i32, b: u8) {
        for y in y0..=y1 {
            for z in z0..=z1 {
                for x in x0..=x1 {
                    self.set(x, y, z, b);
                }
            }
        }
    }
}

struct ColInfo {
    h: i32,
    biome: Biome,
    built: bool,
    top: u8,
}

pub fn generate(gen: &Gen, cx: i32, cz: i32) -> Chunk {
    let x0 = cx * CS;
    let z0 = cz * CS;
    let ctx = gen.ctx(x0 - 12, z0 - 12, x0 + CS + 12, z0 + CS + 12);
    let mut ch = Chunk::new(cx, cz);
    let mut cols: Vec<ColInfo> = Vec::with_capacity((P * P) as usize);

    // 1. terrain
    let mut water = 0;
    let mut nature = 0;
    // the terrain noise for the whole block, four columns at a time
    let noise = gen.noise_block(x0 - 1, z0 - 1, CS + 2, CS + 2);
    for lz in -1..=CS {
        for lx in -1..=CS {
            let (x, z) = (x0 + lx, z0 + lz);
            let (raw, tn, mn) = noise.at(lx + 1, lz + 1);
            let col = gen.column_with(&ctx, x, z, raw, tn, mn);
            let top = fill_column(&mut ch, gen, &ctx, &col, x, z);
            if (0..CS).contains(&lx) && (0..CS).contains(&lz) {
                if col.h < SEA {
                    water += 1;
                } else if !col.built() && top == GRASS {
                    nature += 1;
                }
            }
            cols.push(ColInfo { h: col.h, biome: col.biome, built: col.built(), top });
        }
    }
    ch.feat.water = water as f32 / (CS * CS) as f32;
    ch.feat.nature = nature as f32 / (CS * CS) as f32;

    // 2. small decorations
    let greens: Vec<Vec2> = ctx.towns.iter().filter_map(|t| gen.green(t)).map(|(gx, gz)| Vec2::new(gx as f32 + 0.5, gz as f32 + 0.5)).collect();
    for lz in 0..CS {
        for lx in 0..CS {
            let c = &cols[((lz + 1) * P + lx + 1) as usize];
            if c.built || c.h < SEA {
                continue;
            }
            let (x, z) = (x0 + lx, z0 + lz);
            // nothing grows on a village green: it's mown, and it's where the fair,
            // the cricket and the skating go
            if greens.iter().any(|g| Vec2::new(x as f32 + 0.5 - g.x, z as f32 + 0.5 - g.y).length() < Gen::GREEN_R) {
                continue;
            }
            let r = unit(hash2(gen.seed ^ 0xdec0, x, z));
            let y = c.h + 1;
            // wildflowers grow in drifts rather than evenly
            let drift = fbm(gen.seed ^ 0xf10e, x as f32 / 22.0, z as f32 / 22.0, 2);
            let b = match (c.top, c.biome) {
                (GRASS, Biome::Plains) if drift > 0.2 && r < 0.3 => POPPY,
                (GRASS, Biome::Forest) if drift > 0.12 && r < 0.4 => BLUEBELL,
                (GRASS, Biome::Plains) => {
                    if r < 0.09 {
                        TALLGRASS
                    } else if r < 0.12 {
                        [FLOWER_R, FLOWER_Y, FLOWER_B][(r * 1000.0) as usize % 3]
                    } else if r < 0.126 {
                        BUSH
                    } else if r < 0.128 {
                        ROCK
                    } else {
                        AIR
                    }
                }
                (GRASS, Biome::Forest) => {
                    if r < 0.07 {
                        TALLGRASS
                    } else if r < 0.09 {
                        BUSH
                    } else if r < 0.1 {
                        FLOWER_Y
                    } else {
                        AIR
                    }
                }
                (GRASS, Biome::Jungle) => {
                    if r < 0.16 {
                        TALLGRASS
                    } else if r < 0.23 {
                        BUSH
                    } else if r < 0.255 {
                        [FLOWER_R, FLOWER_Y, FLOWER_B][(r * 1000.0) as usize % 3]
                    } else {
                        AIR
                    }
                }
                (SAND, Biome::Desert) | (SNOW, _) | (STONE, _) if r < 0.006 => ROCK,
                _ => AIR,
            };
            if b != AIR {
                ch.set_air(x, y, z, b);
            }
        }
    }

    // 3. trees, on a jittered 4x4 grid so they never clump into one voxel
    let tseed = gen.seed ^ 0x77ee;
    for gz in div_floor(z0 - 5, 4)..=div_floor(z0 + CS + 4, 4) {
        for gx in div_floor(x0 - 5, 4)..=div_floor(x0 + CS + 4, 4) {
            let hh = hash2(tseed, gx, gz);
            if unit(hh) > 0.6 {
                continue;
            }
            let tx = gx * 4 + (hh >> 4) as i32 % 4;
            let tz = gz * 4 + (hh >> 8) as i32 % 4;
            let col = gen.column(&ctx, tx, tz);
            if col.built() || col.h <= SEA {
                continue;
            }
            if ctx.roads.iter().any(|s| s.dist(tx, tz) <= 3)
                || ctx.rails.iter().any(|l| l.perp(tx as f32, tz as f32).abs() < 6.0)
                || ctx.chords.iter().any(|c| c.dist(Vec2::new(tx as f32, tz as f32)) < 6.0)
                || ctx.towns.iter().any(|t| gen.green(t).is_some_and(|(gx, gz)| Vec2::new(gx as f32 + 0.5 - tx as f32, gz as f32 + 0.5 - tz as f32).length() < Gen::GREEN_R))
                // the towpath and water stay open to the sky
                || ctx.canals.iter().any(|&cz| (tz - cz).abs() < 7 && gen.canal_open(tx as f32, cz))
                || ctx.towns.iter().any(|t| {
                    let dx = (tx - t.cx) as f32;
                    let dz = (tz - t.cz) as f32;
                    (dx * dx + dz * dz).sqrt() < t.radius as f32 + 4.0
                })
            {
                continue;
            }
            let clearing = fbm(gen.seed ^ 0x1234, tx as f32 / 60.0, tz as f32 / 60.0, 2);
            let p = match col.biome {
                Biome::Forest => 0.75 - clearing.max(0.0) * 1.2,
                Biome::Plains => 0.05 + clearing.max(0.0) * 0.3,
                Biome::Desert => 0.05,
                Biome::Tundra => 0.35,
                Biome::Jungle => 0.9 - clearing.max(0.0) * 1.1,
                Biome::Mountain => {
                    if col.h < SEA + 30 {
                        0.15
                    } else {
                        0.0
                    }
                }
            };
            let r = unit(hash_u32(hh ^ 0xabc));
            if r >= p {
                continue;
            }
            let kind = hash_u32(hh ^ 0x99);
            let y = col.h + 1;
            if col.biome != Biome::Desert && ch.core(tx, tz) && ch.feat.treetops.len() < 6 {
                // the top of the tree as it's built, and where the last leaves cling
                // when it's bare: just above the trunk
                let (top, bare) = match col.biome {
                    Biome::Tundra | Biome::Mountain => pine_top(kind),
                    Biome::Forest => match kind % 10 {
                        0..=5 => oak_top(kind),
                        6..=7 => birch_top(kind),
                        _ => pine_top(kind),
                    },
                    Biome::Jungle => match kind % 10 {
                        0..=6 => kapok_top(kind),
                        _ => palm_top(kind),
                    },
                    _ => oak_top(kind),
                };
                ch.feat.treetops.push([tx as f32 + 0.5, y as f32 + top, tz as f32 + 0.5, y as f32 + bare]);
            }
            match col.biome {
                Biome::Desert => cactus(&mut ch, tx, y, tz, kind),
                Biome::Tundra | Biome::Mountain => pine(&mut ch, tx, y, tz, kind),
                Biome::Forest => match kind % 10 {
                    0..=5 => oak(&mut ch, tx, y, tz, kind),
                    6..=7 => birch(&mut ch, tx, y, tz, kind),
                    _ => pine(&mut ch, tx, y, tz, kind),
                },
                Biome::Plains => oak(&mut ch, tx, y, tz, kind),
                Biome::Jungle => match kind % 10 {
                    0..=6 => kapok(&mut ch, tx, y, tz, kind),
                    _ => palm(&mut ch, tx, y, tz, kind),
                },
            }
        }
    }

    // 4. railway track: one short step along each line, owned by the chunk
    // its midpoint falls in
    for l in &ctx.rails {
        let ends = [(x0, z0), (x0 + CS, z0), (x0, z0 + CS), (x0 + CS, z0 + CS)].map(|(x, z)| l.along(x as f32, z as f32));
        let (lo, hi) = (ends.iter().cloned().fold(f32::MAX, f32::min) as i32 - 3, ends.iter().cloned().fold(f32::MIN, f32::max) as i32 + 3);
        for s in lo..=hi {
            let s = s as f32;
            let (mx, mz) = l.point(s + 0.5, 0.0);
            if !ch.core(mx.floor() as i32, mz.floor() as i32) {
                continue;
            }
            let at = |s: f32| {
                let (x, z) = l.point(s, 0.0);
                Vec3::new(x, gen.rail_level(x, z) + 1.0, z)
            };
            let side = |s: f32| {
                let (tx, tz) = l.tangent(s);
                Vec2::new(-tz, tx)
            };
            let bed = gen.rail_level(mx, mz).floor() + 1.0;
            ch.rails.push(RailSeg { a: at(s), b: at(s + 1.0), na: side(s), nb: side(s + 1.0), bed });
        }
    }

    // junction chords: the same track, curving from one line onto another
    for c in &ctx.chords {
        let step = 1.0;
        let us: Vec<f32> = c.samples(step).collect();
        for w in us.windows(2) {
            let (u0, u1) = (w[0], w[1]);
            let m = c.at((u0 + u1) * 0.5);
            if !ch.core(m.x.floor() as i32, m.y.floor() as i32) {
                continue;
            }
            // a hair above the main line where they overlap at the points,
            // so the two don't flicker against each other
            let at = |u: f32| {
                let p = c.at(u);
                Vec3::new(p.x, gen.rail_level(p.x, p.y) + 1.015, p.y)
            };
            let side = |u: f32| {
                let t = c.tangent(u);
                Vec2::new(-t.y, t.x)
            };
            let bed = gen.rail_level(m.x, m.y).floor() + 1.0;
            ch.rails.push(RailSeg { a: at(u0), b: at(u1), na: side(u0), nb: side(u1), bed });
        }
    }

    // 5. towns
    let towns: Vec<Rc<Town>> = ctx.towns.clone();
    for t in &towns {
        let r = t.radius + 2;
        if t.cx + r < x0 - 1 || t.cx - r > x0 + CS || t.cz + r < z0 - 1 || t.cz - r > z0 + CS {
            if t.rail.is_none() {
                continue;
            }
        }
        build_town(&mut ch, gen, t);
    }

    // ferries: each belongs to the chunk its first landing is in
    for (p, q) in gen.ferry_runs_near(x0, z0, x0 + CS, z0 + CS) {
        if ch.core(p.x.floor() as i32, p.y.floor() as i32) {
            ch.feat.ferries.push([p.x, p.y, q.x, q.y]);
        }
    }

    // weirs across the rivers
    for w in gen.weirs_near(x0 - 1, z0 - 1, x0 + CS, z0 + CS) {
        build_weir(&mut ch, &w);
    }

    // countryside sites
    for st in &ctx.sites {
        if !st.near(x0 - 1, z0 - 1, x0 + CS, z0 + CS, 6.0) {
            continue;
        }
        let ground = |x: i32, z: i32| gen.column(&ctx, x, z).h;
        match st.kind {
            SiteKind::Windmill => build_windmill(&mut ch, st),
            SiteKind::Manor => build_manor(&mut ch, st),
            SiteKind::Ruin => build_ruin(&mut ch, st, &ground),
            SiteKind::Quarry => build_quarry(&mut ch, st, &ground),
            SiteKind::Watermill => build_watermill(&mut ch, st),
            SiteKind::Lighthouse => build_lighthouse(&mut ch, st),
            SiteKind::Harbour => build_harbour(&mut ch, st),
            SiteKind::Mine => build_mine(&mut ch, st),
            SiteKind::StoneCircle => build_stone_circle(&mut ch, st, &ground),
            SiteKind::ChalkFigure => build_chalk_figure(&mut ch, st, &ground),
            SiteKind::Geyser => build_pool(&mut ch, st, &ground, true),
            SiteKind::Oasis => build_pool(&mut ch, st, &ground, false),
            SiteKind::SkiResort => build_ski_resort(&mut ch, st, &ground),
            SiteKind::Monastery => build_monastery(&mut ch, st, &ground),
            SiteKind::Shrine => build_shrine(&mut ch, st, &ground),
            SiteKind::HillFort => build_hill_fort(&mut ch, st, &ground),
            SiteKind::Camp => build_camp(&mut ch, st, &ground),
            SiteKind::Orchard => build_orchard(&mut ch, st, &ground),
            SiteKind::Apiary => build_apiary(&mut ch, st, &ground),
        }
    }

    // anglers on the banks, once everything that could stand there is built
    place_anglers(&mut ch, gen);

    // 6. pastures for grazing animals
    if ch.feat.nature > 0.6 && unit(hash2(gen.seed ^ 0x5ee7, cx, cz)) < 0.25 {
        let lx = 8 + (hash2(gen.seed, cx, cz) % 16) as i32;
        let lz = 8 + (hash2(gen.seed ^ 3, cx, cz) % 16) as i32;
        let c = &cols[((lz + 1) * P + lx + 1) as usize];
        if !c.built && c.h > SEA && matches!(c.biome, Biome::Plains | Biome::Forest) {
            ch.feat.pasture = Some([(x0 + lx) as f32 + 0.5, c.h as f32 + 1.0, (z0 + lz) as f32 + 0.5]);
        }
    }

    ch
}

// ---------- trees ----------

// ---------- towns ----------

// ---------- countryside sites ----------

