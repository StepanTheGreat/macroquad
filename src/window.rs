//! Window and associated to window rendering context related functions.

// miniquad is re-exported for the use in combination with `get_internal_gl`
pub use miniquad;

pub use miniquad::conf::Conf;

pub use miniquad::window::*;

pub fn screen_width() -> f32 {
    let (w, _) = screen_size();
    w / dpi_scale()
}

pub fn screen_height() -> f32 {
    let (_, h) = screen_size();
    h / dpi_scale()
}

/// Request the window size to be the given value. This takes DPI into account.
///
/// Note that the OS might decide to give a different size. Additionally, the size in macroquad won't be updated until the next `next_frame().await`.
pub fn request_new_screen_size(width: f32, height: f32) {
    miniquad::window::set_window_size(
        (width * miniquad::window::dpi_scale()) as u32,
        (height * miniquad::window::dpi_scale()) as u32,
    );
    // We do not set the context.screen_width and context.screen_height here.
    // After `set_window_size` is called, EventHandlerFree::resize will be invoked, setting the size correctly.
    // Because the OS might decide to give a different screen dimension, setting the context.screen_* here would be confusing.
}
