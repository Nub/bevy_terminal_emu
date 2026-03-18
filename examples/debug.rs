use bevy::prelude::*;
use bevy_terminal_emu::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

struct MyTerminal;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TerminalEmuPlugin::<MyTerminal>::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, draw_ui.in_set(TerminalSet::AppTick))
        .add_systems(Update, debug_system)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn a large test sprite to verify basic rendering works
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(200.0, 200.0)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));
}

fn draw_ui(mut terminal_res: ResMut<TerminalResource<MyTerminal>>) {

    terminal_res.0
        .draw(|frame| {
            let area = frame.area();
            let block = Block::default()
                .title(" Debug ")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new("Hello World!").block(block);
            frame.render_widget(paragraph, area);
        })
        .unwrap();
}

fn debug_system(
    atlas: Res<FontAtlasResource<MyTerminal>>,
    images: Res<Assets<Image>>,
    quad: Res<TerminalQuadEntity<MyTerminal>>,
    transforms: Query<&Transform>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    if *frame_count != 5 {
        return;
    }

    // Check if atlas image is loaded
    let img_loaded = images.get(&atlas.image).is_some();
    info!("Atlas image handle loaded: {}", img_loaded);
    if let Some(img) = images.get(&atlas.image) {
        info!("Atlas image size: {}x{}", img.width(), img.height());
    }

    info!("Atlas glyph count: {}", atlas.glyph_count);
    info!("Atlas cell size: {:?}", atlas.cell_size);

    // Check quad entity
    if let Ok(transform) = transforms.get(quad.entity) {
        info!("Quad entity {:?} at {:?}", quad.entity, transform.translation);
    }

    // Check cell data image
    let cell_data_loaded = images.get(&quad.cell_data_image).is_some();
    info!("Cell data image loaded: {}", cell_data_loaded);
    if let Some(img) = images.get(&quad.cell_data_image) {
        info!("Cell data image size: {}x{}", img.width(), img.height());
    }
}
