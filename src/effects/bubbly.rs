use bevy::prelude::*;

use super::{simple_hash, EffectRegion};
use crate::grid::{GridPosition, TerminalCell};

/// Random cell scale animation effect.
///
/// A fraction of cells grow and shrink at different phases,
/// creating a bubbly/effervescent look.
#[derive(Component, Clone, Debug)]
pub struct Bubbly {
    /// Oscillation speed in Hz.
    pub speed: f32,
    /// Fraction of cells that are "active" (0.0 to 1.0).
    pub density: f32,
    /// Maximum scale factor for active cells.
    pub max_scale: f32,
}

impl Default for Bubbly {
    fn default() -> Self {
        Self {
            speed: 0.8,
            density: 0.15,
            max_scale: 1.4,
        }
    }
}

/// System that applies the bubbly effect to cell transforms.
pub fn bubbly_system(
    time: Res<Time>,
    effects: Query<(&Bubbly, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    let t = time.elapsed_secs();

    for (bubbly, region) in effects.iter() {
        let threshold = (bubbly.density * 1000.0) as u32;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let h = simple_hash(pos.col as u32, pos.row as u32);

            // Only activate a fraction of cells
            if (h % 1000) >= threshold {
                continue;
            }

            // Deterministic phase from hash
            let phase = (h % 10000) as f32 / 10000.0 * std::f32::consts::TAU;
            let wave = (t * bubbly.speed * std::f32::consts::TAU + phase).sin();
            let scale = 1.0 + (bubbly.max_scale - 1.0) * ((wave + 1.0) / 2.0);

            transform.scale *= Vec3::splat(scale);
        }
    }
}
