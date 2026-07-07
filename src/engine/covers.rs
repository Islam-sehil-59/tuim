use tokio::sync::mpsc::UnboundedSender;

use crate::{
    engine::events::CoverEvent,
    models::{album::Album, artist::Artist, track::Track},
    services::image::{CoverArt, ImageService},
    state::AppState,
};

pub fn prepare_cover_request(state: &mut AppState, request_key: &str) -> bool {
    if state.cover.request_key.as_deref() == Some(request_key)
        && (state.cover.loading || state.cover.path.is_some())
    {
        return false;
    }

    state.cover.request_key = Some(request_key.to_string());
    state.cover.loading = true;
    state.cover.path = None;
    true
}

pub fn prepare_playback_cover_request(state: &mut AppState, request_key: &str) -> bool {
    if state.playback_cover.request_key.as_deref() == Some(request_key)
        && (state.playback_cover.loading || state.playback_cover.path.is_some())
    {
        return false;
    }

    state.playback_cover.request_key = Some(request_key.to_string());
    state.playback_cover.loading = true;
    state.playback_cover.path = None;
    true
}

pub fn clear_cover_request(state: &mut AppState) {
    state.cover.request_key = None;
    state.cover.loading = false;
    state.cover.path = None;
}

pub fn apply_cover_result(state: &mut AppState, result: CoverEvent) -> Result<(), String> {
    match result {
        Ok(cover) => {
            if state.cover.request_key.as_deref() == Some(cover.request_key.as_str()) {
                state.cover.loading = false;
                state.cover.path = cover.path.clone();
            }
            if state.playback_cover.request_key.as_deref() == Some(cover.request_key.as_str()) {
                state.playback_cover.loading = false;
                state.playback_cover.path = cover.path;
            }
            Ok(())
        }
        Err(error) => {
            state.cover.loading = false;
            state.cover.path = None;
            Err(error)
        }
    }
}

pub fn spawn_cover_fetch(
    image: ImageService,
    track: Track,
    tx: UnboundedSender<Result<CoverArt, String>>,
) {
    tokio::spawn(async move {
        let _ = tx.send(image.fetch_cover_for_track(&track).await);
    });
}

pub fn spawn_album_cover_fetch(
    image: ImageService,
    album: Album,
    tx: UnboundedSender<Result<CoverArt, String>>,
) {
    tokio::spawn(async move {
        let _ = tx.send(image.fetch_cover_for_album(&album).await);
    });
}

pub fn spawn_artist_cover_fetch(
    image: ImageService,
    artist: Artist,
    tx: UnboundedSender<Result<CoverArt, String>>,
) {
    tokio::spawn(async move {
        let _ = tx.send(image.fetch_cover_for_artist(&artist).await);
    });
}
