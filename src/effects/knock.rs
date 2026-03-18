use bevy::prelude::*;

/// Blunt-impact knock effect — all cells in the region jolt in a uniform
/// direction (with slight per-cell deviation), then ease back to rest.
/// Simulates the feel of a heavy weapon strike.
#[derive(Component, Clone, Debug)]
pub struct Knock {
    /// Direction of the knock in radians.
    pub angle: f32,
    /// Maximum displacement in pixels.
    pub amplitude: f32,
    /// Per-cell angular deviation in radians (0 = perfectly uniform).
    pub deviation: f32,
    /// Per-cell rotation strength in radians at peak.
    pub rotation: f32,
    /// How long the effect has been running.
    pub elapsed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Knock {
    fn default() -> Self {
        Self {
            angle: 0.0,
            amplitude: 12.0,
            deviation: 0.3,
            rotation: 0.1,
            elapsed: 0.0,
            duration: 0.6,
            active: true,
        }
    }
}
