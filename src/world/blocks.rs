//! Block ids, colours and shading materials.
//!
//! The material id travels to the GPU in the vertex colour's alpha channel
//! and selects special shading (water, seasonal foliage, glowing windows...).

pub const AIR: u8 = 0;
pub const GRASS: u8 = 1;
pub const DIRT: u8 = 2;
pub const STONE: u8 = 3;
pub const SAND: u8 = 4;
pub const WATER: u8 = 5;
pub const SNOW: u8 = 6;
pub const LOG: u8 = 7;
pub const LEAVES: u8 = 8;
pub const PINE: u8 = 9;
pub const PLANK: u8 = 10;
pub const BRICK: u8 = 11;
pub const COBBLE: u8 = 12;
pub const PATH: u8 = 13;
pub const ROOF_RED: u8 = 14;
pub const ROOF_BLUE: u8 = 15;
pub const ROOF_BROWN: u8 = 16;
pub const ROOF_GREEN: u8 = 17;
pub const ROOF_SLATE: u8 = 18;
pub const WINDOW: u8 = 19;
pub const DOOR: u8 = 20;
pub const LAMP: u8 = 21;
pub const CROP: u8 = 22;
pub const SOIL: u8 = 23;
/// Rails running along x; A is the row north of the track centre, B south.
pub const RAIL_XA: u8 = 24;
pub const RAIL_XB: u8 = 25;
/// Rails running along z; A is the column west of the track centre, B east.
pub const RAIL_ZA: u8 = 26;
pub const RAIL_ZB: u8 = 27;
pub const FENCE: u8 = 28;
pub const FLOWER_R: u8 = 29;
pub const FLOWER_Y: u8 = 30;
pub const FLOWER_B: u8 = 31;
pub const CACTUS: u8 = 32;
pub const PLASTER: u8 = 33;
pub const GRAVEL: u8 = 34;
pub const HAY: u8 = 35;
pub const STONEBRICK: u8 = 36;
pub const LAMPPOST: u8 = 37;
pub const BIRCH: u8 = 38;
pub const TIMBER: u8 = 39;
pub const BENCH: u8 = 40;
pub const PLATFORM: u8 = 41;
pub const ANVIL: u8 = 42;
pub const FORGE: u8 = 43;
pub const TALLGRASS: u8 = 44;
pub const PUMPKIN: u8 = 45;
pub const AWNING_R: u8 = 46;
pub const AWNING_W: u8 = 47;
pub const PLASTER_Y: u8 = 48;
pub const PLASTER_P: u8 = 49;
pub const BUSH: u8 = 50;
pub const ROCK: u8 = 51;
/// Seasonal wildflowers: bluebells in spring woods, poppies in summer meadows.
pub const BLUEBELL: u8 = 52;
pub const POPPY: u8 = 53;
/// Stone used for building walls: the same grey as `STONE`, but kept apart
/// from the ground so the see-through effect can tell a wall from a hillside.
pub const WALLSTONE: u8 = 54;

pub const M_SOLID: u8 = 0;
pub const M_WATER: u8 = 1;
pub const M_LEAF: u8 = 2;
pub const M_GRASS: u8 = 3;
pub const M_WINDOW: u8 = 4;
pub const M_LAMP: u8 = 5;
pub const M_CROP: u8 = 6;
pub const M_CROP_TOP: u8 = 7;
pub const M_PINE: u8 = 8;
pub const M_FLOWER: u8 = 9;
pub const M_FIRE: u8 = 10;
/// Tops of paths, cobbles and gravel: puddles form here in the wet.
pub const M_ROAD: u8 = 11;
/// Tall grass blades: tinted like grass and rippled by the wind.
pub const M_BLADE: u8 = 12;
/// Flowers that only show in spring, or only in summer.
pub const M_SPRING: u8 = 13;
pub const M_SUMMER: u8 = 14;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Empty,
    Full,
    Water,
    Special,
}

pub fn shape(b: u8) -> Shape {
    match b {
        AIR => Shape::Empty,
        WATER => Shape::Water,
        LAMP | CROP | RAIL_XA | RAIL_XB | RAIL_ZA | RAIL_ZB | FENCE | FLOWER_R | FLOWER_Y
        | FLOWER_B | CACTUS | LAMPPOST | BENCH | ANVIL | TALLGRASS | PUMPKIN | BUSH | FORGE | BLUEBELL | POPPY => {
            Shape::Special
        }
        _ => Shape::Full,
    }
}

#[inline]
pub fn opaque(b: u8) -> bool {
    shape(b) == Shape::Full
}

pub fn color(b: u8) -> [u8; 3] {
    match b {
        GRASS => [90, 150, 62],
        DIRT => [138, 98, 64],
        STONE | WALLSTONE => [128, 128, 138],
        SAND => [228, 206, 150],
        WATER => [60, 132, 190],
        SNOW => [238, 242, 250],
        LOG => [108, 76, 50],
        LEAVES => [82, 152, 62],
        PINE => [48, 108, 72],
        PLANK => [182, 132, 82],
        BRICK => [172, 82, 64],
        COBBLE => [156, 150, 142],
        PATH => [192, 160, 112],
        ROOF_RED => [190, 74, 60],
        ROOF_BLUE => [72, 104, 156],
        ROOF_BROWN => [128, 86, 60],
        ROOF_GREEN => [82, 124, 84],
        ROOF_SLATE => [84, 88, 108],
        WINDOW => [96, 148, 186],
        DOOR => [112, 72, 42],
        LAMP => [255, 222, 140],
        CROP => [112, 172, 60],
        SOIL => [112, 78, 52],
        RAIL_XA | RAIL_XB | RAIL_ZA | RAIL_ZB => [92, 66, 46],
        FENCE => [164, 124, 82],
        FLOWER_R => [224, 64, 74],
        FLOWER_Y => [244, 204, 64],
        FLOWER_B => [118, 126, 234],
        CACTUS => [88, 150, 72],
        PLASTER => [238, 228, 206],
        PLASTER_Y => [236, 212, 150],
        PLASTER_P => [222, 186, 190],
        GRAVEL => [124, 116, 108],
        HAY => [222, 192, 92],
        STONEBRICK => [152, 152, 160],
        LAMPPOST => [58, 54, 60],
        BIRCH => [224, 222, 210],
        TIMBER => [94, 64, 44],
        BENCH => [152, 106, 66],
        PLATFORM => [184, 178, 166],
        ANVIL => [62, 62, 72],
        FORGE => [255, 124, 44],
        TALLGRASS => [90, 150, 62],
        PUMPKIN => [232, 132, 42],
        AWNING_R => [204, 64, 62],
        AWNING_W => [242, 240, 230],
        BUSH => [70, 136, 56],
        ROCK => [140, 138, 146],
        BLUEBELL => [96, 104, 214],
        POPPY => [214, 40, 36],
        _ => [255, 0, 255],
    }
}

pub fn material(b: u8) -> u8 {
    match b {
        WATER => M_WATER,
        LEAVES | BUSH => M_LEAF,
        PINE => M_PINE,
        TALLGRASS => M_BLADE,
        WINDOW => M_WINDOW,
        LAMP => M_LAMP,
        CROP => M_CROP,
        FLOWER_R | FLOWER_Y | FLOWER_B => M_FLOWER,
        BLUEBELL => M_SPRING,
        POPPY => M_SUMMER,
        FORGE => M_FIRE,
        _ => M_SOLID,
    }
}
