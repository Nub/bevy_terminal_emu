use bevy::prelude::*;
use bevy_terminal_emu::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TerminalEmuPlugin::default())
        .add_systems(Startup, (setup_camera, spawn_spin_effect))
        .add_systems(Update, draw_ui.in_set(TerminalSet::AppTick))
        .add_systems(Update, spin_system.in_set(TerminalSet::Effects))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// Step 1: Define a component with effect params
#[derive(Component)]
struct SpinEffect {
    speed: f32,
    max_angle: f32,
}

// Step 2: Spawn an effect entity with the component and an EffectRegion
fn spawn_spin_effect(mut commands: Commands) {
    commands.spawn((
        SpinEffect {
            speed: 2.0,
            max_angle: 0.15,
        },
        EffectRegion::all(),
    ));
}

// Step 3: Write a system that queries effects and cells
fn spin_system(
    time: Res<Time>,
    effects: Query<(&SpinEffect, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    let t = time.elapsed_secs();

    for (spin, region) in effects.iter() {
        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) {
                continue;
            }

            // Each cell gets a slightly different phase based on position
            let phase = (pos.col as f32 * 0.3) + (pos.row as f32 * 0.5);
            let angle = spin.max_angle * (spin.speed * t + phase).sin();

            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn draw_ui(terminal_res: Res<TerminalResource>) {
    let mut terminal = terminal_res.0.lock().unwrap();

    terminal
        .draw(|frame| {
            let area = frame.area();
            let block = Block::default()
                .title(" Custom Spin Effect ")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new(
                "Each cell has a custom spin effect applied!\n\n\
                 This demonstrates the 3-step custom effect pattern:\n\
                 1. Define a Component with effect params\n\
                 2. Write a system querying effects + cells\n\
                 3. Register in TerminalSet::Effects\n\n\
                 No traits or registration boilerplate needed.",
            )
            .block(block);
            frame.render_widget(paragraph, area);
        })
        .unwrap();
}
