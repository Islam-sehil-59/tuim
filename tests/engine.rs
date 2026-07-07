use tuim::{
    engine::{
        covers,
        events::CoverEvent,
        lyrics::{self, LyricsApply},
        playback::{self, PlaybackEngine},
        search,
    },
    media::{
        lyrics::{Lyrics, LyricsLookup, LyricsSource},
        playback::{PlaybackSource, PlaybackSourceKind},
    },
    models::{album::Album, artist::Artist, track::Track},
    player::mpv::PlaybackExit,
    providers::provider::{AlbumDetails, ArtistDetails, SearchResults},
    services::image::CoverArt,
    state::{
        AppState,
        lyrics::LyricsState,
        search::{SearchContext, SearchFilter, SearchState},
    },
};

#[test]
fn playback_engine_marks_only_latest_request_as_current() {
    let mut engine = PlaybackEngine::new();

    let first = engine.begin_request();
    let second = engine.begin_request();

    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert!(!engine.take_if_current(first));
    assert!(engine.take_if_current(second));
    assert_eq!(engine.pending_request_id(), None);
}

#[test]
fn search_engine_applies_mixed_search_results() {
    let mut state = SearchState::new();
    let applied = search::apply_search_results(
        &mut state,
        SearchResults {
            tracks: vec![track(1)],
            albums: vec![album(2)],
            artists: vec![artist(3)],
        },
    );

    assert_eq!(
        applied.status_message,
        "Found 1 tracks, 1 albums, and 1 artists."
    );
    assert!(applied.request_selected_cover);
    assert_eq!(state.context, SearchContext::Results);
    assert_eq!(state.filter, SearchFilter::Tracks);
    assert_eq!(state.total_items(), 1);
    state.set_filter(SearchFilter::Albums);
    assert_eq!(state.total_items(), 1);
    state.set_filter(SearchFilter::Artists);
    assert_eq!(state.total_items(), 1);
    assert!(state.results_focused);
}

#[test]
fn search_engine_applies_album_details() {
    let mut state = SearchState::new();
    let applied = search::apply_album_details(
        &mut state,
        AlbumDetails {
            album: album(2),
            tracks: vec![track(10), track(11)],
        },
    );

    assert_eq!(
        applied.status_message,
        "Album: Artist — Album 2 (2 tracks)."
    );
    assert!(!applied.request_selected_cover);
    assert_eq!(state.context, SearchContext::Album);
    assert_eq!(state.album_tracks.len(), 2);
}

#[test]
fn search_engine_applies_artist_details_and_returns_cover_target() {
    let mut state = SearchState::new();
    let applied = search::apply_artist_details(
        &mut state,
        ArtistDetails {
            artist: artist(3),
            albums: vec![album(2)],
            tracks: vec![track(10)],
        },
    );

    assert_eq!(
        applied.status_message,
        "Artist: Artist 3 (1 albums, 1 tracks)."
    );
    assert_eq!(applied.cover_artist.map(|artist| artist.id), Some(3));
    assert_eq!(state.context, SearchContext::Artist);
    assert_eq!(state.total_items(), 2);
}

#[test]
fn playback_engine_applies_successful_playback_start() {
    let mut state = AppState::new();
    let track = track(1);
    let resolution = playback_resolution(None);

    let applied = playback::apply_playback_started(&mut state, track, &resolution);

    assert_eq!(applied.status_message, "Playing: Artist — Track 1");
    assert_eq!(applied.preview_reason, None);
    assert_eq!(
        state.player.now_playing.as_ref().map(|track| track.id),
        Some(1)
    );
    assert!(!state.player.paused);
    assert!(state.player.progress.is_none());
}

#[test]
fn playback_engine_applies_preview_playback_start() {
    let mut state = AppState::new();
    let track = track(1);
    let resolution = playback_resolution(Some(String::from("FULL_REQUIRES_SUBSCRIPTION")));

    let applied = playback::apply_playback_started(&mut state, track, &resolution);

    assert_eq!(
        applied.status_message,
        "Playing preview: Artist — Track 1 (FULL_REQUIRES_SUBSCRIPTION)"
    );
    assert_eq!(
        applied.preview_reason.as_deref(),
        Some("FULL_REQUIRES_SUBSCRIPTION")
    );
}

#[test]
fn playback_engine_clears_state_on_stop_and_failures() {
    let mut state = AppState::new();
    state.player.now_playing = Some(track(1));
    state.player.paused = true;
    state.lyrics.start_loading(1, None);

    let stopped = playback::apply_playback_stopped(&mut state);

    assert_eq!(stopped.status_message, "Playback stopped.");
    assert!(state.player.now_playing.is_none());
    assert!(!state.player.paused);
    assert!(state.lyrics.track_id.is_none());

    playback::apply_playback_resolution_failed(&mut state, "network");
    assert_eq!(state.status.message, "Stream lookup failed: network");

    playback::apply_playback_start_failed(&mut state, "missing mpv");
    assert_eq!(state.status.message, "mpv failed: missing mpv");
}

#[test]
fn playback_engine_applies_player_exit_messages() {
    let mut state = AppState::new();
    state.player.now_playing = Some(track(1));
    state.player.paused = true;
    state.lyrics.start_loading(1, None);

    let applied = playback::apply_player_exit(
        &mut state,
        PlaybackExit {
            message: String::from("Playback ended (mpv reached end of stream)."),
            success: true,
        },
        None,
    );

    match applied {
        playback::PlaybackExitApply::Ended { status_message, .. } => {
            assert_eq!(
                status_message,
                "Playback ended (mpv reached end of stream)."
            );
        }
        _ => panic!("expected ended playback"),
    }
    assert!(!state.player.paused);
    assert!(state.lyrics.track_id.is_none());
}

#[test]
fn playback_engine_applies_preview_exit_message() {
    let mut state = AppState::new();

    let applied = playback::apply_player_exit(
        &mut state,
        PlaybackExit {
            message: String::from("Playback ended (mpv reached end of stream)."),
            success: true,
        },
        Some(String::from("preview")),
    );

    match applied {
        playback::PlaybackExitApply::PreviewEnded { status_message, .. } => {
            assert_eq!(
                status_message,
                "Playback ended: preview-only stream stopped normally (preview)."
            );
        }
        _ => panic!("expected preview-ended playback"),
    }
}

#[test]
fn lyrics_engine_applies_matching_result() {
    let mut state = LyricsState::new();
    state.start_loading(1, Some(180));

    let applied = lyrics::apply_lyrics_result(&mut state, 1, Some(180), Ok(Some(lyrics(1, 180))));

    assert_eq!(applied, LyricsApply::Applied);
    assert!(!state.loading);
    assert!(state.lyrics.is_some());
    assert!(state.error.is_none());
}

#[test]
fn lyrics_engine_ignores_stale_result() {
    let mut state = LyricsState::new();
    state.start_loading(2, Some(200));

    let applied = lyrics::apply_lyrics_result(&mut state, 1, Some(180), Ok(Some(lyrics(1, 180))));

    assert_eq!(applied, LyricsApply::Stale);
    assert!(state.loading);
    assert!(state.lyrics.is_none());
}

#[test]
fn lyrics_engine_applies_empty_and_failed_results() {
    let mut state = LyricsState::new();
    state.start_loading(1, None);

    let applied = lyrics::apply_lyrics_result(&mut state, 1, None, Ok(None));

    assert_eq!(applied, LyricsApply::Applied);
    assert_eq!(state.error.as_deref(), Some("No lyrics found."));

    state.start_loading(1, None);
    let failed = lyrics::apply_lyrics_result(&mut state, 1, None, Err(String::from("timeout")));

    assert_eq!(failed, LyricsApply::Failed(String::from("timeout")));
    assert_eq!(state.error.as_deref(), Some("timeout"));
}

#[test]
fn cover_engine_ignores_stale_failed_cover_fetches() {
    let mut state = AppState::new();

    assert!(covers::prepare_cover_request(&mut state, "track:1"));
    assert!(covers::prepare_cover_request(&mut state, "track:2"));

    let result = covers::apply_cover_result(
        &mut state,
        CoverEvent {
            request_key: String::from("track:1"),
            result: Err(String::from("network failed")),
        },
    );

    assert_eq!(result, Ok(()));
    assert_eq!(state.cover.request_key.as_deref(), Some("track:2"));
    assert!(state.cover.loading);
    assert_eq!(state.cover.path, None);
}

#[test]
fn cover_engine_applies_matching_failed_cover_fetches() {
    let mut state = AppState::new();

    assert!(covers::prepare_cover_request(&mut state, "track:1"));

    let result = covers::apply_cover_result(
        &mut state,
        CoverEvent {
            request_key: String::from("track:1"),
            result: Err(String::from("network failed")),
        },
    );

    assert_eq!(result.err(), Some(String::from("network failed")));
    assert!(!state.cover.loading);
    assert_eq!(state.cover.path, None);
}

#[test]
fn cover_engine_applies_matching_successful_cover_fetches() {
    let mut state = AppState::new();

    assert!(covers::prepare_cover_request(&mut state, "track:1"));

    let result = covers::apply_cover_result(
        &mut state,
        CoverEvent {
            request_key: String::from("track:1"),
            result: Ok(CoverArt {
                request_key: String::from("track:1"),
                path: Some(String::from("/tmp/cover.png")),
            }),
        },
    );

    assert_eq!(result, Ok(()));
    assert!(!state.cover.loading);
    assert_eq!(state.cover.path.as_deref(), Some("/tmp/cover.png"));
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

fn artist(id: u64) -> Artist {
    Artist {
        id,
        name: format!("Artist {id}"),
        picture_id: None,
        description: None,
        album_count: None,
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

fn lyrics(track_id: u64, duration_seconds: u32) -> Lyrics {
    Lyrics {
        lookup: LyricsLookup {
            track_id,
            title: String::from("Track"),
            artist: String::from("Artist"),
            album: Some(String::from("Album")),
            duration_seconds: Some(duration_seconds),
        },
        source: LyricsSource::Lrclib,
        plain: Some(String::from("Line one")),
        synced: Vec::new(),
    }
}
