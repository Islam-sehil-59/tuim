use tuim::state::{
    AppState,
    search::SearchFilter,
    search::{SearchContext, SearchState, SelectedSearchItem},
    settings::{CoverDisplayMode, SettingsState},
};

#[test]
fn search_state_tracks_selection_without_exposing_app_logic() {
    let mut search = SearchState::new();
    search.set_results(tuim::providers::provider::SearchResults {
        tracks: vec![track(1), track(2)],
        albums: Vec::new(),
        artists: Vec::new(),
    });

    assert_eq!(search.selected_track().map(|track| track.id), Some(1));

    assert!(search.select_next());
    assert_eq!(search.selected_track().map(|track| track.id), Some(2));

    assert!(search.select_previous());
    assert_eq!(search.selected_track().map(|track| track.id), Some(1));
}

#[test]
fn search_state_can_select_albums_tab_and_open_album_tracks() {
    let mut search = SearchState::new();
    search.set_results(tuim::providers::provider::SearchResults {
        tracks: vec![track(1)],
        albums: vec![album(10)],
        artists: Vec::new(),
    });

    assert!(matches!(
        search.selected_item(),
        Some(SelectedSearchItem::Track(track)) if track.id == 1
    ));
    search.set_filter(SearchFilter::Albums);
    assert!(matches!(
        search.selected_item(),
        Some(SelectedSearchItem::Album(album)) if album.id == 10
    ));

    search.set_album_tracks(album(10), vec![track(2), track(3)]);

    assert_eq!(search.context, SearchContext::Album);
    assert_eq!(search.selected_track().map(|track| track.id), Some(2));

    search.return_to_results();

    assert_eq!(search.context, SearchContext::Results);
    assert!(search.album_tracks.is_empty());
}

#[test]
fn search_filter_limits_selected_items_without_losing_results() {
    let mut search = SearchState::new();
    search.set_results(tuim::providers::provider::SearchResults {
        tracks: vec![track(1)],
        albums: vec![album(10)],
        artists: vec![artist(30)],
    });

    search.set_filter(SearchFilter::Tracks);
    assert_eq!(search.total_items(), 1);
    assert!(matches!(
        search.selected_item(),
        Some(SelectedSearchItem::Track(track)) if track.id == 1
    ));

    search.set_filter(SearchFilter::Albums);
    assert_eq!(search.total_items(), 1);
    assert!(matches!(
        search.selected_item(),
        Some(SelectedSearchItem::Album(album)) if album.id == 10
    ));

    search.set_filter(SearchFilter::Artists);
    assert_eq!(search.total_items(), 1);
    assert!(matches!(
        search.selected_item(),
        Some(SelectedSearchItem::Artist(artist)) if artist.id == 30
    ));
}

#[test]
fn app_state_contains_future_extension_state_buckets() {
    let state = AppState::new();

    assert_eq!(state.theme.name, "default");
    assert!(state.queue.is_empty());
    assert!(state.cover.path.is_none());
    assert!(state.playback_cover.path.is_none());
    assert_eq!(state.settings.cover_display_mode, CoverDisplayMode::Cover);
}

#[test]
fn settings_cycle_cover_display_modes_from_plain_cover_default() {
    let mut settings = SettingsState::new();

    assert_eq!(settings.cover_display_mode, CoverDisplayMode::Cover);
    settings.cycle_cover_display_mode();
    assert_eq!(settings.cover_display_mode, CoverDisplayMode::CoverRounded);
    settings.cycle_cover_display_mode();
    assert_eq!(settings.cover_display_mode, CoverDisplayMode::VinylStill);
    settings.cycle_cover_display_mode();
    assert_eq!(settings.cover_display_mode, CoverDisplayMode::Cover);
}

fn track(id: u64) -> tuim::models::track::Track {
    tuim::models::track::Track {
        id,
        title: format!("Track {id}"),
        artist: "Artist".to_string(),
        artist_id: 10,
        album: "Album".to_string(),
        album_id: 20,
        cover_id: None,
        isrc: None,
    }
}

fn album(id: u64) -> tuim::models::album::Album {
    tuim::models::album::Album {
        id,
        title: format!("Album {id}"),
        artist: "Artist".to_string(),
        cover_id: None,
        release_date: None,
        track_count: None,
    }
}

fn artist(id: u64) -> tuim::models::artist::Artist {
    tuim::models::artist::Artist {
        id,
        name: format!("Artist {id}"),
        picture_id: None,
        description: None,
        album_count: None,
        track_count: None,
    }
}
