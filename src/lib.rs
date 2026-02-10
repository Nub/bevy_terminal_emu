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

/// The embedded default font (JetBrains Mono Regular).
const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

/// Source of font data for the terminal.
#[derive(Clone, Debug)]
pub enum FontSource {
    /// Use the embedded JetBrains Mono font (default).
    Default,
    /// Use custom font bytes loaded from a file or other source.
    Custom(Vec<u8>),
}

impl FontSource {
    /// Load a font from a file path.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Self {
        let bytes = std::fs::read(path.as_ref())
            .unwrap_or_else(|e| panic!("Failed to read font file {:?}: {}", path.as_ref(), e));
        FontSource::Custom(bytes)
    }

    /// Get the font bytes.
    pub fn bytes(&self) -> &[u8] {
        match self {
            FontSource::Default => DEFAULT_FONT_BYTES,
            FontSource::Custom(bytes) => bytes,
        }
    }
}

pub mod prelude {
    pub use crate::atlas::FontAtlasResource;
    pub use crate::backend::BevyBackend;
    pub use crate::effects::breathe::Breathe;
    pub use crate::effects::bubbly::Bubbly;
    pub use crate::effects::collapse::Collapse;
    pub use crate::effects::explode::Explode;
    pub use crate::effects::glitch::Glitch;
    pub use crate::effects::glow::Glow;
    pub use crate::effects::gravity::{CellVelocity, Gravity};
    pub use crate::effects::jitter::Jitter;
    pub use crate::effects::rainbow::Rainbow;
    pub use crate::effects::ripple::Ripple;
    pub use crate::effects::scatter::Scatter;
    pub use crate::effects::shiny::Shiny;
    pub use crate::effects::slash::Slash;
    pub use crate::effects::wave::Wave;
    pub use crate::effects::{EffectRegion, GridRect};
    pub use crate::grid::{
        BackgroundSprite, BaseTransform, CellEntityIndex, CellStyle, ForegroundSprite,
        GridPosition, TerminalCell,
    };
    pub use crate::input::TerminalInputQueue;
    pub use crate::{
        FontSource, TerminalConfig, TerminalEmuPlugin, TerminalLayout, TerminalResource,
        TerminalSet,
    };
}

/// Configuration for the terminal grid.
#[derive(Resource, Clone, Debug)]
pub struct TerminalConfig {
    /// Number of columns in the terminal.
    pub columns: u16,
    /// Number of rows in the terminal.
    pub rows: u16,
    /// Font size for glyph rasterization.
    pub font_size: f32,
    /// Font to use for glyph rasterization.
    pub font: FontSource,
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
            font_size: 20.0,
            font: FontSource::Default,
            default_fg: Color::srgb(0.9, 0.9, 0.9),
            default_bg: Color::srgb(0.1, 0.1, 0.1),
        }
    }
}

/// Derived layout properties computed from font metrics and terminal dimensions.
/// Created automatically by the plugin — do not construct manually.
#[derive(Resource, Clone, Debug)]
pub struct TerminalLayout {
    /// Width of each cell in pixels (derived from font advance width).
    pub cell_width: f32,
    /// Height of each cell in pixels (derived from font line height).
    pub cell_height: f32,
    /// World-space origin (top-left corner of the grid), centered on screen.
    pub origin: Vec2,
}

impl TerminalLayout {
    /// Sprite size with a small overlap to eliminate sub-pixel gaps between cells.
    pub fn sprite_size(&self) -> Vec2 {
        Vec2::new(self.cell_width + 0.5, self.cell_height + 0.5)
    }

    /// Compute layout from config using font metrics.
    pub fn from_config(config: &TerminalConfig) -> Self {
        let (cell_width, cell_height) = atlas::compute_cell_size(config.font.bytes(), config.font_size);
        Self {
            cell_width,
            cell_height,
            origin: Vec2::new(
                -(config.columns as f32 * cell_width) / 2.0,
                (config.rows as f32 * cell_height) / 2.0,
            ),
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
        let layout = TerminalLayout::from_config(&config);
        let backend = BevyBackend::new(config.columns, config.rows);
        let terminal = ratatui::Terminal::new(backend).expect("Failed to create ratatui terminal");
        let terminal_resource = TerminalResource(Arc::new(Mutex::new(terminal)));

        app.insert_resource(config)
            .insert_resource(layout)
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
                (
                    atlas::expand_font_atlas,
                    atlas::rebuild_font_atlas,
                    sync::sync_buffer_to_entities,
                )
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
                    effects::bubbly::bubbly_system,
                    effects::collapse::collapse_system,
                    effects::explode::explode_system,
                    effects::glitch::glitch_system,
                    effects::glow::glow_system,
                    effects::gravity::gravity_system,
                    effects::jitter::jitter_system,
                    effects::rainbow::rainbow_system,
                    effects::ripple::ripple_system,
                    effects::scatter::scatter_system,
                    effects::shiny::shiny_system,
                    effects::slash::slash_system,
                    effects::wave::wave_system,
                )
                    .in_set(TerminalSet::Effects),
            );
    }
}
