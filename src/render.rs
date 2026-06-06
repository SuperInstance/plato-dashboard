//! Terminal rendering utilities — ANSI escape codes, box drawing, colors, layout.

use std::fmt;

/// Terminal color codes.
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset,
}

impl Color {
    /// Return the ANSI foreground code for this color.
    pub fn fg_code(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[37m",
            Color::Reset => "\x1b[0m",
        }
    }

    /// Return the ANSI background code for this color.
    pub fn bg_code(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[41m",
            Color::Green => "\x1b[42m",
            Color::Yellow => "\x1b[43m",
            Color::Blue => "\x1b[44m",
            Color::Magenta => "\x1b[45m",
            Color::Cyan => "\x1b[46m",
            Color::White => "\x1b[47m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Box-drawing characters for panel borders.
pub mod box_drawing {
    pub const TOP_LEFT: char = '┌';
    pub const TOP_RIGHT: char = '┐';
    pub const BOTTOM_LEFT: char = '└';
    pub const BOTTOM_RIGHT: char = '┘';
    pub const HORIZONTAL: char = '─';
    pub const VERTICAL: char = '│';
    pub const T_DOWN: char = '┬';
    pub const T_UP: char = '┴';
    pub const T_RIGHT: char = '├';
    pub const T_LEFT: char = '┤';
    pub const CROSS: char = '┼';
}

/// Terminal rendering helper.
pub struct Render {
    width: usize,
    height: usize,
}

impl Render {
    /// Create a new renderer with the given terminal dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Create a renderer with default 80x24 dimensions.
    pub fn default_size() -> Self {
        Self::new(80, 24)
    }

    /// Get the terminal width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the terminal height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Generate a clear screen escape sequence.
    pub fn clear_screen() -> String {
        "\x1b[2J\x1b[H".to_string()
    }

    /// Move cursor to position (row, col), 1-indexed.
    pub fn move_cursor(row: usize, col: usize) -> String {
        format!("\x1b[{};{}H", row, col)
    }

    /// Hide the cursor.
    pub fn hide_cursor() -> String {
        "\x1b[?25l".to_string()
    }

    /// Show the cursor.
    pub fn show_cursor() -> String {
        "\x1b[?25h".to_string()
    }

    /// Color a string with a foreground color.
    pub fn colorize(text: &str, color: Color) -> String {
        format!("{}{}{}", color.fg_code(), text, Color::Reset.fg_code())
    }

    /// Draw a horizontal line of the given width using box-drawing characters.
    pub fn horizontal_line(width: usize) -> String {
        format!("{}", box_drawing::HORIZONTAL.to_string().repeat(width))
    }

    /// Draw a box with a title at the given position and dimensions.
    /// Returns lines ready to render.
    pub fn draw_box(title: &str, width: usize, height: usize) -> Vec<String> {
        use crate::render::box_drawing as bd;
        let inner_width = width.saturating_sub(2);
        let mut lines = Vec::new();

        // Top border with title
        let title_str = format!(" {} ", title);
        let title_len = title_str.chars().count();
        let remaining = inner_width.saturating_sub(title_len);
        let top = format!(
            "{}{}{}{}{}",
            bd::TOP_LEFT,
            bd::HORIZONTAL.to_string().repeat(remaining / 2),
            title_str,
            bd::HORIZONTAL.to_string().repeat(remaining - remaining / 2),
            bd::TOP_RIGHT
        );
        lines.push(top);

        // Side borders
        for _ in 1..(height.saturating_sub(1)) {
            lines.push(format!(
                "{}{}{}",
                bd::VERTICAL,
                " ".repeat(inner_width),
                bd::VERTICAL
            ));
        }

        // Bottom border
        if height > 1 {
            lines.push(format!(
                "{}{}{}",
                bd::BOTTOM_LEFT,
                bd::HORIZONTAL.to_string().repeat(inner_width),
                bd::BOTTOM_RIGHT
            ));
        }

        lines
    }

    /// Center text within a given width.
    pub fn center(text: &str, width: usize) -> String {
        let text_len = text.chars().count();
        if text_len >= width {
            return text.to_string();
        }
        let left = (width - text_len) / 2;
        let right = width - text_len - left;
        format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
    }

    /// Pad or truncate text to exact width.
    pub fn pad(text: &str, width: usize) -> String {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() >= width {
            chars[..width].iter().collect()
        } else {
            let pad = width - chars.len();
            format!("{}{}", text, " ".repeat(pad))
        }
    }

    /// Reset all formatting.
    pub fn reset() -> String {
        "\x1b[0m".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_drawing() {
        let lines = Render::draw_box("Test Panel", 30, 5);
        assert_eq!(lines.len(), 5);
        assert!(lines[0].starts_with('┌'));
        assert!(lines[0].ends_with('┐'));
        assert!(lines[4].starts_with('└'));
        assert!(lines[4].ends_with('┘'));
        // Middle lines have vertical bars
        assert!(lines[1].starts_with('│'));
        assert!(lines[1].ends_with('│'));
    }

    #[test]
    fn test_color_formatting() {
        let red = Render::colorize("ALERT", Color::Red);
        assert!(red.contains("\x1b[31m"));
        assert!(red.contains("ALERT"));
        assert!(red.contains("\x1b[0m"));
    }

    #[test]
    fn test_center() {
        let centered = Render::center("hi", 10);
        assert_eq!(centered, "    hi    ");
    }

    #[test]
    fn test_pad_truncate() {
        let padded = Render::pad("hello", 10);
        assert_eq!(padded, "hello     ");
        let truncated = Render::pad("hello world", 5);
        assert_eq!(truncated, "hello");
    }

    #[test]
    fn test_clear_screen() {
        let clear = Render::clear_screen();
        assert!(clear.contains("\x1b[2J"));
    }

    #[test]
    fn test_move_cursor() {
        let mv = Render::move_cursor(5, 10);
        assert_eq!(mv, "\x1b[5;10H");
    }

    #[test]
    fn test_color_codes() {
        assert_eq!(Color::Red.fg_code(), "\x1b[31m");
        assert_eq!(Color::Green.bg_code(), "\x1b[42m");
        assert_eq!(Color::Reset.fg_code(), "\x1b[0m");
    }

    #[test]
    fn test_horizontal_line() {
        let line = Render::horizontal_line(5);
        assert_eq!(line, "─────");
    }

    #[test]
    fn test_hide_show_cursor() {
        assert_eq!(Render::hide_cursor(), "\x1b[?25l");
        assert_eq!(Render::show_cursor(), "\x1b[?25h");
    }
}
