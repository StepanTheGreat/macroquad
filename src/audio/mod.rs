//! Loading and playing sounds.
use crate::fs::load_file;

pub use quad_snd::{
    AudioContext, 
    Sound as SoundId,
    PlaySoundParams
};

/// Load an audio file.
///
/// Attempts to automatically detect the format of the source of data.
/// 
/// ### Warning
/// 1. SoundId is not automatically cleaned, do it yourself, on the appropriate audio context
/// 2. On wasm, a sound that is loaded from bytes, isn't actually fully
/// loaded. Before playing it, use the [quad_snd::Sound::is_loaded] to check whether it's 
/// ready:
/// ```
/// sound.is_loaded()
/// ```
pub fn load_sound(ctx: &AudioContext, path: &str) -> Result<SoundId, miniquad::fs::Error> {
    let data = load_file(path)?;
    Ok(load_sound_from_bytes(ctx, &data))
}

/// Load audio data.
///
/// Attempts to automatically detect the format of the source of data.
/// ### Warning
/// 1. SoundId is not automatically cleaned, do it yourself, on the appropriate audio context
/// 2. On wasm, a sound that is loaded from bytes, isn't actually fully
/// loaded. Before playing it, use the [quad_snd::Sound::is_loaded] to check whether it's 
/// ready:
/// ```
/// sound.is_loaded()
/// ```
pub fn load_sound_from_bytes(ctx: &AudioContext, data: &[u8]) -> SoundId {
    SoundId::load(ctx, data)
}