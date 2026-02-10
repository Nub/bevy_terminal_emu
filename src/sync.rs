use bevy::prelude::*;
use ratatui::style::Modifier;

use crate::atlas::FontAtlasResource;
use crate::color::{ratatui_bg_to_bevy, ratatui_fg_to_bevy};
use crate::grid::{BackgroundSprite, CellEntityIndex, CellStyle, ForegroundSprite};
use crate::{TerminalResource, TerminalConfig};

/// Resource tracking the last synced generation to skip redundant updates.
#[derive(Resource, Default)]
pub struct SyncGeneration(pub u64);

/// Sync the backend buffer contents to cell entity sprites each frame.
///
/// Only processes cells marked dirty by the backend, and uses compare-before-write
/// to avoid triggering Bevy change detection on unchanged components.
pub fn sync_buffer_to_entities(
    terminal_res: Res<TerminalResource>,
    config: Res<TerminalConfig>,
    mut atlas: ResMut<FontAtlasResource>,
    cell_index: Res<CellEntityIndex>,
    mut sync_gen: ResMut<SyncGeneration>,
    mut cell_query: Query<&mut CellStyle>,
    mut bg_query: Query<&mut Sprite, (With<BackgroundSprite>, Without<ForegroundSprite>)>,
    mut fg_query: Query<&mut Sprite, (With<ForegroundSprite>, Without<BackgroundSprite>)>,
    children_query: Query<&Children>,
) {
    let mut terminal = terminal_res.0.lock().unwrap();
    let generation = terminal.backend().generation();

    // Skip if nothing has changed
    if generation == sync_gen.0 {
        return;
    }
    sync_gen.0 = generation;

    // Collect dirty cell indices while holding immutable borrow
    let columns = config.columns as usize;
    let dirty_indices: Vec<usize> = terminal
        .backend()
        .dirty_cells()
        .iter()
        .enumerate()
        .filter_map(|(i, &d)| if d { Some(i) } else { None })
        .collect();

    // Clear dirty flags (needs mutable borrow, but dirty_indices is owned)
    terminal.backend_mut().clear_dirty();

    let buffer = terminal.backend().buffer();
    let space_index = atlas.glyph_map.get(&' ').copied().unwrap_or(0);
    let mut new_glyphs: Vec<char> = Vec::new();

    for idx in dirty_indices {
        if idx >= buffer.len() {
            continue;
        }
        let col = (idx % columns) as u16;
        let row = (idx / columns) as u16;

        let cell = &buffer[idx];
        let symbol = cell.symbol();
        let fg = ratatui_fg_to_bevy(cell.fg, config.default_fg);
        let bg = ratatui_bg_to_bevy(cell.bg, config.default_bg);
        let modifier = cell.modifier;
        let bold = modifier.contains(Modifier::BOLD);
        let italic = modifier.contains(Modifier::ITALIC);
        let underlined = modifier.contains(Modifier::UNDERLINED);
        let dim = modifier.contains(Modifier::DIM);

        let Some(entity) = cell_index.get(col, row) else {
            continue;
        };

        // Update CellStyle only if values actually changed (avoids triggering change detection)
        if let Ok(mut cell_style) = cell_query.get_mut(entity) {
            if cell_style.fg != fg
                || cell_style.bg != bg
                || cell_style.bold != bold
                || cell_style.italic != italic
                || cell_style.underlined != underlined
                || cell_style.dim != dim
                || cell_style.symbol != symbol
            {
                cell_style.fg = fg;
                cell_style.bg = bg;
                cell_style.bold = bold;
                cell_style.italic = italic;
                cell_style.underlined = underlined;
                cell_style.dim = dim;
                cell_style.symbol = symbol.to_string();
            }
        }

        // Update child sprites
        if let Ok(children) = children_query.get(entity) {
            let target_fg = if dim { fg.with_alpha(0.5) } else { fg };

            for child in children.iter() {
                // Update background sprite color only if changed
                if let Ok(mut bg_sprite) = bg_query.get_mut(child) {
                    if bg_sprite.color != bg {
                        bg_sprite.color = bg;
                    }
                }

                // Update foreground sprite color and atlas index only if changed
                if let Ok(mut fg_sprite) = fg_query.get_mut(child) {
                    if fg_sprite.color != target_fg {
                        fg_sprite.color = target_fg;
                    }

                    // Look up glyph in atlas; queue unknown chars for next-frame expansion
                    let ch = symbol.chars().next().unwrap_or(' ');
                    let glyph_index = match atlas.glyph_map.get(&ch) {
                        Some(&glyph_idx) => glyph_idx,
                        None => {
                            if ch != ' ' {
                                new_glyphs.push(ch);
                            }
                            space_index
                        }
                    };

                    // Read atlas index immutably first, only write if different
                    let current_index = fg_sprite.texture_atlas.as_ref().map(|ta| ta.index);
                    if current_index != Some(glyph_index) {
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
