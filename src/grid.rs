use bevy::color::Color;
use bevy::prelude::*;

use crate::atlas::FontAtlasResource;
use crate::{TerminalConfig, TerminalLayout};

/// Marker component for terminal cell entities.
#[derive(Component)]
pub struct TerminalCell;

/// Logical grid position of a cell.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub col: u16,
    pub row: u16,
}

/// Style information for a cell, mirroring ratatui cell data.
#[derive(Component, Clone, Debug)]
pub struct CellStyle {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub dim: bool,
    pub symbol: String,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            fg: Color::WHITE,
            bg: Color::srgb(0.0, 0.0, 0.0),
            bold: false,
            italic: false,
            underlined: false,
            dim: false,
            symbol: " ".to_string(),
        }
    }
}

/// Stores the "home" transform for a cell. Effects offset from this.
#[derive(Component, Clone, Copy, Debug)]
pub struct BaseTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

/// Marker for the background sprite child entity.
#[derive(Component)]
pub struct BackgroundSprite;

/// Marker for the foreground sprite child entity.
#[derive(Component)]
pub struct ForegroundSprite;

/// O(1) lookup of cell entities by grid position.
#[derive(Resource)]
pub struct CellEntityIndex {
    pub entities: Vec<Entity>,
    pub fg_entities: Vec<Entity>,
    pub columns: u16,
    pub rows: u16,
}

impl CellEntityIndex {
    /// Get the parent entity at (col, row).
    pub fn get(&self, col: u16, row: u16) -> Option<Entity> {
        if col < self.columns && row < self.rows {
            Some(self.entities[row as usize * self.columns as usize + col as usize])
        } else {
            None
        }
    }

    /// Get the foreground sprite entity at (col, row).
    pub fn get_fg(&self, col: u16, row: u16) -> Option<Entity> {
        if col < self.columns && row < self.rows {
            Some(self.fg_entities[row as usize * self.columns as usize + col as usize])
        } else {
            None
        }
    }
}

/// Startup system that spawns the grid of cell entities.
pub fn spawn_grid(
    mut commands: Commands,
    config: Res<TerminalConfig>,
    layout: Res<TerminalLayout>,
    atlas: Res<FontAtlasResource>,
) {
    let total = config.columns as usize * config.rows as usize;
    let mut entities = Vec::with_capacity(total);
    let mut fg_entities = Vec::with_capacity(total);

    // Space glyph index (fallback to 0)
    let space_index = atlas.glyph_map.get(&' ').copied().unwrap_or(0);

    let bg_size = layout.bg_sprite_size();

    for row in 0..config.rows {
        for col in 0..config.columns {
            let world_x =
                layout.origin.x + (col as f32) * layout.cell_width + layout.cell_width / 2.0;
            let world_y =
                layout.origin.y - (row as f32) * layout.cell_height - layout.cell_height / 2.0;
            let translation = Vec3::new(world_x, world_y, 0.0);

            // Spawn foreground sprite as a standalone entity first
            let fg_entity = commands
                .spawn((
                    ForegroundSprite,
                    Sprite {
                        image: atlas.image.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas.layout.clone(),
                            index: space_index,
                        }),
                        color: Color::WHITE,
                        custom_size: Some(Vec2::new(layout.cell_width, layout.cell_height)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                ))
                .id();

            // Spawn parent with BG sprite directly on it, then add FG as child
            let cell_entity = commands
                .spawn((
                    TerminalCell,
                    GridPosition { col, row },
                    CellStyle::default(),
                    BackgroundSprite,
                    Sprite::from_color(Color::srgb(0.0, 0.0, 0.0), bg_size),
                    BaseTransform {
                        translation,
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    Transform::from_translation(translation),
                    Visibility::default(),
                ))
                .add_child(fg_entity)
                .id();

            entities.push(cell_entity);
            fg_entities.push(fg_entity);
        }
    }

    commands.insert_resource(CellEntityIndex {
        entities,
        fg_entities,
        columns: config.columns,
        rows: config.rows,
    });
}
