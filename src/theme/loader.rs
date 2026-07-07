use std::fs;

use serde::Deserialize;

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
    let Some(name) = name.filter(|name| !name.trim().is_empty()) else {
        return Ok(("default".to_string(), ThemePalette::default_dark()));
    };

    if name == "default" {
        return Ok(("default".to_string(), ThemePalette::default_dark()));
    }

    let path = paths::themes_dir().join(format!("{name}.json"));
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("could not read theme {}: {error}", path.to_string_lossy()))?;
    let theme: ThemeFile = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    let palette = build_palette(&theme)?;

    Ok((theme.name.unwrap_or_else(|| name.to_string()), palette))
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
