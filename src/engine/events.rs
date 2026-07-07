use crate::{
    media::lyrics::Lyrics,
    media::playback::PlaybackSource,
    models::track::Track,
    providers::{
        provider::ArtistDetails,
        provider::{AlbumDetails, SearchResults},
    },
    services::image::CoverArt,
};

pub enum RuntimeEvent {
    SearchCompleted(Result<SearchResults, String>),
    AlbumLoaded(Result<AlbumDetails, String>),
    ArtistLoaded(Result<ArtistDetails, String>),
    PlaybackResolved {
        request_id: u64,
        track: Track,
        result: Result<PlaybackSource, String>,
    },
    LyricsLoaded {
        track_id: u64,
        duration_seconds: Option<u32>,
        result: Result<Option<Lyrics>, String>,
    },
}

pub type CoverEvent = Result<CoverArt, String>;
