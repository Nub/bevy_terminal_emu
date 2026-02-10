use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{ForegroundSprite, GridPosition, TerminalCell};

/// Pulsing glow effect.
///
/// Modulates foreground sprite alpha and scale with per-cell phase offsets
/// for a shimmering appearance.
#[derive(Component, Clone, Debug)]
pub struct Glow {
    /// Oscillation speed in Hz.
    pub speed: f32,
    /// Intensity of the alpha modulation (0.0 to 1.0).
    pub intensity: f32,
    /// Spatial spread of phase offsets between cells.
    pub spread: f32,
}

impl Default for Glow {
    fn default() -> Self {
        Self {
            speed: 2.0,
            intensity: 0.5,
            spread: 0.4,
        }
    }
}

/// System that applies the glow effect to foreground sprites.
pub fn glow_system(
    time: Res<Time>,
    effects: Query<(&Glow, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &Children, &mut Transform), With<TerminalCell>>,
    mut sprites: Query<&mut Sprite, With<ForegroundSprite>>,
) {
    let t = time.elapsed_secs();

    for (glow, region) in effects.iter() {
        for (pos, children, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let phase_offset = (pos.col as f32 * 0.5 + pos.row as f32 * 0.8) * glow.spread;
            let phase = std::f32::consts::TAU * glow.speed * t + phase_offset;
            let wave = phase.sin();

            // Scale pulse on the cell transform
            let scale = 1.0 + 0.05 * wave;
            transform.scale *= Vec3::splat(scale);

            // Alpha modulation on foreground sprite
            for child in children.iter() {
                if let Ok(mut sprite) = sprites.get_mut(child) {
                    let base_alpha = sprite.color.alpha();
                    let new_alpha = (base_alpha * (1.0 + glow.intensity * wave)).clamp(0.0, 1.0);
                    sprite.color = sprite.color.with_alpha(new_alpha);
                }
            }
        }
    }
}
