use std::collections::{HashMap, HashSet};

use ab_glyph::{Font, FontRef, ScaleFont};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::grid::{BackgroundSprite, BaseTransform, CellEntityIndex, ForegroundSprite, GridPosition};

/// Holds the generated font atlas texture, layout, and glyph mapping.
#[derive(Resource)]
pub struct FontAtlasResource {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub glyph_map: HashMap<char, usize>,
    pub cell_size: UVec2,
    pub font_size: f32,
    /// The font bytes used to build this atlas (kept for rebuilds).
    font_bytes: Vec<u8>,
    /// Characters discovered at runtime that aren't yet in the atlas.
    pub pending_glyphs: HashSet<char>,
    /// Number of glyphs currently in the atlas.
    pub glyph_count: usize,
}

/// Number of columns in the atlas grid.
const ATLAS_COLS: u32 = 16;

/// Raw atlas data before it's stored as Bevy assets.
struct AtlasData {
    image: Image,
    layout: TextureAtlasLayout,
    glyph_map: HashMap<char, usize>,
    cell_size: UVec2,
    glyph_count: usize,
}

/// Return the printable ASCII characters (0x20..=0x7E).
fn ascii_chars() -> Vec<char> {
    (0x20u8..=0x7E).map(|b| b as char).collect()
}

/// Build the font atlas texture and layout for a given font size, font bytes, and character set.
fn build_atlas_data_for_chars(font_bytes: &[u8], font_size: f32, chars: &[char]) -> AtlasData {
    let font = FontRef::try_from_slice(font_bytes).expect("Failed to parse font");
    let scale = ab_glyph::PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);

    let glyph_id = font.glyph_id('M');
    let cell_w = scaled_font.h_advance(glyph_id).ceil() as u32;
    let cell_h = scaled_font.height().ceil() as u32;
    let cell_size = UVec2::new(cell_w, cell_h);

    let glyph_count = chars.len();
    let atlas_rows = ((glyph_count as u32) + ATLAS_COLS - 1) / ATLAS_COLS;

    // Add padding between atlas cells so glyph overflow lands in empty space
    // rather than bleeding into a neighbor's tile.
    let pad: u32 = (cell_w / 2).max(4);
    let stride_w = cell_w + pad;
    let stride_h = cell_h + pad;
    let atlas_width = stride_w * ATLAS_COLS;
    let atlas_height = stride_h * atlas_rows;

    let mut pixel_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];
    let mut glyph_map = HashMap::new();

    let ascent = scaled_font.ascent();

    for (i, &ch) in chars.iter().enumerate() {
        glyph_map.insert(ch, i);

        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(0.0, ascent));

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let grid_col = (i as u32) % ATLAS_COLS;
            let grid_row = (i as u32) / ATLAS_COLS;
            let cell_origin_x = grid_col * stride_w;
            let cell_origin_y = grid_row * stride_h;

            outlined.draw(|px, py, coverage| {
                let x = cell_origin_x as i32 + bounds.min.x as i32 + px as i32;
                let y = cell_origin_y as i32 + bounds.min.y as i32 + py as i32;

                // Allow overflow into this cell's padding but not into the next tile
                if x >= cell_origin_x as i32
                    && y >= cell_origin_y as i32
                    && (x as u32) < cell_origin_x + stride_w
                    && (y as u32) < cell_origin_y + stride_h
                {
                    let idx = (y as u32 * atlas_width + x as u32) as usize * 4;
                    let alpha = (coverage * 255.0).round() as u8;
                    // White glyph, variable alpha
                    pixel_data[idx] = 255;
                    pixel_data[idx + 1] = 255;
                    pixel_data[idx + 2] = 255;
                    // Composite alpha (max with existing)
                    pixel_data[idx + 3] = pixel_data[idx + 3].max(alpha);
                }
            });
        }
    }

    let image = Image::new(
        Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let layout = TextureAtlasLayout::from_grid(
        cell_size,
        ATLAS_COLS,
        atlas_rows,
        Some(UVec2::new(pad, pad)),
        None,
    );

    AtlasData {
        image,
        layout,
        glyph_map,
        cell_size,
        glyph_count,
    }
}

/// Generate the font atlas as a startup system.
pub fn generate_font_atlas(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    config: Res<crate::TerminalConfig>,
) {
    let font_bytes = config.font.bytes().to_vec();
    let chars = ascii_chars();
    let data = build_atlas_data_for_chars(&font_bytes, config.font_size, &chars);
    let image_handle = images.add(data.image);
    let layout_handle = layouts.add(data.layout);

    commands.insert_resource(FontAtlasResource {
        image: image_handle,
        layout: layout_handle,
        glyph_map: data.glyph_map,
        cell_size: data.cell_size,
        font_size: config.font_size,
        font_bytes,
        pending_glyphs: HashSet::new(),
        glyph_count: data.glyph_count,
    });
}

/// Expands the font atlas when new (previously unseen) characters are pending.
/// Runs before `rebuild_font_atlas` so that new glyphs are available for the
/// current frame's sync pass.
pub fn expand_font_atlas(
    mut atlas: ResMut<FontAtlasResource>,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    cell_index: Res<CellEntityIndex>,
    mut fg_query: Query<&mut Sprite, (With<ForegroundSprite>, Without<BackgroundSprite>)>,
    children_query: Query<&Children>,
) {
    if atlas.pending_glyphs.is_empty() {
        return;
    }

    // Drain pending first to release the mutable borrow before accessing font_bytes
    let pending: Vec<char> = atlas.pending_glyphs.drain().collect();

    // Filter pending chars to only those the font can actually render
    let font = FontRef::try_from_slice(&atlas.font_bytes).expect("Failed to parse font");
    let scale = ab_glyph::PxScale::from(atlas.font_size);
    let ascent = font.as_scaled(scale).ascent();

    let new_chars: Vec<char> = pending
        .into_iter()
        .filter(|&ch| {
            let glyph_id = font.glyph_id(ch);
            let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(0.0, ascent));
            font.outline_glyph(glyph).is_some()
        })
        .collect();

    if new_chars.is_empty() {
        return;
    }

    // Merge existing glyph_map keys with new chars, sorted for deterministic ordering
    let mut all_chars: Vec<char> = atlas.glyph_map.keys().copied().collect();
    all_chars.extend(new_chars);
    all_chars.sort();
    all_chars.dedup();

    let data = build_atlas_data_for_chars(&atlas.font_bytes, atlas.font_size, &all_chars);
    let image_handle = images.add(data.image);
    let layout_handle = layouts.add(data.layout);
    atlas.image = image_handle.clone();
    atlas.layout = layout_handle.clone();
    atlas.glyph_map = data.glyph_map;
    atlas.cell_size = data.cell_size;
    atlas.glyph_count = data.glyph_count;

    // Update all foreground sprite handles to point to the new atlas
    for &entity in &cell_index.entities {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(mut fg_sprite) = fg_query.get_mut(child) {
                    fg_sprite.image = image_handle.clone();
                    if let Some(ref mut tex_atlas) = fg_sprite.texture_atlas {
                        tex_atlas.layout = layout_handle.clone();
                    }
                }
            }
        }
    }
}

/// Detects when `TerminalConfig.font_size` has changed and rebuilds the atlas,
/// cell positions, and sprite sizes to match.
pub fn rebuild_font_atlas(
    mut config: ResMut<crate::TerminalConfig>,
    mut atlas: ResMut<FontAtlasResource>,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    cell_index: Res<CellEntityIndex>,
    mut cell_query: Query<(&GridPosition, &mut BaseTransform, &mut Transform)>,
    mut bg_query: Query<&mut Sprite, (With<BackgroundSprite>, Without<ForegroundSprite>)>,
    mut fg_query: Query<&mut Sprite, (With<ForegroundSprite>, Without<BackgroundSprite>)>,
    children_query: Query<&Children>,
) {
    if config.font_size == atlas.font_size {
        return;
    }

    // Scale cell dimensions proportionally
    let ratio = config.font_size / atlas.font_size;
    config.cell_width *= ratio;
    config.cell_height *= ratio;
    config.origin = Vec2::new(
        -(config.columns as f32 * config.cell_width) / 2.0,
        (config.rows as f32 * config.cell_height) / 2.0,
    );

    // Rebuild the atlas at the new font size with all currently known chars
    let mut all_chars: Vec<char> = atlas.glyph_map.keys().copied().collect();
    all_chars.sort();

    let data = build_atlas_data_for_chars(&atlas.font_bytes, config.font_size, &all_chars);
    let image_handle = images.add(data.image);
    let layout_handle = layouts.add(data.layout);
    atlas.image = image_handle.clone();
    atlas.layout = layout_handle.clone();
    atlas.glyph_map = data.glyph_map;
    atlas.cell_size = data.cell_size;
    atlas.font_size = config.font_size;
    atlas.glyph_count = data.glyph_count;

    // Update all cell positions and child sprites
    for (grid_pos, mut base_tf, mut transform) in cell_query.iter_mut() {
        let world_x =
            config.origin.x + (grid_pos.col as f32) * config.cell_width + config.cell_width / 2.0;
        let world_y = config.origin.y
            - (grid_pos.row as f32) * config.cell_height
            - config.cell_height / 2.0;
        let translation = Vec3::new(world_x, world_y, 0.0);
        base_tf.translation = translation;
        transform.translation = translation;

        if let Some(entity) = cell_index.get(grid_pos.col, grid_pos.row) {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if let Ok(mut bg_sprite) = bg_query.get_mut(child) {
                        bg_sprite.custom_size =
                            Some(Vec2::new(config.cell_width, config.cell_height));
                    }
                    if let Ok(mut fg_sprite) = fg_query.get_mut(child) {
                        fg_sprite.custom_size =
                            Some(Vec2::new(config.cell_width, config.cell_height));
                        fg_sprite.image = image_handle.clone();
                        if let Some(ref mut tex_atlas) = fg_sprite.texture_atlas {
                            tex_atlas.layout = layout_handle.clone();
                        }
                    }
                }
            }
        }
    }
}
