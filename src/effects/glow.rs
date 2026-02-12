use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{CellEntityIndex, ForegroundSprite, GridPosition, TerminalCell};

#[derive(Component, Clone, Debug)]
pub struct Glow {
    pub speed: f32,
    pub intensity: f32,
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

pub fn glow_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    effects: Query<(&Glow, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
    cell_index: Res<CellEntityIndex<T>>,
    mut sprites: Query<&mut Sprite, With<ForegroundSprite<T>>>,
) {
    let t = time.elapsed_secs();
    let columns = cell_index.columns as usize;

    for (glow, region) in effects.iter() {
        for (idx, &parent_entity) in cell_index.entities.iter().enumerate() {
            let col = (idx % columns) as u16;
            let row = (idx / columns) as u16;

            if !region.contains(col, row) {
                continue;
            }

            let Ok((pos, mut transform)) = cells.get_mut(parent_entity) else {
                continue;
            };

            let phase_offset = (pos.col as f32 * 0.5 + pos.row as f32 * 0.8) * glow.spread;
            let phase = std::f32::consts::TAU * glow.speed * t + phase_offset;
            let wave = phase.sin();

            let scale = 1.0 + 0.05 * wave;
            transform.scale *= Vec3::splat(scale);

            let fg_entity = cell_index.fg_entities[idx];
            if let Ok(mut sprite) = sprites.get_mut(fg_entity) {
                let base_alpha = sprite.color.alpha();
                let new_alpha = (base_alpha * (1.0 + glow.intensity * wave)).clamp(0.0, 1.0);
                sprite.color = sprite.color.with_alpha(new_alpha);
            }
        }
    }
}
