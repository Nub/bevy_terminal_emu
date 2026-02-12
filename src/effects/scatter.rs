use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};
use crate::TerminalLayout;

/// Explosion effect that scatters cells outward from a center point.
///
/// One-shot: cells fly outward radially, shrinking and spinning over time.
#[derive(Component, Clone, Debug)]
pub struct Scatter {
    /// Origin column (grid coords).
    pub origin_col: f32,
    /// Origin row (grid coords).
    pub origin_row: f32,
    /// Outward speed in pixels per second.
    pub speed: f32,
    /// How long the scatter has been running.
    pub elapsed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Spin speed in radians per second.
    pub spin: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Scatter {
    fn default() -> Self {
        Self {
            origin_col: 40.0,
            origin_row: 12.0,
            speed: 150.0,
            elapsed: 0.0,
            duration: 3.0,
            spin: 3.0,
            active: true,
        }
    }
}

/// System that applies the scatter effect to cell transforms.
pub fn scatter_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    layout: Res<TerminalLayout<T>>,
    mut effects: Query<(&mut Scatter, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
) {
    for (mut scatter, region) in effects.iter_mut() {
        if !scatter.active {
            continue;
        }

        scatter.elapsed += time.delta_secs();

        if scatter.elapsed > scatter.duration {
            scatter.active = false;
            continue;
        }

        let t = scatter.elapsed;
        let progress = t / scatter.duration; // 0.0 -> 1.0

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            // Direction from origin to this cell (in pixel space)
            let dx = (pos.col as f32 - scatter.origin_col) * layout.cell_width;
            let dy = (pos.row as f32 - scatter.origin_row) * -layout.cell_height;
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);

            // Normalized direction
            let nx = dx / dist;
            let ny = dy / dist;

            // Radial displacement grows over time
            let displacement = scatter.speed * t;
            transform.translation.x += nx * displacement;
            transform.translation.y += ny * displacement;

            // Spin increases over time
            let angle = scatter.spin * t * (1.0 + dist * 0.001);
            transform.rotation = Quat::from_rotation_z(angle);

            // Scale shrinks as effect progresses
            let scale = 1.0 - progress * 0.8; // shrink to 0.2
            transform.scale = Vec3::splat(scale.max(0.0));
        }
    }
}
