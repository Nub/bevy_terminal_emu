use bevy::prelude::*;
use bevy_terminal_emu::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

struct MyTerminal;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TerminalEmuPlugin::<MyTerminal>::default())
        .insert_resource(Counter(0))
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (handle_input, draw_ui).chain().in_set(TerminalSet::AppTick),
        )
        .run();
}

#[derive(Resource)]
struct Counter(i32);

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn handle_input(mut queue: ResMut<TerminalInputQueue<MyTerminal>>, mut counter: ResMut<Counter>) {
    while let Some(event) = queue.events.pop_front() {
        if let terminput::Event::Key(key_event) = event {
            match key_event.code {
                terminput::KeyCode::Up => counter.0 += 1,
                terminput::KeyCode::Down => counter.0 -= 1,
                terminput::KeyCode::Char('r') => counter.0 = 0,
                _ => {}
            }
        }
    }
}

fn draw_ui(terminal_res: Res<TerminalResource<MyTerminal>>, counter: Res<Counter>) {
    let mut terminal = terminal_res.0.lock().unwrap();

    terminal
        .draw(|frame| {
            let area = frame.area();
            let block = Block::default()
                .title(" Counter ")
                .borders(Borders::ALL);
            let text = format!(
                "Counter: {}\n\n\
                 Up Arrow:   +1\n\
                 Down Arrow: -1\n\
                 r:          Reset\n\n\
                 Press Ctrl+C to exit.",
                counter.0
            );
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(paragraph, area);
        })
        .unwrap();
}
