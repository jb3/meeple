//! Builders for the single-site structures the planner places (see `gen::sites`).

use super::*;

mod buildings;
mod country;
mod landmarks;
pub(crate) use self::buildings::*;
pub(crate) use self::country::*;
pub(in crate::world::chunk) use self::landmarks::*;

pub(super) fn site_out(face: u8) -> (i32, i32) {
    [(0, -1), (1, 0), (0, 1), (-1, 0)][face as usize % 4]
}
