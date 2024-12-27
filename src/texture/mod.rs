//! Loading and rendering textures. Also render textures, per-pixel image manipulations.

use crate::{utils::Rect, Error};

use crate::graphics::FilterMode;
use miniquad::{MipmapFilterMode, RenderingBackend, TextureId};
use std::sync::Arc;

mod image;
mod texture;
mod atlas;

pub use image::Image;
pub use texture::*;
pub use atlas::*;

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
    width: u32,
    height: u32,
    filter: FilterMode,
    mipmap: MipmapFilterMode
}

impl Texture {
    /// Create this Texture from [Image]
    pub fn from_image(
        backend: &mut dyn RenderingBackend,
        image: &Image
    ) -> Self {
        let texture = image.to_texture(backend);
        Self::from_texture(backend, texture)
    }

    /// Create a new Texture from 
    pub fn from_texture(
        backend: &mut dyn RenderingBackend, 
        texture: TextureId
    ) -> Self {
        let params = backend.texture_params(texture);

        Self {
            texture,
            width: params.width,
            height: params.height,
            filter: params.mag_filter,
            mipmap: params.mipmap_filter
        }
    }

    /// Get the inner [TextureId] of this texture
    pub fn texture(&self) -> &TextureId {
        &self.texture
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn filter(&self) -> &FilterMode {
        &self.filter
    }

    /// Change the filter for this texture. Mipmap is optional
    pub fn set_filter(
        &mut self,
        backend: &mut dyn RenderingBackend, 
        new_filter: FilterMode,
    ) {
        self.filter = new_filter;
        backend.texture_set_filter( 
            self.texture,  
            new_filter, 
            MipmapFilterMode::None  
        ); 
    }

    /// Delete the texture. It's the same as 
    /// ```
    /// backend.delete_texture(texture.texture());
    /// ```
    /// but, it's also a bit more convenient, since it's a self consuming method
    pub fn delete(self, backend: &mut dyn RenderingBackend) {
        backend.delete_texture(self.texture);
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
    mipmap: Option<MipmapFilterMode>
) -> TextureId {
    let texture = backend.new_texture_from_rgba8(width, height, bytes);
    
    if let Some(filter_mode) = filter {
        backend.texture_set_filter(
            texture, 
            filter_mode, 
            mipmap.unwrap_or(MipmapFilterMode::None)
        );
    }

    texture
}

/// Loads an [Image] from a file into CPU memory.
/// 
/// Currently this function returns an [Option] as an ungly workaround
pub fn load_image(path: &str) -> Option<Image> {
    let bytes = match crate::fs::load_file(path) {
        Ok(bytes) => bytes,
        Err(_) => return None
    };

    match Image::from_bytes_with_format(&bytes, None) {
        Ok(img) => Some(img),
        Err(_) => None
    }
}

/// Loads a [TextureId] from a file. This will load an image first, and then convert it
/// into a texture. If you would like to reuse the image - better use
/// [Image::to_texture] instead.
pub fn load_texture(backend: &mut dyn RenderingBackend, path: &str) -> Option<TextureId> {
    let img = load_image(path)?;
    Some(img.to_texture(backend))
}

#[derive(Debug, Clone)]
pub struct RenderPass {
    pub color_texture: TextureId,
    pub depth_texture: Option<TextureId>,
    pub render_pass: miniquad::RenderPass,
}

#[derive(Debug, Clone)]
pub struct RenderTargetParams {
    /// 1 means no multi sampling.
    /// Note that sample_count > 1 is not supported on GL2, GLES2 and WebGL1
    pub sample_count: i32,

    /// depth: true creates a depth render target attachment and allows
    /// such a render target being used for a depth-testing cameras
    pub depth: bool,
}

impl Default for RenderTargetParams {
    fn default() -> RenderTargetParams {
        RenderTargetParams {
            sample_count: 1,
            depth: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderTarget {
    pub texture: TextureId,
    pub render_pass: RenderPass,
}

/// A shortcut to create a render target with sample_count: 1 and no depth buffer
pub fn new_render_target(
    backend: &mut dyn RenderingBackend,
    width: u32, 
    height: u32
) -> RenderTarget {
    new_render_target_ex(backend, width, height, RenderTargetParams::default())
}

/// A shortcut to create a render target with no depth buffer and `sample_count: 4`
pub fn new_render_target_msaa(
    backend: &mut dyn RenderingBackend,
    width: u32, 
    height: u32
) -> RenderTarget {
    new_render_target_ex(
        backend,
        width,
        height,
        RenderTargetParams {
            sample_count: 4,
            ..Default::default()
        },
    )
}

pub fn new_render_target_ex(
    backend: &mut dyn RenderingBackend,
    width: u32, 
    height: u32, 
    params: RenderTargetParams
) -> RenderTarget {
    let color_texture = backend.new_render_texture(miniquad::TextureParams {
        width,
        height,
        sample_count: params.sample_count,
        ..Default::default()
    });

    let depth_texture = if params.depth {
        Some(
            backend.new_render_texture(miniquad::TextureParams {
                width,
                height,
                format: miniquad::TextureFormat::Depth,
                sample_count: params.sample_count,
                ..Default::default()
            }),
        )
    } else {
        None
    };

    let render_pass;
    let texture;
    if params.sample_count != 0 {
        let color_resolve_texture =
            backend.new_render_texture(miniquad::TextureParams {
                width,
                height,
                ..Default::default()
            });
        render_pass = backend.new_render_pass_mrt(
            &[color_texture],
            Some(&[color_resolve_texture]),
            depth_texture,
        );
        texture = color_resolve_texture;
    } else {
        render_pass = backend.new_render_pass_mrt(&[color_texture], None, depth_texture);
        texture = color_texture;
    }

    let render_pass = RenderPass {
        color_texture: texture.clone(),
        depth_texture: None,
        render_pass: Arc::new(render_pass),
    };

    RenderTarget {
        texture,
        render_pass,
    }
}

/// Batches textures into a single, large atlas. A useful optimization if you have multiple
/// smaller textures and you would like to combine them to avoid issuing multiple draw calls per each
pub struct TextureBatcher {
    unbatched: Vec<Texture>,
    atlas: atlas::TextureAtlas,
}

impl TextureBatcher {
    pub fn new(backend: &mut dyn RenderingBackend) -> Self {
        Self {
            unbatched: Vec::new(),
            atlas: atlas::TextureAtlas::new(backend, FilterMode::Linear),
        }
    }

    pub fn add_unbatched(&mut self, texture: Texture) {
        self.unbatched.push(texture);
    }

    pub fn get_texture_rect(
        &mut self, 
        backend: &mut dyn RenderingBackend, 
        texture: &TextureId
    ) -> Option<(TextureId, Rect)> {
        let id = SpriteKey::Texture(texture);
        let uv_rect = self.atlas.get_uv_rect(backend, id)?;
        Some((self.atlas.texture(backend), uv_rect))
    }

    /// Get all unbatched textures and combine them into a single texture
    pub fn build(&mut self) {
        for texture in self.unbatched.drain(0..) {
            let sprite: Image = texture.get_texture_data();
            let id = SpriteKey::Texture(texture.raw_miniquad_id());

            self.atlas.cache_sprite(id, sprite);
        }

        // ? It seems like this code is for debugging only purposes, so I'll leave it for now
        // TODO: Do something about telemetry
        // let texture = self.atlas.texture();
        // let (w, h) = backend.texture_size(texture);
        // crate::telemetry::log_string(&format!("Atlas: {} {}", w, h));
    }
}