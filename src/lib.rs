pub mod atlas;
pub mod backend;
pub mod color;
pub mod effects;
pub mod grid;
pub mod input;
pub mod sync;

use std::sync::{Arc, Mutex};

use bevy::color::Color;
use bevy::prelude::*;

use backend::BevyBackend;
use input::TerminalInputQueue;
use sync::SyncGeneration;

pub mod prelude {
    pub use crate::atlas::FontAtlasResource;
    pub use crate::backend::BevyBackend;
    pub use crate::effects::breathe::Breathe;
    pub use crate::effects::collapse::Collapse;
    pub use crate::effects::explode::Explode;
    pub use crate::effects::glitch::Glitch;
    pub use crate::effects::gravity::{CellVelocity, Gravity};
    pub use crate::effects::jitter::Jitter;
    pub use crate::effects::ripple::Ripple;
    pub use crate::effects::scatter::Scatter;
    pub use crate::effects::slash::Slash;
    pub use crate::effects::wave::Wave;
    pub use crate::effects::{EffectRegion, GridRect};
    pub use crate::grid::{
        BackgroundSprite, BaseTransform, CellEntityIndex, CellStyle, ForegroundSprite,
        GridPosition, TerminalCell,
    };
    pub use crate::input::TerminalInputQueue;
    pub use crate::{TerminalConfig, TerminalEmuPlugin, TerminalResource, TerminalSet};
}

/// Configuration for the terminal grid.
#[derive(Resource, Clone, Debug)]
pub struct TerminalConfig {
    /// Number of columns in the terminal.
    pub columns: u16,
    /// Number of rows in the terminal.
    pub rows: u16,
    /// Width of each cell in pixels.
    pub cell_width: f32,
    /// Height of each cell in pixels.
    pub cell_height: f32,
    /// World-space origin (top-left corner of the grid).
    pub origin: Vec2,
    /// Font size for glyph rasterization.
    pub font_size: f32,
    /// Default foreground color.
    pub default_fg: Color,
    /// Default background color.
    pub default_bg: Color,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            columns: 80,
            rows: 24,
            cell_width: 10.0,
            cell_height: 20.0,
            origin: Vec2::new(-400.0, 240.0),
            font_size: 20.0,
            default_fg: Color::srgb(0.9, 0.9, 0.9),
            default_bg: Color::srgb(0.1, 0.1, 0.1),
        }
    }
}

/// Shared resource wrapping the ratatui Terminal<BevyBackend> in an Arc<Mutex<>>
/// for access from both the ratatui app tick and the Bevy sync system.
#[derive(Resource, Clone)]
pub struct TerminalResource(pub Arc<Mutex<ratatui::Terminal<BevyBackend>>>);

/// System sets for ordering terminal systems.
///
/// Usage: add custom systems to `TerminalSet::AppTick` for your ratatui draw logic,
/// or to `TerminalSet::Effects` for custom visual effects.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerminalSet {
    /// User's ratatui draw + input handling runs here.
    AppTick,
    /// Buffer → entity sync.
    Sync,
    /// Reset transforms to base positions.
    ResetTransforms,
    /// All effects run here. Add custom effect systems to this set.
    Effects,
}

/// The main plugin that sets up the terminal emulator.
pub struct TerminalEmuPlugin {
    pub config: TerminalConfig,
}

impl Default for TerminalEmuPlugin {
    fn default() -> Self {
        Self {
            config: TerminalConfig::default(),
        }
    }
}

impl Plugin for TerminalEmuPlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone();
        let backend = BevyBackend::new(config.columns, config.rows);
        let terminal = ratatui::Terminal::new(backend).expect("Failed to create ratatui terminal");
        let terminal_resource = TerminalResource(Arc::new(Mutex::new(terminal)));

        app.insert_resource(config)
            .insert_resource(terminal_resource)
            .insert_resource(TerminalInputQueue::default())
            .insert_resource(SyncGeneration::default())
            // Configure system set ordering
            .configure_sets(
                Update,
                (
                    TerminalSet::AppTick,
                    TerminalSet::Sync,
                    TerminalSet::ResetTransforms,
                    TerminalSet::Effects,
                )
                    .chain(),
            )
            // Startup: generate atlas, then spawn grid (chained because grid needs atlas)
            .add_systems(
                Startup,
                (atlas::generate_font_atlas, grid::spawn_grid).chain(),
            )
            // Update systems in their respective sets
            .add_systems(
                Update,
                input::forward_input.in_set(TerminalSet::AppTick),
            )
            .add_systems(
                Update,
                (atlas::rebuild_font_atlas, sync::sync_buffer_to_entities)
                    .chain()
                    .in_set(TerminalSet::Sync),
            )
            .add_systems(
                Update,
                effects::reset_transforms.in_set(TerminalSet::ResetTransforms),
            )
            .add_systems(
                Update,
                (
                    effects::breathe::breathe_system,
                    effects::collapse::collapse_system,
                    effects::explode::explode_system,
                    effects::glitch::glitch_system,
                    effects::gravity::gravity_system,
                    effects::jitter::jitter_system,
                    effects::ripple::ripple_system,
                    effects::scatter::scatter_system,
                    effects::slash::slash_system,
                    effects::wave::wave_system,
                )
                    .in_set(TerminalSet::Effects),
            );
    }
}
