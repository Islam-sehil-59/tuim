use tuim::state::{AppState, view::View};

#[test]
fn app_state_starts_in_search_input_mode() {
    let state = AppState::new();

    assert_eq!(state.current_view, View::Search);
    assert!(state.search.query.is_empty());
    assert!(state.search.results.is_empty());
    assert_eq!(state.search.selected, 0);
    assert!(!state.search.results_focused);
    assert!(state.player.now_playing.is_none());
    assert!(state.status.message.contains("Type query"));
    assert!(state.status.message.contains("Left/Right tabs"));
    assert_eq!(state.theme.name, "default");
}

#[test]
fn app_state_starts_with_no_cover_request() {
    let state = AppState::new();

    assert_eq!(state.cover.request_key, None);
    assert!(!state.cover.loading);
    assert_eq!(state.cover.path, None);
}
