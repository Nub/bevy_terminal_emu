use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{ForegroundSprite, GridPosition, TerminalCell};

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
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// System that applies the shiny sweep effect to foreground sprites.
pub fn shiny_system(
    time: Res<Time>,
    effects: Query<(&Shiny, &EffectRegion)>,
    cells: Query<(&GridPosition, &Children), With<TerminalCell>>,
    mut sprites: Query<&mut Sprite, With<ForegroundSprite>>,
) {
    let t = time.elapsed_secs();

    for (shiny, region) in effects.iter() {
        let cos_a = shiny.angle.cos();
        let sin_a = shiny.angle.sin();
        // Diagonal length of the grid (generous upper bound)
        let diagonal = 200.0_f32;
        let band_pos = (t * shiny.speed) % diagonal - shiny.width;
        let half_width = shiny.width / 2.0;

        for (pos, children) in cells.iter() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            // Project cell position onto the sweep direction
            let proj = pos.col as f32 * cos_a + pos.row as f32 * sin_a;
            let dist = (proj - band_pos).abs();

            if dist > half_width {
                continue;
            }

            // Smoothstep falloff from edge to center
            let falloff = 1.0 - smoothstep(0.0, half_width, dist);
            let boost = 1.0 + shiny.brightness * falloff;

            for child in children.iter() {
                if let Ok(mut sprite) = sprites.get_mut(child) {
                    let [r, g, b, a] = sprite.color.to_srgba().to_f32_array();
                    sprite.color =
                        Color::srgba((r * boost).min(1.0), (g * boost).min(1.0), (b * boost).min(1.0), a);
                }
            }
        }
    }
}
