use std::marker::PhantomData;

use bevy::prelude::*;
use ratatui::style::Modifier;

use crate::atlas::FontAtlasResource;
use crate::color::{ratatui_bg_to_bevy, ratatui_fg_to_bevy};
use crate::grid::{BackgroundSprite, CellEntityIndex, CellStyle, ForegroundSprite};
use crate::{TerminalResource, TerminalConfig};

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

/// Sync the backend buffer contents to cell entity sprites each frame.
///
/// Only processes cells marked dirty by the backend, and uses compare-before-write
/// to avoid triggering Bevy change detection on unchanged components.
pub fn sync_buffer_to_entities<T: 'static + Send + Sync>(
    terminal_res: Res<TerminalResource<T>>,
    config: Res<TerminalConfig<T>>,
    mut atlas: ResMut<FontAtlasResource<T>>,
    cell_index: Res<CellEntityIndex<T>>,
    mut sync_gen: ResMut<SyncGeneration<T>>,
    mut cell_query: Query<(&mut CellStyle, &mut Sprite), With<BackgroundSprite<T>>>,
    mut fg_query: Query<&mut Sprite, (With<ForegroundSprite<T>>, Without<BackgroundSprite<T>>)>,
) {
    let mut terminal = terminal_res.0.lock().unwrap();
    let generation = terminal.backend().generation();

    // Skip if nothing has changed
    if generation == sync_gen.generation {
        return;
    }
    sync_gen.generation = generation;

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

        // Update CellStyle + BG sprite on parent entity
        if let Ok((mut cell_style, mut bg_sprite)) = cell_query.get_mut(entity) {
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

            if bg_sprite.color != bg {
                bg_sprite.color = bg;
            }
        }

        // Update foreground sprite via direct entity lookup
        let fg_entity = cell_index.fg_entities[idx];
        if let Ok(mut fg_sprite) = fg_query.get_mut(fg_entity) {
            let target_fg = if dim { fg.with_alpha(0.5) } else { fg };
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

    // Schedule newly discovered glyphs for atlas expansion next frame
    if !new_glyphs.is_empty() {
        atlas.pending_glyphs.extend(new_glyphs);
    }
}
