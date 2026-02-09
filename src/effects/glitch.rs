use bevy::prelude::*;

use super::{simple_hash, EffectRegion};
use crate::grid::{GridPosition, TerminalCell};

/// CRT-style horizontal row displacement effect.
///
/// Entire rows randomly shift left/right; the pattern changes rapidly.
#[derive(Component, Clone, Debug)]
pub struct Glitch {
    /// Maximum horizontal displacement in pixels.
    pub max_offset: f32,
    /// Fraction of rows affected each frame (0.0 to 1.0).
    pub intensity: f32,
    /// How many times per second the glitch pattern changes.
    pub frequency: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Glitch {
    fn default() -> Self {
        Self {
            max_offset: 30.0,
            intensity: 0.3,
            frequency: 8.0,
            active: true,
        }
    }
}

/// System that applies the glitch effect to cell transforms.
pub fn glitch_system(
    time: Res<Time>,
    effects: Query<(&Glitch, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    let t = time.elapsed_secs();

    for (glitch, region) in effects.iter() {
        if !glitch.active {
            continue;
        }

        let time_slot = (t * glitch.frequency) as u32;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            // Determine if this row is affected using a hash of (row, time_slot)
            let row_hash = simple_hash(pos.row as u32, time_slot);
            let row_frac = (row_hash % 1000) as f32 / 1000.0;

            if row_frac < glitch.intensity {
                // Compute offset for this row: deterministic but looks random
                let offset_hash = simple_hash(pos.row as u32, time_slot.wrapping_add(7919));
                let offset_frac = (offset_hash % 2000) as f32 / 1000.0 - 1.0; // -1.0 to 1.0
                transform.translation.x += offset_frac * glitch.max_offset;
            }
        }
    }
}
