pub mod atlas;
pub mod backend;
pub mod color;
pub mod effects;
pub mod input;
pub mod render;
pub mod render_effects;
pub mod render_quad;
pub mod render_sync;

use std::marker::PhantomData;

use bevy::asset::embedded_asset;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;

use backend::BevyBackend;
use input::TerminalInputQueue;
use render::TerminalMaterial;
use render_sync::SyncGeneration;

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
    pub use crate::effects::jitter::Jitter;
    pub use crate::effects::knock::Knock;
    pub use crate::effects::rainbow::Rainbow;
    pub use crate::effects::ripple::Ripple;
    pub use crate::effects::scatter::Scatter;
    pub use crate::effects::shiny::Shiny;
    pub use crate::effects::slash::Slash;
    pub use crate::effects::wave::Wave;
    pub use crate::effects::{EffectRegion, GridRect, TargetTerminal};
    pub use crate::input::TerminalInputQueue;
    pub use crate::render::{TerminalEffectUniforms, TerminalQuadEntity};
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

/// Resource wrapping the ratatui Terminal<BevyBackend>.
///
/// Lock-free: Bevy's resource system handles exclusive access via `ResMut`.
#[derive(Resource)]
pub struct TerminalResource<T: 'static + Send + Sync>(
    pub ratatui::Terminal<BevyBackend>,
    PhantomData<T>,
);

impl<T: 'static + Send + Sync> TerminalResource<T> {
    pub fn new(terminal: ratatui::Terminal<BevyBackend>) -> Self {
        Self(terminal, PhantomData)
    }
}

/// System sets for ordering terminal systems.
///
/// Usage: add custom systems to `TerminalSet::AppTick` for your ratatui draw logic.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerminalSet {
    /// User's ratatui draw + input handling runs here.
    AppTick,
    /// Buffer → GPU texture sync.
    Sync,
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

        // Register Material2d plugin and embed shader once (shared across all terminal instances)
        if !app.world().contains_resource::<TerminalMaterialPluginRegistered>() {
            app.insert_resource(TerminalMaterialPluginRegistered);
            embedded_asset!(app, "shaders/terminal.wgsl");
            app.add_plugins(Material2dPlugin::<TerminalMaterial>::default());
        }

        // Only configure system set ordering once (first plugin instance)
        if !app.world().contains_resource::<TerminalSetConfigured>() {
            app.insert_resource(TerminalSetConfigured);
            app.configure_sets(
                Update,
                (TerminalSet::AppTick, TerminalSet::Sync).chain(),
            );
        }

        // Startup: generate atlas, then spawn quad (chained because quad needs atlas)
        app.add_systems(
            Startup,
            (
                atlas::generate_font_atlas::<T>,
                render_quad::spawn_terminal_quad::<T>,
            )
                .chain(),
        );

        // Update systems
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
                render_sync::sync_buffer_to_texture::<T>,
                render_sync::handle_atlas_update::<T>,
                render_effects::aggregate_effects::<T>,
            )
                .chain()
                .in_set(TerminalSet::Sync),
        );
    }
}

/// Marker resource to ensure TerminalSet is only configured once.
#[derive(Resource)]
struct TerminalSetConfigured;

/// Marker resource to ensure Material2dPlugin is only registered once.
#[derive(Resource)]
struct TerminalMaterialPluginRegistered;

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
        _marker: PhantomData,
    }
}
