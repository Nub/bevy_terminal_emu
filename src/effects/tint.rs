use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{CellEntityIndex, ForegroundSprite};

/// Color tint effect that lerps foreground sprite colors toward a target color.
///
/// Strength 0.0 = no tint, 1.0 = full replacement with the tint color.
#[derive(Component, Clone, Debug)]
pub struct Tint {
    /// The color to tint toward.
    pub color: Color,
    /// Blend strength: 0.0 = original color, 1.0 = fully tinted.
    pub strength: f32,
}

/// System that applies the tint effect to foreground sprites.
pub fn tint_system<T: 'static + Send + Sync>(
    effects: Query<(&Tint, &EffectRegion), With<TargetTerminal<T>>>,
    cell_index: Res<CellEntityIndex<T>>,
    mut sprites: Query<&mut Sprite, With<ForegroundSprite<T>>>,
) {
    let columns = cell_index.columns as usize;

    for (tint, region) in effects.iter() {
        let [tr, tg, tb, _] = tint.color.to_srgba().to_f32_array();
        let s = tint.strength.clamp(0.0, 1.0);

        for (idx, &fg_entity) in cell_index.fg_entities.iter().enumerate() {
            let col = (idx % columns) as u16;
            let row = (idx / columns) as u16;

            if !region.contains(col, row) {
                continue;
            }

            if let Ok(mut sprite) = sprites.get_mut(fg_entity) {
                let [r, g, b, a] = sprite.color.to_srgba().to_f32_array();
                sprite.color = Color::srgba(
                    r + (tr - r) * s,
                    g + (tg - g) * s,
                    b + (tb - b) * s,
                    a,
                );
            }
        }
    }
}
