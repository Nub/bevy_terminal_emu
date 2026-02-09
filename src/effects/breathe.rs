use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{GridPosition, TerminalCell};

/// Rhythmic scale pulse effect.
///
/// Cells oscillate in scale with a sinusoidal pattern, with per-cell phase offsets.
#[derive(Component, Clone, Debug)]
pub struct Breathe {
    /// Minimum scale factor.
    pub min_scale: f32,
    /// Maximum scale factor.
    pub max_scale: f32,
    /// Oscillation frequency in Hz.
    pub speed: f32,
    /// Phase spread factor — higher values create more visible staggering between cells.
    pub phase_spread: f32,
}

impl Default for Breathe {
    fn default() -> Self {
        Self {
            min_scale: 0.8,
            max_scale: 1.2,
            speed: 1.5,
            phase_spread: 0.3,
        }
    }
}

/// System that applies the breathe effect to cell transforms.
pub fn breathe_system(
    time: Res<Time>,
    effects: Query<(&Breathe, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    let t = time.elapsed_secs();

    for (breathe, region) in effects.iter() {
        let mid = (breathe.min_scale + breathe.max_scale) / 2.0;
        let range = (breathe.max_scale - breathe.min_scale) / 2.0;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let phase_offset =
                (pos.col as f32 * 0.7 + pos.row as f32 * 1.1) * breathe.phase_spread;
            let wave = (std::f32::consts::TAU * breathe.speed * t + phase_offset).sin();
            let scale = mid + range * wave;

            transform.scale = Vec3::splat(scale);
        }
    }
}
