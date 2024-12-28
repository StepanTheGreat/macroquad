use miniquad::{FilterMode, RenderingBackend, TextureId};

use crate::{color::Color, utils::Rect};

use super::new_texture_from_rgba8;

/// Loads an [Image] from a file into CPU memory.
///
/// Currently this function returns an [Option] as an ungly workaround
pub fn load_image(path: &str) -> Option<Image> {
    let bytes = match crate::fs::load_file(path) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };

    match Image::from_bytes_with_format(&bytes, None) {
        Ok(img) => Some(img),
        Err(_) => None,
    }
}

/// Image, data stored in CPU memory
#[derive(Clone)]
pub struct Image {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes.len()", &self.bytes.len())
            .finish()
    }
}

impl Image {
    /// Creates an empty Image.
    ///
    /// ```
    /// # use macroquad::prelude::*;
    /// let image = Image::empty();
    /// ```
    pub const fn empty() -> Image {
        Image {
            width: 0,
            height: 0,
            bytes: Vec::new(),
        }
    }

    /// Creates an Image from a slice of bytes that contains an encoded image.
    ///
    /// If `format` is None, it will make an educated guess on the
    /// [ImageFormat][image::ImageFormat].
    ///
    /// # Example
    ///
    /// ```
    /// # use macroquad::prelude::*;
    /// let icon = Image::from_file_with_format(
    ///     include_bytes!("../examples/rust.png"),
    ///     Some(ImageFormat::Png),
    ///     );
    /// ```
    pub fn from_bytes_with_format(
        bytes: &[u8],
        format: Option<image::ImageFormat>,
    ) -> Result<Self, image::error::ImageError> {
        let img = match format {
            Some(fmt) => image::load_from_memory_with_format(bytes, fmt)?.to_rgba8(),
            None => image::load_from_memory(bytes)?.to_rgba8(),
        };

        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Ok(Self {
            width,
            height,
            bytes,
        })
    }

    /// Create an image from a provided GPU Texture.
    ///
    /// This is an expensive operation, so avoid it doing often
    pub fn from_texture(backend: &mut dyn RenderingBackend, texture: &TextureId) -> Self {
        let (width, height) = backend.texture_size(*texture);
        let mut image = Self {
            width: width as _,
            height: height as _,
            bytes: vec![0; width as usize * height as usize * 4],
        };
        backend.texture_read_pixels(*texture, &mut image.bytes);
        image
    }

    /// Creates an Image filled with the provided [Color].
    pub fn gen_image_color(width: u16, height: u16, color: Color) -> Image {
        let mut bytes = vec![0; width as usize * height as usize * 4];
        for i in 0..width as usize * height as usize {
            bytes[i * 4] = (color.r * 255.) as u8;
            bytes[i * 4 + 1] = (color.g * 255.) as u8;
            bytes[i * 4 + 2] = (color.b * 255.) as u8;
            bytes[i * 4 + 3] = (color.a * 255.) as u8;
        }
        Image {
            width,
            height,
            bytes,
        }
    }

    /// Updates this image from a slice of [Color]s.
    pub fn update(&mut self, colors: &[Color]) {
        assert!(self.width as usize * self.height as usize == colors.len());

        for (i, color) in colors.iter().enumerate() {
            self.bytes[i * 4] = (color.r * 255.) as u8;
            self.bytes[i * 4 + 1] = (color.g * 255.) as u8;
            self.bytes[i * 4 + 2] = (color.b * 255.) as u8;
            self.bytes[i * 4 + 3] = (color.a * 255.) as u8;
        }
    }

    /// Returns the width of this image.
    pub const fn width(&self) -> usize {
        self.width as usize
    }

    /// Returns the height of this image.
    pub const fn height(&self) -> usize {
        self.height as usize
    }

    /// Returns this image's data as a slice of 4-byte arrays.
    pub fn get_image_data(&self) -> &[[u8; 4]] {
        use std::slice;

        unsafe {
            slice::from_raw_parts(
                self.bytes.as_ptr() as *const [u8; 4],
                self.width as usize * self.height as usize,
            )
        }
    }

    /// Returns this image's data as a mutable slice of 4-byte arrays.
    pub fn get_image_data_mut(&mut self) -> &mut [[u8; 4]] {
        use std::slice;

        unsafe {
            slice::from_raw_parts_mut(
                self.bytes.as_mut_ptr() as *mut [u8; 4],
                self.width as usize * self.height as usize,
            )
        }
    }

    /// Modifies a pixel [Color] in this image.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        assert!(x < self.width as u32);
        assert!(y < self.height as u32);

        let width = self.width;

        self.get_image_data_mut()[(y * width as u32 + x) as usize] = color.into();
    }

    /// Returns a pixel [Color] from this image.
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        self.get_image_data()[(y * self.width as u32 + x) as usize].into()
    }

    /// Returns an Image from a rect inside this image.
    pub fn sub_image(&self, rect: Rect) -> Image {
        let width = rect.w as usize;
        let height = rect.h as usize;
        let mut bytes = vec![0; width * height * 4];

        let x = rect.x as usize;
        let y = rect.y as usize;
        let mut n = 0;
        for y in y..y + height {
            for x in x..x + width {
                bytes[n] = self.bytes[y * self.width as usize * 4 + x * 4];
                bytes[n + 1] = self.bytes[y * self.width as usize * 4 + x * 4 + 1];
                bytes[n + 2] = self.bytes[y * self.width as usize * 4 + x * 4 + 2];
                bytes[n + 3] = self.bytes[y * self.width as usize * 4 + x * 4 + 3];
                n += 4;
            }
        }
        Image {
            width: width as u16,
            height: height as u16,
            bytes,
        }
    }

    /// Blends this image with another image (of identical dimensions)
    /// Inspired by  OpenCV saturated blending
    pub fn blend(&mut self, other: &Image) {
        assert!(
            self.width as usize * self.height as usize
                == other.width as usize * other.height as usize
        );

        for i in 0..self.bytes.len() / 4 {
            let c1: Color = Color {
                r: self.bytes[i * 4] as f32 / 255.,
                g: self.bytes[i * 4 + 1] as f32 / 255.,
                b: self.bytes[i * 4 + 2] as f32 / 255.,
                a: self.bytes[i * 4 + 3] as f32 / 255.,
            };
            let c2: Color = Color {
                r: other.bytes[i * 4] as f32 / 255.,
                g: other.bytes[i * 4 + 1] as f32 / 255.,
                b: other.bytes[i * 4 + 2] as f32 / 255.,
                a: other.bytes[i * 4 + 3] as f32 / 255.,
            };
            let new_color: Color = Color {
                r: f32::min(c1.r * c1.a + c2.r * c2.a, 1.),
                g: f32::min(c1.g * c1.a + c2.g * c2.a, 1.),
                b: f32::min(c1.b * c1.a + c2.b * c2.a, 1.),
                a: f32::max(c1.a, c2.a) + (1. - f32::max(c1.a, c2.a)) * f32::min(c1.a, c2.a),
            };
            self.bytes[i * 4] = (new_color.r * 255.) as u8;
            self.bytes[i * 4 + 1] = (new_color.g * 255.) as u8;
            self.bytes[i * 4 + 2] = (new_color.b * 255.) as u8;
            self.bytes[i * 4 + 3] = (new_color.a * 255.) as u8;
        }
    }

    /// Overlays an image on top of this one.
    /// Slightly different from blending two images,
    /// overlaying a completely transparent image has no effect
    /// on the original image, though blending them would.
    pub fn overlay(&mut self, other: &Image) {
        assert!(
            self.width as usize * self.height as usize
                == other.width as usize * other.height as usize
        );

        for i in 0..self.bytes.len() / 4 {
            let c1: Color = Color {
                r: self.bytes[i * 4] as f32 / 255.,
                g: self.bytes[i * 4 + 1] as f32 / 255.,
                b: self.bytes[i * 4 + 2] as f32 / 255.,
                a: self.bytes[i * 4 + 3] as f32 / 255.,
            };
            let c2: Color = Color {
                r: other.bytes[i * 4] as f32 / 255.,
                g: other.bytes[i * 4 + 1] as f32 / 255.,
                b: other.bytes[i * 4 + 2] as f32 / 255.,
                a: other.bytes[i * 4 + 3] as f32 / 255.,
            };
            let new_color: Color = Color {
                r: f32::min(c1.r * (1. - c2.a) + c2.r * c2.a, 1.),
                g: f32::min(c1.g * (1. - c2.a) + c2.g * c2.a, 1.),
                b: f32::min(c1.b * (1. - c2.a) + c2.b * c2.a, 1.),
                a: f32::min(c1.a + c2.a, 1.),
            };

            self.bytes[i * 4] = (new_color.r * 255.) as u8;
            self.bytes[i * 4 + 1] = (new_color.g * 255.) as u8;
            self.bytes[i * 4 + 2] = (new_color.b * 255.) as u8;
            self.bytes[i * 4 + 3] = (new_color.a * 255.) as u8;
        }
    }

    /// Saves this image as a PNG file.
    /// This method is not supported on web and will panic.
    pub fn export_png(&self, path: &str) {
        let mut bytes = vec![0; self.width as usize * self.height as usize * 4];

        // flip the image before saving
        for y in 0..self.height as usize {
            for x in 0..self.width as usize * 4 {
                bytes[y * self.width as usize * 4 + x] =
                    self.bytes[(self.height as usize - y - 1) * self.width as usize * 4 + x];
            }
        }

        image::save_buffer(
            path,
            &bytes[..],
            self.width as _,
            self.height as _,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }

    /// Create a raw [TextureId] from an [Image]. This is a simplified version of
    /// [Image::to_texture_ex], which doesn't enforce any specific filters/mipmap settings
    ///
    /// ### Warning
    /// The texture returned is raw [miniquad::TextureId]. It's not cleaned up automatically like in
    /// macroquad, so it's your responsibility to handle it properly.
    pub fn to_texture(&self, backend: &mut dyn RenderingBackend) -> TextureId {
        self.to_texture_ex(backend, None)
    }

    /// Create a raw [TextureId] from an [Image]. This is a more detailed version of
    /// [Image::to_texture]
    ///
    /// ### Warning
    /// The texture returned is raw [miniquad::TextureId]. It's not cleaned up automatically like in
    /// macroquad, so it's your responsibility to handle it properly.
    pub fn to_texture_ex(
        &self,
        backend: &mut dyn RenderingBackend,
        filter: Option<FilterMode>,
    ) -> TextureId {
        new_texture_from_rgba8(backend, self.width, self.height, &self.bytes, filter)
    }
}
