//! Most common types that can be glob-imported `use macroquad::prelude::*` for convenience.

pub use crate::input::*;
pub use crate::text::*;
pub use crate::texture::*;
pub use crate::window::*;

pub use crate::color::{colors::*, Color};
pub use crate::graphics::{DrawMode, GlPipeline, Renderer};
pub use glam;
pub use miniquad::{
    conf::Conf, Comparison, PipelineParams, ShaderError, ShaderSource, UniformDesc, UniformType,
};
pub use quad_rand as rand;

pub use crate::logging::*;

pub use crate::color_u8;

pub use image::ImageFormat;
