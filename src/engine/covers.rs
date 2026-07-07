use tokio::sync::mpsc::UnboundedSender;

use crate::{
    engine::events::CoverEvent,
    models::{album::Album, artist::Artist, track::Track},
    services::image::ImageService,
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
    match result.result {
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
            let mut matched = false;
            if state.cover.request_key.as_deref() == Some(result.request_key.as_str()) {
                state.cover.loading = false;
                state.cover.path = None;
                matched = true;
            }
            if state.playback_cover.request_key.as_deref() == Some(result.request_key.as_str()) {
                state.playback_cover.loading = false;
                state.playback_cover.path = None;
                matched = true;
            }
            if !matched {
                return Ok(());
            }
            Err(error)
        }
    }
}

pub fn spawn_cover_fetch(image: ImageService, track: Track, tx: UnboundedSender<CoverEvent>) {
    let request_key = format!("track:{}", track.id);
    tokio::spawn(async move {
        let result = image.fetch_cover_for_track(&track).await;
        let _ = tx.send(CoverEvent {
            request_key,
            result,
        });
    });
}

pub fn spawn_album_cover_fetch(image: ImageService, album: Album, tx: UnboundedSender<CoverEvent>) {
    let request_key = format!("album:{}", album.id);
    tokio::spawn(async move {
        let result = image.fetch_cover_for_album(&album).await;
        let _ = tx.send(CoverEvent {
            request_key,
            result,
        });
    });
}

pub fn spawn_artist_cover_fetch(
    image: ImageService,
    artist: Artist,
    tx: UnboundedSender<CoverEvent>,
) {
    let request_key = format!("artist:{}", artist.id);
    tokio::spawn(async move {
        let result = image.fetch_cover_for_artist(&artist).await;
        let _ = tx.send(CoverEvent {
            request_key,
            result,
        });
    });
}
