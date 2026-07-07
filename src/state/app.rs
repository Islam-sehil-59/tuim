use crate::state::{
    cover::CoverState, library::LibraryState, lyrics::LyricsState, player::PlayerState,
    queue::Queue, search::SearchState, settings::SettingsState, status::StatusState,
    theme::ThemeState, view::View, visualizer::VisualizerState,
};

pub struct AppState {
    pub queue: Queue,
    pub search: SearchState,
    pub player: PlayerState,
    pub current_view: View,
    pub status: StatusState,
    pub cover: CoverState,
    pub playback_cover: CoverState,
    pub lyrics: LyricsState,
    pub library: LibraryState,
    pub settings: SettingsState,
    pub theme: ThemeState,
    pub visualizer: VisualizerState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
            search: SearchState::new(),
            player: PlayerState::new(),
            current_view: View::Search,
            status: StatusState::new(
                "Type query, Enter search/play, Tab focus, Left/Right tabs, Shift+D download, Shift+Up/Down volume, F3 help.",
            ),
            cover: CoverState::new(),
            playback_cover: CoverState::new(),
            lyrics: LyricsState::new(),
            library: LibraryState::new(),
            settings: SettingsState::new(),
            theme: ThemeState::new(),
            visualizer: VisualizerState::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
