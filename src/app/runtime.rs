use crate::{
    engine::{
        events::RuntimeEvent,
        lyrics::{self, LyricsApply},
        playback::{self, PlaybackEngine},
        search,
    },
    models::{artist::Artist, track::Track},
    player::traits::Player,
    state::AppState,
};

#[derive(Debug)]
pub enum RuntimeEffect {
    Log(String),
    RequestSelectedCover,
    RequestArtistCover(Artist),
    RequestTrackCover(Track),
    RequestLyrics {
        track: Track,
        duration_seconds: Option<u32>,
    },
}

#[derive(Default, Debug)]
pub struct RuntimeOutcome {
    pub effects: Vec<RuntimeEffect>,
}

impl RuntimeOutcome {
    fn push(&mut self, effect: RuntimeEffect) {
        self.effects.push(effect);
    }
}

pub fn apply_runtime_event<P: Player>(
    state: &mut AppState,
    playback_engine: &mut PlaybackEngine,
    player: &mut P,
    active_preview_reason: &mut Option<String>,
    event: RuntimeEvent,
) -> RuntimeOutcome {
    let mut outcome = RuntimeOutcome::default();

    match event {
        RuntimeEvent::SearchCompleted(result) => match result {
            Ok(results) => {
                let applied = search::apply_search_results(&mut state.search, results);
                state.status.message = applied.status_message;
                if let Some(message) = applied.log_message {
                    outcome.push(RuntimeEffect::Log(message));
                }
                if applied.request_selected_cover {
                    outcome.push(RuntimeEffect::RequestSelectedCover);
                }
            }
            Err(error) => {
                state.status.message = format!("Search failed: {error}");
                outcome.push(RuntimeEffect::Log(format!("search failed: {error}")));
            }
        },
        RuntimeEvent::AlbumLoaded(result) => match result {
            Ok(details) => {
                let applied = search::apply_album_details(&mut state.search, details);
                state.status.message = applied.status_message;
                if let Some(message) = applied.log_message {
                    outcome.push(RuntimeEffect::Log(message));
                }
                if applied.request_selected_cover {
                    outcome.push(RuntimeEffect::RequestSelectedCover);
                }
            }
            Err(error) => {
                state.status.message = format!("Album load failed: {error}");
                outcome.push(RuntimeEffect::Log(format!("album load failed: {error}")));
            }
        },
        RuntimeEvent::ArtistLoaded(result) => match result {
            Ok(details) => {
                let applied = search::apply_artist_details(&mut state.search, details);
                state.status.message = applied.status_message;
                if let Some(message) = applied.log_message {
                    outcome.push(RuntimeEffect::Log(message));
                }
                if let Some(artist) = applied.cover_artist {
                    outcome.push(RuntimeEffect::RequestArtistCover(artist));
                }
            }
            Err(error) => {
                state.status.message = format!("Artist load failed: {error}");
                outcome.push(RuntimeEffect::Log(format!("artist load failed: {error}")));
            }
        },
        RuntimeEvent::PlaybackResolved {
            request_id,
            track,
            result,
        } => {
            if !playback_engine.take_if_current(request_id) {
                outcome.push(RuntimeEffect::Log(format!(
                    "ignoring stale playback resolution request_id={request_id} pending={:?}",
                    playback_engine.pending_request_id()
                )));
                return outcome;
            }

            match result {
                Ok(resolution) => {
                    outcome.push(RuntimeEffect::Log(format!(
                        "playback resolved request_id={request_id} kind={:?} source={} quality={:?} preview_reason={:?}",
                        resolution.kind,
                        resolution.url,
                        resolution.quality,
                        resolution.preview_reason
                    )));

                    *active_preview_reason = resolution.preview_reason.clone();
                    match player.play(&resolution) {
                        Ok(()) => {
                            let applied =
                                playback::apply_playback_started(state, track.clone(), &resolution);
                            *active_preview_reason = applied.preview_reason;
                            outcome.push(RuntimeEffect::RequestTrackCover(track.clone()));
                            outcome.push(RuntimeEffect::RequestLyrics {
                                track,
                                duration_seconds: None,
                            });
                        }
                        Err(error) => {
                            *active_preview_reason = None;
                            playback::apply_playback_start_failed(state, error.to_string());
                            outcome
                                .push(RuntimeEffect::Log(format!("mpv failed to start: {error}")));
                        }
                    }
                }
                Err(error) => {
                    *active_preview_reason = None;
                    playback::apply_playback_resolution_failed(state, error.clone());
                    outcome.push(RuntimeEffect::Log(format!(
                        "playback resolution failed: {error}"
                    )));
                }
            }
        }
        RuntimeEvent::LyricsLoaded {
            track_id,
            duration_seconds,
            result,
        } => {
            match lyrics::apply_lyrics_result(&mut state.lyrics, track_id, duration_seconds, result)
            {
                LyricsApply::Applied | LyricsApply::Stale => {}
                LyricsApply::Failed(error) => {
                    outcome.push(RuntimeEffect::Log(format!("lyrics load failed: {error}")));
                }
            }
        }
    }

    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::events::RuntimeEvent,
        media::playback::{PlaybackSource, PlaybackSourceKind},
        models::{album::Album, track::Track},
        player::error::PlayerError,
        providers::provider::SearchResults,
        state::AppState,
    };

    struct FakePlayer {
        result: Result<(), PlayerError>,
        played: Vec<String>,
    }

    impl Default for FakePlayer {
        fn default() -> Self {
            Self {
                result: Ok(()),
                played: Vec::new(),
            }
        }
    }

    impl Player for FakePlayer {
        fn play(&mut self, source: &PlaybackSource) -> Result<(), PlayerError> {
            self.played.push(source.url.clone());
            self.result.clone()
        }
    }

    #[test]
    fn runtime_applies_search_results_and_requests_selected_cover() {
        let mut state = AppState::new();
        let mut playback = PlaybackEngine::new();
        let mut player = FakePlayer::default();
        let mut preview_reason = None;

        let outcome = apply_runtime_event(
            &mut state,
            &mut playback,
            &mut player,
            &mut preview_reason,
            RuntimeEvent::SearchCompleted(Ok(SearchResults {
                tracks: vec![track(1)],
                albums: vec![album(2)],
                artists: Vec::new(),
            })),
        );

        assert_eq!(state.search.total_items(), 1);
        assert!(
            outcome
                .effects
                .iter()
                .any(|effect| matches!(effect, RuntimeEffect::RequestSelectedCover))
        );
        assert!(outcome
            .effects
            .iter()
            .any(|effect| matches!(effect, RuntimeEffect::Log(message) if message.contains("search completed"))));
    }

    #[test]
    fn runtime_ignores_stale_playback_resolution() {
        let mut state = AppState::new();
        let mut playback = PlaybackEngine::new();
        let current = playback.begin_request();
        let stale = current + 1;
        let mut player = FakePlayer::default();
        let mut preview_reason = None;

        let outcome = apply_runtime_event(
            &mut state,
            &mut playback,
            &mut player,
            &mut preview_reason,
            RuntimeEvent::PlaybackResolved {
                request_id: stale,
                track: track(1),
                result: Ok(playback_resolution(None)),
            },
        );

        assert!(state.player.now_playing.is_none());
        assert!(player.played.is_empty());
        assert_eq!(playback.pending_request_id(), Some(current));
        assert!(outcome
            .effects
            .iter()
            .any(|effect| matches!(effect, RuntimeEffect::Log(message) if message.contains("stale playback"))));
    }

    #[test]
    fn runtime_starts_resolved_playback_and_requests_media_followups() {
        let mut state = AppState::new();
        let mut playback = PlaybackEngine::new();
        let request_id = playback.begin_request();
        let mut player = FakePlayer::default();
        let mut preview_reason = None;

        let outcome = apply_runtime_event(
            &mut state,
            &mut playback,
            &mut player,
            &mut preview_reason,
            RuntimeEvent::PlaybackResolved {
                request_id,
                track: track(1),
                result: Ok(playback_resolution(Some(String::from("preview")))),
            },
        );

        assert_eq!(player.played, vec!["https://cdn.example.test/track.flac"]);
        assert_eq!(
            state.player.now_playing.as_ref().map(|track| track.id),
            Some(1)
        );
        assert_eq!(preview_reason.as_deref(), Some("preview"));
        assert!(outcome.effects.iter().any(
            |effect| matches!(effect, RuntimeEffect::RequestTrackCover(track) if track.id == 1)
        ));
        assert!(outcome
            .effects
            .iter()
            .any(|effect| matches!(effect, RuntimeEffect::RequestLyrics { track, duration_seconds: None } if track.id == 1)));
    }

    #[test]
    fn runtime_reports_player_start_failure() {
        let mut state = AppState::new();
        let mut playback = PlaybackEngine::new();
        let request_id = playback.begin_request();
        let mut player = FakePlayer {
            result: Err(PlayerError::Spawn(String::from("missing mpv"))),
            played: Vec::new(),
        };
        let mut preview_reason = Some(String::from("old"));

        let outcome = apply_runtime_event(
            &mut state,
            &mut playback,
            &mut player,
            &mut preview_reason,
            RuntimeEvent::PlaybackResolved {
                request_id,
                track: track(1),
                result: Ok(playback_resolution(None)),
            },
        );

        assert_eq!(state.status.message, "mpv failed: missing mpv");
        assert_eq!(preview_reason, None);
        assert!(outcome.effects.iter().any(
            |effect| matches!(effect, RuntimeEffect::Log(message) if message.contains("mpv failed"))
        ));
    }

    fn track(id: u64) -> Track {
        Track {
            id,
            title: format!("Track {id}"),
            artist: String::from("Artist"),
            artist_id: 10,
            album: String::from("Album"),
            album_id: 20,
            cover_id: None,
            isrc: None,
        }
    }

    fn album(id: u64) -> Album {
        Album {
            id,
            title: format!("Album {id}"),
            artist: String::from("Artist"),
            cover_id: None,
            release_date: None,
            track_count: None,
        }
    }

    fn playback_resolution(preview_reason: Option<String>) -> PlaybackSource {
        PlaybackSource {
            url: String::from("https://cdn.example.test/track.flac"),
            kind: PlaybackSourceKind::DirectUrl,
            quality: Some(String::from("FLAC")),
            format: None,
            headers: Vec::new(),
            is_preview: preview_reason.is_some(),
            preview_reason,
            expires_at: None,
        }
    }
}
