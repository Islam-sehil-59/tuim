use ratatui::style::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemePalette {
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub focused_border: Color,
    pub muted_text: Color,
    pub selected_text: Color,
    pub selected_background: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub progress_empty: Color,
    pub progress_fill: Color,
    pub visualizer: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
    pub status_playing: Color,
    pub status_paused: Color,
    pub status_stopped: Color,
    pub status_loading: Color,
    pub lyrics_current: Color,
    pub lyrics_previous_next: Color,
    pub cache_marker: Color,
    pub table_header: Color,
    pub footer_text: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
}

impl ThemePalette {
    pub fn default_dark() -> Self {
        Self {
            background: Color::Rgb(0x15, 0x15, 0x15),
            foreground: Color::Rgb(0xed, 0xec, 0xec),
            border: Color::Rgb(0xac, 0xa3, 0xa3),
            focused_border: Color::Rgb(0xe0, 0xe0, 0xe0),
            muted_text: Color::Rgb(0x8a, 0x8a, 0x8a),
            selected_text: Color::Rgb(0x15, 0x15, 0x15),
            selected_background: Color::Rgb(0xd8, 0xd8, 0xd8),
            accent: Color::Rgb(0x3a, 0x8c, 0xff),
            accent_secondary: Color::Rgb(0x48, 0xb8, 0x4a),
            progress_empty: Color::Rgb(0x44, 0x44, 0x44),
            progress_fill: Color::Rgb(0x3a, 0x8c, 0xff),
            visualizer: Color::Rgb(0x3a, 0x8c, 0xff),
            warning: Color::Rgb(0xd9, 0xa4, 0x41),
            error: Color::Rgb(0xff, 0x5f, 0x5f),
            success: Color::Rgb(0x48, 0xb8, 0x4a),
            status_playing: Color::Rgb(0x48, 0xb8, 0x4a),
            status_paused: Color::Rgb(0xd9, 0xa4, 0x41),
            status_stopped: Color::Rgb(0x8a, 0x8a, 0x8a),
            status_loading: Color::Rgb(0x3a, 0x8c, 0xff),
            lyrics_current: Color::White,
            lyrics_previous_next: Color::Rgb(0x8a, 0x8a, 0x8a),
            cache_marker: Color::Rgb(0x48, 0xb8, 0x4a),
            table_header: Color::Rgb(0xbc, 0xbc, 0xbc),
            footer_text: Color::Rgb(0x8a, 0x8a, 0x8a),
            selected_fg: Color::Rgb(0x15, 0x15, 0x15),
            selected_bg: Color::Rgb(0xd8, 0xd8, 0xd8),
        }
    }

    pub fn from_color_strings(
        accent: &str,
        selected_fg: &str,
        selected_bg: &str,
    ) -> Result<Self, String> {
        let mut palette = Self::default_dark();
        palette.accent = parse_color(accent)?;
        palette.progress_fill = palette.accent;
        palette.visualizer = palette.accent;
        palette.status_loading = palette.accent;
        palette.selected_fg = parse_color(selected_fg)?;
        palette.selected_text = palette.selected_fg;
        palette.selected_bg = parse_color(selected_bg)?;
        palette.selected_background = palette.selected_bg;
        Ok(palette)
    }

    pub fn with_color(mut self, name: &str, value: &str) -> Result<Self, String> {
        let color = parse_color(value)?;
        match name {
            "background" => self.background = color,
            "foreground" => self.foreground = color,
            "border" => self.border = color,
            "focused_border" => self.focused_border = color,
            "muted_text" => self.muted_text = color,
            "selected_text" | "selected_fg" => {
                self.selected_text = color;
                self.selected_fg = color;
            }
            "selected_background" | "selected_bg" => {
                self.selected_background = color;
                self.selected_bg = color;
            }
            "accent" => self.accent = color,
            "accent_secondary" => self.accent_secondary = color,
            "progress_empty" => self.progress_empty = color,
            "progress_fill" => self.progress_fill = color,
            "visualizer" => self.visualizer = color,
            "warning" => self.warning = color,
            "error" => self.error = color,
            "success" => self.success = color,
            "status_playing" => self.status_playing = color,
            "status_paused" => self.status_paused = color,
            "status_stopped" => self.status_stopped = color,
            "status_loading" => self.status_loading = color,
            "lyrics_current" => self.lyrics_current = color,
            "lyrics_previous_next" => self.lyrics_previous_next = color,
            "cache_marker" => self.cache_marker = color,
            "table_header" => self.table_header = color,
            "footer_text" => self.footer_text = color,
            _ => return Err(format!("unknown theme color: {name}")),
        }
        Ok(self)
    }
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self::default_dark()
    }
}

fn parse_color(value: &str) -> Result<Color, String> {
    let color = value.trim().to_ascii_lowercase();
    match color.as_str() {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "dark_gray" | "dark_grey" => Ok(Color::DarkGray),
        "light_red" => Ok(Color::LightRed),
        "light_green" => Ok(Color::LightGreen),
        "light_yellow" => Ok(Color::LightYellow),
        "light_blue" => Ok(Color::LightBlue),
        "light_magenta" => Ok(Color::LightMagenta),
        "light_cyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        _ => parse_hex_color(&color),
    }
}

fn parse_hex_color(value: &str) -> Result<Color, String> {
    let hex = value
        .strip_prefix('#')
        .ok_or_else(|| format!("unsupported color value: {value}"))?;

    if hex.len() != 6 {
        return Err(format!("hex colors must use #rrggbb: {value}"));
    }

    let red = u8::from_str_radix(&hex[0..2], 16).map_err(|error| error.to_string())?;
    let green = u8::from_str_radix(&hex[2..4], 16).map_err(|error| error.to_string())?;
    let blue = u8::from_str_radix(&hex[4..6], 16).map_err(|error| error.to_string())?;

    Ok(Color::Rgb(red, green, blue))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_and_hex_colors() {
        assert_eq!(parse_color("green"), Ok(Color::Green));
        assert_eq!(parse_color("#112233"), Ok(Color::Rgb(0x11, 0x22, 0x33)));
    }

    #[test]
    fn rejects_invalid_colors() {
        assert!(parse_color("not-a-color").is_err());
        assert!(parse_color("#123").is_err());
    }

    #[test]
    fn legacy_theme_colors_fill_new_palette_fields() {
        let palette = ThemePalette::from_color_strings("#112233", "#eeeeee", "#010203").unwrap();

        assert_eq!(palette.accent, Color::Rgb(0x11, 0x22, 0x33));
        assert_eq!(palette.progress_fill, Color::Rgb(0x11, 0x22, 0x33));
        assert_eq!(palette.visualizer, Color::Rgb(0x11, 0x22, 0x33));
        assert_eq!(palette.selected_text, Color::Rgb(0xee, 0xee, 0xee));
        assert_eq!(palette.selected_background, Color::Rgb(0x01, 0x02, 0x03));
    }

    #[test]
    fn palette_fields_can_be_overridden_individually() {
        let palette = ThemePalette::default_dark()
            .with_color("visualizer", "#123456")
            .unwrap()
            .with_color("footer_text", "light_blue")
            .unwrap();

        assert_eq!(palette.visualizer, Color::Rgb(0x12, 0x34, 0x56));
        assert_eq!(palette.footer_text, Color::LightBlue);
    }
}
