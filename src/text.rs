//! Functions to load fonts and draw text.

use std::collections::HashMap;

use crate::{texture::{Image, Texture}, Error};

use glam::{vec3, Mat4};
use miniquad::{FilterMode, RenderingBackend, TextureId};

use crate::texture::{SpriteKey, TextureAtlas};

#[derive(Debug, Clone)]
pub(crate) struct CharacterInfo {
    pub offset_x: i32,
    pub offset_y: i32,
    pub advance: f32,
    pub sprite: SpriteKey,
}

/// A FontAtlas is a loaded font, a GPU texture storing all characters and a character map to access
/// said characters.
///
/// ### Warning
/// Once again, it's your responsibility to manage and properly clean this after. TextureAtlas is essentially
/// a [TextureId] in miniquad, so after using it - delete it using [RenderingBackend]
pub struct FontAtlas {
    pub(crate) font: fontdue::Font,
    pub(crate) atlas: TextureAtlas,
    pub(crate) characters: HashMap<(char, u16), CharacterInfo>,
}

/// World space dimensions of the text, measured by "measure_text" function
#[derive(Debug, Default, Clone, Copy)]
pub struct TextDimensions {
    /// Distance from very left to very right of the rasterized text
    pub width: f32,
    /// Distance from the bottom to the top of the text.
    pub height: f32,
    /// Height offset from the baseline of the text.
    /// "draw_text(.., X, Y, ..)" will be rendered in a "Rect::new(X, Y - dimensions.offset_y, dimensions.width, dimensions.height)"
    /// For reference check "text_measures" example.
    pub offset_y: f32,
}

impl std::fmt::Debug for FontAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("font", &"fontdue::Font")
            .finish()
    }
}

impl FontAtlas {
    pub fn load_from_bytes(atlas: TextureAtlas, bytes: &[u8]) -> Result<FontAtlas, Error> {
        Ok(FontAtlas {
            font: fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())?,
            characters: HashMap::new(),
            atlas,
        })
    }

    pub fn set_atlas(&mut self, atlas: TextureAtlas) {
        self.atlas = atlas;
    }

    pub(crate) fn set_characters(&mut self, characters: HashMap<(char, u16), CharacterInfo>) {
        self.characters = characters;
    }

    pub(crate) fn ascent(&self, font_size: f32) -> f32 {
        self.font.horizontal_line_metrics(font_size).unwrap().ascent
    }

    pub(crate) fn descent(&self, font_size: f32) -> f32 {
        self.font
            .horizontal_line_metrics(font_size)
            .unwrap()
            .descent
    }

    pub(crate) fn cache_glyph(&mut self, character: char, size: u16) {
        if self.contains(character, size) {
            return;
        }

        let (metrics, bitmap) = self.font.rasterize(character, size as f32);

        if metrics.advance_height != 0.0 {
            panic!("Vertical fonts are not supported");
        }

        let (width, height) = (metrics.width as u16, metrics.height as u16);

        let sprite = self.atlas.new_unique_id();
        self.atlas.cache_sprite(
            sprite,
            Image {
                bytes: bitmap
                    .iter()
                    .flat_map(|coverage| vec![255, 255, 255, *coverage])
                    .collect(),
                width,
                height,
            },
        );
        let advance = metrics.advance_width;

        let (offset_x, offset_y) = (metrics.xmin, metrics.ymin);

        let character_info = CharacterInfo {
            advance,
            offset_x,
            offset_y,
            sprite,
        };

        self.characters.insert((character, size), character_info);
    }

    pub(crate) fn get(&self, character: char, size: u16) -> Option<CharacterInfo> {
        self.characters.get(&(character, size)).cloned()
    }
    /// Returns whether the character has been cached
    pub(crate) fn contains(&self, character: char, size: u16) -> bool {
        self.characters.contains_key(&(character, size))
    }

    pub(crate) fn measure_text(
        &mut self,
        text: &str,
        font_size: u16,
        font_scale_x: f32,
        font_scale_y: f32,
    ) -> TextDimensions {
        let dpi_scaling = miniquad::window::dpi_scale();
        let font_size = (font_size as f32 * dpi_scaling).ceil() as u16;

        let mut width = 0.0;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for character in text.chars() {
            // Well, yup, not all characters can be known
            if !self.contains(character, font_size) {
                self.cache_glyph(character, font_size);
            }

            let font_data = &self.characters[&(character, font_size)];
            let offset_y = font_data.offset_y as f32 * font_scale_y;

            // let atlas = &self.atlas;
            let glyph = self.atlas.get(font_data.sprite).unwrap().rect;
            width += font_data.advance * font_scale_x;
            min_y = min_y.min(offset_y);
            max_y = max_y.max(glyph.h * font_scale_y + offset_y);
        }

        TextDimensions {
            width: width / dpi_scaling,
            height: (max_y - min_y) / dpi_scaling,
            offset_y: max_y / dpi_scaling,
        }
    }
}

impl FontAtlas {
    /// List of ascii characters, may be helpful in combination with "populate_font_cache"
    pub fn ascii_character_list() -> Vec<char> {
        (0..255).filter_map(::std::char::from_u32).collect()
    }

    /// List of latin characters
    pub fn latin_character_list() -> Vec<char> {
        "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM1234567890!@#$%^&*(){}[].,:"
            .chars()
            .collect()
    }

    pub fn populate_font_cache(&mut self, characters: &[char], size: u16) {
        for character in characters {
            self.cache_glyph(*character, size);
        }
    }

    /// Sets the [FilterMode](https://docs.rs/miniquad/latest/miniquad/graphics/enum.FilterMode.html#) of this font's texture atlas.
    ///
    /// Use Nearest if you need integer-ratio scaling for pixel art, for example.
    ///
    /// # Example
    /// ```
    /// # use macroquad::prelude::*;
    /// # #[macroquad::main("test")]
    /// # async fn main() {
    /// let font = Font::default();
    /// font.set_filter(FilterMode::Linear);
    /// # }
    /// ```
    pub fn set_filter(
        &mut self,
        backend: &mut dyn RenderingBackend,
        filter_mode: miniquad::FilterMode,
    ) {
        self.atlas.set_filter(backend, filter_mode);
    }

    /// Get a reference to the inner atlas. 
    /// 
    /// This can be used to clean the GPU texture when it's no longer used
    pub fn atlas(&self) -> &TextureAtlas {
        &self.atlas
    }
}

/// Load font from file with "path"
pub fn load_ttf_font(
    backend: &mut dyn RenderingBackend,
    path: &str,
    filter: FilterMode,
) -> Result<FontAtlas, Error> {
    let bytes = crate::fs::load_file(path)
        .map_err(|_| Error::FontError("The Font file couldn't be loaded"))?;

    load_ttf_font_from_bytes(backend, &bytes[..], filter)
}

/// Load font from bytes array, may be use in combination with include_bytes!
/// ```ignore
/// let font = load_ttf_font_from_bytes(include_bytes!("font.ttf"));
/// ```
pub fn load_ttf_font_from_bytes(
    backend: &mut dyn RenderingBackend,
    bytes: &[u8],
    filter: FilterMode,
) -> Result<FontAtlas, Error> {
    let atlas = TextureAtlas::new(backend, filter);
    let mut font = FontAtlas::load_from_bytes(atlas, bytes)?;

    font.populate_font_cache(&FontAtlas::ascii_character_list(), 15);

    // Unneccessary
    // font.set_filter(filter);

    Ok(font)
}

pub fn measure_text(
    text: &str,
    font: &mut FontAtlas,
    font_size: u16,
    font_scale: f32,
) -> TextDimensions {
    font.measure_text(text, font_size, font_scale, font_scale)
}

/// From given font size in world space gives
/// (font_size, font_scale and font_aspect) params to make rasterized font
/// looks good in currently active camera
pub fn camera_font_scale(projection: Mat4, world_font_size: f32) -> (u16, f32, f32) {
    let (scr_w, scr_h) = miniquad::window::screen_size();

    let cam_space = projection.inverse().transform_vector3(vec3(2., 2., 0.));
    let (cam_w, cam_h) = (cam_space.x.abs(), cam_space.y.abs());

    let screen_font_size = world_font_size * scr_h / cam_h;

    let font_size = screen_font_size as u16;

    (font_size, cam_h / scr_h, scr_h / scr_w * cam_w / cam_h)
}
