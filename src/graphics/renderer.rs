//! Legacy module, code should be either removed or moved to different modules

use miniquad::*;

pub use miniquad::{TextureId as MiniquadTexture, UniformDesc};

use crate::{color::Color, logging::warn, tobytes::ToBytes, Error};

use std::{collections::BTreeMap, marker::PhantomData};

pub(crate) use super::{AsVertex, Vertex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawMode {
    Triangles,
    Lines,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlPipeline<V>
where V: AsVertex { 
    id: usize,
    _m: PhantomData<V>
}

impl<V> GlPipeline<V>
where V: AsVertex {
    const fn new(id: usize) -> Self {
        Self {
            id,
            _m: PhantomData
        }
    }
}

struct DrawCall<V>
where V: AsVertex {
    vertices_count: usize,
    indices_count: usize,
    vertices_start: usize,
    indices_start: usize,

    clip: Option<(i32, i32, i32, i32)>,
    viewport: Option<(i32, i32, i32, i32)>,
    texture: Option<miniquad::TextureId>,

    model: glam::Mat4,

    draw_mode: DrawMode,
    pipeline: GlPipeline<V>,
    uniforms: Option<Vec<u8>>,
    render_pass: Option<RenderPass>,
    capture: bool,
}

impl<V> DrawCall<V>
where V: AsVertex {
    const fn new(
        texture: Option<miniquad::TextureId>,
        model: glam::Mat4,
        draw_mode: DrawMode,
        pipeline: GlPipeline<V>,
        uniforms: Option<Vec<u8>>,
        render_pass: Option<RenderPass>,
    ) -> Self {
        Self {
            vertices_start: 0,
            indices_start: 0,
            vertices_count: 0,
            indices_count: 0,
            viewport: None,
            clip: None,
            texture,
            model,
            draw_mode,
            pipeline,
            uniforms,
            render_pass,
            capture: false,
        }
    }
}

struct RendererState<V>
where V: AsVertex {
    texture: Option<miniquad::TextureId>,
    draw_mode: DrawMode,
    clip: Option<(i32, i32, i32, i32)>,
    viewport: Option<(i32, i32, i32, i32)>,
    model_stack: Vec<glam::Mat4>,
    pipeline: Option<GlPipeline<V>>,
    depth_test_enable: bool,

    break_batching: bool,

    render_pass: Option<RenderPass>,
    capture: bool,
}

impl<V> RendererState<V>
where V: AsVertex {
    fn model(&self) -> glam::Mat4 {
        *self.model_stack.last().unwrap()
    }
}

#[derive(Clone, Debug)]
struct Uniform {
    name: String,
    uniform_type: UniformType,
    byte_offset: usize,
    byte_size: usize,
}

#[derive(Clone)]
struct PipelineExt<V>
where V: AsVertex {
    pipeline: miniquad::Pipeline,
    uniforms: Vec<Uniform>,
    uniforms_data: Vec<u8>,
    textures: Vec<String>,
    textures_data: BTreeMap<String, MiniquadTexture>,
    _m: PhantomData<V>
}

impl<V> PipelineExt<V>
where V: AsVertex {
    fn set_uniform<T>(&mut self, name: &str, uniform: T) {
        let uniform_meta = self.uniforms.iter().find(
            |Uniform {
                 name: uniform_name, ..
             }| uniform_name == name,
        );
        if uniform_meta.is_none() {
            warn!("Trying to set non-existing uniform: {}", name);
            return;
        }
        let uniform_meta = uniform_meta.unwrap();
        let uniform_format = uniform_meta.uniform_type;
        let uniform_byte_size = uniform_format.size();
        let uniform_byte_offset = uniform_meta.byte_offset;

        if std::mem::size_of::<T>() != uniform_byte_size {
            warn!(
                "Trying to set uniform {} sized {} bytes value of {} bytes",
                name,
                uniform_byte_size,
                std::mem::size_of::<T>()
            );
            return;
        }
        if uniform_byte_size != uniform_meta.byte_size {
            warn!("set_uniform do not support uniform arrays");
            return;
        }
        macro_rules! transmute_uniform {
            ($uniform_size:expr, $byte_offset:expr, $n:expr) => {
                if $uniform_size == $n {
                    let data: [u8; $n] = unsafe { std::mem::transmute_copy(&uniform) };

                    for i in 0..$uniform_size {
                        self.uniforms_data[$byte_offset + i] = data[i];
                    }
                }
            };
        }
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 4);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 8);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 12);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 16);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 64);
    }

    fn set_uniform_array<T: ToBytes>(&mut self, name: &str, uniform: &[T]) {
        let uniform_meta = self.uniforms.iter().find(
            |Uniform {
                 name: uniform_name, ..
             }| uniform_name == name,
        );
        if uniform_meta.is_none() {
            warn!("Trying to set non-existing uniform: {}", name);
            return;
        }
        let uniform_meta = uniform_meta.unwrap();
        let uniform_byte_size = uniform_meta.byte_size;
        let uniform_byte_offset = uniform_meta.byte_offset;

        let data = unsafe { uniform.to_bytes() };
        if data.len() != uniform_byte_size {
            warn!(
                "Trying to set uniform {} sized {} bytes value of {} bytes",
                name,
                uniform_byte_size,
                std::mem::size_of::<T>()
            );
            return;
        }

        // for i in 0..uniform_byte_size {
        //     self.uniforms_data[uniform_byte_offset + i] = data[i];
        // }

        // That's from clippy. I'm leaving the original as well
        self.uniforms_data[uniform_byte_offset..(uniform_byte_size + uniform_byte_offset)]
            .copy_from_slice(&data[..uniform_byte_size]);
    }
}

const MAX_PIPELINES: usize = 32;

struct PipelineStorage<V>
where V: AsVertex {
    pipelines: [Option<PipelineExt<V>>; MAX_PIPELINES],
    pipelines_amount: usize,
}

impl<V> PipelineStorage<V>
where V: AsVertex {
    const TRIANGLES_PIPELINE: GlPipeline<V> = GlPipeline::new(0);
    const LINES_PIPELINE: GlPipeline<V> = GlPipeline::new(1);
    const TRIANGLES_DEPTH_PIPELINE: GlPipeline<V> = GlPipeline::new(2);
    const LINES_DEPTH_PIPELINE: GlPipeline<V> = GlPipeline::new(3);

    fn new(ctx: &mut dyn RenderingBackend) -> Self {
        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader::meta(),
            )
            .unwrap_or_else(|e| panic!("Failed to load shader: {}", e));

        let params = PipelineParams {
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            ..Default::default()
        };

        let mut storage = Self {
            pipelines: Default::default(),
            pipelines_amount: 0,
        };

        let triangles_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                ..params
            },
            vec![],
            vec![],
        );
        assert_eq!(triangles_pipeline, Self::TRIANGLES_PIPELINE);

        let lines_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Lines,
                ..params
            },
            vec![],
            vec![],
        );
        assert_eq!(lines_pipeline, Self::LINES_PIPELINE);

        let triangles_depth_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                primitive_type: PrimitiveType::Triangles,
                ..params
            },
            vec![],
            vec![],
        );
        assert_eq!(triangles_depth_pipeline, Self::TRIANGLES_DEPTH_PIPELINE);

        let lines_depth_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                primitive_type: PrimitiveType::Lines,
                ..params
            },
            vec![],
            vec![],
        );
        assert_eq!(lines_depth_pipeline, Self::LINES_DEPTH_PIPELINE);

        storage
    }

    fn make_pipeline(
        &mut self,
        backend: &mut dyn RenderingBackend,
        shader: ShaderId,
        params: PipelineParams,
        mut uniforms: Vec<UniformDesc>,
        textures: Vec<String>,
    ) -> GlPipeline<V> {
        let pipeline = backend.new_pipeline(
            &[BufferLayout::default()],
            &(V::attributes()),
            shader,
            params,
        );

        let id = self
            .pipelines
            .iter()
            .position(|p| p.is_none())
            .unwrap_or_else(|| panic!("Pipelines amount exceeded"));

        let mut max_offset = 0;

        for (name, kind) in shader::uniforms().into_iter().rev() {
            uniforms.insert(0, UniformDesc::new(name, kind));
        }

        let uniforms = uniforms
            .iter()
            .scan(0, |offset, uniform| {
                let byte_size = uniform.uniform_type.size() * uniform.array_count;
                let uniform = Uniform {
                    name: uniform.name.clone(),
                    uniform_type: uniform.uniform_type,
                    byte_size,
                    byte_offset: *offset,
                };
                *offset += byte_size;
                max_offset = *offset;

                Some(uniform)
            })
            .collect();

        self.pipelines[id] = Some(PipelineExt {
            pipeline,
            uniforms,
            uniforms_data: vec![0; max_offset],
            textures,
            textures_data: BTreeMap::new(),
            _m: PhantomData
        });
        self.pipelines_amount += 1;

        GlPipeline::new(id)
    }

    /// Get the default pipeline by draw mode and depth flag
    const fn get_default_pipeline_by(
        &self,
        draw_mode: DrawMode,
        depth_enabled: bool,
    ) -> GlPipeline<V> {
        match (draw_mode, depth_enabled) {
            (DrawMode::Triangles, false) => Self::TRIANGLES_PIPELINE,
            (DrawMode::Triangles, true) => Self::TRIANGLES_DEPTH_PIPELINE,
            (DrawMode::Lines, false) => Self::LINES_PIPELINE,
            (DrawMode::Lines, true) => Self::LINES_DEPTH_PIPELINE,
        }
    }

    /// Find a pipeline by pipeline ID ([GlPipeline])
    fn get_pipeline_mut(&mut self, pip: &GlPipeline<V>) -> Option<&mut PipelineExt<V>> {
        (self.pipelines[pip.id]).as_mut()
    }

    /// Check whether this storage has the specified pipeline
    fn has_pipeline(&self, pip: &GlPipeline<V>) -> bool {
        self.pipelines[pip.id].is_some()
    }

    fn delete_pipeline(&mut self, pip: GlPipeline<V>) {
        self.pipelines[pip.id] = None;
    }
}

/// This structure does a lot:
/// 1. It batches draw calls (i.e. unifies similar drawcalls or smaller ones into larger ones)
/// 2. It performs draw calls on the supplied rendering context
/// 3. It creates pipelines
pub struct Renderer<V = Vertex>
where
    V: AsVertex,
{
    pipelines: PipelineStorage<V>,

    draw_calls: Vec<DrawCall<V>>,
    draw_calls_bindings: Vec<Bindings>,
    draw_calls_count: usize,
    state: RendererState<V>,
    start_time: f64,

    pub(crate) white_texture: miniquad::TextureId,
    max_vertices: usize,
    max_indices: usize,

    batch_vertex_buffer: Vec<V>,
    batch_index_buffer: Vec<u16>,
}

impl<V> Renderer<V>
where V: AsVertex {
    pub fn new(
        ctx: &mut dyn miniquad::RenderingBackend,
        max_vertices: usize,
        max_indices: usize,
    ) -> Self {
        let white_texture = ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);

        Self {
            pipelines: PipelineStorage::new(ctx),
            state: RendererState {
                clip: None,
                viewport: None,
                texture: None,
                model_stack: vec![glam::Mat4::IDENTITY],
                draw_mode: DrawMode::Triangles,
                pipeline: None,
                break_batching: false,
                depth_test_enable: false,
                render_pass: None,
                capture: false,
            },
            draw_calls: Vec::with_capacity(200),
            draw_calls_bindings: Vec::with_capacity(200),
            draw_calls_count: 0,
            start_time: miniquad::date::now(),

            white_texture,
            batch_vertex_buffer: Vec::with_capacity(max_vertices),
            batch_index_buffer: Vec::with_capacity(max_indices),
            max_vertices,
            max_indices,
        }
    }

    pub fn make_pipeline(
        &mut self,
        ctx: &mut dyn miniquad::RenderingBackend,
        shader: miniquad::ShaderSource,
        params: PipelineParams,
        uniforms: Vec<UniformDesc>,
        textures: Vec<String>,
    ) -> Result<GlPipeline<V>, Error> {
        let mut shader_meta: ShaderMeta = shader::meta();

        for uniform in &uniforms {
            shader_meta.uniforms.uniforms.push(uniform.clone());
        }

        for texture in &textures {
            if texture == "Texture" {
                panic!(
                    "you can't use name `Texture` for your texture. This name is reserved for the texture that will be drawn with that material"
                );
            }
            shader_meta.images.push(texture.clone());
        }

        // let source = match shader {
        //     ShaderSource::Glsl { fragment, .. } => fragment,
        //     ShaderSource::Msl { program } => program,
        // };

        let shader = ctx.new_shader(shader, shader_meta)?;

        Ok(self
            .pipelines
            .make_pipeline(ctx, shader, params, uniforms, textures))
    }

    /// Clear the framebuffer with a specified color, then clear the draw calls
    pub fn clear(&mut self, ctx: &mut dyn miniquad::RenderingBackend, color: Color) {
        let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

        if let Some(current_pass) = self.state.render_pass {
            ctx.begin_pass(Some(current_pass), clear);
        } else {
            ctx.begin_default_pass(clear);
        }
        ctx.end_render_pass();

        self.clear_draw_calls();
    }

    /// Reset only draw calls state
    pub fn clear_draw_calls(&mut self) {
        self.draw_calls_count = 0;
    }

    /// Reset internal state to known default
    pub fn reset(&mut self) {
        self.state.clip = None;
        self.state.texture = None;
        self.state.model_stack = vec![glam::Mat4::IDENTITY];
        self.draw_calls_count = 0;
    }

    pub fn draw(&mut self, ctx: &mut dyn miniquad::RenderingBackend, projection: glam::Mat4) {
        let white_texture = self.white_texture;

        for _ in 0..self.draw_calls.len() - self.draw_calls_bindings.len() {
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<V>(self.max_vertices),
            );
            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<u16>(self.max_indices),
            );
            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![white_texture, white_texture],
            };

            self.draw_calls_bindings.push(bindings);
        }
        assert_eq!(self.draw_calls_bindings.len(), self.draw_calls.len());

        let (screen_width, screen_height) = miniquad::window::screen_size();
        let time = (miniquad::date::now() - self.start_time) as f32;
        let time = glam::vec4(time, time.sin(), time.cos(), 0.);

        for (dc, bindings) in self.draw_calls[0..self.draw_calls_count]
            .iter_mut()
            .zip(self.draw_calls_bindings.iter_mut())
        {
            // ! We unwrap here, since a draw call can't possibly be added with a pipeline that isn't available
            // ! in the storage
            let pipeline = self.pipelines.get_pipeline_mut(&dc.pipeline).unwrap();

            let (width, height) = if let Some(render_pass) = dc.render_pass {
                let render_texture = ctx.render_pass_texture(render_pass);
                let (width, height) = ctx.texture_size(render_texture);
                (width, height)
            } else {
                (screen_width as u32, screen_height as u32)
            };

            if let Some(render_pass) = dc.render_pass {
                ctx.begin_pass(Some(render_pass), PassAction::Nothing);
            } else {
                ctx.begin_default_pass(PassAction::Nothing);
            }

            ctx.buffer_update(
                bindings.vertex_buffers[0],
                BufferSource::slice(
                    &self.batch_vertex_buffer
                        [dc.vertices_start..(dc.vertices_start + dc.vertices_count)],
                ),
            );
            ctx.buffer_update(
                bindings.index_buffer,
                BufferSource::slice(
                    &self.batch_index_buffer
                        [dc.indices_start..(dc.indices_start + dc.indices_count)],
                ),
            );

            bindings.images[0] = dc.texture.unwrap_or(white_texture);
            bindings
                .images
                .resize(1 + pipeline.textures.len(), white_texture);

            for (pos, name) in pipeline.textures.iter().enumerate() {
                if let Some(texture) = pipeline.textures_data.get(name).copied() {
                    bindings.images[2 + pos] = texture;
                }
            }

            ctx.apply_pipeline(&pipeline.pipeline);
            if let Some((x, y, w, h)) = dc.viewport {
                ctx.apply_viewport(x, y, w, h);
            } else {
                ctx.apply_viewport(0, 0, width as i32, height as i32);
            }
            if let Some(clip) = dc.clip {
                ctx.apply_scissor_rect(clip.0, height as i32 - (clip.1 + clip.3), clip.2, clip.3);
            } else {
                ctx.apply_scissor_rect(0, 0, width as i32, height as i32);
            }
            ctx.apply_bindings(bindings);

            if let Some(ref uniforms) = dc.uniforms {
                // for i in 0..uniforms.len() {
                //     pipeline.uniforms_data[i] = uniforms[i];
                // }
                pipeline.uniforms_data[..uniforms.len()].copy_from_slice(&uniforms[..]);
            }
            pipeline.set_uniform("Projection", projection);
            pipeline.set_uniform("Model", dc.model);
            pipeline.set_uniform("_Time", time);
            ctx.apply_uniforms_from_bytes(
                pipeline.uniforms_data.as_ptr(),
                pipeline.uniforms_data.len(),
            );
            ctx.draw(0, dc.indices_count as i32, 1);
            ctx.end_render_pass();

            // TODO: Telemetry
            // if dc.capture {
            //     telemetry::track_drawcall(&pipeline.pipeline, bindings, dc.indices_count);
            // }

            dc.vertices_count = 0;
            dc.indices_count = 0;
            dc.vertices_start = 0;
            dc.indices_start = 0;
        }

        self.draw_calls_count = 0;
        self.batch_index_buffer.clear();
        self.batch_vertex_buffer.clear();
    }

    pub(crate) fn with_capture(&mut self, capture: bool) {
        self.state.capture = capture;
    }

    pub const fn get_active_render_pass(&self) -> Option<RenderPass> {
        self.state.render_pass
    }

    pub const fn is_depth_test_enabled(&self) -> bool {
        self.state.depth_test_enable
    }

    pub fn with_render_pass(&mut self, render_pass: Option<RenderPass>) {
        self.state.render_pass = render_pass;
    }

    pub fn with_depth_test(&mut self, enable: bool) {
        self.state.depth_test_enable = enable;
    }

    pub fn with_texture(&mut self, texture: Option<&TextureId>) {
        // If you ask me why... Idk
        self.state.texture = texture.copied();
    }

    pub fn with_scissor(&mut self, clip: Option<(i32, i32, i32, i32)>) {
        self.state.clip = clip;
    }

    pub fn with_viewport(&mut self, viewport: Option<(i32, i32, i32, i32)>) {
        self.state.viewport = viewport;
    }

    pub fn get_viewport(&self) -> (i32, i32, i32, i32) {
        self.state.viewport.unwrap_or((
            0,
            0,
            crate::window::screen_width() as _,
            crate::window::screen_height() as _,
        ))
    }

    pub fn push_model_matrix(&mut self, matrix: glam::Mat4) {
        self.state.model_stack.push(self.state.model() * matrix);
    }

    pub fn pop_model_matrix(&mut self) -> Option<glam::Mat4> {
        if self.state.model_stack.len() > 1 {
            self.state.model_stack.pop()
        } else {
            None
        }
    }

    pub fn with_pipeline(&mut self, pipeline: Option<GlPipeline<V>>) {
        if self.state.pipeline == pipeline {
            return;
        }

        if let Some(ref pip) = pipeline {
            assert!(
                self.has_pipeline(pip),
                "The provided pipeline isn't present in the renderer"
            )
        }

        self.state.break_batching = true;
        self.state.pipeline = pipeline;
    }

    pub fn with_draw_mode(&mut self, mode: DrawMode) {
        self.state.draw_mode = mode;
    }

    /// TODO: Document this
    pub fn push_geometry(&mut self, vertices: &[V], indices: &[u16]) {
        if vertices.len() >= self.max_vertices || indices.len() >= self.max_indices {
            warn!("geometry() exceeded max drawcall size, clamping");
        }

        let vertices = &vertices[0..self.max_vertices.min(vertices.len())];
        let indices = &indices[0..self.max_indices.min(indices.len())];

        let pip = self.state.pipeline.unwrap_or(
            self.pipelines
                .get_default_pipeline_by(self.state.draw_mode, self.state.depth_test_enable),
        );

        let previous_dc_ix = if self.draw_calls_count == 0 {
            None
        } else {
            Some(self.draw_calls_count - 1)
        };
        let previous_dc = previous_dc_ix.and_then(|ix| self.draw_calls.get(ix));

        if previous_dc.map_or(true, |draw_call| {
            draw_call.texture != self.state.texture
                || draw_call.clip != self.state.clip
                || draw_call.viewport != self.state.viewport
                || draw_call.model != self.state.model()
                || draw_call.pipeline != pip
                || draw_call.render_pass != self.state.render_pass
                || draw_call.draw_mode != self.state.draw_mode
                || draw_call.vertices_count >= self.max_vertices - vertices.len()
                || draw_call.indices_count >= self.max_indices - indices.len()
                || draw_call.capture != self.state.capture
                || self.state.break_batching
        }) {
            let uniforms = self.state.pipeline.map(|pipeline| {
                self.pipelines
                    .get_pipeline_mut(&pipeline)
                    .unwrap()
                    .uniforms_data
                    .clone()
            });

            if self.draw_calls_count >= self.draw_calls.len() {
                self.draw_calls.push(DrawCall::new(
                    self.state.texture,
                    self.state.model(),
                    self.state.draw_mode,
                    pip,
                    uniforms.clone(),
                    self.state.render_pass,
                ));
            }

            self.draw_calls[self.draw_calls_count].texture = self.state.texture;
            self.draw_calls[self.draw_calls_count].uniforms = uniforms;
            self.draw_calls[self.draw_calls_count].vertices_count = 0;
            self.draw_calls[self.draw_calls_count].indices_count = 0;
            self.draw_calls[self.draw_calls_count].clip = self.state.clip;
            self.draw_calls[self.draw_calls_count].viewport = self.state.viewport;
            self.draw_calls[self.draw_calls_count].model = self.state.model();
            self.draw_calls[self.draw_calls_count].pipeline = pip;
            self.draw_calls[self.draw_calls_count].render_pass = self.state.render_pass;
            self.draw_calls[self.draw_calls_count].capture = self.state.capture;
            self.draw_calls[self.draw_calls_count].indices_start = self.batch_index_buffer.len();
            self.draw_calls[self.draw_calls_count].vertices_start = self.batch_vertex_buffer.len();

            self.draw_calls_count += 1;
            self.state.break_batching = false;
        };

        let dc = &mut self.draw_calls[self.draw_calls_count - 1];

        self.batch_vertex_buffer.extend(vertices);
        self.batch_index_buffer
            .extend(indices.iter().map(|x| *x + dc.vertices_count as u16));

        dc.vertices_count += vertices.len();
        dc.indices_count += indices.len();

        dc.texture = self.state.texture;
    }

    pub fn delete_pipeline(&mut self, pipeline: GlPipeline<V>) {
        self.pipelines.delete_pipeline(pipeline);
    }

    /// Check whether this renderer has this specific pipeline
    pub fn has_pipeline(&self, pipeline: &GlPipeline<V>) -> bool {
        self.pipelines.has_pipeline(pipeline)
    }

    pub fn set_uniform<T>(&mut self, pipeline: &GlPipeline<V>, name: &str, uniform: T) {
        self.state.break_batching = true;

        self.pipelines
            .get_pipeline_mut(pipeline)
            .expect("The provided pipeline has to be present in the renderer")
            .set_uniform(name, uniform);
    }

    pub fn set_uniform_array<T: ToBytes>(
        &mut self,
        pipeline: &GlPipeline<V>,
        name: &str,
        uniform: &[T],
    ) {
        self.state.break_batching = true;

        self.pipelines
            .get_pipeline_mut(pipeline)
            .expect("The provided pipeline has to be present in the renderer")
            .set_uniform_array(name, uniform);
    }

    /// Set a texture under specified name for the provided pipeline
    pub fn set_texture(&mut self, pipeline: &GlPipeline<V>, name: &str, texture: TextureId) {
        let pipeline = self
            .pipelines
            .get_pipeline_mut(pipeline)
            .expect("The provided pipeline has to be present in the renderer");

        pipeline
            .textures
            .iter()
            .find(|x| *x == name)
            .unwrap_or_else(|| {
                panic!(
                    "can't find texture with name '{}', there are only these names: {:?}",
                    name, pipeline.textures
                )
            });
        *pipeline
            .textures_data
            .entry(name.to_owned())
            .or_insert(texture) = texture;
    }

    /// Update the maximum amount of vertices and indices this renderer can accept per draw call.
    ///
    /// ### Attention
    /// This is an expensive operation, as it also changes every existing draw call in the renderer,
    /// removes all drawcalls, and then finally recreates all vertex/index buffers bindings.
    pub fn update_drawcall_capacity(
        &mut self,
        backend: &mut dyn miniquad::RenderingBackend,
        max_vertices: usize,
        max_indices: usize,
    ) {
        self.max_vertices = max_vertices;
        self.max_indices = max_indices;
        self.draw_calls_count = 0;

        for draw_call in &mut self.draw_calls {
            draw_call.indices_start = 0;
            draw_call.vertices_start = 0;
        }

        for binding in &mut self.draw_calls_bindings {
            backend.delete_buffer(binding.index_buffer);

            for vertex_buffer in &binding.vertex_buffers {
                backend.delete_buffer(*vertex_buffer);
            }

            let vertex_buffer = backend.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<V>(self.max_vertices),
            );

            let index_buffer = backend.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<u16>(self.max_indices),
            );

            *binding = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![self.white_texture],
            };
        }
    }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    attribute vec4 normal;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D Texture;

    void main() {
        gl_FragColor = color * texture2D(Texture, uv) ;
    }"#;

    pub const METAL: &str = r#"
#include <metal_stdlib>
    using namespace metal;

    struct Uniforms
    {
        float4x4 Model;
        float4x4 Projection;
    };

    struct Vertex
    {
        float3 position    [[attribute(0)]];
        float2 texcoord    [[attribute(1)]];
        float4 color0      [[attribute(2)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
        float2 uv [[user(locn1)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = uniforms.Model * uniforms.Projection * float4(v.position, 1);
        out.color = v.color0 / 255.0;
        out.uv = v.texcoord;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return in.color * tex.sample(texSmplr, in.uv);
    }
    "#;
    pub fn uniforms() -> Vec<(&'static str, UniformType)> {
        vec![
            ("Projection", UniformType::Mat4),
            ("Model", UniformType::Mat4),
            ("_Time", UniformType::Float4),
        ]
    }

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: uniforms()
                    .into_iter()
                    .map(|(name, kind)| UniformDesc::new(name, kind))
                    .collect(),
            },
        }
    }
}
