use bevy::prelude::*;

/// A simple sine wave effect that oscillates cells vertically.
#[derive(Component, Clone, Debug)]
pub struct Wave {
    /// Maximum displacement in pixels.
    pub amplitude: f32,
    /// Wavelength in grid columns.
    pub wavelength: f32,
    /// Speed of the wave (columns per second).
    pub speed: f32,
    /// Axis of wave propagation: if true, wave travels along rows; if false, along columns.
    pub horizontal: bool,
}

impl Default for Wave {
    fn default() -> Self {
        Self {
            amplitude: 5.0,
            wavelength: 8.0,
            speed: 4.0,
            horizontal: true,
        }
    }
}
