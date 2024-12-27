//! Everything graphics related

use glam::{vec3, vec4, Vec2, Vec3, Vec4};
use miniquad::{VertexAttribute, VertexFormat};
use crate::color::Color;

mod renderer;
pub use renderer::*;

pub mod geometry;

pub mod camera;
pub use camera::{Camera, Camera2D, Camera3D};

pub mod material;
pub use material::{Material, MaterialParams, use_default_material, use_material};

/// A vertex trait that you can implement on any type you want to turn into a Vertex.
/// 
/// Attention: the desired type should be [`repr(C)`], and all its fields have to implement [`ToBytes`] 
pub unsafe trait AsVertex
where Self: Clone + Copy {
    /// Get [`VertexAttribute`]s of this Vertex. This is required when constructing pipelines
    fn attributes() -> Vec<VertexAttribute>;
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub color: [u8; 4],
    /// Normal is not used by macroquad and is completely optional.
    /// Might be usefull for custom shaders.
    /// While normal is not used by macroquad, it is completely safe to use it
    /// to pass arbitary user data, hence Vec4.
    pub normal: Vec4,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32, color: Color) -> Vertex {
        Vertex {
            position: vec3(x, y, z),
            uv: vec2(u, v),
            color: color.into(),
            normal: vec4(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn new2(position: Vec3, uv: Vec2, color: Color) -> Vertex {
        Vertex {
            position,
            uv,
            color: color.into(),
            normal: vec4(0.0, 0.0, 0.0, 0.0),
        }
    }
}

unsafe impl AsVertex for Vertex {
    fn attributes() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::new("position", VertexFormat::Float3),
            VertexAttribute::new("texcoord", VertexFormat::Float2),
            VertexAttribute::new("color0", VertexFormat::Byte4),
            VertexAttribute::new("normal", VertexFormat::Float4),
        ]
    }
}

pub struct Mesh<V>
where V: AsVertex {
    pub vertices: Vec<V>,
    pub indices: Vec<u16>,
    pub texture: Option<miniquad::TextureId>,
}