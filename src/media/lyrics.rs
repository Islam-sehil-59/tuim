use crate::models::track::Track;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LyricsLookup {
    pub track_id: u64,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_seconds: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Lyrics {
    pub lookup: LyricsLookup,
    pub source: LyricsSource,
    pub plain: Option<String>,
    pub synced: Vec<SyncedLyricLine>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SyncedLyricLine {
    pub timestamp_ms: u32,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LyricsSource {
    Lrclib,
    Provider(String),
}

impl LyricsLookup {
    pub fn from_track(track: &Track, duration_seconds: Option<u32>) -> Self {
        Self {
            track_id: track.id,
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: Some(track.album.clone()).filter(|album| !album.is_empty()),
            duration_seconds,
        }
    }

    pub fn cache_key(&self) -> String {
        sanitize_cache_key(&format!(
            "lyrics:{}:{}:{}:{}",
            self.track_id,
            normalize_key_part(&self.artist),
            normalize_key_part(&self.title),
            self.duration_seconds
                .map(|duration| duration.to_string())
                .unwrap_or_else(|| String::from("no_duration"))
        ))
    }
}

impl Lyrics {
    pub fn has_synced_lines(&self) -> bool {
        !self.synced.is_empty()
    }
}

fn normalize_key_part(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn sanitize_cache_key(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_from_track_preserves_metadata_for_provider_queries() {
        let track = Track {
            id: 42,
            title: "Song".to_string(),
            artist: "Artist".to_string(),
            artist_id: 1,
            album: "Album".to_string(),
            album_id: 2,
            cover_id: None,
            isrc: None,
        };

        let lookup = LyricsLookup::from_track(&track, Some(123));

        assert_eq!(lookup.track_id, 42);
        assert_eq!(lookup.album.as_deref(), Some("Album"));
        assert_eq!(lookup.duration_seconds, Some(123));
        assert_eq!(lookup.cache_key(), "lyrics_42_artist_song_123");
    }
}
