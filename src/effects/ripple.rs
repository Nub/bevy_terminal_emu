use bevy::prelude::*;

/// A ripple effect that displaces cells in a wave pattern from an origin point.
#[derive(Component, Clone, Debug)]
pub struct Ripple {
    /// Origin column of the ripple.
    pub origin_col: f32,
    /// Origin row of the ripple.
    pub origin_row: f32,
    /// Maximum displacement in pixels.
    pub amplitude: f32,
    /// Wavelength in grid cells.
    pub wavelength: f32,
    /// Speed of wave propagation (cells per second).
    pub speed: f32,
    /// Accumulated phase offset.
    pub phase: f32,
    /// Exponential damping factor (higher = faster falloff).
    pub damping: f32,
}

impl Default for Ripple {
    fn default() -> Self {
        Self {
            origin_col: 40.0,
            origin_row: 12.0,
            amplitude: 8.0,
            wavelength: 6.0,
            speed: 10.0,
            phase: 0.0,
            damping: 0.1,
        }
    }
}
