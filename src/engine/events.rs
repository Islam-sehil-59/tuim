use crate::{
    media::downloads::DownloadSummary,
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
    DownloadCompleted(DownloadSummary),
}

pub struct CoverEvent {
    pub request_key: String,
    pub result: Result<CoverArt, String>,
}
