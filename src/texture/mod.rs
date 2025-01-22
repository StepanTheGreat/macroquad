//! Loading and rendering textures. Also render textures, per-pixel image manipulations.

use miniquad::{FilterMode, MipmapFilterMode, RenderingBackend, TextureFormat, TextureId};

mod atlas;
mod image;
mod target;

pub use atlas::*;
pub use image::Image;
pub use target::*;

/// Loads a [TextureId] from a file. This will load an image first, and then convert it
/// into a texture. If you would like to reuse the image - better use
/// [Image::to_texture] instead.
pub fn load_texture(backend: &mut dyn RenderingBackend, path: &str) -> Option<TextureId> {
    let img = image::load_image(path)?;
    Some(img.to_texture(backend))
}

/// A texture storage and state struct for said struct.
///
/// Now, I know what you're thinking - this crate's sole purpose was to eliminate abstractions,
/// and now it provides even more abstractions? You're absolutely correct. One stupid issue I've ran into though,
/// is that a lot of operations in macroquad (like drawing textures) simply rely on the global context. Why?
/// To get the global [RenderingBackend], and check for a texture's size.
///
/// To avoid getting in your way - this abstraction exists solely for drawing/texture manipulation purposes.
#[derive(Clone, Debug)]
pub struct Texture {
    texture: TextureId,
    width: u16,
    height: u16,
    filter: FilterMode,
}

impl Texture {
    /// Load a texture from path.
    ///
    /// This operation returns an [Option], as it can fail
    pub fn load(backend: &mut dyn RenderingBackend, path: &str) -> Option<Self> {
        let texture = load_texture(backend, path)?;
        Some(Self::from_texture(backend, texture))
    }

    /// Create this Texture from [Image]
    pub fn from_image(backend: &mut dyn RenderingBackend, image: &Image) -> Self {
        let texture = image.to_texture(backend);
        Self::from_texture(backend, texture)
    }

    /// Create a texture from provided size parameters and RGBA bytes
    pub fn from_rgba8(
        backend: &mut dyn RenderingBackend,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Self {
        let texture = backend.new_texture_from_rgba8(width as _, height as _, bytes);
        Self::from_texture(backend, texture)
    }

    /// Create a new Texture from
    pub fn from_texture(backend: &mut dyn RenderingBackend, texture: TextureId) -> Self {
        let params = backend.texture_params(texture);

        Self {
            texture,
            width: params.width as _,
            height: params.height as _,
            filter: params.mag_filter,
        }
    }

    /// Get the inner [TextureId] of this texture
    pub fn texture(&self) -> &TextureId {
        &self.texture
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn filter(&self) -> &FilterMode {
        &self.filter
    }

    /// Change the filter for this texture. Mipmap is optional
    pub fn set_filter(&mut self, backend: &mut dyn RenderingBackend, new_filter: FilterMode) {
        self.filter = new_filter;
        backend.texture_set_filter(self.texture, new_filter, MipmapFilterMode::None);
    }

    /// Update the data of this image with provided bytes
    pub fn update_with_bytes(&mut self, backend: &mut dyn RenderingBackend, bytes: &[u8]) {
        texture_update_from_bytes(
            backend,
            &self.texture,
            self.width as _,
            self.height as _,
            bytes,
        );
    }

    /// Update the data of the image with a provided image
    pub fn update_with_image(&mut self, backend: &mut dyn RenderingBackend, image: &Image) {
        texture_update(backend, &self.texture, image);
    }

    /// Take the data from the current framebuffer, and add add it to this texture.
    ///
    /// # Safety
    /// This is unsafe, read about it at [texture_grab_screen]
    pub unsafe fn grab_screen(&mut self, backend: &mut dyn RenderingBackend) {
        texture_grab_screen(backend, &self.texture);
    }

    /// Get an image from this texture's data.
    ///
    /// This is equivalent to:
    /// ```
    /// Image::from_texture(backend, texture)
    /// ```
    pub fn to_image(&self, backend: &mut dyn RenderingBackend) -> Image {
        Image::from_texture(backend, &self.texture)
    }
}

/// Create a texture from RGBA byte array and specified size, filter and mipmap filter information.
///
/// ### Warning
/// The texture returned is raw [miniquad::TextureId]. It's not cleaned up automatically like in
/// macroquad, so it's your responsibility to handle it properly.
pub fn new_texture_from_rgba8(
    backend: &mut dyn RenderingBackend,
    width: u16,
    height: u16,
    bytes: &[u8],
    filter: Option<FilterMode>,
) -> TextureId {
    let texture = backend.new_texture_from_rgba8(width, height, bytes);

    if let Some(filter_mode) = filter {
        backend.texture_set_filter(texture, filter_mode, MipmapFilterMode::None);
    }

    texture
}

/// Update a texture with data from an image.
///
/// ### Warning
/// The width and height of both the image and texture has to be identical.
pub fn texture_update(backend: &mut dyn RenderingBackend, texture: &TextureId, image: &Image) {
    let (width, height) = backend.texture_size(*texture);

    assert_eq!(
        width, image.width as u32,
        "The image should have the same width as the texture"
    );
    assert_eq!(
        height, image.height as u32,
        "The image should have the same height as the texture"
    );

    backend.texture_update(*texture, &image.bytes);
}

/// Updates the texture data from an array of bytes. This is almost identical to [texture_update],
/// but you can use bytes and size arguments directly
pub fn texture_update_from_bytes(
    backend: &mut dyn RenderingBackend,
    texture: &TextureId,
    width: u32,
    height: u32,
    bytes: &[u8],
) {
    let (texture_width, texture_height) = backend.texture_size(*texture);

    assert_eq!(
        texture_width, width,
        "The texture should have the same width as the one provided"
    );
    assert_eq!(
        texture_height, height,
        "The texture should have the same height as the one provided"
    );

    backend.texture_update(*texture, bytes);
}

/// Update only a part of the texture with specified image data.
///
/// This is identical to calling the backend directly:
/// ```
/// backend.texture_update_part(
///     texture,
///     x_offset,
///     y_offset,
///     width,
///     height,
///     &image.bytes,
/// );
/// ```
pub fn texture_update_part(
    backend: &mut dyn RenderingBackend,
    texture: &TextureId,
    image: &Image,
    x_offset: i32,
    y_offset: i32,
    width: i32,
    height: i32,
) {
    backend.texture_update_part(*texture, x_offset, y_offset, width, height, &image.bytes);
}

/// Get the width of the texture.
///
/// This is identical to calling:
/// ```
/// let (width, _) = backend.texture_size(texture);
/// ```
pub fn texture_width(backend: &mut dyn RenderingBackend, texture: &TextureId) -> u32 {
    let (width, _) = backend.texture_size(*texture);
    width
}

/// Returns the height of this texture.
///
/// This is identical to calling:
/// ```
/// let (_, height) = backend.texture_size(texture);
/// ```
pub fn texture_height(backend: &mut dyn RenderingBackend, texture: &TextureId) -> u32 {
    let (_, height) = backend.texture_size(*texture);
    height
}

/// Get the width and height of the texture.
///
/// This is identical to calling:
/// ```
/// let (width, height) = backend.texture_size(texture);
/// ```
pub fn texture_size(backend: &mut dyn RenderingBackend, texture: &TextureId) -> (u32, u32) {
    backend.texture_size(*texture)
}

/// Set [FilterMode] for the texture.
///
/// This is identical to calling (Yes, it doesn't set a mipmap filter, so for that you would need to call
/// the backend yourself):
/// ```
/// backend.texture_set_filter(
///     texture,
///     filter_mode,
///     miniquad::MipmapFilterMode::None,
/// );
/// ```
pub fn texture_set_filter(
    backend: &mut dyn RenderingBackend,
    texture: &TextureId,
    filter_mode: FilterMode,
) {
    backend.texture_set_filter(*texture, filter_mode, miniquad::MipmapFilterMode::None);
}

/// Copy the current framebuffer (screen) to a specified texture
///
/// # Safety
/// 1. This function is unsafe, as it performs OpenGL calls directly
/// 2. It only works only on OpenGL backend (Metal isn't implemented), so there's a high chance it will panic
///    if you use it on Metal targets
#[allow(unreachable_patterns)]
pub unsafe fn texture_grab_screen(backend: &mut dyn RenderingBackend, texture: &TextureId) {
    let params = backend.texture_params(*texture);
    let raw_id = match unsafe { backend.texture_raw_id(*texture) } {
        miniquad::RawId::OpenGl(id) => id,
        _ => unimplemented!(),
    };
    let internal_format = match params.format {
        TextureFormat::RGB8 => miniquad::gl::GL_RGB,
        TextureFormat::RGBA8 => miniquad::gl::GL_RGBA,
        TextureFormat::RGBA16F => miniquad::gl::GL_RGBA,
        TextureFormat::Depth => miniquad::gl::GL_DEPTH_COMPONENT,
        TextureFormat::Depth32 => miniquad::gl::GL_DEPTH_COMPONENT,
        #[cfg(target_arch = "wasm32")]
        TextureFormat::Alpha => miniquad::gl::GL_ALPHA,
        #[cfg(not(target_arch = "wasm32"))]
        TextureFormat::Alpha => miniquad::gl::GL_R8,
    };

    unsafe {
        miniquad::gl::glBindTexture(miniquad::gl::GL_TEXTURE_2D, raw_id);
        miniquad::gl::glCopyTexImage2D(
            miniquad::gl::GL_TEXTURE_2D,
            0,
            internal_format,
            0,
            0,
            params.width as _,
            params.height as _,
            0,
        );
    }
}

/// Returns an [Image] from the pixel data in this texture.
///
/// This operation can be expensive.
pub fn get_texture_data(backend: &mut dyn RenderingBackend, texture: &TextureId) -> Image {
    let (width, height) = backend.texture_size(*texture);
    let mut image = Image {
        width: width as _,
        height: height as _,
        bytes: vec![0; width as usize * height as usize * 4],
    };

    backend.texture_read_pixels(*texture, &mut image.bytes);

    image
}
