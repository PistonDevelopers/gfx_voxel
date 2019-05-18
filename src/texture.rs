//! Create textures and build texture atlas.

use std::collections::HashMap;
use std::collections::hash_map::Entry::{ Occupied, Vacant };
use std::fmt::{ Debug, Formatter, Error };
use std::path::{ Path, PathBuf };
use std::mem;

use wgpu::{ Texture, TextureFormat, TextureDescriptor, TextureDimension, Device };
use image::{
    self,
    ImageBuffer,
    RgbaImage,
    DynamicImage,
    ImageResult,
    SubImage,
    ImageError,
    GenericImageView,
    Pixel
};

// Loads RGBA image from path.
fn load_rgba8(path: &Path) -> ImageResult<RgbaImage> {
    image::open(path).map(|img| match img {
        DynamicImage::ImageRgba8(img) => img,
        img => img.to_rgba()
    })
}

/// An enumeration of ColorMap errors.
pub enum ColorMapError {
    /// The image opening error.
    Img(ImageError),

    /// The image size error.
    Size(u32, u32, String)
}

impl Debug for ColorMapError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            ColorMapError::Img(ref e) => e.fmt(f),
            ColorMapError::Size(w, h, ref path) =>
                format!("ColorMap expected 256x256, found {}x{} in '{}'", w, h, path).fmt(f)
        }
    }
}

impl From<ImageError> for ColorMapError {
    fn from(img_err: ImageError) -> Self {
        ColorMapError::Img(img_err)
    }
}

/// A 256x256 image that stores colors.
pub struct ColorMap(RgbaImage);

impl ColorMap {
    /// Creates a new `ColorMap` from path.
    pub fn from_path<P>(path: P) -> Result<Self, ColorMapError>
        where P: AsRef<Path>
    {
        let img = try!(load_rgba8(path.as_ref()));

        match img.dimensions() {
            (256, 256) => Ok(ColorMap(img)),
            (w, h) => Err(ColorMapError::Size(w, h, path.as_ref().display().to_string()))
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

        let col = self.0.get_pixel(x as u32, y as u32).channels();
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
    pub fn new<P>(path: P, unit_width: u32, unit_height: u32) -> Self
        where P: Into<PathBuf>
    {
        AtlasBuilder {
            image: ImageBuffer::new(unit_width * 4, unit_height * 4),
            path: path.into(),
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
        if let Some(&pos) = self.tile_positions.get(name) {
            return pos
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
        if let Some(&alpha) = self.min_alpha_cache.get(&(x, y, w, h)) {
            return alpha
        }

        let tile = SubImage::new(&mut self.image, x, y, w, h);
        let min_alpha = tile.pixels().map(|(_, _, p)| p[3]).min().unwrap_or(0);
        self.min_alpha_cache.insert((x, y, w, h), min_alpha);
        min_alpha
    }

    /// Returns the _current_ atlas size.
    /// Subject to change during every call to `load`!
    pub fn get_size(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    /// Returns the complete texture atlas as a texture.
    pub fn complete(self, device: &mut Device) -> Texture
    {
        let size = self.image.dimensions();
        let texture_extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            size: texture_extent,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::TRANSFER_DST,
        });

        let texels = self.image.into_raw();


        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        
        let temp_buf = device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsage::TRANSFER_SRC)
            .fill_from_slice(&texels);
        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * size.0,
                image_height: size.1,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                array_layer: 0,
		mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            texture_extent,
        );


        // Done
        let init_command_buf = init_encoder.finish();
        device.get_queue().submit(&[init_command_buf]);

        texture
    }
}
