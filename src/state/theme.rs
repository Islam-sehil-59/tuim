use crate::theme::{loader, palette::ThemePalette};

pub struct ThemeState {
    pub name: String,
    pub palette: ThemePalette,
}

impl ThemeState {
    pub fn new() -> Self {
        Self {
            name: "default".to_string(),
            palette: ThemePalette::default_dark(),
        }
    }

    pub fn load(active_theme: Option<&str>) -> Result<Self, String> {
        let (name, palette) = loader::load_theme(active_theme)?;

        Ok(Self { name, palette })
    }
}

impl Default for ThemeState {
    fn default() -> Self {
        Self::new()
    }
}
