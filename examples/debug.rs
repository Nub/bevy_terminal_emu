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

fn draw_ui(terminal_res: Res<TerminalResource<MyTerminal>>) {
    let mut terminal = terminal_res.0.lock().unwrap();

    terminal
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
    layouts: Res<Assets<TextureAtlasLayout>>,
    fg_sprites: Query<&Sprite, With<ForegroundSprite<MyTerminal>>>,
    bg_sprites: Query<&Sprite, With<BackgroundSprite<MyTerminal>>>,
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

    // Check if atlas layout is loaded
    let layout_loaded = layouts.get(&atlas.layout).is_some();
    info!("Atlas layout handle loaded: {}", layout_loaded);
    if let Some(layout) = layouts.get(&atlas.layout) {
        info!("Atlas layout size: {:?}, textures count: {}", layout.size, layout.textures.len());
    }

    // Check a FG sprite's image handle matches atlas
    for sprite in fg_sprites.iter().take(1) {
        let fg_img_matches = sprite.image == atlas.image;
        info!("FG sprite image handle matches atlas: {}", fg_img_matches);
        info!("FG sprite image == default: {}", sprite.image == Handle::default());
    }

    // Check a BG sprite's image handle
    for sprite in bg_sprites.iter().take(1) {
        info!("BG sprite image == default: {}", sprite.image == Handle::default());
        info!("BG sprite color: {:?}", sprite.color);
        info!("BG sprite custom_size: {:?}", sprite.custom_size);
    }

    // Try spawning a test sprite manually to see if it renders
    info!("=== If we can see the BG sprites, the grid cells should be visible at (-395..405, -230..230) ===");
}
