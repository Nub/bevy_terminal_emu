# bevy_terminal_emu

A Bevy library that renders [ratatui](https://github.com/ratatui/ratatui) apps where each character is an independent Bevy entity. Write your UI with ratatui, then apply per-cell visual effects — waves, explosions, glitches, and more — powered by Bevy's ECS.

## How It Works

1. You write a normal ratatui app using `Terminal::draw()`
2. `bevy_terminal_emu` syncs the ratatui buffer to a grid of Bevy sprite entities (one parent + two children per cell: background and foreground glyph)
3. Effect systems run each frame, modifying cell transforms additively
4. Bevy renders the result as GPU-accelerated sprites

## Quick Start

```rust
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
                .title(" Hello ")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new("Each character is a Bevy entity!")
                .block(block);
            frame.render_widget(paragraph, area);
        })
        .unwrap();
}
```

## Built-in Effects

| Effect | Type | Description |
|--------|------|-------------|
| **Wave** | Continuous | Sine wave vertical oscillation |
| **Ripple** | Continuous | Radial wave from center point |
| **Breathe** | Continuous | Rhythmic scale pulse |
| **Jitter** | Continuous | Per-cell random vibration |
| **Glitch** | Continuous | CRT-style horizontal row shift |
| **Gravity** | Continuous | Downward acceleration with per-cell velocity |
| **Collapse** | One-shot | Cells fall with staggered timing |
| **Scatter** | One-shot | Smooth radial explosion from center |
| **Explode** | One-shot | Chaotic explosion with randomized velocity and spin |
| **Slash** | One-shot | Diagonal swipe across screen |

Spawn any effect by adding its component alongside an `EffectRegion`:

```rust
commands.spawn((Wave::default(), EffectRegion::all()));
```

### Region Targeting

Effects can target subsets of the grid using `EffectRegion` with include/exclude rectangles:

```rust
// Only affect the left half of the screen
commands.spawn((
    Ripple::default(),
    EffectRegion {
        include: vec![GridRect { col: 0, row: 0, width: 40, height: 24 }],
        exclude: vec![],
    },
));
```

## Custom Effects

Define a component, write a system, register it in `TerminalSet::Effects`:

```rust
#[derive(Component)]
struct SpinEffect { speed: f32, max_angle: f32 }

fn spin_system(
    time: Res<Time>,
    effects: Query<(&SpinEffect, &EffectRegion)>,
    mut cells: Query<(&GridPosition, &mut Transform), With<TerminalCell>>,
) {
    let t = time.elapsed_secs();
    for (spin, region) in effects.iter() {
        for (pos, mut transform) in cells.iter_mut() {
            if !region.contains(pos.col, pos.row) { continue; }
            let phase = (pos.col as f32 * 0.3) + (pos.row as f32 * 0.5);
            transform.rotation = Quat::from_rotation_z(
                spin.max_angle * (spin.speed * t + phase).sin()
            );
        }
    }
}

// In your app:
app.add_systems(Update, spin_system.in_set(TerminalSet::Effects));
```

## Input Handling

Keyboard events are forwarded as [terminput](https://docs.rs/terminput) events via `TerminalInputQueue`:

```rust
fn handle_input(mut queue: ResMut<TerminalInputQueue>) {
    while let Some(event) = queue.events.pop_front() {
        if let terminput::Event::Key(key_event) = event {
            match key_event.code {
                terminput::KeyCode::Up => { /* ... */ }
                terminput::KeyCode::Char('q') => { /* ... */ }
                _ => {}
            }
        }
    }
}
```

## Configuration

```rust
TerminalEmuPlugin {
    config: TerminalConfig {
        columns: 80,
        rows: 24,
        cell_width: 10.0,
        cell_height: 20.0,
        font_size: 20.0,
        ..default()
    },
}
```

## System Sets

Systems are ordered via `TerminalSet`:

1. **`AppTick`** — Your ratatui draw + input handling
2. **`Sync`** — Buffer-to-entity sync
3. **`ResetTransforms`** — Reset cell transforms to base positions
4. **`Effects`** — All effect systems run here

## Examples

```sh
cargo run --example basic           # Minimal ratatui app
cargo run --example counter         # Interactive counter with keyboard input
cargo run --example custom_effect   # Custom spin effect pattern
cargo run --example effects_browser # Interactive browser for all 10 effects
```

## Dependencies

- [Bevy](https://bevyengine.org/) 0.18
- [ratatui](https://ratatui.rs/) 0.30
- [terminput](https://docs.rs/terminput) 0.3
- [ab_glyph](https://docs.rs/ab_glyph) 0.2

## License

MIT OR Apache-2.0
