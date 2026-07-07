use std::fs;

use serde::Deserialize;

use ratatui::style::Color;

use crate::{config::paths, theme::palette::ThemePalette};

#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: Option<String>,
    #[serde(default)]
    colors: ThemeColors,
    #[serde(flatten)]
    root_colors: ThemeColors,
}

#[derive(Debug, Default, Deserialize)]
struct ThemeColors {
    background: Option<String>,
    foreground: Option<String>,
    border: Option<String>,
    focused_border: Option<String>,
    muted_text: Option<String>,
    selected_text: Option<String>,
    selected_background: Option<String>,
    accent: Option<String>,
    accent_secondary: Option<String>,
    progress_empty: Option<String>,
    progress_fill: Option<String>,
    visualizer: Option<String>,
    warning: Option<String>,
    error: Option<String>,
    success: Option<String>,
    status_playing: Option<String>,
    status_paused: Option<String>,
    status_stopped: Option<String>,
    status_loading: Option<String>,
    lyrics_current: Option<String>,
    lyrics_previous_next: Option<String>,
    cache_marker: Option<String>,
    table_header: Option<String>,
    footer_text: Option<String>,
    selected_fg: Option<String>,
    selected_bg: Option<String>,
}

pub fn load_theme(name: Option<&str>) -> Result<(String, ThemePalette), String> {
    let theme_name = name.filter(|n| !n.trim().is_empty()).unwrap_or("default");

    let path = paths::themes_dir().join(format!("{theme_name}.json"));

    if !path.exists() && theme_name == "default" {
        let palette = ThemePalette::default_dark();
        let json =
            serde_json::to_string_pretty(&theme_to_map(&palette)).map_err(|e| e.to_string())?;
        if std::fs::create_dir_all(path.parent().unwrap())
            .and_then(|()| std::fs::write(&path, &json))
            .is_err()
        {
            return Ok(("default".to_string(), palette));
        }
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) if theme_name == "default" => {
            return Ok(("default".to_string(), ThemePalette::default_dark()));
        }
        Err(error) => {
            return Err(format!(
                "could not read theme {}: {error}",
                path.to_string_lossy()
            ));
        }
    };

    if text.trim().is_empty() && theme_name == "default" {
        return Ok(("default".to_string(), ThemePalette::default_dark()));
    }
    if text.trim().is_empty() {
        return Err(format!("theme file is empty: {}", path.to_string_lossy()));
    }

    let theme: ThemeFile = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    let palette = build_palette(&theme)?;

    Ok((
        theme.name.unwrap_or_else(|| theme_name.to_string()),
        palette,
    ))
}

fn theme_to_map(palette: &ThemePalette) -> std::collections::BTreeMap<&str, String> {
    use std::collections::BTreeMap;
    fn hex(c: Color) -> String {
        match c {
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
            Color::Black => "#000000".to_string(),
            Color::Red => "#ff0000".to_string(),
            Color::Green => "#00ff00".to_string(),
            Color::Yellow => "#ffff00".to_string(),
            Color::Blue => "#0000ff".to_string(),
            Color::Magenta => "#ff00ff".to_string(),
            Color::Cyan => "#00ffff".to_string(),
            Color::Gray => "#808080".to_string(),
            Color::DarkGray => "#404040".to_string(),
            Color::LightRed => "#ff4444".to_string(),
            Color::LightGreen => "#44ff44".to_string(),
            Color::LightYellow => "#ffff44".to_string(),
            Color::LightBlue => "#4444ff".to_string(),
            Color::LightMagenta => "#ff44ff".to_string(),
            Color::LightCyan => "#44ffff".to_string(),
            Color::White => "#ffffff".to_string(),
            _ => "#000000".to_string(),
        }
    }
    BTreeMap::from_iter([
        ("background", hex(palette.background)),
        ("foreground", hex(palette.foreground)),
        ("border", hex(palette.border)),
        ("focused_border", hex(palette.focused_border)),
        ("muted_text", hex(palette.muted_text)),
        ("selected_text", hex(palette.selected_text)),
        ("selected_background", hex(palette.selected_background)),
        ("accent", hex(palette.accent)),
        ("accent_secondary", hex(palette.accent_secondary)),
        ("progress_empty", hex(palette.progress_empty)),
        ("progress_fill", hex(palette.progress_fill)),
        ("visualizer", hex(palette.visualizer)),
        ("warning", hex(palette.warning)),
        ("error", hex(palette.error)),
        ("success", hex(palette.success)),
        ("status_playing", hex(palette.status_playing)),
        ("status_paused", hex(palette.status_paused)),
        ("status_stopped", hex(palette.status_stopped)),
        ("status_loading", hex(palette.status_loading)),
        ("lyrics_current", hex(palette.lyrics_current)),
        ("lyrics_previous_next", hex(palette.lyrics_previous_next)),
        ("cache_marker", hex(palette.cache_marker)),
        ("table_header", hex(palette.table_header)),
        ("footer_text", hex(palette.footer_text)),
    ])
}

fn build_palette(theme: &ThemeFile) -> Result<ThemePalette, String> {
    let mut palette = ThemePalette::default_dark();
    for (name, value) in theme.colors.entries().chain(theme.root_colors.entries()) {
        palette = palette.with_color(name, value)?;
    }
    Ok(palette)
}

impl ThemeColors {
    fn entries(&self) -> impl Iterator<Item = (&'static str, &str)> {
        [
            ("background", self.background.as_deref()),
            ("foreground", self.foreground.as_deref()),
            ("border", self.border.as_deref()),
            ("focused_border", self.focused_border.as_deref()),
            ("muted_text", self.muted_text.as_deref()),
            ("selected_text", self.selected_text.as_deref()),
            ("selected_background", self.selected_background.as_deref()),
            ("accent", self.accent.as_deref()),
            ("accent_secondary", self.accent_secondary.as_deref()),
            ("progress_empty", self.progress_empty.as_deref()),
            ("progress_fill", self.progress_fill.as_deref()),
            ("visualizer", self.visualizer.as_deref()),
            ("warning", self.warning.as_deref()),
            ("error", self.error.as_deref()),
            ("success", self.success.as_deref()),
            ("status_playing", self.status_playing.as_deref()),
            ("status_paused", self.status_paused.as_deref()),
            ("status_stopped", self.status_stopped.as_deref()),
            ("status_loading", self.status_loading.as_deref()),
            ("lyrics_current", self.lyrics_current.as_deref()),
            ("lyrics_previous_next", self.lyrics_previous_next.as_deref()),
            ("cache_marker", self.cache_marker.as_deref()),
            ("table_header", self.table_header.as_deref()),
            ("footer_text", self.footer_text.as_deref()),
            ("selected_fg", self.selected_fg.as_deref()),
            ("selected_bg", self.selected_bg.as_deref()),
        ]
        .into_iter()
        .filter_map(|(name, value)| value.map(|value| (name, value)))
    }
}
