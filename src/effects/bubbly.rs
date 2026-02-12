use bevy::prelude::*;

use super::{simple_hash, EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};

#[derive(Component, Clone, Debug)]
pub struct Bubbly {
    pub speed: f32,
    pub density: f32,
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

pub fn bubbly_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    effects: Query<(&Bubbly, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
) {
    let t = time.elapsed_secs();

    for (bubbly, region) in effects.iter() {
        let threshold = (bubbly.density * 1000.0) as u32;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let h = simple_hash(pos.col as u32, pos.row as u32);

            if (h % 1000) >= threshold {
                continue;
            }

            let phase = (h % 10000) as f32 / 10000.0 * std::f32::consts::TAU;
            let wave = (t * bubbly.speed * std::f32::consts::TAU + phase).sin();
            let scale = 1.0 + (bubbly.max_scale - 1.0) * ((wave + 1.0) / 2.0);

            transform.scale *= Vec3::splat(scale);
        }
    }
}
