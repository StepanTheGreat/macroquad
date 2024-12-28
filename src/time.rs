/// A simple time state struct that simplifies time management a bit.
///
/// It's fairly easy to implement your own, but if it saves you time - you can try this one
/// instead.
#[derive(Clone, Debug, Copy)]
pub struct Timer {
    start_time: f64,
    last_frame_time: f64,
    delta_time: f64,
}

impl Timer {
    /// Create automatically using [miniquad::date::now]
    pub fn new() -> Self {
        let time = miniquad::date::now();
        Self::from_time(time)
    }

    /// Create from provided time
    pub fn from_time(time: f64) -> Self {
        Self {
            start_time: time,
            last_frame_time: time,
            delta_time: 1.0 / 60.0,
        }
    }

    /// Update automatically using [miniquad::date::now]
    pub fn update(&mut self) -> f32 {
        let now = miniquad::date::now();
        self.delta_time = now - self.last_frame_time;
        self.last_frame_time = now;
        self.delta_time as _
    }

    /// Update manually from the provided time
    pub fn update_from_time(&mut self, time: f64) {
        self.delta_time = time - self.last_frame_time;
        self.last_frame_time = time;
    }

    /// Get timer delta time
    pub fn delta(&self) -> f32 {
        self.delta_time as _
    }

    /// Get the start time of this timer
    pub fn start_time(&self) -> f32 {
        self.start_time as _
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
