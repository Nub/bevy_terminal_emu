use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;

use crate::atlas::FontAtlasResource;
use crate::render::{GridParams, TerminalMaterial, TerminalQuadEntity};
use crate::render_sync::{create_cell_data_image, update_atlas_params};
use crate::{TerminalConfig, TerminalLayout};

/// Startup system that spawns a single quad entity for the terminal.
///
/// Replaces the old `spawn_grid` which created `cols × rows × 2` entities.
pub fn spawn_terminal_quad<T: 'static + Send + Sync>(
    mut commands: Commands,
    config: Res<TerminalConfig<T>>,
    layout: Res<TerminalLayout<T>>,
    atlas: Res<FontAtlasResource<T>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerminalMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let cols = config.columns;
    let rows = config.rows;

    // Create cell data texture
    let cell_data_image = create_cell_data_image(cols, rows);
    let cell_data_handle = images.add(cell_data_image);

    // Build grid params
    let mut grid_params = GridParams {
        columns: cols as u32,
        rows: rows as u32,
        cell_width: layout.cell_width,
        cell_height: layout.cell_height,
        atlas_columns: 16,
        atlas_glyph_count: 0,
        atlas_stride_w: 0.0,
        atlas_stride_h: 0.0,
        atlas_cell_w: 0.0,
        atlas_cell_h: 0.0,
        atlas_tex_width: 1.0,
        atlas_tex_height: 1.0,
        time: 0.0,
        _pad: 0.0,
    };
    update_atlas_params(&mut grid_params, &atlas, &images);

    // Create material
    let material = TerminalMaterial {
        cell_data: cell_data_handle.clone(),
        font_atlas: atlas.image.clone(),
        grid_params,
        effect_params: crate::render::TerminalEffectUniforms::default(),
    };
    let material_handle = materials.add(material);

    // Quad size = total terminal dimensions in world space
    let quad_width = cols as f32 * layout.cell_width;
    let quad_height = rows as f32 * layout.cell_height;

    // The mesh is a simple rectangle
    let mesh = Rectangle::new(quad_width, quad_height);
    let mesh_handle = meshes.add(mesh);

    // Position: center of the terminal grid
    let center_x = layout.origin.x + quad_width / 2.0;
    let center_y = layout.origin.y - quad_height / 2.0;

    let entity = commands
        .spawn((
            Mesh2d(mesh_handle),
            MeshMaterial2d(material_handle.clone()),
            Transform::from_xyz(center_x, center_y, config.z_layer),
            Visibility::default(),
        ))
        .id();

    commands.insert_resource(TerminalQuadEntity::<T>::new(
        entity,
        material_handle,
        cell_data_handle,
    ));
}
