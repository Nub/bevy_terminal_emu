use std::collections::VecDeque;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

/// Queue of terminal input events for the ratatui app to consume.
#[derive(Resource, Default)]
pub struct TerminalInputQueue {
    pub events: VecDeque<terminput::Event>,
}

/// System that forwards Bevy keyboard events to the terminal input queue.
pub fn forward_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut queue: ResMut<TerminalInputQueue>,
) {
    for event in keyboard_events.read() {
        // Only process key presses (not releases)
        if event.state != ButtonState::Pressed {
            continue;
        }

        if let Some(terminal_event) = bevy_key_to_terminal_event(event, &keys) {
            queue.events.push_back(terminal_event);
        }
    }
}

/// Convert a Bevy KeyboardInput into a terminput Event.
fn bevy_key_to_terminal_event(
    event: &KeyboardInput,
    keys: &ButtonInput<KeyCode>,
) -> Option<terminput::Event> {
    let mut modifiers = terminput::KeyModifiers::NONE;

    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
        modifiers |= terminput::KeyModifiers::CTRL;
    }
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        modifiers |= terminput::KeyModifiers::SHIFT;
    }
    if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) {
        modifiers |= terminput::KeyModifiers::ALT;
    }
    if keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight) {
        modifiers |= terminput::KeyModifiers::SUPER;
    }

    let code = bevy_keycode_to_terminput(event)?;

    let kind = if event.repeat {
        terminput::KeyEventKind::Repeat
    } else {
        terminput::KeyEventKind::Press
    };

    Some(terminput::Event::Key(
        terminput::KeyEvent::new(code)
            .modifiers(modifiers)
            .kind(kind),
    ))
}

/// Map a Bevy KeyboardInput to a terminput KeyCode.
fn bevy_keycode_to_terminput(event: &KeyboardInput) -> Option<terminput::KeyCode> {
    // First try to get a character from the logical key / text
    if let Some(ref text) = event.text {
        if let Some(ch) = text.chars().next() {
            if !ch.is_control() {
                return Some(terminput::KeyCode::Char(ch));
            }
        }
    }

    // Fall back to physical key mapping
    match event.key_code {
        KeyCode::Space => Some(terminput::KeyCode::Char(' ')),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(terminput::KeyCode::Enter),
        KeyCode::Escape => Some(terminput::KeyCode::Esc),
        KeyCode::Backspace => Some(terminput::KeyCode::Backspace),
        KeyCode::Tab => Some(terminput::KeyCode::Tab),
        KeyCode::ArrowUp => Some(terminput::KeyCode::Up),
        KeyCode::ArrowDown => Some(terminput::KeyCode::Down),
        KeyCode::ArrowLeft => Some(terminput::KeyCode::Left),
        KeyCode::ArrowRight => Some(terminput::KeyCode::Right),
        KeyCode::Home => Some(terminput::KeyCode::Home),
        KeyCode::End => Some(terminput::KeyCode::End),
        KeyCode::PageUp => Some(terminput::KeyCode::PageUp),
        KeyCode::PageDown => Some(terminput::KeyCode::PageDown),
        KeyCode::Delete => Some(terminput::KeyCode::Delete),
        KeyCode::Insert => Some(terminput::KeyCode::Insert),
        KeyCode::F1 => Some(terminput::KeyCode::F(1)),
        KeyCode::F2 => Some(terminput::KeyCode::F(2)),
        KeyCode::F3 => Some(terminput::KeyCode::F(3)),
        KeyCode::F4 => Some(terminput::KeyCode::F(4)),
        KeyCode::F5 => Some(terminput::KeyCode::F(5)),
        KeyCode::F6 => Some(terminput::KeyCode::F(6)),
        KeyCode::F7 => Some(terminput::KeyCode::F(7)),
        KeyCode::F8 => Some(terminput::KeyCode::F(8)),
        KeyCode::F9 => Some(terminput::KeyCode::F(9)),
        KeyCode::F10 => Some(terminput::KeyCode::F(10)),
        KeyCode::F11 => Some(terminput::KeyCode::F(11)),
        KeyCode::F12 => Some(terminput::KeyCode::F(12)),
        KeyCode::CapsLock => Some(terminput::KeyCode::CapsLock),
        KeyCode::ScrollLock => Some(terminput::KeyCode::ScrollLock),
        KeyCode::NumLock => Some(terminput::KeyCode::NumLock),
        KeyCode::Pause => Some(terminput::KeyCode::Pause),
        // Skip modifier-only keys
        KeyCode::ShiftLeft
        | KeyCode::ShiftRight
        | KeyCode::ControlLeft
        | KeyCode::ControlRight
        | KeyCode::AltLeft
        | KeyCode::AltRight
        | KeyCode::SuperLeft
        | KeyCode::SuperRight => None,
        _ => None,
    }
}
