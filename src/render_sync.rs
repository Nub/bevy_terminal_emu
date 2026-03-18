use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use ratatui::style::Modifier;
use std::marker::PhantomData;

use crate::atlas::FontAtlasResource;
use crate::color::{ratatui_bg_to_bevy, ratatui_fg_to_bevy};
use crate::render::{pack_color_rgba8, GridParams, TerminalMaterial, TerminalQuadEntity};
use crate::{TerminalConfig, TerminalResource};

/// Resource tracking the last synced generation to skip redundant updates.
#[derive(Resource)]
pub struct SyncGeneration<T: 'static + Send + Sync> {
    pub generation: u64,
    _marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync> Default for SyncGeneration<T> {
    fn default() -> Self {
        Self {
            generation: 0,
            _marker: PhantomData,
        }
    }
}

/// Create the initial cell data texture (Rgba32Uint, cols × rows).
pub fn create_cell_data_image(columns: u16, rows: u16) -> Image {
    let w = columns as u32;
    let h = rows as u32;
    // Rgba32Uint: 4 × u32 per pixel = 16 bytes per pixel
    let data = vec![0u8; (w * h * 16) as usize];
    let mut image = Image::new(
        Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba32Uint,
        RenderAssetUsages::default(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    image
}

/// Sync the ratatui backend buffer into the cell data GPU texture.
///
/// Replaces the old entity-per-cell sync — writes directly to texel data.
pub fn sync_buffer_to_texture<T: 'static + Send + Sync>(
    mut terminal_res: ResMut<TerminalResource<T>>,
    config: Res<TerminalConfig<T>>,
    mut atlas: ResMut<FontAtlasResource<T>>,
    quad: Res<TerminalQuadEntity<T>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerminalMaterial>>,
    mut sync_gen: ResMut<SyncGeneration<T>>,
    time: Res<Time>,
) {
    let generation = terminal_res.0.backend().generation();

    if generation == sync_gen.generation {
        // Even if buffer hasn't changed, update time uniform for effects
        if let Some(mat) = materials.get_mut(&quad.material_handle) {
            mat.grid_params.time = time.elapsed_secs();
        }
        return;
    }
    sync_gen.generation = generation;

    let space_index = atlas.glyph_map.get(&' ').copied().unwrap_or(0);
    let mut new_glyphs: Vec<char> = Vec::new();

    // Get the image data
    let Some(image) = images.get_mut(&quad.cell_data_image) else {
        return;
    };
    let Some(ref mut image_data) = image.data else {
        return;
    };

    let bytes_per_pixel = 16usize; // Rgba32Uint = 4 × u32

    // Iterate dirty cells inline — no Vec allocation.
    // Scope the immutable borrows so clear_dirty() can take &mut afterwards.
    {
        let buffer = terminal_res.0.backend().buffer();
        let dirty_cells = terminal_res.0.backend().dirty_cells();

        for (idx, &dirty) in dirty_cells.iter().enumerate() {
            if !dirty || idx >= buffer.len() {
                continue;
            }

            let cell = &buffer[idx];
            let symbol = cell.symbol();
            let fg = ratatui_fg_to_bevy(cell.fg, config.default_fg);
            let bg = ratatui_bg_to_bevy(cell.bg, config.default_bg);
            let modifier = cell.modifier;

            // Look up glyph index
            let ch = symbol.chars().next().unwrap_or(' ');
            let glyph_index = match atlas.glyph_map.get(&ch) {
                Some(&gi) => gi as u32,
                None => {
                    if ch != ' ' {
                        new_glyphs.push(ch);
                    }
                    space_index as u32
                }
            };

            // Pack modifiers
            let mod_bits = {
                let mut m = 0u32;
                if modifier.contains(Modifier::BOLD) {
                    m |= 1;
                }
                if modifier.contains(Modifier::ITALIC) {
                    m |= 2;
                }
                if modifier.contains(Modifier::UNDERLINED) {
                    m |= 4;
                }
                if modifier.contains(Modifier::DIM) {
                    m |= 8;
                }
                m
            };

            let fg_packed = pack_color_rgba8(fg);
            let bg_packed = pack_color_rgba8(bg);

            // Write into image data at the correct offset
            let pixel_offset = idx * bytes_per_pixel;
            if pixel_offset + bytes_per_pixel <= image_data.len() {
                let data = &mut image_data[pixel_offset..pixel_offset + bytes_per_pixel];
                data[0..4].copy_from_slice(&glyph_index.to_le_bytes());
                data[4..8].copy_from_slice(&mod_bits.to_le_bytes());
                data[8..12].copy_from_slice(&fg_packed.to_le_bytes());
                data[12..16].copy_from_slice(&bg_packed.to_le_bytes());
            }
        }
    }

    terminal_res.0.backend_mut().clear_dirty();

    // Schedule newly discovered glyphs for atlas expansion
    if !new_glyphs.is_empty() {
        atlas.pending_glyphs.extend(new_glyphs);
    }

    // Update material uniforms
    if let Some(mat) = materials.get_mut(&quad.material_handle) {
        mat.grid_params.time = time.elapsed_secs();
        // Update atlas info in case it changed (e.g. after expansion)
        update_atlas_params(&mut mat.grid_params, &atlas, &images);
    }
}

/// Update GridParams with current atlas metrics.
pub fn update_atlas_params(
    params: &mut GridParams,
    atlas: &FontAtlasResource<impl Send + Sync + 'static>,
    images: &Assets<Image>,
) {
    let atlas_cols = 16u32;
    let cell_w = atlas.cell_size.x;
    let cell_h = atlas.cell_size.y;
    let pad = (cell_w / 2).max(4);
    let stride_w = cell_w + pad;
    let stride_h = cell_h + pad;

    params.atlas_columns = atlas_cols;
    params.atlas_glyph_count = atlas.glyph_count as u32;
    params.atlas_stride_w = stride_w as f32;
    params.atlas_stride_h = stride_h as f32;
    params.atlas_cell_w = cell_w as f32;
    params.atlas_cell_h = cell_h as f32;

    if let Some(atlas_image) = images.get(&atlas.image) {
        params.atlas_tex_width = atlas_image.width() as f32;
        params.atlas_tex_height = atlas_image.height() as f32;
    }
}

/// Handle atlas expansion: when the font atlas is rebuilt with new glyphs,
/// update the material's font_atlas handle and cell data (mark all dirty).
pub fn handle_atlas_update<T: 'static + Send + Sync>(
    atlas: Res<FontAtlasResource<T>>,
    mut terminal_res: ResMut<TerminalResource<T>>,
    quad: Res<TerminalQuadEntity<T>>,
    mut materials: ResMut<Assets<TerminalMaterial>>,
    images: Res<Assets<Image>>,
) {
    if !atlas.is_changed() {
        return;
    }

    if let Some(mat) = materials.get_mut(&quad.material_handle) {
        mat.font_atlas = atlas.image.clone();
        update_atlas_params(&mut mat.grid_params, &atlas, &images);
    }

    // Mark all cells dirty so they re-sync with the new glyph indices
    terminal_res.0.backend_mut().mark_all_dirty();
}
