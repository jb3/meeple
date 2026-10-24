//! Procedural names for towns and their residents.

use crate::util::{hash_u32, Rng};

const PREFIX: &[&str] = &[
    "Ash", "Bram", "Bel", "Cor", "Dun", "Elm", "Fern", "Glen", "Hol", "Kings", "Lark", "Mil",
    "Nor", "Oak", "Pen", "Ral", "Stan", "Thorn", "Wil", "Wyn", "Briar", "Cop", "Hazel", "Red",
    "Stone", "Marsh", "Apple", "Barley", "Clover", "Honey", "Moss", "Pebble", "Willow", "Sorrel",
    "Tansy", "Hollow", "Bright", "Fox", "Heron", "Otter", "Rook", "Swan",
];

const SUFFIX: &[&str] = &[
    "ford", "by", "ton", "wick", "stead", "bury", "dale", "mere", "field", "brook", "haven",
    "worth", "ley", "combe", "holt", "well", "bridge", "minster", "gate", "moor", "end", "thorpe",
];

const FIRST: &[&str] = &[
    "Ada", "Alba", "Ash", "Bea", "Bram", "Cass", "Clem", "Dot", "Eli", "Edda", "Fen", "Flo",
    "Gil", "Greta", "Hal", "Hazel", "Ivo", "Isla", "Jem", "Juno", "Kit", "Lark", "Len", "Lottie",
    "Milo", "Moss", "Nell", "Nico", "Odo", "Opal", "Pip", "Poppy", "Quill", "Rosa", "Rue", "Sid",
    "Sky", "Tam", "Tess", "Ulla", "Vera", "Wren", "Wynn", "Yara", "Zed", "Bo", "Cleo", "Dara",
    "Fern", "Ivy", "Jory", "Maple", "Otto", "Perry", "Sage", "Toby", "Willa", "Arlo", "Elsie",
    "Rowan", "Sorrel", "Bryn", "Cato", "Hester", "Nim", "Pell",
];

const SURNAMES: &[&str] = &[
    "Marlow", "Thatcher", "Fairweather", "Cooper", "Hartley", "Bellamy", "Ashdown", "Wheeler",
    "Gosling", "Penny", "Tanner", "Larkin", "Fletcher", "Merriweather", "Crabtree", "Mellor",
    "Dunmore", "Hobbs", "Pickering", "Sallow", "Tilling", "Underhill", "Wick", "Yarrow", "Bracken",
    "Cobb", "Dapple", "Fennimore", "Goodbody", "Hazelwood", "Ketteridge", "Lovegrove",
];

/// A household's family name.
pub fn surname(seed: u32) -> &'static str {
    SURNAMES[hash_u32(seed ^ 0x5a4e) as usize % SURNAMES.len()]
}

pub fn town_name(seed: u32) -> String {
    let mut r = Rng::new(hash_u32(seed ^ 0x51ab) as u64);
    let mut s = String::from(*r.pick(PREFIX));
    s.push_str(r.pick(SUFFIX));
    s
}

pub fn person_name(seed: u32) -> &'static str {
    FIRST[hash_u32(seed ^ 0x9e11) as usize % FIRST.len()]
}
