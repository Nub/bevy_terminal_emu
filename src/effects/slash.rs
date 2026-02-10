use bevy::prelude::*;

use super::EffectRegion;
use crate::grid::{GridPosition, TerminalCell};
use crate::{TerminalConfig, TerminalLayout};

/// Diagonal swipe effect — a line sweeps across the screen and cells near it
/// get displaced perpendicular to the slash direction.
///
/// One-shot: the slash line travels from one corner to the opposite, displacing
/// cells as it passes. Cells spring back after the line moves on.
#[derive(Component, Clone, Debug)]
pub struct Slash {
    /// How long the slash has been running.
    pub elapsed: f32,
    /// Total duration of one sweep.
    pub duration: f32,
    /// Maximum perpendicular displacement in pixels.
    pub amplitude: f32,
    /// Width of the displacement band (in grid cells).
    pub width: f32,
    /// Angle of the slash line in radians (0 = horizontal, PI/4 = diagonal).
    pub angle: f32,
    /// Whether the effect is currently active.
    pub active: bool,
}

impl Default for Slash {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            duration: 1.5,
            amplitude: 25.0,
            width: 6.0,
            angle: std::f32::consts::FRAC_PI_4,
            active: true,
        }
    }
}

/// System that applies the slash effect to cell transforms.
pub fn slash_system(
    time: Res<Time>,
    config: Res<TerminalConfig>,
    layout: Res<TerminalLayout>,
    mut effects: Query<(&mut Slash, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    for (mut slash, region) in effects.iter_mut() {
        if !slash.active {
            continue;
        }

        slash.elapsed += time.delta_secs();

        if slash.elapsed > slash.duration {
            slash.active = false;
            continue;
        }

        let progress = slash.elapsed / slash.duration; // 0.0 -> 1.0

        // Slash direction vector (unit)
        let dir_x = slash.angle.cos();
        let dir_y = slash.angle.sin();

        // Perpendicular direction (for displacement)
        let perp_x = -dir_y;
        let perp_y = dir_x;

        // The diagonal extent of the grid in the slash direction
        let cols = config.columns as f32;
        let rows = config.rows as f32;

        // Project all four corners onto the slash direction to find the range
        let corners = [
            (0.0, 0.0),
            (cols, 0.0),
            (0.0, rows),
            (cols, rows),
        ];
        let mut min_proj = f32::MAX;
        let mut max_proj = f32::MIN;
        for (cx, cy) in corners {
            let p = cx * dir_x + cy * dir_y;
            min_proj = min_proj.min(p);
            max_proj = max_proj.max(p);
        }

        // Current position of the slash line along its travel
        let sweep_range = max_proj - min_proj + slash.width * 2.0;
        let line_pos = min_proj - slash.width + progress * sweep_range;

        let half_width = slash.width / 2.0;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            // Project cell position onto slash direction
            let cell_proj = pos.col as f32 * dir_x + pos.row as f32 * dir_y;

            // Distance from cell to the current slash line position
            let dist = (cell_proj - line_pos).abs();

            if dist < half_width {
                // Smooth falloff: 1.0 at center, 0.0 at edges
                let t = 1.0 - dist / half_width;
                let strength = t * t * (3.0 - 2.0 * t); // smoothstep

                // Displace perpendicular to the slash line
                let disp = slash.amplitude * strength;
                transform.translation.x += perp_x * disp * layout.cell_width;
                transform.translation.y += perp_y * disp * -layout.cell_height;

                // Slight rotation for dramatic effect
                let rotation = 0.15 * strength;
                transform.rotation = Quat::from_rotation_z(rotation);

                // Slight scale bump at the slash line
                let scale = 1.0 + 0.3 * strength;
                transform.scale = Vec3::splat(scale);
            }
        }
    }
}
