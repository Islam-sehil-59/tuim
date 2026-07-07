use std::{future::Future, pin::Pin};

use crate::{
    media::playback::PlaybackSource,
    models::{album::Album, artist::Artist, track::Track},
};

pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'a>>;

#[derive(Clone, Debug, Default)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

#[derive(Clone, Debug)]
pub struct AlbumDetails {
    pub album: Album,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug)]
pub struct ArtistDetails {
    pub artist: Artist,
    pub albums: Vec<Album>,
    pub tracks: Vec<Track>,
}

pub trait MusicProvider {
    fn search<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, SearchResults>;

    fn search_tracks<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<Track>>;

    fn search_albums<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<Album>>;

    fn search_artists<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<Artist>>;

    fn fetch_album(&self, album_id: u64) -> ProviderFuture<'_, Album>;

    fn fetch_album_details(&self, album_id: u64) -> ProviderFuture<'_, AlbumDetails>;

    fn fetch_artist(&self, artist_id: u64) -> ProviderFuture<'_, Artist>;

    fn fetch_artist_details(&self, artist_id: u64) -> ProviderFuture<'_, ArtistDetails>;

    fn resolve_playback<'a>(&'a self, track: &'a Track) -> ProviderFuture<'a, PlaybackSource>;
}
