use miniquad::{RenderingBackend, TextureId};

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
        let color_resolve_texture = backend.new_render_texture(miniquad::TextureParams {
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
        render_pass,
    };

    RenderTarget {
        texture,
        render_pass,
    }
}