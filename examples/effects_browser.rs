use std::collections::HashMap;

use bevy::prelude::*;
use bevy_terminal_emu::prelude::*;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(bevy::color::Color::BLACK))
        .add_plugins(TerminalEmuPlugin {
            config: TerminalConfig {
                columns: 160,
                rows: 48,
                ..Default::default()
            },
        })
        .insert_resource(BrowserState::new())
        .insert_resource(ActiveEffectEntities::default())
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (handle_input, sync_effects, draw_ui)
                .chain()
                .in_set(TerminalSet::AppTick),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Describes one effect entry in the browser.
struct EffectEntry {
    name: &'static str,
    description: &'static str,
    active: bool,
}

/// Region preset for targeting effects to a sub-area.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RegionPreset {
    Full,
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    Center,
}

impl RegionPreset {
    const ALL: [RegionPreset; 6] = [
        RegionPreset::Full,
        RegionPreset::LeftHalf,
        RegionPreset::RightHalf,
        RegionPreset::TopHalf,
        RegionPreset::BottomHalf,
        RegionPreset::Center,
    ];

    fn name(self) -> &'static str {
        match self {
            RegionPreset::Full => "Full",
            RegionPreset::LeftHalf => "Left Half",
            RegionPreset::RightHalf => "Right Half",
            RegionPreset::TopHalf => "Top Half",
            RegionPreset::BottomHalf => "Bottom Half",
            RegionPreset::Center => "Center",
        }
    }

    fn to_effect_region(self) -> EffectRegion {
        match self {
            RegionPreset::Full => EffectRegion::all(),
            RegionPreset::LeftHalf => EffectRegion {
                include: vec![GridRect { col: 0, row: 0, width: 80, height: 48 }],
                exclude: vec![],
            },
            RegionPreset::RightHalf => EffectRegion {
                include: vec![GridRect { col: 80, row: 0, width: 80, height: 48 }],
                exclude: vec![],
            },
            RegionPreset::TopHalf => EffectRegion {
                include: vec![GridRect { col: 0, row: 0, width: 160, height: 24 }],
                exclude: vec![],
            },
            RegionPreset::BottomHalf => EffectRegion {
                include: vec![GridRect { col: 0, row: 24, width: 160, height: 24 }],
                exclude: vec![],
            },
            RegionPreset::Center => EffectRegion {
                include: vec![GridRect { col: 40, row: 12, width: 80, height: 24 }],
                exclude: vec![],
            },
        }
    }
}

/// Browser UI state.
#[derive(Resource)]
struct BrowserState {
    selected: usize,
    effects: Vec<EffectEntry>,
    region_index: usize,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            selected: 0,
            region_index: 0,
            effects: vec![
                EffectEntry {
                    name: "Wave",
                    description: "Sine wave vertical oscillation",
                    active: false,
                },
                EffectEntry {
                    name: "Ripple",
                    description: "Radial wave from center point",
                    active: false,
                },
                EffectEntry {
                    name: "Collapse",
                    description: "Cells fall with gravity stagger",
                    active: false,
                },
                EffectEntry {
                    name: "Gravity",
                    description: "Continuous downward acceleration",
                    active: false,
                },
                EffectEntry {
                    name: "Glitch",
                    description: "CRT-style horizontal row shift",
                    active: false,
                },
                EffectEntry {
                    name: "Scatter",
                    description: "Explosion from center point",
                    active: false,
                },
                EffectEntry {
                    name: "Breathe",
                    description: "Rhythmic scale pulse",
                    active: false,
                },
                EffectEntry {
                    name: "Jitter",
                    description: "Per-cell random vibration",
                    active: false,
                },
                EffectEntry {
                    name: "Slash",
                    description: "Diagonal swipe across screen",
                    active: false,
                },
                EffectEntry {
                    name: "Explode",
                    description: "Chaotic explosion with random spin",
                    active: false,
                },
                EffectEntry {
                    name: "Rainbow",
                    description: "Foreground hue cycling through spectrum",
                    active: false,
                },
                EffectEntry {
                    name: "Glow",
                    description: "Pulsing alpha and scale shimmer",
                    active: false,
                },
                EffectEntry {
                    name: "Shiny",
                    description: "Sweeping diagonal highlight band",
                    active: false,
                },
                EffectEntry {
                    name: "Bubbly",
                    description: "Random cells grow and shrink",
                    active: false,
                },
            ],
        }
    }

    fn current_region(&self) -> RegionPreset {
        RegionPreset::ALL[self.region_index]
    }
}

/// Tracks spawned effect entities so we can despawn them on toggle-off.
#[derive(Resource, Default)]
struct ActiveEffectEntities {
    map: HashMap<usize, Entity>,
}

fn handle_input(
    mut queue: ResMut<TerminalInputQueue>,
    mut state: ResMut<BrowserState>,
    mut active: ResMut<ActiveEffectEntities>,
    mut config: ResMut<TerminalConfig>,
    mut commands: Commands,
    cells: Query<Entity, With<TerminalCell>>,
) {
    while let Some(event) = queue.events.pop_front() {
        if let terminput::Event::Key(key_event) = event {
            if key_event.kind != terminput::KeyEventKind::Press {
                continue;
            }
            match key_event.code {
                terminput::KeyCode::Up => {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
                terminput::KeyCode::Down => {
                    if state.selected + 1 < state.effects.len() {
                        state.selected += 1;
                    }
                }
                terminput::KeyCode::Enter | terminput::KeyCode::Char(' ') => {
                    let idx = state.selected;
                    state.effects[idx].active = !state.effects[idx].active;
                }
                terminput::KeyCode::Char('r') => {
                    for effect in &mut state.effects {
                        effect.active = false;
                    }
                }
                terminput::KeyCode::Char('e') => {
                    // Cycle region preset
                    state.region_index = (state.region_index + 1) % RegionPreset::ALL.len();

                    // Despawn all active effects so they respawn with the new region
                    for entity in active.map.values() {
                        commands.entity(*entity).despawn();
                    }
                    active.map.clear();

                    // Remove CellVelocity from all cells (in case Gravity was active)
                    for cell_entity in cells.iter() {
                        commands.entity(cell_entity).remove::<CellVelocity>();
                    }

                    // Re-activate effects that were toggled on so sync_effects respawns them
                    // (they're still marked active in state, but no longer in active.map)
                }
                terminput::KeyCode::Char('=') | terminput::KeyCode::Char('+')
                    if key_event.modifiers.contains(terminput::KeyModifiers::CTRL) =>
                {
                    config.font_size = (config.font_size + 2.0).min(48.0);
                }
                terminput::KeyCode::Char('-')
                    if key_event.modifiers.contains(terminput::KeyModifiers::CTRL) =>
                {
                    config.font_size = (config.font_size - 2.0).max(8.0);
                }
                _ => {}
            }
        }
    }
}

fn sync_effects(
    mut commands: Commands,
    state: Res<BrowserState>,
    mut active: ResMut<ActiveEffectEntities>,
    cells: Query<Entity, With<TerminalCell>>,
    mut collapses: Query<&mut Collapse>,
    mut scatters: Query<&mut Scatter>,
    mut slashes: Query<&mut Slash>,
    mut explodes: Query<&mut Explode>,
) {
    let region = state.current_region().to_effect_region();

    for (idx, effect) in state.effects.iter().enumerate() {
        let is_spawned = active.map.contains_key(&idx);

        if effect.active && !is_spawned {
            // Spawn the effect entity with the current region
            let entity = match idx {
                0 => commands.spawn((Wave::default(), region.clone())).id(),
                1 => commands.spawn((Ripple::default(), region.clone())).id(),
                2 => commands.spawn((Collapse::default(), region.clone())).id(),
                3 => commands.spawn((Gravity::default(), region.clone())).id(),
                4 => commands.spawn((Glitch::default(), region.clone())).id(),
                5 => commands.spawn((Scatter::default(), region.clone())).id(),
                6 => commands.spawn((Breathe::default(), region.clone())).id(),
                7 => commands.spawn((Jitter::default(), region.clone())).id(),
                8 => commands.spawn((Slash::default(), region.clone())).id(),
                9 => commands.spawn((Explode::default(), region.clone())).id(),
                10 => commands.spawn((Rainbow::default(), region.clone())).id(),
                11 => commands.spawn((Glow::default(), region.clone())).id(),
                12 => commands.spawn((Shiny::default(), region.clone())).id(),
                13 => commands.spawn((Bubbly::default(), region.clone())).id(),
                _ => unreachable!(),
            };
            active.map.insert(idx, entity);

            // Gravity needs CellVelocity on all cells
            if idx == 3 {
                for cell_entity in cells.iter() {
                    commands.entity(cell_entity).insert(CellVelocity::default());
                }
            }
        } else if effect.active && is_spawned {
            // For one-shot effects: re-toggling resets the animation
            if idx == 2 {
                if let Some(&entity) = active.map.get(&idx) {
                    if let Ok(mut collapse) = collapses.get_mut(entity) {
                        if !collapse.active {
                            collapse.elapsed = 0.0;
                            collapse.active = true;
                        }
                    }
                }
            }
            if idx == 5 {
                if let Some(&entity) = active.map.get(&idx) {
                    if let Ok(mut scatter) = scatters.get_mut(entity) {
                        if !scatter.active {
                            scatter.elapsed = 0.0;
                            scatter.active = true;
                        }
                    }
                }
            }
            if idx == 8 {
                if let Some(&entity) = active.map.get(&idx) {
                    if let Ok(mut slash) = slashes.get_mut(entity) {
                        if !slash.active {
                            slash.elapsed = 0.0;
                            slash.active = true;
                        }
                    }
                }
            }
            if idx == 9 {
                if let Some(&entity) = active.map.get(&idx) {
                    if let Ok(mut explode) = explodes.get_mut(entity) {
                        if !explode.active {
                            explode.elapsed = 0.0;
                            explode.active = true;
                        }
                    }
                }
            }
        } else if !effect.active && is_spawned {
            // Despawn the effect entity
            if let Some(entity) = active.map.remove(&idx) {
                commands.entity(entity).despawn();
            }

            // Remove CellVelocity when Gravity is toggled off
            if idx == 3 {
                for cell_entity in cells.iter() {
                    commands.entity(cell_entity).remove::<CellVelocity>();
                }
            }
        }
    }
}

fn draw_ui(terminal_res: Res<TerminalResource>, state: Res<BrowserState>, config: Res<TerminalConfig>) {
    let mut terminal = terminal_res.0.lock().unwrap();

    terminal
        .draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(38), Constraint::Min(1)])
                .split(area);

            // Left panel: effect list
            let mut lines: Vec<Line> = Vec::new();
            for (i, effect) in state.effects.iter().enumerate() {
                let checkbox = if effect.active { "[x]" } else { "[ ]" };
                let cursor = if i == state.selected { "> " } else { "  " };

                let style = if i == state.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if effect.active {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{} {}", cursor, checkbox, effect.name),
                        style,
                    ),
                ]));
                lines.push(Line::from(vec![Span::styled(
                    format!("     {}", effect.description),
                    Style::default().fg(Color::DarkGray),
                )]));
            }

            // Region info below effect list
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!("  Region: {}", state.current_region().name()),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )]));

            let list_block = Block::default()
                .title(" Effects ")
                .borders(Borders::ALL);
            let list = Paragraph::new(lines).block(list_block);
            frame.render_widget(list, chunks[0]);

            // Right panel: demo content + instructions
            let demo_lines = vec![
                Line::from("Hello, terminal!"),
                Line::from(""),
                Line::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
                Line::from("abcdefghijklmnopqrstuvwxyz"),
                Line::from("0123456789 !@#$%^&*()"),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "--- Box Drawing ---",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from("┌──────┬──────┐  ╔══════╗"),
                Line::from("│ thin │ box  │  ║double║"),
                Line::from("├──────┼──────┤  ╠══════╣"),
                Line::from("│ line │ draw │  ║ line ║"),
                Line::from("└──────┴──────┘  ╚══════╝"),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "--- Block Elements ---",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    Span::styled("░░", Style::default().fg(Color::Red)),
                    Span::styled("▒▒", Style::default().fg(Color::Yellow)),
                    Span::styled("▓▓", Style::default().fg(Color::Green)),
                    Span::styled("██", Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled("▀▄▀▄", Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled("▌▐▌▐", Style::default().fg(Color::Blue)),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "--- Symbols & Arrows ---",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from("← → ↑ ↓ ↔ ↕ ◀ ▶ ▲ ▼"),
                Line::from("● ○ ■ □ ▪ ▫ ◆ ◇ ★ ☆"),
                Line::from("✓ ✗ ⚡ ♠ ♣ ♥ ♦ … · •"),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "--- Color Swatch ---",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                // Foreground colors
                Line::from(vec![
                    Span::styled(" Red ", Style::default().fg(Color::Red)),
                    Span::styled(" Green ", Style::default().fg(Color::Green)),
                    Span::styled(" Blue ", Style::default().fg(Color::Blue)),
                    Span::styled(" Yellow ", Style::default().fg(Color::Yellow)),
                    Span::styled(" Cyan ", Style::default().fg(Color::Cyan)),
                    Span::styled(" Magenta ", Style::default().fg(Color::Magenta)),
                ]),
                // Background colors
                Line::from(vec![
                    Span::styled(" Red ", Style::default().fg(Color::White).bg(Color::Red)),
                    Span::styled(" Green ", Style::default().fg(Color::Black).bg(Color::Green)),
                    Span::styled(" Blue ", Style::default().fg(Color::White).bg(Color::Blue)),
                    Span::styled(" Yellow ", Style::default().fg(Color::Black).bg(Color::Yellow)),
                    Span::styled(" Cyan ", Style::default().fg(Color::Black).bg(Color::Cyan)),
                    Span::styled(" Magenta ", Style::default().fg(Color::White).bg(Color::Magenta)),
                ]),
                // RGB gradient
                Line::from(vec![
                    Span::styled("  ", Style::default().bg(Color::Rgb(255, 0, 0))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(255, 127, 0))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(255, 255, 0))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(0, 255, 0))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(0, 255, 255))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(0, 127, 255))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(0, 0, 255))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(127, 0, 255))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(255, 0, 255))),
                    Span::styled("  ", Style::default().bg(Color::Rgb(255, 0, 127))),
                ]),
                // Styled text samples
                Line::from(vec![
                    Span::styled("Bold", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled("Dim", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                    Span::raw(" "),
                    Span::styled("Italic", Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)),
                    Span::raw(" "),
                    Span::styled("Underline", Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED)),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "--- Controls ---",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(format!("  Font size: {:.0}", config.font_size)),
                Line::from("  Up/Down  Navigate   Ctrl+/-  Font"),
                Line::from("  Enter    Toggle     e  Region"),
                Line::from("  r  Reset all        Ctrl+C  Quit"),
            ];

            let demo_block = Block::default()
                .title(" Preview ")
                .borders(Borders::ALL);
            let demo = Paragraph::new(demo_lines)
                .block(demo_block)
                .centered();
            frame.render_widget(demo, chunks[1]);
        })
        .unwrap();
}
