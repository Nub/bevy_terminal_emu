use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CellVelocity {
    pub velocity: Vec2,
}

#[derive(Component, Clone, Debug)]
pub struct Gravity {
    pub acceleration: Vec2,
    pub damping: f32,
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

pub fn gravity_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    effects: Query<(&Gravity, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<
        (&GridPosition, &mut Transform, &mut CellVelocity),
        With<TerminalCell<T>>,
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

            vel.velocity += gravity.acceleration * dt;

            if gravity.damping > 0.0 {
                let damping_factor = (1.0 - gravity.damping).powf(dt);
                vel.velocity *= damping_factor;
            }

            transform.translation.x += vel.velocity.x * dt;
            transform.translation.y += vel.velocity.y * dt;
        }
    }
}
