use std::{env, path::PathBuf};

pub fn config_dir() -> PathBuf {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home).join("tuim");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".config").join("tuim");
    }

    env::temp_dir().join("tuim")
}

pub fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn keybinds_path() -> PathBuf {
    config_dir().join("keybinds.json")
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

pub fn downloads_dir() -> PathBuf {
    if let Ok(xdg_music_dir) = env::var("XDG_MUSIC_DIR") {
        return PathBuf::from(xdg_music_dir).join("tuim");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join("Music").join("tuim");
    }

    env::temp_dir().join("tuim-downloads")
}
