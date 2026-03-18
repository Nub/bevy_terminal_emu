use bevy::prelude::*;

/// Sweeping highlight band effect.
///
/// A bright band sweeps diagonally across the grid, boosting foreground RGB.
#[derive(Component, Clone, Debug)]
pub struct Shiny {
    /// Speed of the sweep (grid units per second).
    pub speed: f32,
    /// Width of the highlight band in grid units.
    pub width: f32,
    /// Angle of the sweep in radians (0 = horizontal, PI/2 = vertical).
    pub angle: f32,
    /// Maximum brightness multiplier at the center of the band.
    pub brightness: f32,
}

impl Default for Shiny {
    fn default() -> Self {
        Self {
            speed: 8.0,
            width: 6.0,
            angle: 0.5,
            brightness: 2.0,
        }
    }
}

/// Smoothstep interpolation.
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
