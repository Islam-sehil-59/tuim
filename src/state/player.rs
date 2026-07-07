use crate::models::track::Track;
use crate::player::mpv::PlaybackProgress;

pub struct PlayerState {
    pub now_playing: Option<Track>,
    pub progress: Option<PlaybackProgress>,
    pub paused: bool,
    pub attached_playback: bool,
    pub source_quality: Option<String>,
    pub volume: Option<u8>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            now_playing: None,
            progress: None,
            paused: false,
            attached_playback: false,
            source_quality: None,
            volume: None,
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}
