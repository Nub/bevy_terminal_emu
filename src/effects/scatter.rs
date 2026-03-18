use bevy::prelude::*;

/// Explosion effect that scatters cells outward from a center point.
///
/// One-shot: cells fly outward radially, shrinking and spinning over time.
#[derive(Component, Clone, Debug)]
pub struct Scatter {
    /// Origin column (grid coords).
    pub origin_col: f32,
    /// Origin row (grid coords).
    pub origin_row: f32,
    /// Outward speed in pixels per second.
    pub speed: f32,
    /// How long the scatter has been running.
    pub elapsed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Spin speed in radians per second.
    pub spin: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Scatter {
    fn default() -> Self {
        Self {
            origin_col: 40.0,
            origin_row: 12.0,
            speed: 150.0,
            elapsed: 0.0,
            duration: 3.0,
            spin: 3.0,
            active: true,
        }
    }
}
