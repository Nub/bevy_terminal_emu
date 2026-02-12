use bevy::prelude::*;

use super::{simple_hash, EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};

#[derive(Component, Clone, Debug)]
pub struct Glitch {
    pub max_offset: f32,
    pub intensity: f32,
    pub frequency: f32,
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

pub fn glitch_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    effects: Query<(&Glitch, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
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

            let row_hash = simple_hash(pos.row as u32, time_slot);
            let row_frac = (row_hash % 1000) as f32 / 1000.0;

            if row_frac < glitch.intensity {
                let offset_hash = simple_hash(pos.row as u32, time_slot.wrapping_add(7919));
                let offset_frac = (offset_hash % 2000) as f32 / 1000.0 - 1.0;
                transform.translation.x += offset_frac * glitch.max_offset;
            }
        }
    }
}
