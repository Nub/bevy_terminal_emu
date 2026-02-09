use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{GridPosition, TerminalCell};

/// A collapse effect that makes cells fall with gravity, staggered by row.
#[derive(Component, Clone, Debug)]
pub struct Collapse {
    /// Gravity acceleration in pixels/sec².
    pub gravity: f32,
    /// How long the collapse has been running.
    pub elapsed: f32,
    /// Total duration before the collapse is complete.
    pub duration: f32,
    /// Stagger delay per row (seconds).
    pub stagger_per_row: f32,
    /// Whether the collapse is active.
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

/// System that applies the collapse effect to cell transforms.
pub fn collapse_system(
    time: Res<Time>,
    mut effects: Query<(&mut Collapse, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
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

            // Kinematic equation: displacement = 0.5 * g * t²
            let fall = 0.5 * collapse.gravity * t * t;

            // Apply downward (negative Y in Bevy 2D)
            transform.translation.y -= fall;
        }
    }
}
