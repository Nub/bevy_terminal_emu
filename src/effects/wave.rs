use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{GridPosition, TerminalCell};

/// A simple sine wave effect that oscillates cells vertically.
#[derive(Component, Clone, Debug)]
pub struct Wave {
    /// Maximum displacement in pixels.
    pub amplitude: f32,
    /// Wavelength in grid columns.
    pub wavelength: f32,
    /// Speed of the wave (columns per second).
    pub speed: f32,
    /// Axis of wave propagation: if true, wave travels along rows; if false, along columns.
    pub horizontal: bool,
}

impl Default for Wave {
    fn default() -> Self {
        Self {
            amplitude: 5.0,
            wavelength: 8.0,
            speed: 4.0,
            horizontal: true,
        }
    }
}

/// System that applies the wave effect to cell transforms.
pub fn wave_system(
    time: Res<Time>,
    effects: Query<(&Wave, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    let t = time.elapsed_secs();

    for (wave, region) in effects.iter() {
        let two_pi = std::f32::consts::TAU;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let position_along = if wave.horizontal {
                pos.col as f32
            } else {
                pos.row as f32
            };

            let displacement =
                wave.amplitude * (two_pi * (position_along / wave.wavelength - wave.speed * t)).sin();

            transform.translation.y += displacement;
        }
    }
}
