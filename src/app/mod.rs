pub mod actions;
pub mod events;
mod input;
mod runtime;

use std::time::Duration;

use crossterm::{
    event::{KeyEvent, MouseEvent},
    terminal,
};
use ratatui::{Terminal, layout::Rect, prelude::CrosstermBackend};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
    config::keybinds::Keybinds,
    config::paths,
    engine::{
        covers,
        events::{CoverEvent, RuntimeEvent},
        lyrics,
        playback::{self, PlaybackEngine},
        queue::{self, QueueInsert},
        search,
    },
    log::app_log,
    media::downloads::DownloadRequest,
    player::mpv::MpvPlayer,
    providers::monochrome::MonochromeProvider,
    providers::provider::SearchResults,
    services::{image::ImageService, lyrics::LyricsService},
    state::{
        AppState,
        search::{SearchFilter, SelectedSearchItem},
        settings::SettingsState,
        theme::ThemeState,
        view::View,
    },
    ui::Ui,
};

use self::{
    actions::Action,
    events::{AppEvent, next},
};

pub struct App {
    state: AppState,
    ui: Ui,
    provider: MonochromeProvider,
    image: ImageService,
    lyrics: LyricsService,
    keybinds: Keybinds,
    player: MpvPlayer,
    should_quit: bool,
    cover_tx: UnboundedSender<CoverEvent>,
    cover_rx: UnboundedReceiver<CoverEvent>,
    runtime_tx: UnboundedSender<RuntimeEvent>,
    runtime_rx: UnboundedReceiver<RuntimeEvent>,
    playback: PlaybackEngine,
    active_preview_reason: Option<String>,
    last_track_id: Option<u64>,
}

impl App {
    pub fn new() -> Self {
        let (cover_tx, cover_rx) = unbounded_channel();
        let (runtime_tx, runtime_rx) = unbounded_channel();
        app_log("session started");

        let mut state = AppState::new();
        state.settings = SettingsState::load();
        let keybinds = match Keybinds::load() {
            Ok(keybinds) => keybinds,
            Err(error) => {
                state.status.message = format!("Keybind load failed, using defaults: {error}");
                Keybinds::default()
            }
        };
        match ThemeState::load(state.settings.active_theme.as_deref()) {
            Ok(theme) => state.theme = theme,
            Err(error) => {
                state.theme = ThemeState::new();
                state.status.message = format!("Theme load failed, using default: {error}");
            }
        }

        let audio_quality = state.settings.audio_quality;
        let deezer_fallback_url = state.settings.deezer_fallback_api_url.clone();

        let mut player = MpvPlayer::new();
        if player.is_running() {
            state.player.attached_playback = true;
            state.status.message = String::from("Attached to existing playback session.");
        }

        Self {
            state,
            ui: Ui::new(),
            provider: MonochromeProvider::new_with_deezer_url(deezer_fallback_url)
                .with_quality(audio_quality),
            image: ImageService::new(),
            lyrics: LyricsService::new(),
            keybinds,
            player,
            should_quit: false,
            cover_tx,
            cover_rx,
            runtime_tx,
            runtime_rx,
            playback: PlaybackEngine::new(),
            active_preview_reason: None,
            last_track_id: None,
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> std::io::Result<()> {
        while !self.should_quit {
            self.drain_cover_updates();
            self.drain_runtime_updates();
            self.poll_player_exit();
            self.poll_player_progress();

            self.try_reset_cava_on_new_track();
            self.ui.sync_visualizer_state(&mut self.state);
            let mut cover_rect = None;
            terminal.draw(|frame| {
                cover_rect = self.ui.render(frame, &self.state);
            })?;

            if let Some(rect) = cover_rect
                && let Err(error) = self.ui.sync_cover(rect, &self.state)
            {
                self.state.status.message = format!("Cover render failed: {error}");
                app_log(&format!("cover render failed: {error}"));
            }

            if let Some(event) = next(Duration::from_millis(33))? {
                let action = match event {
                    AppEvent::Key(key_event) => self.action_from_key(key_event),
                    AppEvent::Mouse(mouse_event) => self.action_from_mouse(mouse_event),
                };

                self.handle_action(action);
            }
        }

        let stop_if_paused = self.state.player.paused || self.player.is_paused().unwrap_or(false);
        self.player.shutdown_for_app_exit(stop_if_paused);
        self.ui.cleanup()?;
        app_log("session ended");
        Ok(())
    }

    pub fn mouse_enabled(&self) -> bool {
        self.state.settings.mouse_enabled
    }

    fn action_from_key(&self, key_event: KeyEvent) -> Action {
        input::action_from_key(&self.state, &self.keybinds, key_event)
    }

    fn action_from_mouse(&mut self, mouse_event: MouseEvent) -> Action {
        let (width, height) = terminal::size().unwrap_or((100, 30));
        let input_action = input::action_from_mouse(
            &mut self.state,
            mouse_event,
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
        );
        if input_action.request_selected_cover {
            self.request_cover_for_selected();
        }

        input_action.action
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::InputChar(c) => {
                self.state.search.query.push(c);
            }
            Action::BackspaceSearch => {
                self.state.search.query.pop();
            }
            Action::Search => {
                let query = self.state.search.query.trim().to_string();
                if query.is_empty() {
                    self.show_downloaded_tracks();
                    return;
                }

                app_log(&format!("search requested query={query}"));
                self.state.status.message = format!("Searching for \"{query}\"...");
                search::spawn_search(self.provider.clone(), query, self.runtime_tx.clone());
            }
            Action::FocusSearch => {
                self.state.search.results_focused = false;
                self.state.status.message = String::from("Search input focused.");
            }
            Action::ClearSearch => {
                self.state.search.query.clear();
                self.state.search.results_focused = false;
                self.state.status.message = String::from("Search cleared.");
            }
            Action::SetSearchFilterTracks => {
                self.state.search.set_filter(SearchFilter::Tracks);
                self.state.status.message = String::from("Search filter: tracks.");
                self.request_cover_for_selected();
            }
            Action::SetSearchFilterAlbums => {
                self.state.search.set_filter(SearchFilter::Albums);
                self.state.status.message = String::from("Search filter: albums.");
                self.request_cover_for_selected();
            }
            Action::SetSearchFilterArtists => {
                self.state.search.set_filter(SearchFilter::Artists);
                self.state.status.message = String::from("Search filter: artists.");
                self.request_cover_for_selected();
            }
            Action::SelectPreviousSearchFilter => {
                let filter = self.state.search.filter.previous();
                self.state.search.set_filter(filter);
                self.state.status.message = format!("Search filter: {}.", filter.label());
                self.request_cover_for_selected();
            }
            Action::SelectNextSearchFilter => {
                let filter = self.state.search.filter.next();
                self.state.search.set_filter(filter);
                self.state.status.message = format!("Search filter: {}.", filter.label());
                self.request_cover_for_selected();
            }
            Action::SelectNext => {
                if self.state.search.select_next() {
                    self.request_cover_for_selected();
                }
            }
            Action::SelectPrev => {
                if self.state.search.select_previous() {
                    self.request_cover_for_selected();
                }
            }
            Action::ToggleFocus => {
                if self.state.search.results_focused {
                    self.state.search.results_focused = false;
                } else if self.state.search.total_items() > 0 {
                    self.state.search.results_focused = true;
                } else {
                    self.state.status.message = String::from("No search results to focus.");
                }
            }
            Action::PlaySelected => {
                let Some(item) = self.state.search.selected_item() else {
                    self.state.status.message = String::from("No track selected.");
                    return;
                };

                match item {
                    SelectedSearchItem::Album(album) => {
                        let album_id = album.id;
                        self.state.status.message =
                            format!("Loading album: {} — {}...", album.artist, album.title);
                        search::spawn_album_load(
                            self.provider.clone(),
                            album_id,
                            self.runtime_tx.clone(),
                        );
                    }
                    SelectedSearchItem::Track(track) => {
                        self.request_playback(track.clone());
                    }
                    SelectedSearchItem::Artist(artist) => {
                        let artist_id = artist.id;
                        self.state.status.message = format!("Loading artist: {}...", artist.name);
                        search::spawn_artist_load(
                            self.provider.clone(),
                            artist_id,
                            self.runtime_tx.clone(),
                        );
                    }
                }
            }
            Action::BackToSearchResults => {
                self.state.search.return_to_results();
                self.state.status.message = String::from("Search results.");
            }
            Action::PlayAlbum => {
                match queue::replace_with_search_context(&mut self.state.queue, &self.state.search)
                {
                    Ok(track) => self.request_playback(track),
                    Err(message) => self.state.status.message = message,
                }
            }
            Action::QueueAlbum => {
                match queue::append_search_context(&mut self.state.queue, &self.state.search) {
                    Ok(count) => {
                        let context_label =
                            queue::search_context_label_lower(self.state.search.context);
                        self.state.status.message =
                            format!("Queued {context_label}: {count} tracks.");
                    }
                    Err(message) => self.state.status.message = message,
                }
            }
            Action::AddSelectedToQueue => {
                let Some(track) = self.state.search.selected_track().cloned() else {
                    self.state.status.message = String::from("No track selected.");
                    return;
                };

                let title = track.title.clone();
                let artist = track.artist.clone();
                queue::add_track(&mut self.state.queue, track, QueueInsert::End);
                self.state.status.message = format!("Queued: {artist} — {title}");
            }
            Action::AddSelectedNext => {
                let Some(track) = self.state.search.selected_track().cloned() else {
                    self.state.status.message = String::from("No track selected.");
                    return;
                };

                let title = track.title.clone();
                let artist = track.artist.clone();
                queue::add_track(&mut self.state.queue, track, QueueInsert::Next);
                self.state.status.message = format!("Queued next: {artist} — {title}");
            }
            Action::CycleCoverDisplayMode => {
                self.state.settings.cycle_cover_display_mode();
                self.state.status.message = format!(
                    "Cover display: {}.",
                    self.state.settings.cover_display_mode.label()
                );
                if let Err(error) = self.state.settings.save() {
                    self.state.status.message =
                        format!("Cover display changed, but settings save failed: {error}");
                }
            }
            Action::CycleAudioQuality => {
                self.state.settings.cycle_audio_quality();
                self.provider
                    .set_audio_quality(self.state.settings.audio_quality);
                self.state.status.message = format!(
                    "Audio quality: {}.",
                    self.state.settings.audio_quality.label()
                );
                if let Err(error) = self.state.settings.save() {
                    self.state.status.message =
                        format!("Audio quality changed, but settings save failed: {error}");
                }
            }
            Action::SwitchToSearch => {
                self.state.current_view = View::Search;
                self.state.status.message = String::from("Search view.");
            }
            Action::SwitchToQueue => {
                self.state.current_view = View::Queue;
                self.state.status.message =
                    format!("Queue view: {} tracks.", self.state.queue.items.len());
            }
            Action::SwitchToLyrics => {
                self.state.current_view = View::Lyrics;
                self.state.status.message = String::from("Lyrics view.");
            }
            Action::SwitchToHelp => {
                self.state.current_view = View::Help;
                self.state.status.message = String::from("Help view.");
            }
            Action::TogglePause => match self.player.toggle_pause() {
                Ok(()) => {
                    playback::apply_pause_toggled(&mut self.state);
                }
                Err(error) => {
                    self.state.status.message = format!("Pause failed: {error}");
                }
            },
            Action::StopPlayback => {
                if self.player.stop() {
                    let applied = playback::apply_playback_stopped(&mut self.state);
                    self.active_preview_reason = applied.preview_reason;
                } else {
                    self.state.status.message = String::from("Nothing is playing.");
                }
            }
            Action::SeekBackward => match self.player.seek_relative(-10) {
                Ok(()) => self.state.status.message = String::from("Seeked back 10s."),
                Err(error) => self.state.status.message = format!("Seek failed: {error}"),
            },
            Action::SeekForward => match self.player.seek_relative(10) {
                Ok(()) => self.state.status.message = String::from("Seeked forward 10s."),
                Err(error) => self.state.status.message = format!("Seek failed: {error}"),
            },
            Action::VolumeUp => match self.player.change_volume(5) {
                Ok(()) => self.state.status.message = String::from("Volume up."),
                Err(error) => self.state.status.message = format!("Volume failed: {error}"),
            },
            Action::VolumeDown => match self.player.change_volume(-5) {
                Ok(()) => self.state.status.message = String::from("Volume down."),
                Err(error) => self.state.status.message = format!("Volume failed: {error}"),
            },
            Action::ToggleMute => match self.player.toggle_mute() {
                Ok(()) => self.state.status.message = String::from("Mute toggled."),
                Err(error) => self.state.status.message = format!("Mute failed: {error}"),
            },
            Action::DownloadSelected => {
                let Some(track) = self.state.search.selected_track().cloned() else {
                    self.state.status.message = String::from("Select a track to download.");
                    return;
                };

                let plan =
                    DownloadRequest::single_track(track.clone(), paths::downloads_dir()).plan();
                self.state.status.message = format!(
                    "Download not started yet: {} - {} -> {}",
                    track.artist,
                    track.title,
                    plan.target_file.display()
                );
            }
            Action::ShowDownloaded => {
                self.show_downloaded_tracks();
            }
            Action::QueueSelectNext => {
                self.state.queue.select_next();
            }
            Action::QueueSelectPrev => {
                self.state.queue.select_previous();
            }
            Action::PlayQueueSelected => {
                let Some(track) = self.state.queue.selected_track().cloned() else {
                    self.state.status.message = String::from("Queue is empty.");
                    return;
                };

                self.state.queue.set_current_to_selected();
                self.request_playback(track);
            }
            Action::PlayQueueNext => {
                if !self.play_next_queued_track() {
                    self.state.status.message = String::from("No next queued track.");
                }
            }
            Action::PlayQueuePrevious => {
                if !self.play_previous_queued_track() {
                    self.state.status.message = String::from("No previous queued track.");
                }
            }
            Action::RemoveQueueSelected => {
                let Some(track) = self.state.queue.remove_selected() else {
                    self.state.status.message = String::from("Queue is empty.");
                    return;
                };

                self.state.status.message =
                    format!("Removed from queue: {} — {}", track.artist, track.title);
            }
            Action::LyricsScrollUp => {
                self.state.lyrics.scroll_up();
            }
            Action::LyricsScrollDown => {
                self.state.lyrics.scroll_down();
            }
            Action::Quit => {
                app_log("quit requested");
                self.should_quit = true;
            }
            Action::None => {}
        }
    }

    fn play_next_queued_track(&mut self) -> bool {
        let Some(track) = self.state.queue.next().cloned() else {
            return false;
        };

        self.request_playback(track);
        true
    }

    fn play_previous_queued_track(&mut self) -> bool {
        let Some(track) = self.state.queue.previous().cloned() else {
            return false;
        };

        self.request_playback(track);
        true
    }

    fn request_playback(&mut self, track: crate::models::track::Track) {
        if let Some(request_id) = self.playback.pending_request_id() {
            app_log(&format!(
                "concurrent playback request requested while request_id={request_id} is still resolving"
            ));
        }
        if self.player.is_running() {
            app_log("concurrent playback request while mpv is currently active");
        }

        let started = playback::begin_playback_request(&mut self.state, &mut self.playback, &track);
        app_log(&started.log_message);
        playback::spawn_playback_resolution(
            self.provider.clone(),
            started.request_id,
            track,
            self.runtime_tx.clone(),
        );
    }

    fn show_downloaded_tracks(&mut self) {
        let tracks = self
            .state
            .library
            .downloaded_tracks
            .iter()
            .map(|item| item.track.clone())
            .collect::<Vec<_>>();
        if tracks.is_empty() {
            self.state.status.message = String::from("No downloaded tracks yet.");
            return;
        }

        let count = tracks.len();
        self.state.search.set_results(SearchResults {
            tracks,
            albums: Vec::new(),
            artists: Vec::new(),
        });
        self.state.search.results_focused = true;
        self.state.status.message = format!("Downloaded tracks: {count}.");
        self.request_cover_for_selected();
    }

    fn request_cover_for_selected(&mut self) {
        match self.state.search.selected_item() {
            Some(SelectedSearchItem::Album(album)) => {
                let album = album.clone();
                let request_key = format!("album:{}", album.id);
                if !covers::prepare_cover_request(&mut self.state, &request_key) {
                    return;
                }

                covers::spawn_album_cover_fetch(self.image.clone(), album, self.cover_tx.clone());
            }
            Some(SelectedSearchItem::Track(track)) => {
                self.request_cover_for_track(track.clone());
            }
            Some(SelectedSearchItem::Artist(artist)) => {
                self.request_cover_for_artist(artist.clone());
            }
            None => {
                covers::clear_cover_request(&mut self.state);
            }
        }
    }

    fn request_cover_for_artist(&mut self, artist: crate::models::artist::Artist) {
        let request_key = format!("artist:{}", artist.id);
        if !covers::prepare_cover_request(&mut self.state, &request_key) {
            return;
        }

        covers::spawn_artist_cover_fetch(self.image.clone(), artist, self.cover_tx.clone());
    }

    fn request_cover_for_track(&mut self, track: crate::models::track::Track) {
        let request_key = format!("track:{}", track.id);
        if !covers::prepare_cover_request(&mut self.state, &request_key) {
            return;
        }

        covers::spawn_cover_fetch(self.image.clone(), track, self.cover_tx.clone());
    }

    fn request_playback_cover_for_track(&mut self, track: crate::models::track::Track) {
        let request_key = format!("track:{}", track.id);
        if !covers::prepare_playback_cover_request(&mut self.state, &request_key) {
            return;
        }

        covers::spawn_cover_fetch(self.image.clone(), track, self.cover_tx.clone());
    }

    fn drain_cover_updates(&mut self) {
        while let Ok(result) = self.cover_rx.try_recv() {
            if let Err(error) = covers::apply_cover_result(&mut self.state, result) {
                self.state.status.message = format!("Cover fetch failed: {error}");
                app_log(&format!("cover fetch failed: {error}"));
            }
        }
    }

    fn drain_runtime_updates(&mut self) {
        while let Ok(event) = self.runtime_rx.try_recv() {
            let outcome = runtime::apply_runtime_event(
                &mut self.state,
                &mut self.playback,
                &mut self.player,
                &mut self.active_preview_reason,
                event,
            );
            self.apply_runtime_outcome(outcome);
        }
    }

    fn apply_runtime_outcome(&mut self, outcome: runtime::RuntimeOutcome) {
        for effect in outcome.effects {
            match effect {
                runtime::RuntimeEffect::Log(message) => app_log(&message),
                runtime::RuntimeEffect::RequestSelectedCover => self.request_cover_for_selected(),
                runtime::RuntimeEffect::RequestArtistCover(artist) => {
                    self.request_cover_for_artist(artist);
                }
                runtime::RuntimeEffect::RequestTrackCover(track) => {
                    self.request_playback_cover_for_track(track);
                }
                runtime::RuntimeEffect::RequestLyrics {
                    track,
                    duration_seconds,
                } => {
                    self.request_lyrics_for_track(&track, duration_seconds);
                }
            }
        }
    }

    fn poll_player_exit(&mut self) {
        if let Some(exit) = self.player.poll_exit() {
            let was_success = exit.success;
            let applied = playback::apply_player_exit(
                &mut self.state,
                exit,
                self.active_preview_reason.take(),
            );
            match applied {
                playback::PlaybackExitApply::Ended { log_message, .. } => {
                    app_log(&log_message);
                    if was_success
                        && self.now_playing_matches_queue_current()
                        && self.play_next_queued_track()
                    {
                        return;
                    }
                }
                playback::PlaybackExitApply::PreviewEnded { log_message, .. }
                | playback::PlaybackExitApply::Failed { log_message, .. } => {
                    app_log(&log_message);
                }
            }
        }
    }

    fn poll_player_progress(&mut self) {
        if self.state.player.now_playing.is_none() {
            self.state.player.progress = self.player.poll_progress();
            self.state.player.attached_playback = self.state.player.progress.is_some();
            if let Some(progress) = &self.state.player.progress {
                self.state.player.volume = progress.volume;
            }
            return;
        }

        self.state.player.progress = self.player.poll_progress();
        self.state.player.attached_playback = self.state.player.progress.is_some();
        if let Some(progress) = &self.state.player.progress {
            self.state.player.volume = progress.volume;
        }
        let Some(duration) = self
            .state
            .player
            .progress
            .as_ref()
            .and_then(|progress| progress.duration_secs)
            .map(|duration| duration.round() as u32)
            .filter(|duration| *duration > 0)
        else {
            return;
        };
        if self.state.lyrics.requested_duration_seconds.is_some() {
            return;
        }
        let Some(track) = self.state.player.now_playing.clone() else {
            return;
        };
        if self.state.lyrics.track_id == Some(track.id) {
            self.request_lyrics_for_track(&track, Some(duration));
        }
    }

    fn now_playing_matches_queue_current(&self) -> bool {
        let Some(now_playing) = &self.state.player.now_playing else {
            return false;
        };
        let Some(queue_current) = self.state.queue.current() else {
            return false;
        };

        now_playing.id == queue_current.id
    }

    fn try_reset_cava_on_new_track(&mut self) {
        let current_id = self.state.player.now_playing.as_ref().map(|t| t.id);
        if current_id.is_some() && current_id != self.last_track_id {
            self.last_track_id = current_id;
            self.ui.reset_cava();
        }
        if current_id.is_none() {
            self.last_track_id = None;
        }
    }

    fn request_lyrics_for_track(
        &mut self,
        track: &crate::models::track::Track,
        duration_seconds: Option<u32>,
    ) {
        self.state.lyrics.start_loading(track.id, duration_seconds);
        let lookup = crate::media::lyrics::LyricsLookup::from_track(track, duration_seconds);
        lyrics::spawn_lyrics_load(self.lyrics.clone(), lookup, self.runtime_tx.clone());
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{App, actions::Action};
    use crate::{
        models::{album::Album, artist::Artist, track::Track},
        providers::provider::SearchResults,
        state::search::{SearchContext, SearchFilter},
    };

    #[test]
    fn text_input_shortcuts_do_not_fire_when_search_input_is_focused() {
        let mut app = App::new();

        assert_eq!(
            app.action_from_key(key(KeyCode::Char('v'), KeyModifiers::NONE)),
            Action::InputChar('v')
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('/'), KeyModifiers::NONE)),
            Action::InputChar('/')
        );

        app.state.search.context = SearchContext::Album;
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('b'), KeyModifiers::NONE)),
            Action::InputChar('b')
        );
    }

    #[test]
    fn result_shortcuts_only_fire_with_expected_focus_and_modifiers() {
        let mut app = App::new();
        app.state.search.set_results(SearchResults {
            tracks: vec![track(1)],
            albums: vec![album(2)],
            artists: Vec::new(),
        });

        assert_eq!(
            app.action_from_key(key(KeyCode::Char('v'), KeyModifiers::NONE)),
            Action::CycleCoverDisplayMode
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('/'), KeyModifiers::NONE)),
            Action::None
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL)),
            Action::None
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('a'), KeyModifiers::NONE)),
            Action::AddSelectedToQueue
        );
    }

    #[test]
    fn search_results_use_left_right_tabs_and_shift_arrows_for_volume() {
        let mut app = App::new();
        app.state.search.set_results(SearchResults {
            tracks: vec![track(1)],
            albums: vec![album(2)],
            artists: vec![artist(3)],
        });

        assert_eq!(app.state.search.filter, SearchFilter::Tracks);
        assert_eq!(
            app.action_from_key(key(KeyCode::Right, KeyModifiers::NONE)),
            Action::SelectNextSearchFilter
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Left, KeyModifiers::NONE)),
            Action::SelectPreviousSearchFilter
        );

        app.state.player.attached_playback = true;
        assert_eq!(
            app.action_from_key(key(KeyCode::Up, KeyModifiers::SHIFT)),
            Action::VolumeUp
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Down, KeyModifiers::SHIFT)),
            Action::VolumeDown
        );
    }

    #[test]
    fn help_and_playback_have_practical_fallback_shortcuts() {
        let mut app = App::new();

        assert_eq!(
            app.action_from_key(key(KeyCode::F(3), KeyModifiers::NONE)),
            Action::SwitchToHelp
        );

        app.state.search.set_results(SearchResults {
            tracks: vec![track(1)],
            albums: Vec::new(),
            artists: Vec::new(),
        });
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('?'), KeyModifiers::SHIFT)),
            Action::SwitchToHelp
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('h'), KeyModifiers::NONE)),
            Action::SwitchToHelp
        );

        app.state.player.now_playing = Some(track(1));
        app.state.search.results_focused = false;
        assert_eq!(
            app.action_from_key(key(KeyCode::Char(' '), KeyModifiers::NONE)),
            Action::TogglePause
        );
    }

    #[test]
    fn album_shortcuts_accept_shifted_lowercase_terminal_events() {
        let mut app = App::new();
        app.state.search.set_album_tracks(album(2), vec![track(1)]);

        assert_eq!(
            app.action_from_key(key(KeyCode::Char('p'), KeyModifiers::SHIFT)),
            Action::PlayAlbum
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('q'), KeyModifiers::SHIFT)),
            Action::QueueAlbum
        );
    }

    #[test]
    fn artist_detail_shortcuts_play_and_queue_all_tracks() {
        let mut app = App::new();
        app.state
            .search
            .set_artist_results(artist(10), vec![album(2)], vec![track(1)]);

        assert_eq!(
            app.action_from_key(key(KeyCode::Char('p'), KeyModifiers::SHIFT)),
            Action::PlayAlbum
        );
        assert_eq!(
            app.action_from_key(key(KeyCode::Char('q'), KeyModifiers::SHIFT)),
            Action::QueueAlbum
        );
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn track(id: u64) -> Track {
        Track {
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

    fn album(id: u64) -> Album {
        Album {
            id,
            title: format!("Album {id}"),
            artist: "Artist".to_string(),
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
}
