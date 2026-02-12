use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};

#[derive(Component, Clone, Debug)]
pub struct Collapse {
    pub gravity: f32,
    pub elapsed: f32,
    pub duration: f32,
    pub stagger_per_row: f32,
    pub active: bool,
}

impl Default for Collapse {
    fn default() -> Self {
        Self {
            gravity: 800.0,
            elapsed: 0.0,
            duration: 3.0,
            stagger_per_row: 0.05,
            active: true,
        }
    }
}

pub fn collapse_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    mut effects: Query<(&mut Collapse, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
) {
    for (mut collapse, region) in effects.iter_mut() {
        if !collapse.active {
            continue;
        }

        collapse.elapsed += time.delta_secs();

        if collapse.elapsed > collapse.duration {
            collapse.active = false;
            continue;
        }

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let row_delay = pos.row as f32 * collapse.stagger_per_row;
            let t = (collapse.elapsed - row_delay).max(0.0);
            let fall = 0.5 * collapse.gravity * t * t;
            transform.translation.y -= fall;
        }
    }
}
