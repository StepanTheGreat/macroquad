//! Loading and rendering textures. Also render textures, per-pixel image manipulations.

use crate::{
    color::Color, 
    utils::Rect, 
    prelude::Renderer, 
    text::atlas::SpriteKey, 
    Error
};

use crate::graphics::{FilterMode, DrawMode, Vertex};
use glam::{vec2, Vec2};
use miniquad::{MipmapFilterMode, RenderingBackend, TextureId};
use std::sync::Arc;

mod image;
mod texture;

pub use image::Image;
pub use texture::*;

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
pub fn load_image(path: &str) -> Result<Image, Error> {
    let bytes = crate::fs::load_file(path);

    Image::from_bytes_with_format(&bytes, None)
}

/// Loads a [TextureId] from a file. This will load an image first, and then convert it
/// into a texture. If you would like to reuse the image - better use
/// [Image::to_texture] instead.
pub fn load_texture(backend: &mut dyn RenderingBackend, path: &str) -> Result<TextureId, Error> {
    let img = load_image(path)?;
    Ok(img.to_texture(backend))
}

#[derive(Debug, Clone)]
pub struct RenderPass {
    pub color_texture: Texture2D,
    pub depth_texture: Option<Texture2D>,
    pub(crate) render_pass: Arc<miniquad::RenderPass>,
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

impl RenderPass {
    /// Returns the miniquad handle for this render pass.
    pub fn raw_miniquad_id(&self) -> miniquad::RenderPass {
        *self.render_pass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        if Arc::strong_count(&self.render_pass) < 2 {
            let context = get_quad_context();
            context.delete_render_pass(*self.render_pass);
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderTarget {
    pub texture: Texture2D,
    pub render_pass: RenderPass,
}

/// A shortcut to create a render target with sample_count: 1 and no depth buffer
pub fn render_target(width: u32, height: u32) -> RenderTarget {
    render_target_ex(width, height, RenderTargetParams::default())
}

/// A shortcut to create a render target with no depth buffer and `sample_count: 4`
pub fn render_target_msaa(width: u32, height: u32) -> RenderTarget {
    render_target_ex(
        width,
        height,
        RenderTargetParams {
            sample_count: 4,
            ..Default::default()
        },
    )
}

pub fn render_target_ex(width: u32, height: u32, params: RenderTargetParams) -> RenderTarget {
    let context = get_context();

    let color_texture = get_quad_context().new_render_texture(miniquad::TextureParams {
        width,
        height,
        sample_count: params.sample_count,
        ..Default::default()
    });
    let depth_texture = if params.depth {
        Some(
            get_quad_context().new_render_texture(miniquad::TextureParams {
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
            get_quad_context().new_render_texture(miniquad::TextureParams {
                width,
                height,
                ..Default::default()
            });
        render_pass = get_quad_context().new_render_pass_mrt(
            &[color_texture],
            Some(&[color_resolve_texture]),
            depth_texture,
        );
        texture = color_resolve_texture;
    } else {
        render_pass = get_quad_context().new_render_pass_mrt(&[color_texture], None, depth_texture);
        texture = color_texture;
    }

    let texture = Texture2D {
        texture: context.textures.store_texture(texture),
    };

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

#[derive(Debug, Clone)]
pub struct DrawTextureParams {
    pub dest_size: Option<Vec2>,

    /// Part of texture to draw. If None - draw the whole texture.
    /// Good use example: drawing an image from texture atlas.
    /// Is None by default
    pub source: Option<Rect>,

    /// Rotation in radians
    pub rotation: f32,

    /// Mirror on the X axis
    pub flip_x: bool,

    /// Mirror on the Y axis
    pub flip_y: bool,

    /// Rotate around this point.
    /// When `None`, rotate around the texture's center.
    /// When `Some`, the coordinates are in screen-space.
    /// E.g. pivot (0,0) rotates around the top left corner of the screen, not of the
    /// texture.
    pub pivot: Option<Vec2>,
}

impl Default for DrawTextureParams {
    fn default() -> DrawTextureParams {
        DrawTextureParams {
            dest_size: None,
            source: None,
            rotation: 0.0,
            pivot: None,
            flip_x: false,
            flip_y: false,
        }
    }
}

pub fn draw_texture(gl: &mut Renderer<Vertex>, texture: &Texture2D, x: f32, y: f32, color: Color) {
    draw_texture_ex(gl, texture, x, y, color, Default::default());
}

pub fn draw_texture_ex(
    gl: &mut Renderer<Vertex>,
    texture: &Texture2D,
    x: f32,
    y: f32,
    color: Color,
    params: DrawTextureParams,
) {
    let [mut width, mut height] = texture.size().to_array();

    let Rect {
        x: mut sx,
        y: mut sy,
        w: mut sw,
        h: mut sh,
    } = params.source.unwrap_or(Rect {
        x: 0.,
        y: 0.,
        w: width,
        h: height,
    });

    let texture_opt = context
        .texture_batcher
        .get(texture)
        .map(|(batched_texture, uv)| {
            let [batched_width, batched_height] = batched_texture.size().to_array();
            sx = ((sx / width) * uv.w + uv.x) * batched_width;
            sy = ((sy / height) * uv.h + uv.y) * batched_height;
            sw = (sw / width) * uv.w * batched_width;
            sh = (sh / height) * uv.h * batched_height;

            width = batched_width;
            height = batched_height;

            batched_texture
        });
    let texture = texture_opt.as_ref().unwrap_or(texture);

    let (mut w, mut h) = match params.dest_size {
        Some(dst) => (dst.x, dst.y),
        _ => (sw, sh),
    };
    let mut x = x;
    let mut y = y;
    if params.flip_x {
        x += w;
        w = -w;
    }
    if params.flip_y {
        y += h;
        h = -h;
    }

    let pivot = params.pivot.unwrap_or(vec2(x + w / 2., y + h / 2.));
    let m = pivot;
    let p = [
        vec2(x, y) - pivot,
        vec2(x + w, y) - pivot,
        vec2(x + w, y + h) - pivot,
        vec2(x, y + h) - pivot,
    ];
    let r = params.rotation;
    let p = [
        vec2(
            p[0].x * r.cos() - p[0].y * r.sin(),
            p[0].x * r.sin() + p[0].y * r.cos(),
        ) + m,
        vec2(
            p[1].x * r.cos() - p[1].y * r.sin(),
            p[1].x * r.sin() + p[1].y * r.cos(),
        ) + m,
        vec2(
            p[2].x * r.cos() - p[2].y * r.sin(),
            p[2].x * r.sin() + p[2].y * r.cos(),
        ) + m,
        vec2(
            p[3].x * r.cos() - p[3].y * r.sin(),
            p[3].x * r.sin() + p[3].y * r.cos(),
        ) + m,
    ];
    #[rustfmt::skip]
    let vertices = [
        Vertex::new(p[0].x, p[0].y, 0.,  sx      /width,  sy      /height, color),
        Vertex::new(p[1].x, p[1].y, 0., (sx + sw)/width,  sy      /height, color),
        Vertex::new(p[2].x, p[2].y, 0., (sx + sw)/width, (sy + sh)/height, color),
        Vertex::new(p[3].x, p[3].y, 0.,  sx      /width, (sy + sh)/height, color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    gl.texture(&mut gl, Some(texture));
    gl.draw_mode(DrawMode::Triangles);
    gl.geometry(&vertices, &indices);
}

/// Get pixel data from screen buffer and return an Image (screenshot)
pub fn get_screen_data() -> Image {
    get_context().perform_render_passes();

    let context = get_context();

    let texture_id = get_quad_context().new_render_texture(miniquad::TextureParams {
        width: context.screen_width as _,
        height: context.screen_height as _,
        ..Default::default()
    });

    let texture = Texture2D {
        texture: context.textures.store_texture(texture_id),
    };

    texture.grab_screen();

    texture.get_texture_data()
}

/// Batches textures into a single, large atlas. A useful optimization if you have multiple
/// smaller textures, and you would like to combine them to avoid issuing multiple draw-calls per each texture
pub struct TextureBatcher {
    unbatched: Vec<Texture2D>,
    atlas: crate::text::atlas::Atlas,
}

impl TextureBatcher {
    pub fn new(ctx: &mut dyn miniquad::RenderingBackend) -> Self {
        Self {
            unbatched: Vec::new(),
            atlas: crate::text::atlas::Atlas::new(ctx, miniquad::FilterMode::Linear),
        }
    }

    pub fn add_unbatched(&mut self, texture: &Texture2D) {
        self.unbatched.push(texture.weak_clone());
    }

    pub fn get(&mut self, texture: &Texture2D) -> Option<(Texture2D, Rect)> {
        let id = SpriteKey::Texture(texture.raw_miniquad_id());
        let uv_rect = self.atlas.get_uv_rect(id)?;
        Some((Texture2D::unmanaged(self.atlas.texture()), uv_rect))
    }
}

/// Build an atlas out of all currently loaded texture
/// Later on all draw_texture calls with texture available in the atlas will use
/// the one from the atlas
/// NOTE: the GPU memory and texture itself in Texture2D will still be allocated
/// and Texture->Image conversions will work with Texture2D content, not the atlas
pub fn build_textures_atlas() {
    let context = get_context();

    for texture in context.texture_batcher.unbatched.drain(0..) {
        let sprite: Image = texture.get_texture_data();
        let id = SpriteKey::Texture(texture.raw_miniquad_id());

        context.texture_batcher.atlas.cache_sprite(id, sprite);
    }

    let texture = context.texture_batcher.atlas.texture();
    let (w, h) = get_quad_context().texture_size(texture);
    crate::telemetry::log_string(&format!("Atlas: {} {}", w, h));
}

#[doc(hidden)]
/// Macroquad do not have track of all loaded fonts.
/// Fonts store their characters as ID's in the atlas.
/// There fore resetting the atlas will render all fonts unusable.
pub unsafe fn reset_textures_atlas() {
    let context = get_context();
    context.fonts_storage = crate::text::FontStorage::new(&mut *context.quad_context);
    context.texture_batcher = Batcher::new(&mut *context.quad_context);
}
