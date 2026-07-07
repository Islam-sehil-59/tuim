use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::{
    Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde::Serialize;
use tokio::fs;

use crate::{
    config::paths,
    media::{
        downloads::{DownloadRequest, DownloadSummary, DownloadedFile, sidecar_path, target_path},
        lyrics::Lyrics,
        playback::{PlaybackSource, PlaybackSourceKind},
    },
    models::track::Track,
    providers::{monochrome::MonochromeProvider, provider::MusicProvider},
    services::{image::ImageService, lyrics::LyricsService},
};

#[derive(Clone)]
pub struct DownloadService {
    client: Client,
}

#[derive(Clone, Debug)]
pub struct DownloadJob {
    pub label: String,
    pub tracks: Vec<Track>,
    pub output_dir: PathBuf,
    pub collection: Option<String>,
}

#[derive(Serialize)]
struct TrackMetadata<'a> {
    track_id: u64,
    title: &'a str,
    artist: &'a str,
    artist_id: u64,
    album: &'a str,
    album_id: u64,
    isrc: Option<&'a str>,
    cover_id: Option<&'a str>,
    source_quality: Option<&'a str>,
    source_format: Option<&'a str>,
    lyrics_embedded: bool,
    artist_rating: Option<u8>,
    downloaded_at_unix: u64,
}

impl DownloadService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn download_job(
        &self,
        provider: MonochromeProvider,
        image: ImageService,
        lyrics: LyricsService,
        job: DownloadJob,
    ) -> DownloadSummary {
        let mut files = Vec::new();
        let mut failed = Vec::new();
        let total = job.tracks.len().max(1);

        for (index, track) in job.tracks.into_iter().enumerate() {
            let mut request = DownloadRequest::single_track(track.clone(), job.output_dir.clone());
            request.collection = job.collection.clone();
            request.index = if total > 1 { Some(index + 1) } else { None };

            match self
                .download_track(&provider, &image, &lyrics, request)
                .await
            {
                Ok(downloaded) => files.push(downloaded),
                Err(error) => failed.push((track, error)),
            }
        }

        DownloadSummary {
            label: job.label,
            files,
            failed,
            fatal_error: None,
        }
    }

    async fn download_track(
        &self,
        provider: &MonochromeProvider,
        image: &ImageService,
        lyrics: &LyricsService,
        request: DownloadRequest,
    ) -> Result<DownloadedFile, String> {
        let source = provider.resolve_playback(&request.track).await?;
        if !matches!(source.kind, PlaybackSourceKind::DirectUrl) {
            return Err(String::from(
                "download requires a direct stream; provider returned a manifest",
            ));
        }

        let extension = extension_for_source(&source);
        let audio_path = target_path(&request, extension);
        if let Some(parent) = audio_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|error| error.to_string())?;
        }

        let bytes = self.fetch_bytes(&source).await?;
        fs::write(&audio_path, bytes)
            .await
            .map_err(|error| error.to_string())?;

        let fetched_lyrics = if request.include_lyrics {
            let lookup = crate::media::lyrics::LyricsLookup::from_track(&request.track, None);
            lyrics.fetch(lookup).await.unwrap_or(None)
        } else {
            None
        };

        if let Some(lyrics) = &fetched_lyrics {
            write_lyrics_sidecar(&audio_path, lyrics).await?;
        }

        if request.include_cover
            && let Ok(cover) = image.fetch_cover_for_track(&request.track).await
            && let Some(path) = cover.path
        {
            copy_cover_sidecar(Path::new(&path), &audio_path).await?;
        }

        write_metadata_sidecar(
            &audio_path,
            &request.track,
            &source,
            fetched_lyrics.as_ref(),
        )
        .await?;

        Ok(DownloadedFile {
            track: request.track,
            path: audio_path,
        })
    }

    async fn fetch_bytes(&self, source: &PlaybackSource) -> Result<Vec<u8>, String> {
        let response = self
            .client
            .get(&source.url)
            .headers(headers(&source.headers)?)
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;

        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|error| error.to_string())
    }
}

impl Default for DownloadService {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadJob {
    pub fn tracks(label: impl Into<String>, tracks: Vec<Track>) -> Self {
        Self {
            label: label.into(),
            tracks,
            output_dir: paths::downloads_dir(),
            collection: None,
        }
    }

    pub fn collection(
        label: impl Into<String>,
        collection: impl Into<String>,
        tracks: Vec<Track>,
    ) -> Self {
        Self {
            label: label.into(),
            tracks,
            output_dir: paths::downloads_dir(),
            collection: Some(collection.into()),
        }
    }
}

fn headers(values: &[(String, String)]) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    for (name, value) in values {
        let name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| error.to_string())?;
        let value = HeaderValue::from_str(value).map_err(|error| error.to_string())?;
        headers.insert(name, value);
    }
    Ok(headers)
}

fn extension_for_source(source: &PlaybackSource) -> &'static str {
    match source.quality.as_deref() {
        Some("FLAC") | Some("LOSSLESS") | Some("HI_RES_LOSSLESS") => "flac",
        _ => "m4a",
    }
}

async fn copy_cover_sidecar(source: &Path, audio_path: &Path) -> Result<(), String> {
    let cover_path = sidecar_path(audio_path, "cover.png");
    fs::copy(source, cover_path)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

async fn write_lyrics_sidecar(audio_path: &Path, lyrics: &Lyrics) -> Result<(), String> {
    let text = if lyrics.has_synced_lines() {
        lyrics
            .synced
            .iter()
            .map(|line| format_lrc_line(line.timestamp_ms, &line.text))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        lyrics.plain.clone().unwrap_or_default()
    };

    if text.trim().is_empty() {
        return Ok(());
    }

    fs::write(sidecar_path(audio_path, "lyrics.lrc"), text)
        .await
        .map_err(|error| error.to_string())
}

async fn write_metadata_sidecar(
    audio_path: &Path,
    track: &Track,
    source: &PlaybackSource,
    lyrics: Option<&Lyrics>,
) -> Result<(), String> {
    let metadata = TrackMetadata {
        track_id: track.id,
        title: &track.title,
        artist: &track.artist,
        artist_id: track.artist_id,
        album: &track.album,
        album_id: track.album_id,
        isrc: track.isrc.as_deref(),
        cover_id: track.cover_id.as_deref(),
        source_quality: source.quality.as_deref(),
        source_format: source.format.as_deref(),
        lyrics_embedded: lyrics.is_some(),
        artist_rating: None,
        downloaded_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0),
    };
    let text = serde_json::to_string_pretty(&metadata).map_err(|error| error.to_string())?;

    fs::write(sidecar_path(audio_path, "metadata.json"), text)
        .await
        .map_err(|error| error.to_string())
}

fn format_lrc_line(timestamp_ms: u32, text: &str) -> String {
    let minutes = timestamp_ms / 60_000;
    let seconds = (timestamp_ms % 60_000) / 1_000;
    let centiseconds = (timestamp_ms % 1_000) / 10;
    format!("[{minutes:02}:{seconds:02}.{centiseconds:02}]{text}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_lrc_lines_from_milliseconds() {
        assert_eq!(format_lrc_line(83_450, "hello"), "[01:23.45]hello");
    }
}
