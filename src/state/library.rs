use std::path::PathBuf;

use crate::models::track::Track;

#[derive(Clone, Debug)]
pub struct DownloadedTrack {
    pub track: Track,
    pub path: PathBuf,
}

pub struct LibraryState {
    pub downloaded_tracks: Vec<DownloadedTrack>,
}

impl LibraryState {
    pub fn new() -> Self {
        Self {
            downloaded_tracks: Vec::new(),
        }
    }

    pub fn mark_downloaded(&mut self, track: Track, path: PathBuf) {
        if let Some(existing) = self
            .downloaded_tracks
            .iter_mut()
            .find(|item| item.track.id == track.id)
        {
            existing.path = path;
            existing.track = track;
            return;
        }

        self.downloaded_tracks
            .insert(0, DownloadedTrack { track, path });
    }

    pub fn is_downloaded(&self, track_id: u64) -> bool {
        self.downloaded_tracks
            .iter()
            .any(|item| item.track.id == track_id)
    }

    pub fn download_path(&self, track_id: u64) -> Option<&PathBuf> {
        self.downloaded_tracks
            .iter()
            .find(|item| item.track.id == track_id)
            .map(|item| &item.path)
    }
}

impl Default for LibraryState {
    fn default() -> Self {
        Self::new()
    }
}
