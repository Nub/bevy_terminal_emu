use bevy::prelude::*;
use bevy_terminal_emu::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TerminalEmuPlugin::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, draw_ui.in_set(TerminalSet::AppTick))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn draw_ui(terminal_res: Res<TerminalResource>) {
    let mut terminal = terminal_res.0.lock().unwrap();

    terminal
        .draw(|frame| {
            let area = frame.area();
            let block = Block::default()
                .title(" Hello bevy_terminal_emu ")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new(
                "This is a ratatui app rendered as Bevy sprites!\n\n\
                 Each character is an independent Bevy entity.\n\
                 Effects can target regions of cells.\n\n\
                 Press Ctrl+C to exit.",
            )
            .block(block);
            frame.render_widget(paragraph, area);
        })
        .unwrap();
}
