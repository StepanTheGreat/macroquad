//! Custom materials - shaders, uniforms.

use crate::{graphics::GlPipeline, tobytes::ToBytes, Error};
use miniquad::{PipelineParams, RenderingBackend, TextureId, UniformDesc};

use super::Renderer;

/// A material with custom shaders and uniforms, textures and pipeline params
/// 
/// ### Warning
/// This is essentially an abstraction of [miniquad::Pipeline] for a specific renderer.
/// 2 things can go wrong however:
/// 1. This material provides will not clear itself after, so this memory management is after you
/// (A renderer only has 32 possible pipeline slots, not that many)
/// 2. A material is inherently bound to a specific renderer from which you created it. That means that if you
/// try to use a material on a renderer that doesn't have it - it will panic.  
#[derive(Clone, PartialEq)]
pub struct Material {
    pipeline: GlPipeline,
}

impl std::fmt::Debug for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Material").finish()
    }
}

impl Material {
    fn from_pipeline(pipeline: GlPipeline) -> Self {
        Self {
            pipeline
        }
    }

    /// Set GPU uniform value for this material.
    /// "name" should be from "uniforms" list used for material creation.
    /// Otherwise uniform value would be silently ignored.
    pub fn set_uniform<T>(&self, renderer: &mut Renderer, name: &str, uniform: T) {
        renderer.set_uniform(&self.pipeline, name, uniform);
    }

    pub fn set_uniform_array<T: ToBytes>(&self, renderer: &mut Renderer, name: &str, uniform: &[T]) {
        renderer.set_uniform_array(&self.pipeline, name, uniform);
    }

    pub fn set_texture(&self, renderer: &mut Renderer, name: &str, texture: &TextureId) {
        renderer.set_texture(&self.pipeline, name, *texture);
    }
}

/// Params used for material loading.
/// It is not possible to change material params at runtime, so this
/// struct is used only once - at "load_material".
#[derive(Default)]
pub struct MaterialParams {
    /// miniquad pipeline configuration for this material.
    /// Things like blending, culling, depth dest
    pub pipeline_params: PipelineParams,

    /// List of custom uniforms used in this material
    pub uniforms: Vec<UniformDesc>,

    /// List of textures used in this material
    pub textures: Vec<String>,
}

/// Create a new material on the specified renderer, with specified shader source and material params
/// 
/// ### Warning
/// 1. Materials are essentially pipelines, with no cleanup guarantees. Its your responsibility
/// to properly clean it after use.
/// 2. Given materials can only be used with renderers with which you created said materials.
/// Since pipelines (from said materials) are renderer bound, using them on another renderer will cause a panic
pub fn load_material(
    backend: &mut dyn RenderingBackend,
    renderer: &mut Renderer,
    shader: miniquad::ShaderSource,
    params: MaterialParams,
) -> Result<Material, Error> {

    let pipeline = renderer.make_pipeline(
        backend,
        shader,
        params.pipeline_params,
        params.uniforms,
        params.textures,
    )?;

    Ok(Material::from_pipeline(pipeline))
}

/// All following macroquad rendering calls will use the given material.
/// 
/// ### Attention
/// This function will panic if you use it on a renderer that doesn't have said material.
/// To check: use [has_material]
pub fn use_material(renderer: &mut Renderer, material: &Material) {
    renderer.with_pipeline(Some(material.pipeline));
}

/// Check whether the provided renderer contains the specified material.
/// 
/// It's highly important to only use materials given by the same renderers, to avoid
/// panics
pub fn has_material(renderer: &mut Renderer, material: &Material) -> bool {
    renderer.has_pipeline(&material.pipeline)
}

/// Use default renderer material.
/// 
/// This is essentially:
/// ```
/// renderer.with_pipeline(None);
/// ```
pub fn use_default_material(renderer: &mut Renderer) {
    renderer.with_pipeline(None);
}


// I'm leaving this for now, could be a separate crate feature in the future.
mod preprocessor {
    type IncludeFilename = String;
    type IncludeContent = String;

    #[derive(Debug, Clone)]
    pub struct PreprocessorConfig {
        pub includes: Vec<(IncludeFilename, IncludeContent)>,
    }
    impl Default for PreprocessorConfig {
        fn default() -> PreprocessorConfig {
            PreprocessorConfig { includes: vec![] }
        }
    }

    impl PreprocessorConfig {}

    pub fn preprocess_shader(source: &str, config: &PreprocessorConfig) -> String {
        let mut res = source.chars().collect::<Vec<_>>();

        fn find(data: &[char], n: &mut usize, target: &str) -> bool {
            if *n >= data.len() {
                return false;
            }
            let target = target.chars().collect::<Vec<_>>();

            'outer: for i in *n..data.len() {
                for j in 0..target.len() {
                    if data[i + j] != target[j] {
                        *n += 1;
                        continue 'outer;
                    }
                }
                return true;
            }
            false
        }

        fn skip_character(data: &[char], n: &mut usize, target: char) {
            while *n < data.len() && data[*n] == target {
                *n += 1;
            }
        }

        let mut i = 0;
        while find(&res, &mut i, "#include") {
            let directive_start_ix = i;
            i += "#include".len();
            skip_character(&res, &mut i, ' ');
            assert!(res[i] == '\"');
            i += 1;
            let filename_start_ix = i;
            find(&res, &mut i, "\"");
            let filename_end_ix = i;
            let filename = res[filename_start_ix..filename_end_ix]
                .iter()
                .cloned()
                .collect::<String>();

            let include_content = config
                .includes
                .iter()
                .find(|(name, _)| name == &filename)
                .expect(&format!(
                    "Include file {} in not on \"includes\" list",
                    filename
                ));

            let _ = res
                .splice(
                    directive_start_ix..filename_end_ix + 1,
                    include_content.1.chars(),
                )
                .collect::<Vec<_>>();
        }

        res.into_iter().collect()
    }

    #[test]
    fn preprocessor_test() {
        let shader_string = r#"
#version blah blah

asd
asd

#include "hello.glsl"

qwe
"#;

        let preprocessed = r#"
#version blah blah

asd
asd

iii
jjj

qwe
"#;

        let result = preprocess_shader(
            shader_string,
            &PreprocessorConfig {
                includes: vec![("hello.glsl".to_string(), "iii\njjj".to_string())],
                ..Default::default()
            },
        );

        assert_eq!(result, preprocessed);
    }
}
