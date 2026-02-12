use bevy::prelude::*;

use super::{EffectRegion, TargetTerminal};
use crate::grid::{GridPosition, TerminalCell};
use crate::TerminalLayout;

/// Slash effect — a blade cuts across the region along a line, splitting cells
/// apart perpendicular to the cut as it passes, like cutting cloth.
///
/// The cut animates from one end of the line to the other. Cells behind the
/// blade's wavefront get displaced outward (perpendicular to the cut).
/// Displacement is strongest at the center of the line and fades toward edges.
/// After the blade finishes its pass, the split eases closed.
#[derive(Component, Clone, Debug)]
pub struct Slash {
    /// How long the slash has been running.
    pub elapsed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Maximum perpendicular displacement in pixels.
    pub amplitude: f32,
    /// Width of the displacement band (in grid cells) on each side of the line.
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
            duration: 0.5,
            amplitude: 8.0,
            width: 4.0,
            angle: std::f32::consts::FRAC_PI_4,
            active: true,
        }
    }
}

/// System that applies the slash effect to cell transforms.
pub fn slash_system<T: 'static + Send + Sync>(
    time: Res<Time>,
    layout: Res<TerminalLayout<T>>,
    mut effects: Query<(&mut Slash, &EffectRegion), With<TargetTerminal<T>>>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell<T>>>,
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

        // Phase 1 (0.0–0.5): blade travels along the line, opening the cut
        // Phase 2 (0.5–1.0): cut eases closed
        let cut_phase = progress.min(0.5) / 0.5; // 0→1 during phase 1, stays 1 in phase 2
        let close_phase = if progress > 0.5 {
            let t = (progress - 0.5) / 0.5;
            1.0 - t * t // ease out
        } else {
            1.0
        };

        // Direction along the cut line
        let along_x = slash.angle.cos();
        let along_y = slash.angle.sin();

        // Perpendicular to the cut (displacement direction)
        let perp_x = -along_y;
        let perp_y = along_x;

        // Find region center and half-extent along the cut direction
        let (center_col, center_row, half_along, _half_perp) =
            if let Some(rect) = region.include.first() {
                let cx = rect.col as f32 + rect.width as f32 / 2.0;
                let cy = rect.row as f32 + rect.height as f32 / 2.0;
                let ex = rect.width as f32 / 2.0;
                let ey = rect.height as f32 / 2.0;
                // Extent of the region projected onto each axis
                let along_ext = (ex * along_x).abs() + (ey * along_y).abs();
                let perp_ext = (ex * perp_x).abs() + (ey * perp_y).abs();
                (cx, cy, along_ext, perp_ext)
            } else {
                (75.0, 25.0, 50.0, 25.0)
            };

        // Project center onto both axes
        let center_along = center_col * along_x + center_row * along_y;
        let center_perp = center_col * perp_x + center_row * perp_y;

        // The blade wavefront: starts at one end, travels to the other
        let blade_start = center_along - half_along;
        let blade_end = center_along + half_along;
        let blade_pos = blade_start + cut_phase * (blade_end - blade_start);

        let half_width = slash.width / 2.0;

        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            let col = pos.col as f32;
            let row = pos.row as f32;

            // Project cell onto the along-line and perpendicular axes
            let cell_along = col * along_x + row * along_y;
            let cell_perp = col * perp_x + row * perp_y;

            // Perpendicular distance from the cut line
            let perp_dist = (cell_perp - center_perp).abs();

            if perp_dist >= half_width {
                continue;
            }

            // Has the blade passed this cell?
            if cell_along > blade_pos {
                continue;
            }

            // Perpendicular falloff: 1.0 at the line, 0.0 at band edges
            let perp_t = 1.0 - perp_dist / half_width;
            let perp_strength = perp_t * perp_t * (3.0 - 2.0 * perp_t); // smoothstep

            // Along-line falloff: strongest at center, fades to edges
            let along_dist = (cell_along - center_along).abs();
            let along_t = (1.0 - along_dist / half_along.max(1.0)).max(0.0);
            let along_strength = along_t; // linear falloff along the cut

            // Wavefront softness: cells very close to the blade tip get partial displacement
            let tip_dist = blade_pos - cell_along;
            let tip_blend = (tip_dist / 2.0).min(1.0); // ramp over 2 cells

            let strength = perp_strength * along_strength * tip_blend * close_phase;

            // Which side of the cut line is this cell on?
            let side = if cell_perp >= center_perp { 1.0 } else { -1.0 };

            // Displace perpendicular to the cut (cells split apart)
            let disp = slash.amplitude * strength * side;
            transform.translation.x += perp_x * disp * layout.cell_width;
            transform.translation.y += perp_y * disp * -layout.cell_height;

            // Slight rotation following the cut
            let rotation = 0.08 * strength * side;
            transform.rotation *= Quat::from_rotation_z(rotation);

            // Subtle scale bump near the cut line
            let scale = 1.0 + 0.1 * strength;
            transform.scale *= Vec3::splat(scale);
        }
    }
}
