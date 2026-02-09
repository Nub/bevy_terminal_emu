use std::collections::HashMap;

use ab_glyph::{Font, FontRef, ScaleFont};
use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Holds the generated font atlas texture, layout, and glyph mapping.
#[derive(Resource)]
pub struct FontAtlasResource {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub glyph_map: HashMap<char, usize>,
    pub cell_size: UVec2,
}

/// The embedded monospace font bytes (JetBrains Mono).
const FONT_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

/// Printable ASCII range for atlas generation.
const GLYPH_START: u8 = 0x20;
const GLYPH_END: u8 = 0x7E;
const GLYPH_COUNT: usize = (GLYPH_END - GLYPH_START + 1) as usize; // 95

/// Number of columns in the atlas grid.
const ATLAS_COLS: u32 = 16;

/// Generate the font atlas as a startup system.
pub fn generate_font_atlas(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    config: Res<crate::TerminalConfig>,
) {
    let font = FontRef::try_from_slice(FONT_BYTES).expect("Failed to parse embedded font");
    let scale = ab_glyph::PxScale::from(config.font_size);
    let scaled_font = font.as_scaled(scale);

    // Determine cell size from the advance width of 'M' and the font height.
    let glyph_id = font.glyph_id('M');
    let cell_w = scaled_font.h_advance(glyph_id).ceil() as u32;
    let cell_h = scaled_font.height().ceil() as u32;
    let cell_size = UVec2::new(cell_w, cell_h);

    let atlas_rows = ((GLYPH_COUNT as u32) + ATLAS_COLS - 1) / ATLAS_COLS;
    let atlas_width = cell_w * ATLAS_COLS;
    let atlas_height = cell_h * atlas_rows;

    // RGBA8 buffer, initialized to transparent
    let mut pixel_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];
    let mut glyph_map = HashMap::new();

    let ascent = scaled_font.ascent();

    for (i, code) in (GLYPH_START..=GLYPH_END).enumerate() {
        let ch = code as char;
        glyph_map.insert(ch, i);

        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(0.0, ascent));

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let grid_col = (i as u32) % ATLAS_COLS;
            let grid_row = (i as u32) / ATLAS_COLS;
            let cell_origin_x = grid_col * cell_w;
            let cell_origin_y = grid_row * cell_h;

            outlined.draw(|px, py, coverage| {
                let x = cell_origin_x as i32 + bounds.min.x as i32 + px as i32;
                let y = cell_origin_y as i32 + bounds.min.y as i32 + py as i32;

                if x >= 0
                    && y >= 0
                    && (x as u32) < atlas_width
                    && (y as u32) < atlas_height
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

    let image_handle = images.add(image);

    let layout = TextureAtlasLayout::from_grid(cell_size, ATLAS_COLS, atlas_rows, None, None);
    let layout_handle = layouts.add(layout);

    commands.insert_resource(FontAtlasResource {
        image: image_handle,
        layout: layout_handle,
        glyph_map,
        cell_size,
    });
}
