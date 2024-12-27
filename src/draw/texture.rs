use glam::{vec2, Vec2};
use crate::{
    color::Color, graphics::{DrawMode, Renderer, Vertex}, texture::Texture, utils::Rect
};

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
    fn default() -> Self {
        Self {
            dest_size: None,
            source: None,
            rotation: 0.0,
            pivot: None,
            flip_x: false,
            flip_y: false,
        }
    }
}

pub fn draw_texture(renderer: &mut Renderer<Vertex>, texture: &Texture, x: f32, y: f32, color: Color) {
    draw_texture_ex(renderer, texture, x, y, color, Default::default());
}

pub fn draw_texture_ex(
    renderer: &mut Renderer<Vertex>,
    texture: &Texture,
    x: f32,
    y: f32,
    color: Color,
    params: DrawTextureParams,
) {
    let (width, height) = (texture.width() as f32, texture.height() as f32);
    let Rect {
        x: sx,
        y: sy,
        w: sw,
        h: sh,
    } = params.source.unwrap_or(Rect {
        x: 0.,
        y: 0.,
        w: width,
        h: height,
    });

    // ! Code relating to texture batching. I think it should be somewhat manual instead
    // let texture_opt = context
    //     .texture_batcher
    //     .get(texture)
    //     .map(|(batched_texture, uv)| {
    //         let [batched_width, batched_height] = batched_texture.size().to_array();
    //         sx = ((sx / width) * uv.w + uv.x) * batched_width;
    //         sy = ((sy / height) * uv.h + uv.y) * batched_height;
    //         sw = (sw / width) * uv.w * batched_width;
    //         sh = (sh / height) * uv.h * batched_height;

    //         width = batched_width;
    //         height = batched_height;

    //         batched_texture
    //     });
    // let texture = texture_opt.as_ref().unwrap_or(texture);

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

    renderer.with_texture(Some(texture.texture()));
    renderer.with_draw_mode(DrawMode::Triangles);
    renderer.push_geometry(&vertices, &indices);
}