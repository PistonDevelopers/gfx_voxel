//! Helper methods and structures for working with cubes.
//!
//! ```ignore
//!         3  ---------  2
//!           /       / |
//!          /  up   /  |
//!      6  -------- 7  | 1
//!        |        |  /
//! west   |  south | /  east
//!        |        |/
//!      5  -------- 4
//! ```
//!
//!
//! ```ignore
//!         7  ---------  6
//!           /       / |
//!          /  up   /  |
//!      2  -------- 3  | 5
//!        |        |  /
//! east   |  north | /  west
//!        |        |/
//!      1  -------- 0
//! ```

use std::str::FromStr;

use vecmath::Vector3;

pub use self::Face::{
    Down,
    Up,
    North,
    South,
    West,
    East,
};

// Cube faces (clockwise).
pub const QUADS: &'static [[uint, ..4], ..6] = &[
    [1, 0, 5, 4], // down
    [7, 6, 3, 2], // up
    [0, 1, 2, 3], // north
    [4, 5, 6, 7], // south
    [5, 0, 3, 6], // west
    [1, 4, 7, 2]  // east
];

// Cube vertices.
pub const VERTICES: &'static [Vector3<f32>, ..8] = &[
    // This is the north surface
    [0.0, 0.0, 0.0], // 0
    [1.0, 0.0, 0.0], // 1
    [1.0, 1.0, 0.0], // 2
    [0.0, 1.0, 0.0], // 3

    // This is the south surface
    [1.0, 0.0, 1.0], // 4
    [0.0, 0.0, 1.0], // 5
    [0.0, 1.0, 1.0], // 6
    [1.0, 1.0, 1.0]  // 7
];

/// A value representing face direction.
#[repr(uint)]
#[deriving(PartialEq, Eq, FromPrimitive, Show)]
pub enum Face {
    /// Facing down.
    Down = 0,
    /// Facing up.
    Up = 1,
    /// Facing north.
    North = 2,
    /// Facing south.
    South = 3,
    /// Facing west.
    West = 4,
    /// Facing east.
    East = 5
}

impl Face {
    /// Computes vertices of the face.
    pub fn vertices(self, base: Vector3<f32>, scale: Vector3<f32>) -> [Vector3<f32>, ..4] {
        use array::*;

        let [x, y, z] = base;
        let [sx, sy, sz] = scale;

        QUADS[self as uint].map(|i| VERTICES[i]).map(|[vx, vy, vz]| {
            [x + sx * vx, y + sy * vy, z + sz * vz]
        })
    }

    /// Gets the direction of face.
    pub fn direction(self) -> [i32, ..3] {
        match self {
            Down => [0, -1, 0],
            Up => [0, 1, 0],
            North => [0, 0, -1],
            South => [0, 0, 1],
            West => [-1, 0, 0],
            East => [1, 0, 0]
        }
    }

    /// Gets the face in a specific direction.
    pub fn from_direction(d: [i32, ..3]) -> Option<Face> {
        Some(match d {
            [0, -1, 0] => Down,
            [0, 1, 0] => Up,
            [0, 0, -1] => North,
            [0, 0, 1] => South,
            [-1, 0, 0] => West,
            [1, 0, 0] => East,
            _ => return None
        })
    }
}

impl FromStr for Face {
    fn from_str(s: &str) -> Option<Face> {
        Some(match s {
            "down" => Down,
            "up" => Up,
            "north" => North,
            "south" => South,
            "west" => West,
            "east" => East,
            _ => return None
        })
    }
}

/// Iterates through each face on a cube.
pub struct FaceIterator {
    face: uint,
}

impl FaceIterator {
    /// Creates a new face iterator.
    pub fn new() -> FaceIterator {
        FaceIterator {
            face: 0
        }
    }
}

impl Iterator<Face> for FaceIterator {
    fn next(&mut self) -> Option<Face> {
        match self.face {
            x if x < 6 => {
                self.face += 1;
                FromPrimitive::from_uint(x)
            },
            _ => None
        }
    }
}
