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

/// A 3D vector.
pub type Vector3<T> = [T; 3];

pub use self::Face::{
    Down,
    Up,
    North,
    South,
    West,
    East,
};

// Cube faces (clockwise).
pub const QUADS: &'static [[usize; 4]; 6] = &[
    [1, 0, 5, 4], // down
    [7, 6, 3, 2], // up
    [0, 1, 2, 3], // north
    [4, 5, 6, 7], // south
    [5, 0, 3, 6], // west
    [1, 4, 7, 2]  // east
];

// Cube vertices.
pub const VERTICES: &'static [Vector3<f32>; 8] = &[
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
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Face {
    /// Facing down.
    Down,
    /// Facing up.
    Up,
    /// Facing north.
    North,
    /// Facing south.
    South,
    /// Facing west.
    West,
    /// Facing east.
    East
}

impl Face {
    /// Computes vertices of the face.
    pub fn vertices(self, base: Vector3<f32>, scale: Vector3<f32>) -> [Vector3<f32>; 4] {
        use array::*;

        QUADS[self as usize].map(|i| VERTICES[i]).map(|v| {
            [
                base[0] + scale[0] * v[0],
                base[1] + scale[1] * v[1],
                base[2] + scale[2] * v[2]
            ]
        })
    }

    /// Gets the direction of face.
    pub fn direction(self) -> [i32; 3] {
        match self {
            Down  => [ 0, -1,  0],
            Up    => [ 0,  1,  0],
            North => [ 0,  0, -1],
            South => [ 0,  0,  1],
            West  => [-1,  0,  0],
            East  => [ 1,  0,  0]
        }
    }

    /// Gets the face in a specific direction.
    pub fn from_direction(d: [i32; 3]) -> Option<Self> {
        Some(match (d[0], d[1], d[2]) {
            ( 0, -1,  0) => Down,
            ( 0,  1,  0) => Up,
            ( 0,  0, -1) => North,
            ( 0,  0,  1) => South,
            (-1,  0,  0) => West,
            ( 1,  0,  0) => East,
            _ => return None
        })
    }

    /// Convert number to face.
    pub fn from_usize(number: usize) -> Option<Self> {
        Some(match number {
            0 => Down,
            1 => Up,
            2 => North,
            3 => South,
            4 => West,
            5 => East,
            _ => return None
        })
    }
}

/// The error parsing face from string.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ParseError;

impl FromStr for Face {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(match s {
            "down"  => Down,
            "up"    => Up,
            "north" => North,
            "south" => South,
            "west"  => West,
            "east"  => East,
            _ => return Err(ParseError)
        })
    }
}

/// Iterates through each face on a cube.
#[derive(Copy, Clone)]
pub struct FaceIterator(usize);

impl FaceIterator {
    /// Creates a new face iterator.
    pub fn new() -> Self {
        FaceIterator(0)
    }
}

impl Iterator for FaceIterator {
    type Item = Face;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let face = self.0;
        if face < 6 {
            self.0 += 1;
            Face::from_usize(face)
        } else {
            None
        }
    }
}
