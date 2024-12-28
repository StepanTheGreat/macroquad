//! Input state management

use std::collections::{HashMap, HashSet};

use glam::{vec2, Vec2};
use miniquad::{window::screen_size, KeyCode, KeyMods, MouseButton};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TouchPhase {
    Started,
    Stationary,
    Moved,
    Ended,
    Cancelled,
}

impl From<miniquad::TouchPhase> for TouchPhase {
    fn from(miniquad_phase: miniquad::TouchPhase) -> TouchPhase {
        match miniquad_phase {
            miniquad::TouchPhase::Started => TouchPhase::Started,
            miniquad::TouchPhase::Moved => TouchPhase::Moved,
            miniquad::TouchPhase::Ended => TouchPhase::Ended,
            miniquad::TouchPhase::Cancelled => TouchPhase::Cancelled,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Touch {
    pub id: u64,
    pub phase: TouchPhase,
    pub position: Vec2,
}

/// Convert a position in pixels to a position in the range [-1; 1].
fn convert_to_local(pixel_pos: Vec2) -> Vec2 {
    let (width, height) = screen_size();
    (vec2(pixel_pos.x / width, pixel_pos.y / height) * 2.0) - vec2(1.0, 1.0)
}

/// A simple state struct that simplifies input management.
///
/// Simply feed it data and it will update everything for you:
/// - All state update methods are prefixed with `update_`
/// - All other methods are getters for the inner state.
///
/// Some update methods will ask for `cursor_grabbed`, which you should keep track of yourself.
pub struct InputState {
    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,
    mouse_down: HashSet<MouseButton>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
    touches: HashMap<u64, Touch>,
    chars_pressed_queue: Vec<char>,
    mouse_position: Vec2,
    last_mouse_position: Option<Vec2>,
    mouse_wheel: Vec2,

    /// Whether to conver touch input to mouse input
    pub simulate_mouse_with_touch: bool,
}

impl InputState {
    pub fn new(simulate_mouse_with_touch: bool) -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            chars_pressed_queue: Vec::new(),
            mouse_down: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_released: HashSet::new(),
            touches: HashMap::new(),
            mouse_position: vec2(0., 0.),
            last_mouse_position: None,
            mouse_wheel: vec2(0., 0.),

            simulate_mouse_with_touch,
        }
    }

    /// Since we don't want to use the same information throughout multiple frames,
    /// we need to reset it. Call this at the end of each frame
    pub fn post_frame_cleanup(&mut self) {
        self.mouse_wheel = Vec2::new(0., 0.);
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.last_mouse_position = Some(self.mouse_position_local());

        self.touches.retain(|_, touch| {
            touch.phase != TouchPhase::Ended && touch.phase != TouchPhase::Cancelled
        });

        // change all Started or Moved touches to Stationary
        for touch in self.touches.values_mut() {
            if touch.phase == TouchPhase::Started || touch.phase == TouchPhase::Moved {
                touch.phase = TouchPhase::Stationary;
            }
        }
    }

    pub fn update_raw_mouse_motion(&mut self, x: f32, y: f32) {
        self.mouse_position += Vec2::new(x, y);
    }

    pub fn update_mouse_motion_event(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
    }

    pub fn update_mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.mouse_wheel.x = x;
        self.mouse_wheel.y = y;
    }

    pub fn update_mouse_button_down_event(
        &mut self,
        btn: MouseButton,
        x: f32,
        y: f32,
        cursor_grabbed: bool,
    ) {
        self.mouse_down.insert(btn);
        self.mouse_pressed.insert(btn);

        if !cursor_grabbed {
            self.mouse_position = Vec2::new(x, y);
        }
    }

    pub fn update_mouse_button_up_event(
        &mut self,
        btn: MouseButton,
        x: f32,
        y: f32,
        cursor_grabbed: bool,
    ) {
        self.mouse_down.remove(&btn);
        self.mouse_released.insert(btn);

        if !cursor_grabbed {
            self.mouse_position = Vec2::new(x, y);
        }
    }

    pub fn update_touch_event(
        &mut self,
        phase: TouchPhase,
        id: u64,
        x: f32,
        y: f32,
        cursor_grabbed: bool,
    ) {
        self.touches.insert(
            id,
            Touch {
                id,
                phase,
                position: Vec2::new(x, y),
            },
        );

        if self.simulate_mouse_with_touch {
            if phase == TouchPhase::Started {
                self.update_mouse_button_down_event(MouseButton::Left, x, y, cursor_grabbed);
            }

            if phase == TouchPhase::Ended {
                self.update_mouse_button_up_event(MouseButton::Left, x, y, cursor_grabbed);
            }

            if phase == TouchPhase::Moved {
                self.update_mouse_motion_event(x, y);
            }
        }
    }

    pub fn update_char_event(&mut self, character: char, _mods: KeyMods, _rep: bool) {
        self.chars_pressed_queue.push(character);
    }

    pub fn update_key_down_event(&mut self, keycode: KeyCode, _mods: KeyMods, repeat: bool) {
        self.keys_down.insert(keycode);
        if !repeat {
            self.keys_pressed.insert(keycode);
        }
    }

    pub fn update_key_up_event(&mut self, keycode: KeyCode, _mods: KeyMods) {
        self.keys_down.remove(&keycode);
        self.keys_released.insert(keycode);
    }

    /// Return mouse position in pixels.
    pub fn mouse_position(&self) -> (f32, f32) {
        (
            self.mouse_position.x / miniquad::window::dpi_scale(),
            self.mouse_position.y / miniquad::window::dpi_scale(),
        )
    }

    /// Return mouse position in range [-1; 1].
    pub fn mouse_position_local(&self) -> Vec2 {
        let (pixels_x, pixels_y) = self.mouse_position();

        convert_to_local(Vec2::new(pixels_x, pixels_y))
    }

    /// Returns the difference between the current mouse position and the mouse position on the previous frame.
    pub fn mouse_delta_position(&self) -> Vec2 {
        let current_position = self.mouse_position_local();
        let last_position = self.last_mouse_position.unwrap_or(current_position);

        // Calculate the delta
        last_position - current_position
    }

    /// Return touches with positions in pixels.
    ///
    /// Attention: This method will clone its inner touch vector, so better reuse the same values instead
    pub fn touches(&self) -> Vec<Touch> {
        self.touches.values().cloned().collect()
    }

    /// Return touches with positions in range [-1; 1].
    ///
    /// The same warning as with [`InputState::touches`]
    pub fn touches_local(&self) -> Vec<Touch> {
        self.touches
            .values()
            .map(|touch| {
                let mut touch = touch.clone();
                touch.position = convert_to_local(touch.position);
                touch
            })
            .collect()
    }

    pub fn mouse_wheel(&self) -> (f32, f32) {
        (self.mouse_wheel.x, self.mouse_wheel.y)
    }

    /// Detect if the key has been pressed once
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.keys_pressed.contains(&key_code)
    }

    /// Detect if the key is being pressed
    pub fn is_key_down(&self, key_code: KeyCode) -> bool {
        self.keys_down.contains(&key_code)
    }

    /// Detect if the key has been released this frame
    pub fn is_key_released(&self, key_code: KeyCode) -> bool {
        self.keys_released.contains(&key_code)
    }

    /// Return the last pressed char.
    /// Each "get_char_pressed" call will consume a character from the input queue.
    pub fn get_char_pressed(&mut self) -> Option<char> {
        self.chars_pressed_queue.pop()
    }

    /// Return the last pressed key.
    pub fn get_last_key_pressed(&self) -> Option<KeyCode> {
        // TODO: this will return a random key from keys_pressed HashMap instead of the last one, fix me later
        self.keys_pressed.iter().next().cloned()
    }

    pub fn get_keys_pressed(&self) -> &HashSet<KeyCode> {
        &self.keys_pressed
    }

    pub fn get_keys_down(&self) -> &HashSet<KeyCode> {
        &self.keys_down
    }

    pub fn get_keys_released(&self) -> &HashSet<KeyCode> {
        &self.keys_released
    }

    /// Clear the pressed char queue. I'm not sure when it's useful, but I'll keep it for now
    pub fn clear_input_queue(&mut self) {
        self.chars_pressed_queue.clear();
    }

    /// Detect if the button is being pressed
    pub fn is_mouse_button_down(&self, btn: &MouseButton) -> bool {
        self.mouse_down.contains(btn)
    }

    /// Detect if the button has been pressed once
    pub fn is_mouse_button_pressed(&self, btn: &MouseButton) -> bool {
        self.mouse_pressed.contains(btn)
    }

    /// Detect if the button has been released this frame
    pub fn is_mouse_button_released(&self, btn: &MouseButton) -> bool {
        self.mouse_released.contains(btn)
    }
}
