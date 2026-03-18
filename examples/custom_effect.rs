use bevy::prelude::*;
use bevy_terminal_emu::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

struct MyTerminal;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TerminalEmuPlugin::<MyTerminal>::default())
        .add_systems(Startup, (setup_camera, spawn_effects))
        .add_systems(Update, draw_ui.in_set(TerminalSet::AppTick))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_effects(mut commands: Commands) {
    // Spawn built-in effects targeting the terminal.
    // Each effect component + EffectRegion + TargetTerminal = one entity.
    commands.spawn((
        Wave {
            amplitude: 0.15,
            speed: 2.0,
            wavelength: 4.0,
            ..Default::default()
        },
        EffectRegion::all(),
        TargetTerminal::<MyTerminal>::default(),
    ));

    commands.spawn((
        Glow {
            speed: 1.5,
            intensity: 0.8,
            spread: 3.0,
        },
        EffectRegion::all(),
        TargetTerminal::<MyTerminal>::default(),
    ));
}

fn draw_ui(mut terminal_res: ResMut<TerminalResource<MyTerminal>>) {

    terminal_res.0
        .draw(|frame| {
            let area = frame.area();
            let block = Block::default()
                .title(" GPU Effects Demo ")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new(
                "Effects are now GPU-driven via shader uniforms!\n\n\
                 To use an effect:\n\
                 1. Define a component (Wave, Glow, Jitter, etc.)\n\
                 2. Spawn it with EffectRegion + TargetTerminal<T>\n\
                 3. The aggregate_effects system reads components\n\
                    and writes uniforms to the shader each frame.\n\n\
                 No per-cell entity queries needed.",
            )
            .block(block);
            frame.render_widget(paragraph, area);
        })
        .unwrap();
}
