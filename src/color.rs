use bevy::color::Color;
use ratatui::style::Color as RatColor;

/// Convert a ratatui Color to a Bevy Color.
pub fn ratatui_color_to_bevy(color: RatColor) -> Color {
    match color {
        RatColor::Reset => Color::WHITE,
        RatColor::Black => Color::srgb(0.0, 0.0, 0.0),
        RatColor::Red => Color::srgb(0.8, 0.0, 0.0),
        RatColor::Green => Color::srgb(0.0, 0.8, 0.0),
        RatColor::Yellow => Color::srgb(0.8, 0.8, 0.0),
        RatColor::Blue => Color::srgb(0.0, 0.0, 0.8),
        RatColor::Magenta => Color::srgb(0.8, 0.0, 0.8),
        RatColor::Cyan => Color::srgb(0.0, 0.8, 0.8),
        RatColor::Gray => Color::srgb(0.75, 0.75, 0.75),
        RatColor::DarkGray => Color::srgb(0.5, 0.5, 0.5),
        RatColor::LightRed => Color::srgb(1.0, 0.33, 0.33),
        RatColor::LightGreen => Color::srgb(0.33, 1.0, 0.33),
        RatColor::LightYellow => Color::srgb(1.0, 1.0, 0.33),
        RatColor::LightBlue => Color::srgb(0.33, 0.33, 1.0),
        RatColor::LightMagenta => Color::srgb(1.0, 0.33, 1.0),
        RatColor::LightCyan => Color::srgb(0.33, 1.0, 1.0),
        RatColor::White => Color::srgb(1.0, 1.0, 1.0),
        RatColor::Rgb(r, g, b) => Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        RatColor::Indexed(i) => indexed_color(i),
    }
}

/// Convert an 8-bit indexed color to a Bevy Color.
fn indexed_color(index: u8) -> Color {
    match index {
        // Standard 16 colors
        0 => Color::srgb(0.0, 0.0, 0.0),
        1 => Color::srgb(0.8, 0.0, 0.0),
        2 => Color::srgb(0.0, 0.8, 0.0),
        3 => Color::srgb(0.8, 0.8, 0.0),
        4 => Color::srgb(0.0, 0.0, 0.8),
        5 => Color::srgb(0.8, 0.0, 0.8),
        6 => Color::srgb(0.0, 0.8, 0.8),
        7 => Color::srgb(0.75, 0.75, 0.75),
        8 => Color::srgb(0.5, 0.5, 0.5),
        9 => Color::srgb(1.0, 0.33, 0.33),
        10 => Color::srgb(0.33, 1.0, 0.33),
        11 => Color::srgb(1.0, 1.0, 0.33),
        12 => Color::srgb(0.33, 0.33, 1.0),
        13 => Color::srgb(1.0, 0.33, 1.0),
        14 => Color::srgb(0.33, 1.0, 1.0),
        15 => Color::srgb(1.0, 1.0, 1.0),
        // 216-color cube (indices 16..=231)
        16..=231 => {
            let n = index - 16;
            let b = n % 6;
            let g = (n / 6) % 6;
            let r = n / 36;
            Color::srgb(
                if r == 0 { 0.0 } else { (55.0 + 40.0 * r as f32) / 255.0 },
                if g == 0 { 0.0 } else { (55.0 + 40.0 * g as f32) / 255.0 },
                if b == 0 { 0.0 } else { (55.0 + 40.0 * b as f32) / 255.0 },
            )
        }
        // Grayscale ramp (indices 232..=255)
        232..=255 => {
            let v = (8 + 10 * (index - 232) as u32) as f32 / 255.0;
            Color::srgb(v, v, v)
        }
    }
}

/// Convert a ratatui foreground color to a Bevy Color, using a default for Reset.
pub fn ratatui_fg_to_bevy(color: RatColor, default: Color) -> Color {
    if color == RatColor::Reset {
        default
    } else {
        ratatui_color_to_bevy(color)
    }
}

/// Convert a ratatui background color to a Bevy Color, using a default for Reset.
pub fn ratatui_bg_to_bevy(color: RatColor, default: Color) -> Color {
    if color == RatColor::Reset {
        default
    } else {
        ratatui_color_to_bevy(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_colors() {
        let white = ratatui_color_to_bevy(RatColor::White);
        assert_eq!(white, Color::srgb(1.0, 1.0, 1.0));

        let black = ratatui_color_to_bevy(RatColor::Black);
        assert_eq!(black, Color::srgb(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_rgb_color() {
        let color = ratatui_color_to_bevy(RatColor::Rgb(128, 64, 255));
        assert_eq!(color, Color::srgb(128.0 / 255.0, 64.0 / 255.0, 1.0));
    }

    #[test]
    fn test_indexed_grayscale() {
        let color = ratatui_color_to_bevy(RatColor::Indexed(232));
        let v = 8.0 / 255.0;
        assert_eq!(color, Color::srgb(v, v, v));
    }

    #[test]
    fn test_reset_defaults() {
        let default_fg = Color::srgb(0.9, 0.9, 0.9);
        let result = ratatui_fg_to_bevy(RatColor::Reset, default_fg);
        assert_eq!(result, default_fg);
    }
}
