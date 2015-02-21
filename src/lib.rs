#![deny(missing_docs)]
#![feature(core, old_path, std_misc)]

//! A voxel rendering library on top of Gfx.

extern crate gfx_texture;
extern crate gfx;
extern crate image;
extern crate vecmath;

pub mod array;
pub mod cube;
pub mod texture;
