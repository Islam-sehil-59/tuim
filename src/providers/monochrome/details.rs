use serde_json::Value;

use crate::models::{album::Album, artist::Artist, track::Track};
use crate::providers::provider::{AlbumDetails, ArtistDetails};

use super::client::{ApiClient, RequestSpec};
use super::models::*;

impl ApiClient {
    pub async fn fetch_album(&self, album_id: u64) -> Result<Album, String> {
        let payload: AlbumResponse = self
            .request_json(&[
                RequestSpec::new(format!("/album/{album_id}"), Vec::new()),
                RequestSpec::new("/album", vec![("id".to_string(), album_id.to_string())]),
            ])
            .await?;

        let album = payload
            .data
            .or(payload.album)
            .ok_or_else(|| String::from("album payload did not include album data"))?;

        Ok(Album {
            id: album.id,
            title: album.title,
            artist: album.artist.name,
            cover_id: album.cover,
            release_date: None,
            track_count: None,
        })
    }

    pub async fn fetch_album_details(&self, album_id: u64) -> Result<AlbumDetails, String> {
        let payload: Value = self
            .request_json(&[
                RequestSpec::new("/album/", vec![("id".to_string(), album_id.to_string())]),
                RequestSpec::new(format!("/album/{album_id}"), Vec::new()),
            ])
            .await?;

        let data = payload.get("data").unwrap_or(&payload);
        let mut album = album_from_value(data.clone());
        let tracks: Vec<Track> = section_items(data)
            .into_iter()
            .filter_map(track_from_value)
            .collect();

        if album.is_none() {
            album = tracks.first().map(|track| Album {
                id: track.album_id,
                title: track.album.clone(),
                artist: track.artist.clone(),
                cover_id: track.cover_id.clone(),
                release_date: None,
                track_count: Some(tracks.len()),
            });
        }

        let mut album =
            album.ok_or_else(|| String::from("album payload did not include album data"))?;

        if album.artist.is_empty()
            && let Some(track) = tracks.first()
        {
            album.artist = track.artist.clone();
        }
        if album.cover_id.is_none()
            && let Some(track) = tracks.first()
        {
            album.cover_id = track.cover_id.clone();
        }
        album.track_count = Some(tracks.len());

        Ok(AlbumDetails { album, tracks })
    }

    pub async fn fetch_artist(&self, artist_id: u64) -> Result<Artist, String> {
        let payload: ArtistResponse = self
            .request_json(&[
                RequestSpec::new("/artist", vec![("id".to_string(), artist_id.to_string())]),
                RequestSpec::new(format!("/artist/{artist_id}"), Vec::new()),
            ])
            .await?;

        let artist = payload
            .artist
            .or(payload.data)
            .ok_or_else(|| String::from("artist payload did not include artist data"))?;

        Ok(Artist {
            id: artist.id,
            name: artist.name,
            picture_id: artist.picture,
            description: None,
            album_count: None,
            track_count: None,
        })
    }

    pub async fn fetch_artist_details(&self, artist_id: u64) -> Result<ArtistDetails, String> {
        let primary_payload: Value = self
            .request_json(&[
                RequestSpec::new("/artist/", vec![("id".to_string(), artist_id.to_string())]),
                RequestSpec::new(format!("/artist/{artist_id}"), Vec::new()),
            ])
            .await?;

        let primary_data = primary_payload.get("data").unwrap_or(&primary_payload);
        let artist = primary_data
            .get("artist")
            .cloned()
            .or_else(|| {
                primary_data
                    .as_array()
                    .and_then(|items| items.first().cloned())
            })
            .or_else(|| Some(primary_data.clone()))
            .and_then(artist_from_value)
            .ok_or_else(|| String::from("artist payload did not include artist data"))?;

        let mut albums = Vec::new();
        let mut tracks = Vec::new();
        collect_artist_media(primary_data, artist.id, &mut albums, &mut tracks);

        let discography_payload: Value = self
            .request_json(&[RequestSpec::new(
                "/artist/",
                vec![
                    ("f".to_string(), artist_id.to_string()),
                    ("skip_tracks".to_string(), "true".to_string()),
                ],
            )])
            .await
            .unwrap_or(Value::Null);
        let discography_data = discography_payload
            .get("data")
            .unwrap_or(&discography_payload);
        collect_artist_media(discography_data, artist.id, &mut albums, &mut tracks);

        let tracks_payload: Value = self
            .request_json(&[RequestSpec::new(
                "/artist/",
                vec![
                    ("f".to_string(), artist_id.to_string()),
                    ("skip_tracks".to_string(), "true".to_string()),
                    ("offset".to_string(), "0".to_string()),
                    ("limit".to_string(), "15".to_string()),
                ],
            )])
            .await
            .unwrap_or(Value::Null);
        let tracks_data = tracks_payload.get("data").unwrap_or(&tracks_payload);
        collect_artist_media(tracks_data, artist.id, &mut albums, &mut tracks);

        dedup_albums(&mut albums);
        dedup_tracks(&mut tracks);

        let mut artist = artist;
        artist.album_count = Some(albums.len());
        artist.track_count = Some(tracks.len());

        Ok(ArtistDetails {
            artist,
            albums,
            tracks,
        })
    }
}

pub(super) fn search_items(payload: &Value, key: &str) -> Vec<Value> {
    find_section(payload, key)
        .and_then(|section| section.get("items").and_then(Value::as_array))
        .cloned()
        .or_else(|| {
            payload
                .get("data")
                .and_then(|data| data.get("items"))
                .and_then(Value::as_array)
                .cloned()
        })
        .unwrap_or_default()
}

pub(super) fn section_items(payload: &Value) -> Vec<Value> {
    payload
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn find_section<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if let Some(section) = map.get(key)
                && section.get("items").and_then(Value::as_array).is_some()
            {
                return Some(section);
            }

            if value.get("items").and_then(Value::as_array).is_some() {
                return Some(value);
            }

            map.values().find_map(|value| find_section(value, key))
        }
        Value::Array(items) => items.iter().find_map(|value| find_section(value, key)),
        _ => None,
    }
}

pub(super) fn album_from_value(value: Value) -> Option<Album> {
    let id = value.get("id")?.as_u64()?;
    let title = value.get("title")?.as_str()?.to_string();
    let artist = artist_name(value.get("artist"))
        .or_else(|| {
            value
                .get("artists")
                .and_then(Value::as_array)
                .and_then(|artists| artists.first())
                .and_then(|artist| artist_name(Some(artist)))
        })
        .unwrap_or_default();

    Some(Album {
        id,
        title,
        artist,
        cover_id: value
            .get("cover")
            .and_then(Value::as_str)
            .map(str::to_string),
        release_date: string_field(
            &value,
            &[
                "releaseDate",
                "release_date",
                "releaseDateTime",
                "releaseYear",
                "year",
            ],
        ),
        track_count: value
            .get("numberOfTracks")
            .or_else(|| value.get("trackCount"))
            .or_else(|| value.get("tracksCount"))
            .and_then(Value::as_u64)
            .map(|count| count as usize),
    })
}

pub(super) fn artist_from_value(value: Value) -> Option<Artist> {
    Some(Artist {
        id: value.get("id")?.as_u64()?,
        name: value.get("name")?.as_str()?.to_string(),
        picture_id: value
            .get("picture")
            .or_else(|| value.get("image"))
            .and_then(Value::as_str)
            .map(str::to_string),
        description: string_field(
            &value,
            &["description", "bio", "biography", "shortDescription"],
        ),
        album_count: value
            .get("albumCount")
            .or_else(|| value.get("albumsCount"))
            .and_then(Value::as_u64)
            .map(|count| count as usize),
        track_count: value
            .get("trackCount")
            .or_else(|| value.get("tracksCount"))
            .and_then(Value::as_u64)
            .map(|count| count as usize),
    })
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
    })
}

fn collect_artist_media(
    value: &Value,
    artist_id: u64,
    albums: &mut Vec<Album>,
    tracks: &mut Vec<Track>,
) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_artist_media(item, artist_id, albums, tracks);
            }
        }
        Value::Object(map) => {
            let item = map.get("item").unwrap_or(value);
            if media_matches_artist(item, artist_id) {
                if let Some(album) = album_from_value(item.clone()) {
                    albums.push(album);
                }
                if let Some(track) = track_from_value(item.clone()) {
                    tracks.push(track);
                }
            }

            for nested in map.values() {
                collect_artist_media(nested, artist_id, albums, tracks);
            }
        }
        _ => {}
    }
}

fn media_matches_artist(value: &Value, artist_id: u64) -> bool {
    value
        .get("artist")
        .and_then(|artist| artist.get("id").or(Some(artist)))
        .and_then(Value::as_u64)
        == Some(artist_id)
        || value
            .get("artists")
            .and_then(Value::as_array)
            .map(|artists| {
                artists
                    .iter()
                    .any(|artist| artist.get("id").and_then(Value::as_u64) == Some(artist_id))
            })
            .unwrap_or(false)
}

fn dedup_albums(albums: &mut Vec<Album>) {
    let mut seen = std::collections::HashSet::new();
    albums.retain(|album| seen.insert(album.id));
}

fn dedup_tracks(tracks: &mut Vec<Track>) {
    let mut seen = std::collections::HashSet::new();
    tracks.retain(|track| seen.insert(track.id));
}

fn track_from_value(value: Value) -> Option<Track> {
    let item = value.get("item").unwrap_or(&value);
    let album = item.get("album")?;
    let artist = item.get("artist").or_else(|| {
        item.get("artists")
            .and_then(Value::as_array)
            .and_then(|artists| artists.first())
    });

    Some(Track {
        id: item.get("id")?.as_u64()?,
        title: item.get("title")?.as_str()?.to_string(),
        artist: artist_name(artist).unwrap_or_default(),
        artist_id: artist
            .and_then(|artist| artist.get("id"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        album: album.get("title")?.as_str()?.to_string(),
        album_id: album.get("id")?.as_u64()?,
        cover_id: album
            .get("cover")
            .and_then(Value::as_str)
            .map(str::to_string),
        isrc: item.get("isrc").and_then(Value::as_str).map(str::to_string),
    })
}

fn artist_name(value: Option<&Value>) -> Option<String> {
    value?
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artist_discography_payload_yields_albums() {
        let payload = serde_json::json!({
            "albums": {
                "items": [
                    {
                        "id": 107105449,
                        "title": "Deux frères",
                        "cover": "85ee3a95-9627-41e1-834b-2df965d26004",
                        "numberOfTracks": 16,
                        "artist": {
                            "id": 7629451,
                            "name": "PNL",
                            "picture": "979bb546-8007-4395-9257-20d521ea791e"
                        }
                    }
                ]
            }
        });
        let mut albums = Vec::new();
        let mut tracks = Vec::new();

        collect_artist_media(&payload, 7629451, &mut albums, &mut tracks);

        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].title, "Deux frères");
        assert_eq!(
            albums[0].cover_id.as_deref(),
            Some("85ee3a95-9627-41e1-834b-2df965d26004")
        );
    }

    #[test]
    fn artist_payload_yields_picture_id() {
        let artist = artist_from_value(serde_json::json!({
            "id": 7629451,
            "name": "PNL",
            "picture": "979bb546-8007-4395-9257-20d521ea791e"
        }))
        .unwrap();

        assert_eq!(
            artist.picture_id.as_deref(),
            Some("979bb546-8007-4395-9257-20d521ea791e")
        );
    }
}
