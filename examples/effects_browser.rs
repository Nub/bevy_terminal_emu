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
        .add_plugins(TerminalEmuPlugin::default())
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

/// Browser UI state.
#[derive(Resource)]
struct BrowserState {
    selected: usize,
    effects: Vec<EffectEntry>,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            selected: 0,
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
            ],
        }
    }
}

/// Tracks spawned effect entities so we can despawn them on toggle-off.
#[derive(Resource, Default)]
struct ActiveEffectEntities {
    map: HashMap<usize, Entity>,
}

fn handle_input(mut queue: ResMut<TerminalInputQueue>, mut state: ResMut<BrowserState>) {
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
) {
    for (idx, effect) in state.effects.iter().enumerate() {
        let is_spawned = active.map.contains_key(&idx);

        if effect.active && !is_spawned {
            // Spawn the effect entity
            let entity = match idx {
                0 => commands.spawn((Wave::default(), EffectRegion::all())).id(),
                1 => commands.spawn((Ripple::default(), EffectRegion::all())).id(),
                2 => commands
                    .spawn((Collapse::default(), EffectRegion::all()))
                    .id(),
                3 => commands
                    .spawn((Gravity::default(), EffectRegion::all()))
                    .id(),
                4 => commands.spawn((Glitch::default(), EffectRegion::all())).id(),
                5 => commands
                    .spawn((Scatter::default(), EffectRegion::all()))
                    .id(),
                6 => commands
                    .spawn((Breathe::default(), EffectRegion::all()))
                    .id(),
                7 => commands.spawn((Jitter::default(), EffectRegion::all())).id(),
                8 => commands.spawn((Slash::default(), EffectRegion::all())).id(),
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
            // For Collapse/Scatter: re-toggling resets the animation
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

fn draw_ui(terminal_res: Res<TerminalResource>, state: Res<BrowserState>) {
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
                Line::from("The quick brown fox jumps"),
                Line::from("over the lazy dog."),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "--- Controls ---",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from("  Up/Down    Navigate"),
                Line::from("  Enter/Space   Toggle"),
                Line::from("  r          Reset all"),
                Line::from("  Ctrl+C     Quit"),
            ];

            let demo_block = Block::default()
                .title(" Preview ")
                .borders(Borders::ALL);
            let demo = Paragraph::new(demo_lines).block(demo_block);
            frame.render_widget(demo, chunks[1]);
        })
        .unwrap();
}
