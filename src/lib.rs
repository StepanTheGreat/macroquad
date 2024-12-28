//! Macroquad abstractions 
//! 
//! A crate designed to expose macroquad abstractions in a modular way

mod graphics;
mod tobytes;

pub mod time;
#[cfg(feature="quad-snd")]
pub mod audio;
pub mod utils;
pub mod color;
pub mod input;
pub mod text;
pub mod draw;
pub mod texture;
pub mod fs;
#[cfg(feature="ui")]
pub mod ui;
pub mod window;

// pub mod telemetry;

mod error;

pub use error::Error;

/// Cross platform random generator.
pub mod rand {
    pub use quad_rand::*;
}

#[cfg(not(feature = "log-rs"))]
/// Logging macros, available with miniquad "log-impl" feature.
pub mod logging {
    pub use miniquad::{debug, error, info, trace, warn};
}
#[cfg(feature = "log-rs")]
// Use logging facade
pub use ::log as logging;
pub use miniquad;