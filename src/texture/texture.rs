use miniquad::{RenderingBackend, TextureFormat, TextureId};
use super::*;

/// Update a texture with data from an image. 
/// 
/// ### Warning
/// The width and height of both the image and texture has to be identical.
pub fn texture_update(
    backend: &mut dyn RenderingBackend, 
    texture: &TextureId, 
    image: &Image
) {
    let (width, height) = backend.texture_size(texture);

    assert_eq!(width, image.width as u32, "The image should have the same width as the texture");
    assert_eq!(height, image.height as u32, "The image should have the same height as the texture");

    backend.texture_update(texture, &image.bytes);
}

/// Updates the texture data from an array of bytes. This is almost identical to [texture_update],
/// but you can use bytes and size arguments directly
pub fn texture_update_from_bytes(
    backend: &mut dyn RenderingBackend,
    texture: &TextureId,
    width: u32, 
    height: u32, 
    bytes: &[u8]
) {
    let (texture_width, texture_height) = backend.texture_size(texture);

    assert_eq!(texture_width, width, "The texture should have the same width as the one provided");
    assert_eq!(texture_height, height, "The texture should have the same height as the one provided");

    backend.texture_update(texture, bytes);
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
    backend.texture_update_part(
        texture,
        x_offset,
        y_offset,
        width,
        height,
        &image.bytes,
    );
}

/// Get the width of the texture.
/// 
/// This is identical to calling:
/// ```
/// let (width, _) = backend.texture_size(texture);
/// ```
pub fn texture_width(backend: &mut dyn RenderingBackend, texture: &TextureId) -> u32 {
    let (width, _) = backend.texture_size(texture);
    width
}

/// Returns the height of this texture.
/// 
/// This is identical to calling:
/// ```
/// let (_, height) = backend.texture_size(texture);
/// ```
pub fn texture_height(backend: &mut dyn RenderingBackend, texture: &TextureId) -> u32 {
    let (_, height) = backend.texture_size(texture);
    height
}

/// Get the width and height of the texture. 
/// 
/// This is identical to calling:
/// ```
/// let (width, height) = backend.texture_size(texture);
/// ```
pub fn texture_size(backend: &mut dyn RenderingBackend, texture: &TextureId) -> (u32, u32) {
    backend.texture_size(texture)
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
    filter_mode: FilterMode
) {
    backend.texture_set_filter(
        texture,
        filter_mode,
        miniquad::MipmapFilterMode::None,
    );
}

/// Copy the current framebuffer (screen) to a specified texture
/// 
/// ### Warning
/// 1. This function is unsafe, as it performs OpenGL calls directly
/// 2. It only works only on OpenGL (Metal isn't implemented), so there's a high chance it will panic
#[allow(unreachable_patterns)]
pub unsafe fn texture_grab_screen(
    backend: &mut dyn RenderingBackend,
    texture: &TextureId
) {
    let params = backend.texture_params(texture);
    let raw_id = match unsafe { backend.texture_raw_id(texture) } {
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
pub fn get_texture_data(
    backend: &mut dyn RenderingBackend,
    texture: &TextureId
) -> Image {
    let (width, height) = backend.texture_size(texture);
    let mut image = Image {
        width: width as _,
        height: height as _,
        bytes: vec![0; width as usize * height as usize * 4],
    };
    
    backend.texture_read_pixels(texture, &mut image.bytes);

    image
}