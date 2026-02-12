use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{CellEntityIndex, ForegroundSprite};

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
pub fn shiny_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    effects: Query<(&Shiny, &EffectRegion), With<TargetTerminal<T>>>,
    cell_index: Res<CellEntityIndex<T>>,
    mut sprites: Query<&mut Sprite, With<ForegroundSprite<T>>>,
) {
    let t = time.elapsed_secs();
    let columns = cell_index.columns as usize;

    for (shiny, region) in effects.iter() {
        let cos_a = shiny.angle.cos();
        let sin_a = shiny.angle.sin();
        // Diagonal length of the grid (generous upper bound)
        let diagonal = 200.0_f32;
        let band_pos = (t * shiny.speed) % diagonal - shiny.width;
        let half_width = shiny.width / 2.0;

        for (idx, &fg_entity) in cell_index.fg_entities.iter().enumerate() {
            let col = (idx % columns) as u16;
            let row = (idx / columns) as u16;

            if !region.contains(col, row) {
                continue;
            }

            // Project cell position onto the sweep direction
            let proj = col as f32 * cos_a + row as f32 * sin_a;
            let dist = (proj - band_pos).abs();

            if dist > half_width {
                continue;
            }

            // Smoothstep falloff from edge to center
            let falloff = 1.0 - smoothstep(0.0, half_width, dist);
            let boost = 1.0 + shiny.brightness * falloff;

            if let Ok(mut sprite) = sprites.get_mut(fg_entity) {
                let [r, g, b, a] = sprite.color.to_srgba().to_f32_array();
                sprite.color =
                    Color::srgba((r * boost).min(1.0), (g * boost).min(1.0), (b * boost).min(1.0), a);
            }
        }
    }
}
