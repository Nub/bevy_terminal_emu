use bevy::prelude::*;

use super::{simple_hash, EffectRegion};
use crate::grid::{GridPosition, TerminalCell};
use crate::TerminalLayout;

/// Blunt-impact knock effect — all cells in the region jolt in a uniform
/// direction (with slight per-cell deviation), then ease back to rest.
/// Simulates the feel of a heavy weapon strike.
#[derive(Component, Clone, Debug)]
pub struct Knock {
    /// Direction of the knock in radians.
    pub angle: f32,
    /// Maximum displacement in pixels.
    pub amplitude: f32,
    /// Per-cell angular deviation in radians (0 = perfectly uniform).
    pub deviation: f32,
    /// Per-cell rotation strength in radians at peak.
    pub rotation: f32,
    /// How long the effect has been running.
    pub elapsed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Knock {
    fn default() -> Self {
        Self {
            angle: 0.0,
            amplitude: 12.0,
            deviation: 0.3,
            rotation: 0.1,
            elapsed: 0.0,
            duration: 0.6,
            active: true,
        }
    }
}

/// System that applies the knock effect to cell transforms.
pub fn knock_system(
    time: Res<Time>,
    layout: Res<TerminalLayout>,
    mut effects: Query<(&mut Knock, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    for (mut knock, region) in effects.iter_mut() {
        if !knock.active {
            continue;
        }

        knock.elapsed += time.delta_secs();

        if knock.elapsed > knock.duration {
            knock.active = false;
            continue;
        }

        let progress = knock.elapsed / knock.duration;

        // Sharp onset, smooth settle: peaks at ~15% then decays
        // Using a damped impulse: t * exp(-decay * t) normalized
        let decay = 4.0;
        let raw = progress * (-decay * progress).exp();
        // Normalize so peak = 1.0 (peak is at 1/decay)
        let peak = (1.0 / decay) * (-1.0_f32).exp();
        let strength = raw / peak;

        let base_dx = knock.angle.cos();
        let base_dy = knock.angle.sin();

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let cell_id = pos.col as u32 * 1000 + pos.row as u32;

            // Per-cell slight deviation from the main knock direction
            let h1 = simple_hash(cell_id, 777);
            let h2 = simple_hash(cell_id, 888);
            let r1 = (h1 % 10000) as f32 / 10000.0; // 0..1
            let r2 = (h2 % 10000) as f32 / 10000.0; // 0..1

            let angle_dev = (r1 - 0.5) * 2.0 * knock.deviation;
            let cos_dev = angle_dev.cos();
            let sin_dev = angle_dev.sin();
            let dx = base_dx * cos_dev - base_dy * sin_dev;
            let dy = base_dx * sin_dev + base_dy * cos_dev;

            // Per-cell amplitude variation (±15%)
            let amp_mult = 0.85 + r2 * 0.3;

            let disp = knock.amplitude * amp_mult * strength;
            transform.translation.x += dx * disp * layout.cell_width;
            transform.translation.y += dy * disp * -layout.cell_height;

            // Slight rotation matching the knock direction
            let rot_dir = if r1 > 0.5 { 1.0 } else { -1.0 };
            let rot = knock.rotation * amp_mult * strength * rot_dir;
            transform.rotation *= Quat::from_rotation_z(rot);
        }
    }
}
