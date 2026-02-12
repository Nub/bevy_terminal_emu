use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};

/// A ripple effect that displaces cells in a wave pattern from an origin point.
#[derive(Component, Clone, Debug)]
pub struct Ripple {
    /// Origin column of the ripple.
    pub origin_col: f32,
    /// Origin row of the ripple.
    pub origin_row: f32,
    /// Maximum displacement in pixels.
    pub amplitude: f32,
    /// Wavelength in grid cells.
    pub wavelength: f32,
    /// Speed of wave propagation (cells per second).
    pub speed: f32,
    /// Accumulated phase offset.
    pub phase: f32,
    /// Exponential damping factor (higher = faster falloff).
    pub damping: f32,
}

impl Default for Ripple {
    fn default() -> Self {
        Self {
            origin_col: 40.0,
            origin_row: 12.0,
            amplitude: 8.0,
            wavelength: 6.0,
            speed: 10.0,
            phase: 0.0,
            damping: 0.1,
        }
    }
}

/// System that applies the ripple effect to cell transforms.
pub fn ripple_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    mut effects: Query<(&mut Ripple, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
) {
    for (mut ripple, region) in effects.iter_mut() {
        ripple.phase += ripple.speed * time.delta_secs();

        let two_pi = std::f32::consts::TAU;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let dx = pos.col as f32 - ripple.origin_col;
            let dy = pos.row as f32 - ripple.origin_row;
            let distance = (dx * dx + dy * dy).sqrt();

            let wave = (two_pi * (distance / ripple.wavelength - ripple.phase)).sin();
            let decay = (-ripple.damping * distance).exp();
            let displacement = ripple.amplitude * wave * decay;

            transform.translation.y += displacement;
        }
    }
}
