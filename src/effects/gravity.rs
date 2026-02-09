use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{GridPosition, TerminalCell};

/// Per-cell velocity for the gravity effect.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CellVelocity {
    pub velocity: Vec2,
}

/// A gravity effect that applies continuous acceleration to cells.
#[derive(Component, Clone, Debug)]
pub struct Gravity {
    /// Acceleration vector in pixels/sec² (e.g., Vec2::new(0.0, -200.0) for downward).
    pub acceleration: Vec2,
    /// Velocity damping factor (0.0 = no damping, 1.0 = full damping per second).
    pub damping: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Gravity {
    fn default() -> Self {
        Self {
            acceleration: Vec2::new(0.0, -200.0),
            damping: 0.0,
            active: true,
        }
    }
}

/// System that applies per-cell gravity and velocity to transforms.
pub fn gravity_system(
    time: Res<Time>,
    effects: Query<(&Gravity, &EffectRegion)>,
    mut cells: Query<
        (&GridPosition, &mut Transform, &mut CellVelocity),
        With<TerminalCell>,
    >,
) {
    let dt = time.delta_secs();

    for (gravity, region) in effects.iter() {
        if !gravity.active {
            continue;
        }

        for (pos, mut transform, mut vel) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            // Apply acceleration
            vel.velocity += gravity.acceleration * dt;

            // Apply damping
            if gravity.damping > 0.0 {
                let damping_factor = (1.0 - gravity.damping).powf(dt);
                vel.velocity *= damping_factor;
            }

            // Apply velocity to transform
            transform.translation.x += vel.velocity.x * dt;
            transform.translation.y += vel.velocity.y * dt;
        }
    }
}
