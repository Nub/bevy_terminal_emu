use bevy::prelude::*;

use super::{simple_hash, EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};
use crate::TerminalLayout;

#[derive(Component, Clone, Debug)]
pub struct Explode {
    pub origin_col: f32,
    pub origin_row: f32,
    pub force: f32,
    pub chaos: f32,
    pub elapsed: f32,
    pub duration: f32,
    pub active: bool,
}

impl Default for Explode {
    fn default() -> Self {
        Self {
            origin_col: 40.0,
            origin_row: 12.0,
            force: 200.0,
            chaos: 0.5,
            elapsed: 0.0,
            duration: 2.5,
            active: true,
        }
    }
}

pub fn explode_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    layout: Res<TerminalLayout<T>>,
    mut effects: Query<(&mut Explode, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
) {
    for (mut explode, region) in effects.iter_mut() {
        if !explode.active {
            continue;
        }

        explode.elapsed += time.delta_secs();

        if explode.elapsed > explode.duration {
            explode.active = false;
            continue;
        }

        let t = explode.elapsed;
        let progress = t / explode.duration;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let cell_id = pos.col as u32 * 1000 + pos.row as u32;

            let h1 = simple_hash(cell_id, 111);
            let h2 = simple_hash(cell_id, 222);
            let h3 = simple_hash(cell_id, 333);
            let h4 = simple_hash(cell_id, 444);

            let r1 = (h1 % 10000) as f32 / 10000.0;
            let r2 = (h2 % 10000) as f32 / 10000.0;
            let r3 = (h3 % 10000) as f32 / 10000.0;
            let r4 = (h4 % 10000) as f32 / 10000.0;

            let dx = (pos.col as f32 - explode.origin_col) * layout.cell_width;
            let dy = (pos.row as f32 - explode.origin_row) * -layout.cell_height;
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);

            let nx = dx / dist;
            let ny = dy / dist;

            let angle_offset = (r1 - 0.5) * std::f32::consts::PI * explode.chaos;
            let cos_off = angle_offset.cos();
            let sin_off = angle_offset.sin();
            let dir_x = nx * cos_off - ny * sin_off;
            let dir_y = nx * sin_off + ny * cos_off;

            let speed_mult = 1.0 + (r2 - 0.5) * explode.chaos;
            let displacement = explode.force * speed_mult * t;

            transform.translation.x += dir_x * displacement;
            transform.translation.y += dir_y * displacement;

            let spin_dir = if r3 > 0.5 { 1.0 } else { -1.0 };
            let spin_speed = 2.0 + r3 * 6.0;
            let angle = spin_dir * spin_speed * t;
            transform.rotation = Quat::from_rotation_z(angle);

            let timing_offset = (r4 - 0.5) * 0.3 * explode.chaos;
            let shrink_progress = (progress + timing_offset).clamp(0.0, 1.0);
            let scale = 1.0 - shrink_progress;
            transform.scale = Vec3::splat(scale.max(0.0));
        }
    }
}
