use glam::{vec2, Vec2};
use miniquad::RenderingBackend;

use crate::{color::{Color, WHITE}, graphics::{Renderer, Vertex}, text::{FontAtlas, TextDimensions}, utils::Rect};


/// Arguments for "draw_text_ex" function such as font, font_size etc
#[derive(Debug, Clone)]
pub struct TextParams<'a> {
    pub font: Option<&'a FontAtlas>,
    /// Base size for character height. The size in pixel used during font rasterizing.
    pub font_size: u16,
    /// The glyphs sizes actually drawn on the screen will be font_size * font_scale
    /// However with font_scale too different from 1.0 letters may be blurry
    pub font_scale: f32,
    /// Font X axis would be scaled by font_scale * font_scale_aspect
    /// and Y axis would be scaled by font_scale
    /// Default is 1.0
    pub font_scale_aspect: f32,
    /// Text rotation in radian
    /// Default is 0.0
    pub rotation: f32,
    pub color: Color,
}

impl<'a> Default for TextParams<'a> {
    fn default() -> TextParams<'a> {
        TextParams {
            font: None,
            font_size: 20,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            color: WHITE,
            rotation: 0.0,
        }
    }
}

/// Draw text with given font_size
/// Returns text size
pub fn draw_text(
    backend: &mut dyn RenderingBackend,
    renderer: &mut Renderer<Vertex>,
    font: &mut FontAtlas,
    text: &str, 
    x: f32, 
    y: f32, 
    font_size: f32, 
    color: Color
) -> TextDimensions {
    draw_text_ex(
        backend,
        renderer,
        font,
        text,
        x,
        y,
        TextParams {
            font_size: font_size as u16,
            font_scale: 1.0,
            color,
            ..Default::default()
        },
    )
}

/// Draw text with custom params such as font, font size and font scale
/// Returns text size
pub fn draw_text_ex(
    backend: &mut dyn RenderingBackend,
    renderer: &mut Renderer,
    font: &mut FontAtlas,
    text: &str, 
    x: f32, 
    y: f32, 
    params: TextParams
) -> TextDimensions {
    if text.is_empty() {
        return TextDimensions::default();
    }

    let dpi_scaling = miniquad::window::dpi_scale();

    let rot = params.rotation;
    let font_scale_x = params.font_scale * params.font_scale_aspect;
    let font_scale_y = params.font_scale;
    let font_size = (params.font_size as f32 * dpi_scaling).ceil() as u16;

    let mut total_width = 0.0;
    let mut max_offset_y = f32::MIN;
    let mut min_offset_y = f32::MAX;

    for character in text.chars() {
        if !font.contains(character, font_size) {
            font.cache_glyph(character, font_size);
        }

        let char_data = &font.characters[&(character, font_size)];
        let offset_x = char_data.offset_x as f32 * font_scale_x;
        let offset_y = char_data.offset_y as f32 * font_scale_y;

        let atlas = &mut font.atlas;
        let glyph = atlas.get(char_data.sprite).unwrap().rect;
        let glyph_scaled_h = glyph.h * font_scale_y;

        min_offset_y = min_offset_y.min(offset_y);
        max_offset_y = max_offset_y.max(glyph_scaled_h + offset_y);

        let rot_cos = rot.cos();
        let rot_sin = rot.sin();
        let dest_x = (offset_x + total_width) * rot_cos + (glyph_scaled_h + offset_y) * rot_sin;
        let dest_y = (offset_x + total_width) * rot_sin + (-glyph_scaled_h - offset_y) * rot_cos;

        let dest = Rect::new(
            dest_x / dpi_scaling + x,
            dest_y / dpi_scaling + y,
            glyph.w / dpi_scaling * font_scale_x,
            glyph.h / dpi_scaling * font_scale_y,
        );

        total_width += char_data.advance * font_scale_x;

        super::draw_texture_ex(
            renderer,
            atlas.texture(backend),
            dest.x,
            dest.y,
            params.color,
            crate::draw::DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(glyph),
                rotation: rot,
                pivot: Some(vec2(dest.x, dest.y)),
                ..Default::default()
            },
        );
    }

    TextDimensions {
        width: total_width / dpi_scaling,
        height: (max_offset_y - min_offset_y) / dpi_scaling,
        offset_y: max_offset_y / dpi_scaling,
    }
}

/// Draw multiline text with the given font_size, line_distance_factor and color.
/// If no line distance but a custom font is given, the fonts line gap will be used as line distance factor if it exists.
pub fn draw_multiline_text(
    backend: &mut dyn RenderingBackend,
    renderer: &mut Renderer<Vertex>,
    font: &mut FontAtlas,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    line_distance_factor: Option<f32>,
    color: Color,
) {
    draw_multiline_text_ex(
        backend,
        renderer,
        font,
        text,
        x,
        y,
        line_distance_factor,
        TextParams {
            font_size: font_size as u16,
            font_scale: 1.0,
            color,
            ..Default::default()
        },
    );
}

/// Draw multiline text with the given line distance and custom params such as font, font size and font scale.
/// If no line distance but a custom font is given, the fonts newline size will be used as line distance factor if it exists.
pub fn draw_multiline_text_ex(
    backend: &mut dyn RenderingBackend,
    renderer: &mut Renderer<Vertex>,
    font: &mut FontAtlas,
    text: &str,
    x: f32,
    mut y: f32,
    line_distance_factor: Option<f32>,
    params: TextParams,
) {
    let line_distance = match line_distance_factor {
        Some(distance) => distance,
        None => {
            let mut font_line_distance = 0.0;
            if let Some(font) = params.font {
                if let Some(metrics) = font.font.horizontal_line_metrics(1.0) {
                    font_line_distance = metrics.new_line_size;
                }
            }
            font_line_distance
        }
    };

    for line in text.lines() {
        draw_text_ex(
            backend, 
            renderer,
            font,
            line,
            x, 
            y,
            params.clone()
        );
        y += line_distance * params.font_size as f32 * params.font_scale;
    }
}

/// Get the text center.
pub fn get_text_center(
    text: &str,
    font: &mut FontAtlas,
    font_size: u16,
    font_scale: f32,
    rotation: f32,
) -> Vec2 {
    let measure = crate::text::measure_text(text, font, font_size, font_scale);

    let x_center = measure.width / 2.0 * rotation.cos() + measure.height / 2.0 * rotation.sin();
    let y_center = measure.width / 2.0 * rotation.sin() - measure.height / 2.0 * rotation.cos();

    Vec2::new(x_center, y_center)
}