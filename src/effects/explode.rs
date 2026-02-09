use bevy::prelude::*;

use super::{simple_hash, EffectRegion};
use crate::grid::{GridPosition, TerminalCell};
use crate::TerminalConfig;

/// Chaotic explosion effect — cells fly outward with randomized velocity,
/// spin, and timing. Differentiates from Scatter (smooth/uniform) by giving
/// each cell unique random behaviour via `simple_hash`.
#[derive(Component, Clone, Debug)]
pub struct Explode {
    /// Origin column (grid coords).
    pub origin_col: f32,
    /// Origin row (grid coords).
    pub origin_row: f32,
    /// Base outward force in pixels per second.
    pub force: f32,
    /// Amount of velocity randomness (0.0 = uniform, 1.0 = very chaotic).
    pub chaos: f32,
    /// How long the effect has been running.
    pub elapsed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Whether the effect is currently active.
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

/// System that applies the explode effect to cell transforms.
pub fn explode_system(
    time: Res<Time>,
    config: Res<TerminalConfig>,
    mut effects: Query<(&mut Explode, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
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

            // Per-cell random values from hash
            let h1 = simple_hash(cell_id, 111);
            let h2 = simple_hash(cell_id, 222);
            let h3 = simple_hash(cell_id, 333);
            let h4 = simple_hash(cell_id, 444);

            // Normalize hashes to 0.0..1.0
            let r1 = (h1 % 10000) as f32 / 10000.0;
            let r2 = (h2 % 10000) as f32 / 10000.0;
            let r3 = (h3 % 10000) as f32 / 10000.0;
            let r4 = (h4 % 10000) as f32 / 10000.0;

            // Direction from origin to this cell (in pixel space)
            let dx = (pos.col as f32 - explode.origin_col) * config.cell_width;
            let dy = (pos.row as f32 - explode.origin_row) * -config.cell_height;
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);

            // Normalized direction
            let nx = dx / dist;
            let ny = dy / dist;

            // Per-cell random angle offset for chaotic spread
            let angle_offset = (r1 - 0.5) * std::f32::consts::PI * explode.chaos;
            let cos_off = angle_offset.cos();
            let sin_off = angle_offset.sin();
            let dir_x = nx * cos_off - ny * sin_off;
            let dir_y = nx * sin_off + ny * cos_off;

            // Per-cell speed variation
            let speed_mult = 1.0 + (r2 - 0.5) * explode.chaos;
            let displacement = explode.force * speed_mult * t;

            transform.translation.x += dir_x * displacement;
            transform.translation.y += dir_y * displacement;

            // Per-cell spin: random direction (CW or CCW) and speed
            let spin_dir = if r3 > 0.5 { 1.0 } else { -1.0 };
            let spin_speed = 2.0 + r3 * 6.0;
            let angle = spin_dir * spin_speed * t;
            transform.rotation = Quat::from_rotation_z(angle);

            // Shrink with random timing offset — some cells pop early, some late
            let timing_offset = (r4 - 0.5) * 0.3 * explode.chaos;
            let shrink_progress = (progress + timing_offset).clamp(0.0, 1.0);
            let scale = 1.0 - shrink_progress;
            transform.scale = Vec3::splat(scale.max(0.0));
        }
    }
}
