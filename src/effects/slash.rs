use bevy::prelude::*;

/// Slash effect — a blade cuts across the region along a line, splitting cells
/// apart perpendicular to the cut as it passes, like cutting cloth.
///
/// The cut animates from one end of the line to the other. Cells behind the
/// blade's wavefront get displaced outward (perpendicular to the cut).
/// Displacement is strongest at the center of the line and fades toward edges.
/// After the blade finishes its pass, the split eases closed.
#[derive(Component, Clone, Debug)]
pub struct Slash {
    /// How long the slash has been running.
    pub elapsed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Maximum perpendicular displacement in pixels.
    pub amplitude: f32,
    /// Width of the displacement band (in grid cells) on each side of the line.
    pub width: f32,
    /// Angle of the slash line in radians (0 = horizontal, PI/4 = diagonal).
    pub angle: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Slash {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            duration: 0.5,
            amplitude: 8.0,
            width: 4.0,
            angle: std::f32::consts::FRAC_PI_4,
            active: true,
        }
    }
}
