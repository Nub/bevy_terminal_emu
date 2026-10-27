use bevy::prelude::*;
use ratatui::backend::Backend;
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

/// Update system: resize the quad mesh, transform, and material grid params
/// when `TerminalLayout` changes (e.g. due to font size change).
pub fn resize_terminal_quad<T: 'static + Send + Sync>(
    config: Res<TerminalConfig<T>>,
    layout: Res<TerminalLayout<T>>,
    quad: Res<TerminalQuadEntity<T>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerminalMaterial>>,
    mut transforms: Query<(&Mesh2d, &mut Transform)>,
) {
    if !layout.is_changed() {
        return;
    }

    let cols = config.columns as f32;
    let rows = config.rows as f32;
    let quad_width = cols * layout.cell_width;
    let quad_height = rows * layout.cell_height;

    // Update the mesh to the new size
    if let Ok((mesh2d, mut transform)) = transforms.get_mut(quad.entity) {
        let new_mesh = Rectangle::new(quad_width, quad_height);
        if let Some(mesh) = meshes.get_mut(&mesh2d.0) {
            *mesh = Mesh::from(new_mesh);
        }

        // Update position to match new layout origin
        let center_x = layout.origin.x + quad_width / 2.0;
        let center_y = layout.origin.y - quad_height / 2.0;
        transform.translation.x = center_x;
        transform.translation.y = center_y;
    }

    // Update cell dimensions in material grid params
    if let Some(mat) = materials.get_mut(&quad.material_handle) {
        mat.grid_params.cell_width = layout.cell_width;
        mat.grid_params.cell_height = layout.cell_height;
    }
}

/// Update system: when `TerminalConfig.columns` / `rows` change, resize the ratatui terminal, its backend and the
/// cell data texture, and recompute the layout (which `resize_terminal_quad` then follows).
pub fn resize_terminal_grid<T: 'static + Send + Sync>(
    config: Res<TerminalConfig<T>>,
    mut layout: ResMut<TerminalLayout<T>>,
    mut terminal_res: ResMut<crate::TerminalResource<T>>,
    atlas: Option<Res<crate::atlas::FontAtlasResource<T>>>,
    quad: Option<Res<TerminalQuadEntity<T>>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerminalMaterial>>,
) {
    let (Some(atlas), Some(quad)) = (atlas, quad) else { return };
    let size = terminal_res.0.backend().size().unwrap_or_default();
    if (size.width, size.height) == (config.columns, config.rows) {
        return;
    }
    let (cols, rows) = (config.columns, config.rows);
    terminal_res.0.backend_mut().resize(cols, rows);
    // The terminal's own buffers follow the backend (a full-screen viewport resizes and clears).
    let _ = terminal_res.0.resize(ratatui::layout::Rect::new(0, 0, cols, rows));
    if let Some(image) = images.get_mut(&quad.cell_data_image) {
        *image = create_cell_data_image(cols, rows);
    }
    if let Some(mat) = materials.get_mut(&quad.material_handle) {
        mat.grid_params.columns = cols as u32;
        mat.grid_params.rows = rows as u32;
    }
    crate::atlas::align_layout_to_atlas(&mut layout, &config, atlas.cell_size, atlas.scale_factor);
    // A changed layout makes resize_terminal_quad size the quad to the new grid.
    layout.set_changed();
}
