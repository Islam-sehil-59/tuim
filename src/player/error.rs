use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum PlayerError {
    NoActivePlayback,
    Ipc(String),
    Spawn(String),
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActivePlayback => write!(f, "No active playback."),
            Self::Ipc(msg) => write!(f, "{msg}"),
            Self::Spawn(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PlayerError {}
