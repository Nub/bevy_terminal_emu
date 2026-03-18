use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

/// GPU-side grid parameters for the terminal shader.
#[derive(Clone, Debug, ShaderType)]
pub struct GridParams {
    /// Number of columns in the terminal.
    pub columns: u32,
    /// Number of rows in the terminal.
    pub rows: u32,
    /// Cell width in pixels (logical).
    pub cell_width: f32,
    /// Cell height in pixels (logical).
    pub cell_height: f32,
    /// Number of columns in the font atlas grid.
    pub atlas_columns: u32,
    /// Total number of glyphs in the atlas.
    pub atlas_glyph_count: u32,
    /// Atlas cell width in texels (including padding stride).
    pub atlas_stride_w: f32,
    /// Atlas cell height in texels (including padding stride).
    pub atlas_stride_h: f32,
    /// Atlas cell width in texels (glyph area only, no padding).
    pub atlas_cell_w: f32,
    /// Atlas cell height in texels (glyph area only, no padding).
    pub atlas_cell_h: f32,
    /// Total atlas texture width in texels.
    pub atlas_tex_width: f32,
    /// Total atlas texture height in texels.
    pub atlas_tex_height: f32,
    /// Elapsed time in seconds (for effects).
    pub time: f32,
    /// Padding to align to 16 bytes.
    pub _pad: f32,
}

/// GPU-side effect parameters — aggregated from all active effect components.
///
/// All effects are encoded in a single uniform struct. Inactive effects have
/// their `*_active` field set to 0 and are skipped by the shader.
///
/// Region support: up to 8 include + 8 exclude rects. Each rect is a `vec4<u32>`
/// containing (col, row, width, height). Empty include = whole screen.
#[derive(Clone, Debug, ShaderType)]
pub struct TerminalEffectUniforms {
    // ── Region rects (must come first for alignment) ──
    pub include_rects: [UVec4; 8],
    pub exclude_rects: [UVec4; 8],
    pub include_count: u32,
    pub exclude_count: u32,

    // ── Wave ──
    pub wave_amplitude: f32,
    pub wave_wavelength: f32,
    pub wave_speed: f32,
    pub wave_horizontal: u32,
    pub wave_active: u32,

    // ── Jitter ──
    pub jitter_amplitude: f32,
    pub jitter_speed: f32,
    pub jitter_rotate: u32,
    pub jitter_max_rotation: f32,
    pub jitter_active: u32,

    // ── Glow ──
    pub glow_speed: f32,
    pub glow_intensity: f32,
    pub glow_spread: f32,
    pub glow_active: u32,

    // ── Rainbow ──
    pub rainbow_speed: f32,
    pub rainbow_saturation: f32,
    pub rainbow_lightness: f32,
    pub rainbow_spread: f32,
    pub rainbow_active: u32,

    // ── Breathe ──
    pub breathe_min_scale: f32,
    pub breathe_max_scale: f32,
    pub breathe_speed: f32,
    pub breathe_phase_spread: f32,
    pub breathe_active: u32,

    // ── Shiny ──
    pub shiny_speed: f32,
    pub shiny_width: f32,
    pub shiny_angle: f32,
    pub shiny_brightness: f32,
    pub shiny_active: u32,

    // ── Glitch ──
    pub glitch_max_offset: f32,
    pub glitch_intensity: f32,
    pub glitch_frequency: f32,
    pub glitch_active: u32,

    // ── Bubbly ──
    pub bubbly_speed: f32,
    pub bubbly_density: f32,
    pub bubbly_max_scale: f32,
    pub bubbly_active: u32,

    // ── Ripple ──
    pub ripple_origin_col: f32,
    pub ripple_origin_row: f32,
    pub ripple_amplitude: f32,
    pub ripple_wavelength: f32,
    pub ripple_speed: f32,
    pub ripple_phase: f32,
    pub ripple_damping: f32,
    pub ripple_active: u32,

    // ── Slash ──
    pub slash_elapsed: f32,
    pub slash_duration: f32,
    pub slash_amplitude: f32,
    pub slash_width: f32,
    pub slash_angle: f32,
    pub slash_active: u32,

    // ── Knock ──
    pub knock_angle: f32,
    pub knock_amplitude: f32,
    pub knock_deviation: f32,
    pub knock_rotation: f32,
    pub knock_elapsed: f32,
    pub knock_duration: f32,
    pub knock_active: u32,

    // ── Explode ──
    pub explode_origin_col: f32,
    pub explode_origin_row: f32,
    pub explode_force: f32,
    pub explode_chaos: f32,
    pub explode_elapsed: f32,
    pub explode_duration: f32,
    pub explode_active: u32,

    // ── Collapse ──
    pub collapse_gravity: f32,
    pub collapse_elapsed: f32,
    pub collapse_duration: f32,
    pub collapse_stagger_per_row: f32,
    pub collapse_active: u32,

    // ── Scatter ──
    pub scatter_origin_col: f32,
    pub scatter_origin_row: f32,
    pub scatter_speed: f32,
    pub scatter_elapsed: f32,
    pub scatter_duration: f32,
    pub scatter_spin: f32,
    pub scatter_active: u32,

    // Padding to 16-byte alignment
    pub _pad0: u32,
}

impl Default for TerminalEffectUniforms {
    fn default() -> Self {
        Self {
            include_rects: [UVec4::ZERO; 8],
            exclude_rects: [UVec4::ZERO; 8],
            include_count: 0,
            exclude_count: 0,
            wave_amplitude: 0.0,
            wave_wavelength: 1.0,
            wave_speed: 0.0,
            wave_horizontal: 0,
            wave_active: 0,
            jitter_amplitude: 0.0,
            jitter_speed: 0.0,
            jitter_rotate: 0,
            jitter_max_rotation: 0.0,
            jitter_active: 0,
            glow_speed: 0.0,
            glow_intensity: 0.0,
            glow_spread: 0.0,
            glow_active: 0,
            rainbow_speed: 0.0,
            rainbow_saturation: 0.0,
            rainbow_lightness: 0.0,
            rainbow_spread: 0.0,
            rainbow_active: 0,
            breathe_min_scale: 1.0,
            breathe_max_scale: 1.0,
            breathe_speed: 0.0,
            breathe_phase_spread: 0.0,
            breathe_active: 0,
            shiny_speed: 0.0,
            shiny_width: 0.0,
            shiny_angle: 0.0,
            shiny_brightness: 0.0,
            shiny_active: 0,
            glitch_max_offset: 0.0,
            glitch_intensity: 0.0,
            glitch_frequency: 0.0,
            glitch_active: 0,
            bubbly_speed: 0.0,
            bubbly_density: 0.0,
            bubbly_max_scale: 1.0,
            bubbly_active: 0,
            ripple_origin_col: 0.0,
            ripple_origin_row: 0.0,
            ripple_amplitude: 0.0,
            ripple_wavelength: 1.0,
            ripple_speed: 0.0,
            ripple_phase: 0.0,
            ripple_damping: 0.0,
            ripple_active: 0,
            slash_elapsed: 0.0,
            slash_duration: 1.0,
            slash_amplitude: 0.0,
            slash_width: 1.0,
            slash_angle: 0.0,
            slash_active: 0,
            knock_angle: 0.0,
            knock_amplitude: 0.0,
            knock_deviation: 0.0,
            knock_rotation: 0.0,
            knock_elapsed: 0.0,
            knock_duration: 1.0,
            knock_active: 0,
            explode_origin_col: 0.0,
            explode_origin_row: 0.0,
            explode_force: 0.0,
            explode_chaos: 0.0,
            explode_elapsed: 0.0,
            explode_duration: 1.0,
            explode_active: 0,
            collapse_gravity: 0.0,
            collapse_elapsed: 0.0,
            collapse_duration: 1.0,
            collapse_stagger_per_row: 0.0,
            collapse_active: 0,
            scatter_origin_col: 0.0,
            scatter_origin_row: 0.0,
            scatter_speed: 0.0,
            scatter_elapsed: 0.0,
            scatter_duration: 1.0,
            scatter_spin: 0.0,
            scatter_active: 0,
            _pad0: 0,
        }
    }
}

/// Custom Material2d for GPU-rendered terminals.
///
/// A single quad with this material replaces the entire entity-per-cell grid.
/// The fragment shader reads cell data from a data texture and samples the font atlas.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerminalMaterial {
    /// Cell data texture (Rgba32Uint): each pixel stores one cell's data.
    /// R = glyph_index, G = modifiers, B = fg_packed_rgba8, A = bg_packed_rgba8
    #[texture(0, sample_type = "u_int", dimension = "2d")]
    pub cell_data: Handle<Image>,

    /// Font atlas texture (standard RGBA).
    #[texture(1)]
    #[sampler(2)]
    pub font_atlas: Handle<Image>,

    /// Grid parameters uniform.
    #[uniform(3)]
    pub grid_params: GridParams,

    /// Effect parameters uniform.
    #[uniform(4)]
    pub effect_params: TerminalEffectUniforms,
}

impl Material2d for TerminalMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_terminal_emu/shaders/terminal.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Resource holding the single quad entity for a terminal instance.
#[derive(Resource)]
pub struct TerminalQuadEntity<T: 'static + Send + Sync> {
    pub entity: Entity,
    pub material_handle: Handle<TerminalMaterial>,
    pub cell_data_image: Handle<Image>,
    _marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync> TerminalQuadEntity<T> {
    pub fn new(
        entity: Entity,
        material_handle: Handle<TerminalMaterial>,
        cell_data_image: Handle<Image>,
    ) -> Self {
        Self {
            entity,
            material_handle,
            cell_data_image,
            _marker: PhantomData,
        }
    }
}

/// Pack an sRGB Color into a u32 (R in lowest byte, A in highest).
pub fn pack_color_rgba8(color: Color) -> u32 {
    let srgba = color.to_srgba();
    let [r, g, b, a] = srgba.to_f32_array();
    let r8 = (r * 255.0).round() as u32 & 0xFF;
    let g8 = (g * 255.0).round() as u32 & 0xFF;
    let b8 = (b * 255.0).round() as u32 & 0xFF;
    let a8 = (a * 255.0).round() as u32 & 0xFF;
    r8 | (g8 << 8) | (b8 << 16) | (a8 << 24)
}
