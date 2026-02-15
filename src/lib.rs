pub mod atlas;
pub mod backend;
pub mod color;
pub mod effects;
pub mod grid;
pub mod input;
pub mod sync;

use std::marker::PhantomData;
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
    pub use crate::effects::knock::Knock;
    pub use crate::effects::gravity::{CellVelocity, Gravity};
    pub use crate::effects::jitter::Jitter;
    pub use crate::effects::rainbow::Rainbow;
    pub use crate::effects::ripple::Ripple;
    pub use crate::effects::scatter::Scatter;
    pub use crate::effects::shiny::Shiny;
    pub use crate::effects::slash::Slash;
    pub use crate::effects::tint::Tint;
    pub use crate::effects::wave::Wave;
    pub use crate::effects::{EffectRegion, GridRect, TargetTerminal};
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
pub struct TerminalConfig<T: 'static + Send + Sync> {
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
    /// Custom origin (top-left of grid) in world space. If None, centered on screen.
    pub origin_override: Option<Vec2>,
    /// Z depth for cell entities (default: 0.0).
    pub z_layer: f32,
    /// Whether this terminal receives keyboard input (default: true).
    pub receive_input: bool,
    /// Override cell dimensions instead of deriving from font metrics.
    /// When set, `TerminalLayout` uses these exact values (no ceil rounding).
    /// The atlas is still rasterized at `font_size` — this only affects grid spacing.
    pub cell_size_override: Option<Vec2>,
    /// Render layer for the terminal grid entities.
    /// When set, `RenderLayers::layer(n)` is inserted on every cell entity
    /// so that an off-screen camera on the same layer can capture them.
    pub render_layer: Option<u8>,
    #[doc(hidden)]
    pub _marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync> Default for TerminalConfig<T> {
    fn default() -> Self {
        Self {
            columns: 80,
            rows: 24,
            font_size: 20.0,
            font: FontSource::Default,
            default_fg: Color::srgb(0.9, 0.9, 0.9),
            default_bg: Color::srgb(0.1, 0.1, 0.1),
            origin_override: None,
            z_layer: 0.0,
            receive_input: true,
            cell_size_override: None,
            render_layer: None,
            _marker: PhantomData,
        }
    }
}

/// Derived layout properties computed from font metrics and terminal dimensions.
/// Created automatically by the plugin — do not construct manually.
#[derive(Resource, Clone, Debug)]
pub struct TerminalLayout<T: 'static + Send + Sync> {
    /// Width of each cell in pixels (derived from font advance width).
    pub cell_width: f32,
    /// Height of each cell in pixels (derived from font line height).
    pub cell_height: f32,
    /// World-space origin (top-left corner of the grid), centered on screen.
    pub origin: Vec2,
    #[doc(hidden)]
    pub _marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync> TerminalLayout<T> {
    /// Background sprite size with a small overlap to fill sub-pixel gaps.
    /// Foreground sprites should use exact cell dimensions to avoid clipping.
    pub fn bg_sprite_size(&self) -> Vec2 {
        Vec2::new(self.cell_width + 0.5, self.cell_height + 0.5)
    }

    /// Compute layout from config using font metrics.
    ///
    /// Cell dimensions are ceil'd to integer pixels so that foreground sprites
    /// can render at an exact 1:1 pixel ratio with the atlas tile — no scaling,
    /// no nearest-filter pixel loss.
    pub fn from_config(config: &TerminalConfig<T>) -> Self {
        let (cell_width, cell_height) = if let Some(override_size) = config.cell_size_override {
            (override_size.x, override_size.y)
        } else {
            let (cw, ch) = atlas::compute_cell_size(config.font.bytes(), config.font_size);
            (cw.ceil(), ch.ceil())
        };
        let origin = config.origin_override.unwrap_or_else(|| {
            Vec2::new(
                -(config.columns as f32 * cell_width) / 2.0,
                (config.rows as f32 * cell_height) / 2.0,
            )
        });
        Self {
            cell_width,
            cell_height,
            origin,
            _marker: PhantomData,
        }
    }
}

/// Shared resource wrapping the ratatui Terminal<BevyBackend> in an Arc<Mutex<>>
/// for access from both the ratatui app tick and the Bevy sync system.
#[derive(Resource, Clone)]
pub struct TerminalResource<T: 'static + Send + Sync>(
    pub Arc<Mutex<ratatui::Terminal<BevyBackend>>>,
    PhantomData<T>,
);

impl<T: 'static + Send + Sync> TerminalResource<T> {
    pub fn new(terminal: ratatui::Terminal<BevyBackend>) -> Self {
        Self(Arc::new(Mutex::new(terminal)), PhantomData)
    }
}

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
/// Generic over a marker type `T` to support multiple independent terminal instances.
pub struct TerminalEmuPlugin<T: 'static + Send + Sync> {
    pub config: TerminalConfig<T>,
}

impl<T: 'static + Send + Sync> Default for TerminalEmuPlugin<T> {
    fn default() -> Self {
        Self {
            config: TerminalConfig::default(),
        }
    }
}

impl<T: 'static + Send + Sync> Plugin for TerminalEmuPlugin<T> {
    fn build(&self, app: &mut App) {
        let config = clone_config(&self.config);
        let layout = TerminalLayout::from_config(&config);
        let backend = BevyBackend::new(config.columns, config.rows);
        let terminal = ratatui::Terminal::new(backend).expect("Failed to create ratatui terminal");
        let terminal_resource = TerminalResource::<T>::new(terminal);

        app.insert_resource(config)
            .insert_resource(layout)
            .insert_resource(terminal_resource)
            .insert_resource(TerminalInputQueue::<T>::default())
            .insert_resource(SyncGeneration::<T>::default());

        // Only configure system set ordering once (first plugin instance)
        if !app.world().contains_resource::<TerminalSetConfigured>() {
            app.insert_resource(TerminalSetConfigured);
            app.configure_sets(
                Update,
                (
                    TerminalSet::AppTick,
                    TerminalSet::Sync,
                    TerminalSet::ResetTransforms,
                    TerminalSet::Effects,
                )
                    .chain(),
            );
        }

        // Startup: generate atlas, then spawn grid (chained because grid needs atlas)
        app.add_systems(
            Startup,
            (atlas::generate_font_atlas::<T>, grid::spawn_grid::<T>).chain(),
        );

        // Update systems in their respective sets
        if self.config.receive_input {
            app.add_systems(
                Update,
                input::forward_input::<T>.in_set(TerminalSet::AppTick),
            );
        }

        app.add_systems(
            Update,
            (
                atlas::expand_font_atlas::<T>,
                atlas::rebuild_font_atlas::<T>,
                sync::sync_buffer_to_entities::<T>,
            )
                .chain()
                .in_set(TerminalSet::Sync),
        )
        .add_systems(
            Update,
            (
                effects::reset_transforms::<T>,
                effects::reset_colors::<T>,
            )
                .in_set(TerminalSet::ResetTransforms),
        )
        .add_systems(
            Update,
            (
                effects::breathe::breathe_system::<T>,
                effects::bubbly::bubbly_system::<T>,
                effects::collapse::collapse_system::<T>,
                effects::explode::explode_system::<T>,
                effects::glitch::glitch_system::<T>,
                effects::glow::glow_system::<T>,
                effects::gravity::gravity_system::<T>,
                effects::jitter::jitter_system::<T>,
                effects::knock::knock_system::<T>,
                effects::rainbow::rainbow_system::<T>,
                effects::ripple::ripple_system::<T>,
                effects::scatter::scatter_system::<T>,
                effects::shiny::shiny_system::<T>,
                effects::slash::slash_system::<T>,
                effects::tint::tint_system::<T>,
                effects::wave::wave_system::<T>,
            )
                .in_set(TerminalSet::Effects),
        );
    }
}

/// Marker resource to ensure TerminalSet is only configured once.
#[derive(Resource)]
struct TerminalSetConfigured;

/// Clone a TerminalConfig without requiring T: Clone (T is only PhantomData).
fn clone_config<T: 'static + Send + Sync>(c: &TerminalConfig<T>) -> TerminalConfig<T> {
    TerminalConfig {
        columns: c.columns,
        rows: c.rows,
        font_size: c.font_size,
        font: c.font.clone(),
        default_fg: c.default_fg,
        default_bg: c.default_bg,
        origin_override: c.origin_override,
        z_layer: c.z_layer,
        receive_input: c.receive_input,
        cell_size_override: c.cell_size_override,
        render_layer: c.render_layer,
        _marker: PhantomData,
    }
}
