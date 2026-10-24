use super::*;

pub(super) fn fill_column(ch: &mut Chunk, gen: &Gen, ctx: &Ctx, col: &Column, x: i32, z: i32) -> u8 {
    let h = col.h;
    let desert = col.biome == Biome::Desert;
    for y in 0..h {
        let b = if y < h - 3 {
            STONE
        } else if desert {
            SAND
        } else {
            DIRT
        };
        ch.set(x, y, z, b);
    }
    let city = col.town.map(|i| ctx.towns[i].city).unwrap_or(false);
    let r = hash2(gen.seed ^ 0x70b, x, z);
    let top = if let Some(p) = col.canal {
        // gravel bed, stone quay edges, towpath
        if p.abs() < 1.6 {
            GRAVEL
        } else if p.abs() < 2.2 {
            STONEBRICK
        } else {
            PATH
        }
    } else if col.quarry {
        if r % 4 == 0 {
            GRAVEL
        } else {
            STONE
        }
    } else if (col.rail.is_some() || col.chord) && col.viaduct.is_none() && col.tunnel.is_none() {
        GRAVEL
    } else if col.plaza {
        COBBLE
    } else if col.street {
        if city {
            COBBLE
        } else {
            PATH
        }
    } else if col.road && h >= SEA {
        PATH
    } else if h < SEA - 1 {
        if r % 3 == 0 {
            GRAVEL
        } else {
            SAND
        }
    } else if h <= SEA + 1 && col.town.is_none() && col.biome != Biome::Tundra {
        SAND
    } else {
        match col.biome {
            Biome::Desert => SAND,
            Biome::Tundra => SNOW,
            Biome::Mountain => {
                if h > SEA + 33 {
                    SNOW
                } else if r % 5 == 0 {
                    GRAVEL
                } else {
                    STONE
                }
            }
            _ => GRASS,
        }
    };
    ch.set(x, h, z, top);
    if h < SEA {
        for y in h + 1..=SEA {
            ch.set(x, y, z, WATER);
        }
        if let (true, Some(d)) = (col.road && col.town.is_none(), gen.ferry_at(ctx, x, z)) {
            // too wide to bridge: a plank landing stage at each end of the
            // crossing, with posts and a lantern; the ferry crosses the open water
            if d <= 1.6 {
                ch.set(x, SEA, z, PLANK);
                if is_bridge_edge(ctx, x, z) {
                    ch.fill_box(x, SEA + 1, z, x, SEA + 2, z, LOG);
                    if (x + z).rem_euclid(2) == 0 {
                        ch.lamp(x, SEA + 3, z);
                    }
                }
            }
        } else if col.road && col.town.is_none() {
            // a humpback stone bridge: the deck climbs from the banks (see
            // `Gen::bridge_deck`) to clear boats, solid abutments at the
            // ends, open water under the crown, a pier every so often on
            // long crossings, and parapets along the edges
            let (deck, d) = gen.bridge_deck(ctx, x, z);
            ch.set(x, deck, z, STONEBRICK);
            if is_bridge_edge(ctx, x, z) {
                ch.set(x, deck + 1, z, COBBLE);
            }
            // piers stand on a lattice of two-cell blocks, so a road of any
            // heading, straight or slanting, gets separate pillars rather
            // than a wall along the row it happens to follow
            let pier = x.rem_euclid(8) < 2 && z.rem_euclid(8) < 2;
            let solid = d <= 2 || (d >= 4 && pier);
            if solid {
                for y in h + 1..deck {
                    ch.set(x, y, z, STONEBRICK);
                }
            }
        }
    }
    // a viaduct: the deck carried on piers and arches over the low ground
    if let Some(bed) = col.viaduct {
        ch.set(x, bed, z, GRAVEL);
        ch.set(x, bed - 1, z, STONEBRICK);
        match (col.along.floor() as i32).rem_euclid(8) {
            0 | 1 => {
                for y in h + 1..bed - 1 {
                    ch.set(x, y, z, STONEBRICK);
                }
            }
            2 | 7 => {
                ch.set(x, bed - 2, z, STONEBRICK);
                ch.set(x, bed - 3, z, STONEBRICK);
            }
            3 | 6 => ch.set(x, bed - 2, z, STONEBRICK),
            _ => {}
        }
    }
    // a tunnel: bored through the hill, lined with stone at the mouths
    if let Some(bed) = col.tunnel {
        ch.set(x, bed, z, GRAVEL);
        for y in bed + 1..=bed + 4 {
            ch.set(x, y, z, AIR);
        }
        if h >= bed + 5 {
            ch.set(x, bed + 5, z, STONEBRICK);
        }
        // near the hillside face: a stone portal arch
        if h < bed + 10 {
            ch.set(x, bed + 6, z, STONEBRICK);
        }
    }
    top
}

/// A bridge cell is an edge if a neighbour across the road direction is off-road.
pub(super) fn is_bridge_edge(ctx: &Ctx, x: i32, z: i32) -> bool {
    let on = |x: i32, z: i32| ctx.roads.iter().any(|s| s.dist(x, z) <= 1);
    !on(x + 1, z) || !on(x - 1, z) || !on(x, z + 1) || !on(x, z - 1)
}
