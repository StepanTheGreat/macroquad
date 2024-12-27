//! Everything graphics related

mod renderer;
pub use renderer::*;

pub mod geometry;

pub mod camera;
pub use camera::{Camera, Camera2D, Camera3D};

pub mod material;
pub use material::{Material, MaterialParams, use_default_material, use_material};