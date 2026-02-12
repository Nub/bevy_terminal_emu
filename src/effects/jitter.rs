use bevy::prelude::*;

use super::{simple_hash, EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};

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

/// System that applies the jitter effect to cell transforms.
pub fn jitter_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    effects: Query<(&Jitter, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
) {
    let t = time.elapsed_secs();

    for (jitter, region) in effects.iter() {
        let time_slot = (t * jitter.speed) as u32;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let cell_id = pos.row as u32 * 1000 + pos.col as u32;

            // X offset
            let hx = simple_hash(cell_id, time_slot);
            let dx = (hx % 2000) as f32 / 1000.0 - 1.0; // -1.0 to 1.0
            transform.translation.x += dx * jitter.amplitude;

            // Y offset
            let hy = simple_hash(cell_id, time_slot.wrapping_add(3571));
            let dy = (hy % 2000) as f32 / 1000.0 - 1.0;
            transform.translation.y += dy * jitter.amplitude;

            // Optional rotation
            if jitter.rotate {
                let hr = simple_hash(cell_id, time_slot.wrapping_add(6947));
                let r = (hr % 2000) as f32 / 1000.0 - 1.0;
                transform.rotation = Quat::from_rotation_z(r * jitter.max_rotation);
            }
        }
    }
}
