pub mod breathe;
pub mod collapse;
pub mod glitch;
pub mod gravity;
pub mod jitter;
pub mod ripple;
pub mod scatter;
pub mod slash;
pub mod wave;

use bevy::prelude::*;

use crate::grid::{BaseTransform, TerminalCell};

/// A rectangle in grid coordinates.
#[derive(Clone, Debug)]
pub struct GridRect {
    pub col: u16,
    pub row: u16,
    pub width: u16,
    pub height: u16,
}

impl GridRect {
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.col
            && col < self.col + self.width
            && row >= self.row
            && row < self.row + self.height
    }
}

/// Defines which cells an effect targets using include/exclude logic.
///
/// - `include`: union of rects to target. If empty, targets all cells.
/// - `exclude`: union of rects to skip (takes priority over include).
#[derive(Component, Clone, Debug)]
pub struct EffectRegion {
    pub include: Vec<GridRect>,
    pub exclude: Vec<GridRect>,
}

impl EffectRegion {
    /// Check if a cell at (col, row) is within this effect region.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        // Check excludes first (they take priority)
        for rect in &self.exclude {
            if rect.contains(col, row) {
                return false;
            }
        }

        // If no include rects, everything (not excluded) is included
        if self.include.is_empty() {
            return true;
        }

        // Otherwise check if in any include rect
        for rect in &self.include {
            if rect.contains(col, row) {
                return true;
            }
        }

        false
    }

    /// Create an EffectRegion that covers the full screen.
    pub fn full_screen(cols: u16, rows: u16) -> Self {
        Self {
            include: vec![GridRect {
                col: 0,
                row: 0,
                width: cols,
                height: rows,
            }],
            exclude: vec![],
        }
    }

    /// Create an EffectRegion that targets everything (empty include = all).
    pub fn all() -> Self {
        Self {
            include: vec![],
            exclude: vec![],
        }
    }
}

/// System that resets all cell transforms to their base positions each frame.
/// This runs before effects so they can additively modify transforms.
pub fn reset_transforms(
    mut query: Query<(&BaseTransform, &mut Transform), With<TerminalCell>>,
) {
    for (base, mut transform) in query.iter_mut() {
        transform.translation = base.translation;
        transform.rotation = base.rotation;
        transform.scale = base.scale;
    }
}

/// Deterministic xor-shift hash for procedural effects (Glitch, Jitter).
/// Avoids pulling in a `rand` dependency.
pub fn simple_hash(a: u32, b: u32) -> u32 {
    let mut h = a.wrapping_mul(2654435761).wrapping_add(b.wrapping_mul(2246822519));
    h ^= h >> 16;
    h = h.wrapping_mul(2246822519);
    h ^= h >> 13;
    h = h.wrapping_mul(3266489917);
    h ^= h >> 16;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_rect_contains() {
        let rect = GridRect { col: 5, row: 10, width: 3, height: 2 };
        assert!(rect.contains(5, 10));
        assert!(rect.contains(7, 11));
        assert!(!rect.contains(8, 10));
        assert!(!rect.contains(5, 12));
        assert!(!rect.contains(4, 10));
    }

    #[test]
    fn test_effect_region_include_exclude() {
        let region = EffectRegion {
            include: vec![GridRect { col: 0, row: 0, width: 10, height: 10 }],
            exclude: vec![GridRect { col: 3, row: 3, width: 2, height: 2 }],
        };

        assert!(region.contains(0, 0));
        assert!(region.contains(9, 9));
        assert!(!region.contains(3, 3)); // excluded
        assert!(!region.contains(4, 4)); // excluded
        assert!(region.contains(5, 5));
        assert!(!region.contains(10, 10)); // outside include
    }

    #[test]
    fn test_effect_region_empty_include() {
        let region = EffectRegion::all();
        assert!(region.contains(0, 0));
        assert!(region.contains(100, 100));
    }
}
