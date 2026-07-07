use std::{fs, path::PathBuf};

use reqwest::Client;
use serde::Deserialize;
use tokio::fs as tokio_fs;

use crate::{
    cache::paths,
    media::lyrics::{Lyrics, LyricsLookup, LyricsSource, SyncedLyricLine},
};

#[derive(Clone)]
pub struct LyricsService {
    client: Client,
    cache_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibResponse {
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
}

impl LyricsService {
    pub fn new() -> Self {
        let cache_dir = paths::lyrics_dir();
        let _ = fs::create_dir_all(&cache_dir);

        Self {
            client: Client::new(),
            cache_dir,
        }
    }

    pub async fn fetch(&self, lookup: LyricsLookup) -> Result<Option<Lyrics>, String> {
        let cache_path = self.cache_path(&lookup);
        if cache_path.exists() {
            let text = tokio_fs::read_to_string(&cache_path)
                .await
                .map_err(|error| error.to_string())?;
            return serde_json::from_str(&text).map_err(|error| error.to_string());
        }

        let mut query = vec![
            ("track_name", lookup.title.clone()),
            ("artist_name", lookup.artist.clone()),
        ];
        if let Some(album) = &lookup.album {
            query.push(("album_name", album.clone()));
        }
        if let Some(duration) = lookup.duration_seconds {
            query.push(("duration", duration.to_string()));
        }

        let response = self
            .client
            .get("https://lrclib.net/api/get")
            .query(&query)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        if response.status().as_u16() == 404 {
            self.write_cache(&cache_path, &None).await?;
            return Ok(None);
        }

        let response = response
            .error_for_status()
            .map_err(|error| error.to_string())?
            .json::<LrclibResponse>()
            .await
            .map_err(|error| error.to_string())?;

        let lyrics = build_lyrics(lookup, response);
        self.write_cache(&cache_path, &lyrics).await?;
        Ok(lyrics)
    }

    fn cache_path(&self, lookup: &LyricsLookup) -> PathBuf {
        self.cache_dir.join(format!("{}.json", lookup.cache_key()))
    }

    async fn write_cache(&self, path: &PathBuf, lyrics: &Option<Lyrics>) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            tokio_fs::create_dir_all(parent)
                .await
                .map_err(|error| error.to_string())?;
        }

        let text = serde_json::to_string_pretty(lyrics).map_err(|error| error.to_string())?;
        tokio_fs::write(path, text)
            .await
            .map_err(|error| error.to_string())
    }
}

impl Default for LyricsService {
    fn default() -> Self {
        Self::new()
    }
}

fn build_lyrics(lookup: LyricsLookup, response: LrclibResponse) -> Option<Lyrics> {
    let synced = response
        .synced_lyrics
        .as_deref()
        .map(parse_lrc)
        .unwrap_or_default();
    let plain = response
        .plain_lyrics
        .filter(|lyrics| !lyrics.trim().is_empty());

    if plain.is_none() && synced.is_empty() {
        return None;
    }

    Some(Lyrics {
        lookup,
        source: LyricsSource::Lrclib,
        plain,
        synced,
    })
}

fn parse_lrc(input: &str) -> Vec<SyncedLyricLine> {
    let mut lines = Vec::new();
    for line in input.lines() {
        let Some((timestamp, text)) = line.strip_prefix('[').and_then(|line| line.split_once(']'))
        else {
            continue;
        };
        let Some(timestamp_ms) = parse_lrc_timestamp(timestamp) else {
            continue;
        };

        lines.push(SyncedLyricLine {
            timestamp_ms,
            text: text.trim().to_string(),
        });
    }

    lines
}

fn parse_lrc_timestamp(input: &str) -> Option<u32> {
    let (minutes, rest) = input.split_once(':')?;
    let (seconds, fraction) = rest.split_once('.').unwrap_or((rest, "0"));
    let minutes = minutes.parse::<u32>().ok()?;
    let seconds = seconds.parse::<u32>().ok()?;
    let fraction_ms = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<u32>().ok()? * 100,
        2 => fraction.parse::<u32>().ok()? * 10,
        _ => fraction.get(0..3)?.parse::<u32>().ok()?,
    };

    Some(minutes * 60_000 + seconds * 1_000 + fraction_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lrc_timestamps_and_text() {
        let lines = parse_lrc("[00:01.20]hello\n[02:03.456]world");

        assert_eq!(
            lines,
            vec![
                SyncedLyricLine {
                    timestamp_ms: 1_200,
                    text: "hello".to_string(),
                },
                SyncedLyricLine {
                    timestamp_ms: 123_456,
                    text: "world".to_string(),
                }
            ]
        );
    }
}
