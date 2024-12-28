use miniquad::{FilterMode, RenderingBackend, TextureId};

use crate::{texture::Image, color::Color};
use crate::utils::Rect;

use std::collections::HashMap;

use super::Texture;

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub rect: Rect,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SpriteKey {
    Texture(TextureId),
    Id(u64),
}

/// A combination of textures in a single, large texture.
pub struct TextureAtlas {
    texture: Texture,
    image: Image,
    pub sprites: HashMap<SpriteKey, Sprite>,
    cursor_x: u16,
    cursor_y: u16,
    max_line_height: u16,

    pub dirty: bool,

    filter: FilterMode,

    unique_id: u64,
}

impl TextureAtlas {
    // pixel gap between glyphs in the atlas
    const GAP: u16 = 2;
    // well..
    const UNIQUENESS_OFFSET: u64 = 100000;

    pub fn new(
        backend: &mut dyn RenderingBackend, 
        filter: FilterMode
    ) -> Self {
        let image = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));
        let mut texture = Texture::from_rgba8(backend, image.width, image.height, &image.bytes);
        
        // TODO: Check whether this causes any issues. Originally, the filter is always set to Nearest,
        // totally ignoring the provided one.
        texture.set_filter(backend, filter);

        Self {
            image,
            texture,
            cursor_x: 0,
            cursor_y: 0,
            dirty: false,
            max_line_height: 0,
            sprites: HashMap::new(),
            filter,
            unique_id: Self::UNIQUENESS_OFFSET,
        }
    }

    /// Get a new unique sprite key
    pub fn new_unique_id(&mut self) -> SpriteKey {
        self.unique_id += 1;

        SpriteKey::Id(self.unique_id)
    }

    /// Change the filter for the atlas texture
    pub fn set_filter(
        &mut self, 
        backend: &mut dyn RenderingBackend, 
        filter_mode: FilterMode
    ) {
        self.filter = filter_mode;
        self.texture.set_filter(backend, filter_mode);
        // backend.texture_set_filter(self.texture, filter_mode, MipmapFilterMode::None);
    }

    pub fn get(&self, key: SpriteKey) -> Option<Sprite> {
        self.sprites.get(&key).cloned()
    }

    pub const fn width(&self) -> u16 {
        self.image.width
    }

    pub const fn height(&self) -> u16 {
        self.image.height
    }

    /// Get the atlas texture.
    /// 
    /// If *dirty*, will immediately update the texture
    pub fn texture(&mut self, backend: &mut dyn RenderingBackend) -> &Texture {
        if self.dirty {
            self.dirty = false;
            let (texture_width, texture_height) = self.texture.size();
            if texture_width != (self.image.width as _) || texture_height != (self.image.height as _) {
                // We're doing here using the rendering backend, since 
                // dropping fields simply isn't possible
                backend.delete_texture(*self.texture.texture());

                self.texture = Texture::from_rgba8(
                    backend,
                    self.image.width,
                    self.image.height,
                    &self.image.bytes[..],
                );
                self.texture.set_filter(backend, self.filter);
                // backend.texture_set_filter(self.texture, self.filter, MipmapFilterMode::None);
            }

            self.texture.update_with_image(backend, &self.image);
            // backend.texture_update(self.texture, &self.image.bytes);
        }

        &self.texture
    }

    /// Try to get a rect in the atlas for the provided sprite key
    pub fn get_uv_rect(&self, key: SpriteKey) -> Option<Rect> {
        self.get(key).map(|sprite| {
            let (w, h) = self.texture.size();

            Rect::new(
                sprite.rect.x / w as f32,
                sprite.rect.y / h as f32,
                sprite.rect.w / w as f32,
                sprite.rect.h / h as f32,
            )
        })
    }

    pub fn cache_sprite(&mut self, key: SpriteKey, sprite: Image) {
        let (width, height) = (sprite.width as usize, sprite.height as usize);

        let x = if self.cursor_x + (width as u16) < self.image.width {
            if height as u16 > self.max_line_height {
                self.max_line_height = height as u16;
            }
            let res = self.cursor_x + Self::GAP;
            self.cursor_x += width as u16 + Self::GAP * 2;
            res
        } else {
            self.cursor_y += self.max_line_height + Self::GAP * 2;
            self.cursor_x = width as u16 + Self::GAP;
            self.max_line_height = height as u16;
            Self::GAP
        };
        let y = self.cursor_y;

        // texture bounds exceeded
        if y + sprite.height > self.image.height || x + sprite.width > self.image.width {
            // reset glyph cache state
            let sprites = self.sprites.drain().collect::<Vec<_>>();
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.max_line_height = 0;

            let old_image = self.image.clone();

            // increase font texture size
            // note: if we tried to fit gigantic texture into a small atlas,
            // new_width will still be not enough. But its fine, it will
            // be regenerated on the recursion call.
            let new_width = self.image.width * 2;
            let new_height = self.image.height * 2;

            self.image = Image::gen_image_color(
                new_width, 
                new_height, 
                Color::new(0.0, 0.0, 0.0, 0.0)
            );

            // recache all previously cached symbols
            for (key, sprite) in sprites {
                let image = old_image.sub_image(sprite.rect);
                self.cache_sprite(key, image);
            }

            // cache the new sprite
            self.cache_sprite(key, sprite);
        } else {
            self.dirty = true;

            for j in 0..height {
                for i in 0..width {
                    self.image.set_pixel(
                        x as u32 + i as u32,
                        y as u32 + j as u32,
                        sprite.get_pixel(i as u32, j as u32),
                    );
                }
            }

            self.sprites.insert(
                key,
                Sprite {
                    rect: Rect::new(x as f32, y as f32, width as f32, height as f32),
                },
            );
        }
    }
}

/// Batches textures into a single, large atlas. A useful optimization if you have multiple
/// smaller textures and you would like to combine them to avoid issuing multiple draw calls per each
pub struct TextureBatcher {
    unbatched: Vec<TextureId>,
    atlas: TextureAtlas,
}

impl TextureBatcher {
    pub fn new(backend: &mut dyn RenderingBackend) -> Self {
        Self {
            unbatched: Vec::new(),
            atlas: TextureAtlas::new(backend, FilterMode::Linear),
        }
    }

    pub fn add_unbatched(&mut self, texture: &Texture) {
        self.unbatched.push(*texture.texture());
    }

    pub fn get_texture_rect<'a>(
        &'a mut self, 
        backend: &mut dyn RenderingBackend, 
        texture: &Texture
    ) -> Option<(&'a Texture, Rect)> {
        let id = SpriteKey::Texture(*texture.texture());
        let uv_rect = self.atlas.get_uv_rect(id)?;
        Some((self.atlas.texture(backend), uv_rect))
    }

    /// Get all unbatched textures and combine them into a single texture
    pub fn build(&mut self, backend: &mut dyn RenderingBackend) {
        for texture in self.unbatched.drain(0..) {
            let sprite: Image = Image::from_texture(backend, &texture);
            let id = SpriteKey::Texture(texture);

            self.atlas.cache_sprite(id, sprite);
        }

        // ? It seems like this code is for debugging only purposes, so I'll leave it for now
        // TODO: Do something about telemetry
        // let texture = self.atlas.texture();
        // let (w, h) = backend.texture_size(texture);
        // crate::telemetry::log_string(&format!("Atlas: {} {}", w, h));
    }
}