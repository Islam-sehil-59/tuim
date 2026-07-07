use crate::{
    engine::events::RuntimeEvent,
    media::lyrics::{Lyrics, LyricsLookup},
    services::lyrics::LyricsService,
    state::lyrics::LyricsState,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Eq, PartialEq)]
pub enum LyricsApply {
    Applied,
    Stale,
    Failed(String),
}

pub fn spawn_lyrics_load(
    service: LyricsService,
    lookup: LyricsLookup,
    tx: UnboundedSender<RuntimeEvent>,
) {
    let track_id = lookup.track_id;
    let duration_seconds = lookup.duration_seconds;
    tokio::spawn(async move {
        let _ = tx.send(RuntimeEvent::LyricsLoaded {
            track_id,
            duration_seconds,
            result: service.fetch(lookup).await,
        });
    });
}

pub fn apply_lyrics_result(
    state: &mut LyricsState,
    track_id: u64,
    duration_seconds: Option<u32>,
    result: Result<Option<Lyrics>, String>,
) -> LyricsApply {
    if state.track_id != Some(track_id) || state.requested_duration_seconds != duration_seconds {
        return LyricsApply::Stale;
    }

    state.loading = false;
    match result {
        Ok(Some(loaded_lyrics)) => {
            state.lyrics = Some(loaded_lyrics);
            state.error = None;
            LyricsApply::Applied
        }
        Ok(None) => {
            state.lyrics = None;
            state.error = Some(String::from("No lyrics found."));
            LyricsApply::Applied
        }
        Err(error) => {
            state.lyrics = None;
            state.error = Some(error.clone());
            LyricsApply::Failed(error)
        }
    }
}
