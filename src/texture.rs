//! Create textures and build texture atlas.

use gfx::{ Resources, Factory };
use image::{ self, GenericImage, ImageBuffer, RgbaImage, Pixel, SubImage };
use std::num::Float;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{ Occupied, Vacant };
use std::mem;
use std::path::{ Path, PathBuf };

pub use gfx_texture::Texture;
pub use gfx_texture::ImageSize;

/// Loads RGBA image from path.
fn load_rgba8(path: &Path) -> Result<RgbaImage, String> {
    Ok(match image::open(path) {
        Ok(image::ImageRgba8(img)) => img,
        Ok(image::ImageRgb8(img)) => {
            let (w, h) = img.dimensions();
            // We're forced to use Box::new on the closure because ImageBuffer
            // is defined in the "image" crate.
            ImageBuffer::from_fn(w, h, |x, y| img.get_pixel(x, y).to_rgba())
        }
        Ok(img) => {
            return Err(format!("Unsupported color type {:?} in '{}'",
                img.color(), path.display()));
        }
        Err(e)  => {
            return Err(format!("Could not load '{}': {:?}", path.display(), e));
        }
    })
}

/// A 256x256 image that stores colors.
pub struct ColorMap {
    image: RgbaImage
}

impl ColorMap {
    /// Creates a new `ColorMap` from path.
    pub fn from_path(path: &Path) -> Result<ColorMap, String> {
        let img = try!(load_rgba8(path));

        match img.dimensions() {
            (256, 256) => Ok(ColorMap {image: img}),
            (w, h) => Err(format!("ColorMap expected 256x256, found {}x{} in '{}'",
                                  w, h, path.display()))
        }
    }

    /// Gets RGB color from the color map.
    pub fn get(&self, x: f32, y: f32) -> [u8; 3] {
        // Clamp to [0.0, 1.0].
        let x = x.max(0.0).min(1.0);
        let y = y.max(0.0).min(1.0);

        // Scale y from [0.0, 1.0] to [0.0, x], forming a triangle.
        let y = x * y;

        // Origin is in the bottom-right corner.
        let x = ((1.0 - x) * 255.0) as u8;
        let y = ((1.0 - y) * 255.0) as u8;

        let col = self.image.get_pixel(x as u32, y as u32).channels();
        [col[0], col[1], col[2]]
    }
}

/// Builds an atlas of textures.
pub struct AtlasBuilder {
    image: RgbaImage,
    // Base path for loading tiles.
    path: PathBuf,
    // Size of an individual tile.
    unit_width: u32,
    unit_height: u32,
    // Size of the entirely occupied square, in tiles.
    completed_tiles_size: u32,
    // Position in the current strip.
    position: u32,
    // Position cache for loaded tiles (in pixels).
    tile_positions: HashMap<String, (u32, u32)>,
    // Lowest-alpha cache for rectangles in the atlas.
    min_alpha_cache: HashMap<(u32, u32, u32, u32), u8>
}

impl AtlasBuilder {
    /// Creates a new `AtlasBuilder`.
    pub fn new(path: PathBuf, unit_width: u32, unit_height: u32) -> AtlasBuilder {
        AtlasBuilder {
            image: ImageBuffer::new(unit_width * 4, unit_height * 4),
            path: path,
            unit_width: unit_width,
            unit_height: unit_height,
            completed_tiles_size: 0,
            position: 0,
            tile_positions: HashMap::new(),
            min_alpha_cache: HashMap::new()
        }
    }

    /// Loads a file into the texture atlas.
    /// Checks if the file is loaded and returns position within the atlas.
    /// The name should be specified without file extension.
    /// PNG is the only supported format.
    pub fn load(&mut self, name: &str) -> (u32, u32) {
        match self.tile_positions.get(name) {
            Some(pos) => return *pos,
            None => {}
        }

        let mut path = self.path.join(name);
        path.set_extension("png");
        let img = load_rgba8(&path).unwrap();

        let (iw, ih) = img.dimensions();
        assert!(iw == self.unit_width);
        assert!((ih % self.unit_height) == 0);
        if ih > self.unit_height {
            println!("ignoring {} extra frames in '{}'", (ih / self.unit_height) - 1, name);
        }

        let (uw, uh) = (self.unit_width, self.unit_height);
        let (w, h) = self.image.dimensions();
        let size = self.completed_tiles_size;

        // Expand the image buffer if necessary.
        if self.position == 0 && (uw * size >= w || uh * size >= h) {
            let old = mem::replace(&mut self.image, ImageBuffer::new(w * 2, h * 2));
            for ix in 0 .. w {
                for iy in 0 .. h {
                    *self.image.get_pixel_mut(ix, iy) = old[(ix, iy)];
                }
            }

            /*
            let mut dest = SubImage::new(&mut self.image, 0, 0, w, h);
            for ((_, _, a), b) in dest.pixels_mut().zip(old.pixels()) {
                *a = *b;
            }
            */
        }

        let (x, y) = if self.position < size {
            (self.position, size)
        } else {
            (size, self.position - size)
        };

        self.position += 1;
        if self.position >= size * 2 + 1 {
            self.position = 0;
            self.completed_tiles_size += 1;
        }

        {
            let (x, y, w, h) = (x * uw, y * uh, uw, uh);
            for ix in 0 .. w {
                for iy in 0 .. h {
                    *self.image.get_pixel_mut(ix + x, iy + y) = img[(ix, iy)];
                }
            }
        }

        /*
        let mut dest = SubImage::new(&mut self.image, x * uw, y * uh, uw, uh);
        for ((_, _, a), b) in dest.pixels_mut().zip(img.pixels()) {
            *a = *b;
        }
        */

        *match self.tile_positions.entry(name.to_string()) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert((x * uw, y * uh))
        }
    }

    /// Finds the minimum alpha value in a given sub texture of the image.
    pub fn min_alpha(&mut self, rect: [u32; 4]) -> u8 {
        let x = rect[0];
        let y = rect[1];
        let w = rect[2];
        let h = rect[3];
        match self.min_alpha_cache.get(&(x, y, w, h)) {
            Some(alpha) => return *alpha,
            None => {}
        }

        let tile = SubImage::new(&mut self.image, x, y, w, h);
        let min_alpha = tile.pixels().map(|(_, _, p)| p[3])
            .min().unwrap_or(0);
        self.min_alpha_cache.insert((x, y, w, h), min_alpha);
        min_alpha
    }

    /// Returns the complete texture atlas as a texture.
    pub fn complete<R: Resources, D: Factory<R>>(self, device: &mut D) -> Texture<R> {
        Texture::from_image_with_mipmap(device, &self.image)
    }
}
