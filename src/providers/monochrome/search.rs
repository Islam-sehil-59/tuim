use crate::models::{album::Album, artist::Artist, track::Track};
use serde_json::Value;

use super::{
    client::{ApiClient, RequestSpec},
    details::{album_from_value, artist_from_value, search_items},
    models::*,
};

impl ApiClient {
    pub async fn search_tracks(&self, query: &str) -> Result<Vec<Track>, String> {
        let payload: SearchResponse = self
            .request_json(&[RequestSpec::new(
                "/search",
                vec![("s".to_string(), query.to_string())],
            )])
            .await?;

        Ok(payload
            .data
            .items
            .into_iter()
            .map(|item| Track {
                id: item.id,
                title: item.title,
                artist: item.artist.name,
                artist_id: item.artist.id,
                album: item.album.title,
                album_id: item.album.id,
                cover_id: item.album.cover,
                isrc: item.isrc,
            })
            .collect())
    }

    pub async fn search_albums(&self, query: &str) -> Result<Vec<Album>, String> {
        let payload: Value = self
            .request_json(&[RequestSpec::new(
                "/search/",
                vec![("al".to_string(), query.to_string())],
            )])
            .await?;

        Ok(search_items(&payload, "albums")
            .into_iter()
            .filter_map(album_from_value)
            .collect())
    }

    pub async fn search_artists(&self, query: &str) -> Result<Vec<Artist>, String> {
        let payload: Value = self
            .request_json(&[RequestSpec::new(
                "/search/",
                vec![("a".to_string(), query.to_string())],
            )])
            .await?;

        Ok(search_items(&payload, "artists")
            .into_iter()
            .filter_map(artist_from_value)
            .collect())
    }
}

pub(super) fn normalize_match_text(value: &str) -> String {
    value
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if character.is_whitespace() {
                Some(' ')
            } else {
                None
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_normalization_ignores_case_and_punctuation() {
        assert_eq!(
            normalize_match_text("Mask Off (feat. Kendrick Lamar)"),
            "mask off feat kendrick lamar"
        );
        assert_eq!(normalize_match_text("  PNL — Au DD "), "pnl au dd");
    }
}
