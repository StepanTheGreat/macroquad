//!
//! `macroquad` is a simple and easy to use game library for Rust programming language.
//!
//! `macroquad` attempts to avoid any rust-specific programming concepts like lifetimes/borrowing, making it very friendly for rust beginners.
//!
//! ## Supported platforms
//!
//! * PC: Windows/Linux/MacOS
//! * HTML5
//! * Android
//! * IOS
//!
//! ## Features
//!
//! * Same code for all supported platforms, no platform dependent defines required
//! * Efficient 2D rendering with automatic geometry batching
//! * Minimal amount of dependencies: build after `cargo clean` takes only 16s on x230(~6years old laptop)
//! * Immediate mode UI library included
//! * Single command deploy for both WASM and Android [build instructions](https://github.com/not-fl3/miniquad/#building-examples)
//! # Example
//! ```no_run
//! use macroquad::prelude::*;
//!
//! #[macroquad::main("BasicShapes")]
//! async fn main() {
//!     loop {
//!         clear_background(RED);
//!
//!         draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
//!         draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
//!         draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
//!         draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);
//!
//!         next_frame().await
//!     }
//! }
//!```

mod graphics;
mod tobytes;

pub mod time;
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

pub mod prelude;

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