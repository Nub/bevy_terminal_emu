use bevy::prelude::*;
use ratatui::style::Modifier;

use crate::atlas::FontAtlasResource;
use crate::color::{ratatui_bg_to_bevy, ratatui_fg_to_bevy};
use crate::grid::{BackgroundSprite, CellEntityIndex, CellStyle, ForegroundSprite, GridPosition};
use crate::{TerminalResource, TerminalConfig};

/// Resource tracking the last synced generation to skip redundant updates.
#[derive(Resource, Default)]
pub struct SyncGeneration(pub u64);

/// Sync the backend buffer contents to cell entity sprites each frame.
pub fn sync_buffer_to_entities(
    terminal_res: Res<TerminalResource>,
    config: Res<TerminalConfig>,
    mut atlas: ResMut<FontAtlasResource>,
    cell_index: Res<CellEntityIndex>,
    mut sync_gen: ResMut<SyncGeneration>,
    mut cell_query: Query<(&GridPosition, &mut CellStyle)>,
    mut bg_query: Query<&mut Sprite, (With<BackgroundSprite>, Without<ForegroundSprite>)>,
    mut fg_query: Query<&mut Sprite, (With<ForegroundSprite>, Without<BackgroundSprite>)>,
    children_query: Query<&Children>,
) {
    let terminal = terminal_res.0.lock().unwrap();
    let backend = terminal.backend();
    let generation = backend.generation();

    // Skip if nothing has changed
    if generation == sync_gen.0 {
        return;
    }
    sync_gen.0 = generation;

    let buffer = backend.buffer();
    let columns = config.columns as usize;
    let space_index = atlas.glyph_map.get(&' ').copied().unwrap_or(0);
    let mut new_glyphs: Vec<char> = Vec::new();

    for (grid_pos, mut cell_style) in cell_query.iter_mut() {
        let idx = grid_pos.row as usize * columns + grid_pos.col as usize;
        if idx >= buffer.len() {
            continue;
        }

        let cell = &buffer[idx];
        let symbol = cell.symbol();
        let fg = ratatui_fg_to_bevy(cell.fg, config.default_fg);
        let bg = ratatui_bg_to_bevy(cell.bg, config.default_bg);
        let modifier = cell.modifier;

        // Update CellStyle component
        cell_style.fg = fg;
        cell_style.bg = bg;
        cell_style.bold = modifier.contains(Modifier::BOLD);
        cell_style.italic = modifier.contains(Modifier::ITALIC);
        cell_style.underlined = modifier.contains(Modifier::UNDERLINED);
        cell_style.dim = modifier.contains(Modifier::DIM);
        cell_style.symbol = symbol.to_string();

        // Update child sprites
        if let Some(entity) = cell_index.get(grid_pos.col, grid_pos.row) {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    // Update background sprite color
                    if let Ok(mut bg_sprite) = bg_query.get_mut(child) {
                        bg_sprite.color = bg;
                    }

                    // Update foreground sprite color and atlas index
                    if let Ok(mut fg_sprite) = fg_query.get_mut(child) {
                        fg_sprite.color = if cell_style.dim {
                            // Dim the foreground by reducing alpha
                            fg.with_alpha(0.5)
                        } else {
                            fg
                        };

                        // Look up glyph in atlas; queue unknown chars for next-frame expansion
                        let ch = symbol.chars().next().unwrap_or(' ');
                        let glyph_index = match atlas.glyph_map.get(&ch) {
                            Some(&idx) => idx,
                            None => {
                                if ch != ' ' {
                                    new_glyphs.push(ch);
                                }
                                space_index
                            }
                        };

                        if let Some(ref mut tex_atlas) = fg_sprite.texture_atlas {
                            tex_atlas.index = glyph_index;
                        }
                    }
                }
            }
        }
    }

    // Schedule newly discovered glyphs for atlas expansion next frame
    if !new_glyphs.is_empty() {
        atlas.pending_glyphs.extend(new_glyphs);
    }
}
