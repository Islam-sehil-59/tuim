use std::{env, path::PathBuf};

pub fn cache_dir() -> PathBuf {
    if let Ok(cache_home) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(cache_home).join("tuim");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("tuim");
    }

    env::temp_dir().join("tuim")
}

pub fn covers_dir() -> PathBuf {
    cache_dir().join("covers")
}

pub fn vinyl_dir() -> PathBuf {
    cache_dir().join("vinyl")
}

pub fn api_dir() -> PathBuf {
    cache_dir().join("api")
}

pub fn lyrics_dir() -> PathBuf {
    cache_dir().join("lyrics")
}
