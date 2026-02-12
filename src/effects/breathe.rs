use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};

#[derive(Component, Clone, Debug)]
pub struct Breathe {
    pub min_scale: f32,
    pub max_scale: f32,
    pub speed: f32,
    pub phase_spread: f32,
}

impl Default for Breathe {
    fn default() -> Self {
        Self {
            min_scale: 0.92,
            max_scale: 1.08,
            speed: 1.0,
            phase_spread: 0.0,
        }
    }
}

pub fn breathe_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    effects: Query<(&Breathe, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
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
