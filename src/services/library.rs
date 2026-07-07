pub struct LibraryService;
use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    models::track::Track,
    state::library::{DownloadedTrack, LibraryState},
};

#[derive(Deserialize)]
struct TrackMetadata {
    track_id: u64,
    title: String,
    artist: String,
    artist_id: u64,
    album: String,
    album_id: u64,
    isrc: Option<String>,
    cover_id: Option<String>,
}

pub fn load_downloaded_tracks(root: &Path) -> LibraryState {
    let mut state = LibraryState::new();
    let mut items = Vec::new();
    collect_downloaded_tracks(root, &mut items);

    for item in items {
        state.mark_downloaded(item.track, item.path);
    }

    state
}

fn collect_downloaded_tracks(dir: &Path, output: &mut Vec<DownloadedTrack>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_downloaded_tracks(&path, output);
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".metadata.json"))
            && let Some(item) = load_downloaded_track(&path)
        {
            output.push(item);
        }
    }
}

fn load_downloaded_track(metadata_path: &Path) -> Option<DownloadedTrack> {
    let text = fs::read_to_string(metadata_path).ok()?;
    let metadata: TrackMetadata = serde_json::from_str(&text).ok()?;
    let audio_path = matching_audio_path(metadata_path)?;

    Some(DownloadedTrack {
        track: Track {
            id: metadata.track_id,
            title: metadata.title,
            artist: metadata.artist,
            artist_id: metadata.artist_id,
            album: metadata.album,
            album_id: metadata.album_id,
            cover_id: metadata.cover_id,
            isrc: metadata.isrc,
        },
        path: audio_path,
    })
}

fn matching_audio_path(metadata_path: &Path) -> Option<PathBuf> {
    let file_name = metadata_path.file_name()?.to_str()?;
    let stem = file_name.strip_suffix(".metadata.json")?;
    let parent = metadata_path.parent()?;

    ["flac", "m4a", "mp3", "aac"]
        .iter()
        .map(|extension| parent.join(format!("{stem}.{extension}")))
        .find(|candidate| candidate.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_audio_path_finds_audio_next_to_metadata() {
        let dir = std::env::temp_dir().join(format!("tuim-library-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Song.metadata.json"), "{}").unwrap();
        fs::write(dir.join("Song.flac"), "").unwrap();

        assert_eq!(
            matching_audio_path(&dir.join("Song.metadata.json")),
            Some(dir.join("Song.flac"))
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
