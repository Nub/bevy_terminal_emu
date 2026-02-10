use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{ForegroundSprite, GridPosition, TerminalCell};

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

/// System that applies the rainbow effect to foreground sprite colors.
pub fn rainbow_system(
    time: Res<Time>,
    effects: Query<(&Rainbow, &EffectRegion)>,
    cells: Query<(&GridPosition, &Children), With<TerminalCell>>,
    mut sprites: Query<&mut Sprite, With<ForegroundSprite>>,
) {
    let t = time.elapsed_secs();

    for (rainbow, region) in effects.iter() {
        for (pos, children) in cells.iter() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let hue = ((pos.col as f32 + pos.row as f32) * rainbow.spread + t * rainbow.speed)
                * 360.0
                % 360.0;

            let color = Color::hsl(hue, rainbow.saturation, rainbow.lightness);

            for child in children.iter() {
                if let Ok(mut sprite) = sprites.get_mut(child) {
                    let alpha = sprite.color.alpha();
                    sprite.color = color.with_alpha(alpha);
                }
            }
        }
    }
}
