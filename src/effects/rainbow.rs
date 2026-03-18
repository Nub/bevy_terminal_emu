use bevy::prelude::*;

/// Rainbow color cycling effect.
///
/// Cycles foreground sprite hue through the spectrum based on grid position and time.
#[derive(Component, Clone, Debug)]
pub struct Rainbow {
    /// Speed of hue cycling (revolutions per second).
    pub speed: f32,
    /// Color saturation (0.0 to 1.0).
    pub saturation: f32,
    /// Color lightness (0.0 to 1.0).
    pub lightness: f32,
    /// Spatial spread — how much hue varies across the grid.
    pub spread: f32,
}

impl Default for Rainbow {
    fn default() -> Self {
        Self {
            speed: 1.0,
            saturation: 1.0,
            lightness: 0.6,
            spread: 0.3,
        }
    }
}
