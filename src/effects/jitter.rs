use bevy::prelude::*;

/// Per-cell random vibration effect.
///
/// Each cell gets a small random offset every frame (hash-based, no `rand` dependency).
#[derive(Component, Clone, Debug)]
pub struct Jitter {
    /// Maximum displacement amplitude in pixels.
    pub amplitude: f32,
    /// How many times per second the jitter pattern changes.
    pub speed: f32,
    /// Whether to apply small random rotation as well.
    pub rotate: bool,
    /// Maximum rotation in radians (when `rotate` is true).
    pub max_rotation: f32,
}

impl Default for Jitter {
    fn default() -> Self {
        Self {
            amplitude: 3.0,
            speed: 20.0,
            rotate: true,
            max_rotation: 0.05,
        }
    }
}
