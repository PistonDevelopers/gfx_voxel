#![deny(missing_docs)]
#![feature(core)]

//! A voxel rendering library on top of Gfx.

extern crate gfx_texture;
extern crate gfx;
extern crate image;
extern crate vecmath;
extern crate array as array_lib;

pub use array_lib as array;

pub mod cube;
pub mod texture;
