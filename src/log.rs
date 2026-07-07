use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const LOG_TARGET_APP: &str = "APP";
const LOG_TARGET_PLAYER: &str = "PLAYER";

pub fn app_log(message: &str) {
    write_log(LOG_TARGET_APP, message);
}

pub fn player_log(message: &str) {
    write_log(LOG_TARGET_PLAYER, message);
}

fn write_log(target: &str, message: &str) {
    let path = log_path("playback.log");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let _ = writeln!(file, "[{ts}] [{target}] {message}");
    }
}

pub(crate) fn log_path(file_name: &str) -> PathBuf {
    if let Ok(cache_home) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(cache_home)
            .join("tuim")
            .join(file_name);
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".cache")
            .join("tuim")
            .join(file_name);
    }

    env::temp_dir().join(file_name)
}
