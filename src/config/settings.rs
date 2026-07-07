use std::fs;

use crate::config::paths;
use crate::state::settings::SettingsState;

pub fn load() -> Result<SettingsState, String> {
    let path = paths::settings_path();
    if !path.exists() {
        return Ok(SettingsState::default());
    }

    let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

pub fn save(settings: &SettingsState) -> Result<(), String> {
    let path = paths::settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, text).map_err(|error| error.to_string())
}
