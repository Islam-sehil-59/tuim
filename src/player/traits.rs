use crate::{media::playback::PlaybackSource, player::error::PlayerError};

pub trait Player {
    fn play(&mut self, source: &PlaybackSource) -> Result<(), PlayerError>;
}
