use tokio::sync::mpsc::UnboundedSender;

use crate::{
    engine::events::RuntimeEvent,
    models::artist::Artist,
    providers::provider::{AlbumDetails, ArtistDetails, SearchResults},
    providers::{monochrome::MonochromeProvider, provider::MusicProvider},
    state::search::SearchState,
};

pub struct SearchApply {
    pub status_message: String,
    pub log_message: Option<String>,
    pub request_selected_cover: bool,
}

pub struct ArtistApply {
    pub status_message: String,
    pub log_message: Option<String>,
    pub cover_artist: Option<Artist>,
}

pub fn spawn_search(
    provider: MonochromeProvider,
    query: String,
    tx: UnboundedSender<RuntimeEvent>,
) {
    tokio::spawn(async move {
        let _ = tx.send(RuntimeEvent::SearchCompleted(provider.search(&query).await));
    });
}

pub fn spawn_album_load(
    provider: MonochromeProvider,
    album_id: u64,
    tx: UnboundedSender<RuntimeEvent>,
) {
    tokio::spawn(async move {
        let _ = tx.send(RuntimeEvent::AlbumLoaded(
            provider.fetch_album_details(album_id).await,
        ));
    });
}

pub fn apply_search_results(search: &mut SearchState, results: SearchResults) -> SearchApply {
    let track_count = results.tracks.len();
    let album_count = results.albums.len();
    let artist_count = results.artists.len();
    search.set_results(results);

    let status_message = if search.total_items() == 0 {
        String::from("No tracks, albums, or artists found.")
    } else {
        format!("Found {track_count} tracks, {album_count} albums, and {artist_count} artists.")
    };

    SearchApply {
        status_message,
        log_message: Some(format!(
            "search completed track_count={} album_count={} artist_count={}",
            search.results.len(),
            search.albums.len(),
            search.artists.len()
        )),
        request_selected_cover: true,
    }
}

pub fn apply_album_details(search: &mut SearchState, details: AlbumDetails) -> SearchApply {
    let title = details.album.title.clone();
    let artist = details.album.artist.clone();
    let count = details.tracks.len();
    search.set_album_tracks(details.album, details.tracks);

    SearchApply {
        status_message: format!("Album: {artist} — {title} ({count} tracks)."),
        log_message: None,
        request_selected_cover: false,
    }
}

pub fn apply_artist_details(search: &mut SearchState, details: ArtistDetails) -> ArtistApply {
    let name = details.artist.name.clone();
    let album_count = details.albums.len();
    let track_count = details.tracks.len();
    let artist = details.artist.clone();
    search.set_artist_results(details.artist, details.albums, details.tracks);

    ArtistApply {
        status_message: format!("Artist: {name} ({album_count} albums, {track_count} tracks)."),
        log_message: None,
        cover_artist: Some(artist),
    }
}

pub fn spawn_artist_load(
    provider: MonochromeProvider,
    artist_id: u64,
    tx: UnboundedSender<RuntimeEvent>,
) {
    tokio::spawn(async move {
        let _ = tx.send(RuntimeEvent::ArtistLoaded(
            provider.fetch_artist_details(artist_id).await,
        ));
    });
}
