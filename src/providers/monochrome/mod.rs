pub mod client;
pub mod deezer;
pub mod details;
pub mod discovery;
mod models;
pub mod playback;
pub mod search;

use crate::{
    media::playback::PlaybackSource,
    models::{album::Album, artist::Artist, track::Track},
    providers::provider::{
        AlbumDetails, ArtistDetails, MusicProvider, ProviderFuture, SearchResults,
    },
    state::settings::AudioQuality,
};

use self::client::ApiClient;

#[derive(Clone)]
pub struct MonochromeProvider {
    api: ApiClient,
    audio_quality: AudioQuality,
}

impl MonochromeProvider {
    pub fn new() -> Self {
        Self {
            api: ApiClient::new(),
            audio_quality: AudioQuality::default(),
        }
    }

    pub fn new_with_deezer_url(deezer_fallback_url: Option<String>) -> Self {
        Self {
            api: ApiClient::new_with_deezer_url(deezer_fallback_url),
            audio_quality: AudioQuality::default(),
        }
    }

    pub fn from_api(api: ApiClient) -> Self {
        Self {
            api,
            audio_quality: AudioQuality::default(),
        }
    }

    pub fn with_quality(mut self, quality: AudioQuality) -> Self {
        self.audio_quality = quality;
        self
    }

    pub fn set_audio_quality(&mut self, quality: AudioQuality) {
        self.audio_quality = quality;
    }

    pub fn api(&self) -> &ApiClient {
        &self.api
    }
}

impl Default for MonochromeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicProvider for MonochromeProvider {
    fn search<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, SearchResults> {
        Box::pin(async move {
            let tracks = self.api.search_tracks(query).await?;
            let albums = self.api.search_albums(query).await.unwrap_or_default();
            let artists = self.api.search_artists(query).await.unwrap_or_default();
            Ok(SearchResults {
                tracks,
                albums,
                artists,
            })
        })
    }

    fn search_tracks<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<Track>> {
        Box::pin(async move { self.api.search_tracks(query).await })
    }

    fn search_albums<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<Album>> {
        Box::pin(async move { self.api.search_albums(query).await })
    }

    fn search_artists<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<Artist>> {
        Box::pin(async move { self.api.search_artists(query).await })
    }

    fn fetch_album(&self, album_id: u64) -> ProviderFuture<'_, Album> {
        Box::pin(async move { self.api.fetch_album(album_id).await })
    }

    fn fetch_album_details(&self, album_id: u64) -> ProviderFuture<'_, AlbumDetails> {
        Box::pin(async move { self.api.fetch_album_details(album_id).await })
    }

    fn fetch_artist(&self, artist_id: u64) -> ProviderFuture<'_, Artist> {
        Box::pin(async move { self.api.fetch_artist(artist_id).await })
    }

    fn fetch_artist_details(&self, artist_id: u64) -> ProviderFuture<'_, ArtistDetails> {
        Box::pin(async move { self.api.fetch_artist_details(artist_id).await })
    }

    fn resolve_playback<'a>(&'a self, track: &'a Track) -> ProviderFuture<'a, PlaybackSource> {
        let quality = self.audio_quality;
        Box::pin(async move {
            self.api
                .resolve_track_playback(track, quality)
                .await
                .map(|resolution| resolution.into_playback_source())
        })
    }
}
