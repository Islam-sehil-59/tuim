use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
    app::actions::Action,
    config::keybinds::Keybinds,
    state::{AppState, search::SearchContext, view::View},
    ui::layout::{self, PlaybackClickZone},
};

#[derive(Debug, PartialEq)]
pub struct InputAction {
    pub action: Action,
    pub request_selected_cover: bool,
}

impl InputAction {
    pub fn new(action: Action) -> Self {
        Self {
            action,
            request_selected_cover: false,
        }
    }

    fn with_selected_cover_request(action: Action) -> Self {
        Self {
            action,
            request_selected_cover: true,
        }
    }
}

pub fn action_from_key(state: &AppState, keybinds: &Keybinds, key_event: KeyEvent) -> Action {
    let ctx = KeyContext {
        state,
        keybinds,
        key_event,
    };

    global_nav_key(&ctx)
        .or_else(|| global_playback_key(&ctx))
        .or_else(|| global_toggle_key(&ctx))
        .or_else(|| view_specific_key(&ctx))
        .or_else(|| {
            if ctx.state.current_view == View::Search
                || ctx.state.current_view == View::Album
                || ctx.state.current_view == View::Artist
            {
                text_input_key(&ctx)
            } else {
                None
            }
        })
        .unwrap_or(Action::None)
}

fn global_nav_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.search_view.matches(ctx.key_event) {
        return Some(Action::SwitchToSearch);
    }
    if ctx.keybinds.queue_view.matches(ctx.key_event) {
        return Some(Action::SwitchToQueue);
    }
    if ctx.keybinds.lyrics_view.matches(ctx.key_event)
        && (ctx.state.current_view != View::Search || ctx.state.search.results_focused)
    {
        return Some(Action::SwitchToLyrics);
    }
    if ctx.keybinds.help_view.matches(ctx.key_event)
        || (ctx.keybinds.help.matches(ctx.key_event)
            && can_use_global_text_shortcut(ctx.state, ctx.key_event))
        || (ctx.keybinds.help_alt.matches(ctx.key_event)
            && (ctx.state.current_view != View::Search || ctx.state.search.results_focused))
    {
        return Some(Action::SwitchToHelp);
    }

    None
}

fn global_playback_key(ctx: &KeyContext) -> Option<Action> {
    if !playback_controls_available(ctx.state) {
        return None;
    }
    if ctx.keybinds.pause.matches(ctx.key_event) {
        return Some(Action::TogglePause);
    }
    if ctx.keybinds.volume_up.matches(ctx.key_event) {
        return Some(Action::VolumeUp);
    }
    if ctx.keybinds.volume_down.matches(ctx.key_event) {
        return Some(Action::VolumeDown);
    }
    if can_use_global_text_shortcut(ctx.state, ctx.key_event)
        && ctx.keybinds.mute.matches(ctx.key_event)
    {
        return Some(Action::ToggleMute);
    }

    None
}

fn global_toggle_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.cover_mode.matches(ctx.key_event)
        && (ctx.state.current_view != View::Search || ctx.state.search.results_focused)
    {
        return Some(Action::CycleCoverDisplayMode);
    }
    if ctx.keybinds.audio_quality.matches(ctx.key_event)
        && (ctx.state.current_view != View::Search || ctx.state.search.results_focused)
    {
        return Some(Action::CycleAudioQuality);
    }

    None
}

fn view_specific_key(ctx: &KeyContext) -> Option<Action> {
    match ctx.state.current_view {
        View::Help => help_view_key(ctx),
        View::Lyrics => lyrics_view_key(ctx),
        View::Queue => queue_view_key(ctx),
        View::Search | View::Album | View::Artist => search_view_key(ctx),
    }
}

fn help_view_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.quit.matches(ctx.key_event)
        || KeyCode::Char('q') == ctx.key_event.code && ctx.key_event.modifiers == KeyModifiers::NONE
    {
        return Some(Action::SwitchToSearch);
    }

    None
}

fn lyrics_view_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.quit.matches(ctx.key_event)
        || ctx.keybinds.back.matches(ctx.key_event)
        || ctx.keybinds.lyrics_view.matches(ctx.key_event)
    {
        return Some(Action::SwitchToSearch);
    }
    if ctx.keybinds.select_previous.matches(ctx.key_event) {
        return Some(Action::LyricsScrollUp);
    }
    if ctx.keybinds.select_next.matches(ctx.key_event) {
        return Some(Action::LyricsScrollDown);
    }
    if ctx.keybinds.pause.matches(ctx.key_event) && playback_controls_available(ctx.state) {
        return Some(Action::TogglePause);
    }
    if ctx.keybinds.seek_backward.matches(ctx.key_event) {
        return Some(Action::SeekBackward);
    }
    if ctx.keybinds.seek_forward.matches(ctx.key_event) {
        return Some(Action::SeekForward);
    }

    None
}

fn queue_view_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.quit.matches(ctx.key_event) {
        return Some(Action::Quit);
    }
    if ctx.keybinds.seek_backward.matches(ctx.key_event) {
        return Some(Action::SeekBackward);
    }
    if ctx.keybinds.seek_forward.matches(ctx.key_event) {
        return Some(Action::SeekForward);
    }
    if ctx.keybinds.pause.matches(ctx.key_event) && playback_controls_available(ctx.state) {
        return Some(Action::TogglePause);
    }
    if ctx.keybinds.select_previous.matches(ctx.key_event) {
        return Some(Action::QueueSelectPrev);
    }
    if ctx.keybinds.select_next.matches(ctx.key_event) {
        return Some(Action::QueueSelectNext);
    }
    if ctx.keybinds.play_selected.matches(ctx.key_event) {
        return Some(Action::PlayQueueSelected);
    }
    if ctx.keybinds.queue_remove.matches(ctx.key_event) {
        return Some(Action::RemoveQueueSelected);
    }
    if ctx.keybinds.queue_next_track.matches(ctx.key_event) {
        return Some(Action::PlayQueueNext);
    }
    if ctx.keybinds.queue_previous_track.matches(ctx.key_event) {
        return Some(Action::PlayQueuePrevious);
    }
    if ctx.keybinds.stop.matches(ctx.key_event) {
        return Some(Action::StopPlayback);
    }

    None
}

fn search_view_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.quit.matches(ctx.key_event) {
        return Some(Action::Quit);
    }
    if ctx.state.search.results_focused {
        return search_results_key(ctx);
    }

    search_input_key(ctx)
}

fn search_results_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.toggle_focus.matches(ctx.key_event) {
        return Some(Action::ToggleFocus);
    }
    if ctx.keybinds.clear_search.matches(ctx.key_event) {
        return Some(Action::ClearSearch);
    }
    if ctx.keybinds.select_previous.matches(ctx.key_event) {
        return Some(Action::SelectPrev);
    }
    if ctx.keybinds.select_next.matches(ctx.key_event) {
        return Some(Action::SelectNext);
    }
    if ctx.keybinds.play_selected.matches(ctx.key_event) {
        return Some(Action::PlaySelected);
    }
    if ctx.keybinds.add_queue.matches(ctx.key_event) {
        return Some(Action::AddSelectedToQueue);
    }
    if ctx.keybinds.add_next.matches(ctx.key_event) {
        return Some(Action::AddSelectedNext);
    }
    if ctx.keybinds.download_selected.matches(ctx.key_event) {
        return Some(Action::DownloadSelected);
    }
    if ctx.keybinds.stop.matches(ctx.key_event) {
        return Some(Action::StopPlayback);
    }
    if ctx.keybinds.pause.matches(ctx.key_event) && playback_controls_available(ctx.state) {
        return Some(Action::TogglePause);
    }

    if ctx.state.search.context == SearchContext::Results {
        if ctx.keybinds.seek_backward.matches(ctx.key_event) {
            return Some(Action::SelectPreviousSearchFilter);
        }
        if ctx.keybinds.seek_forward.matches(ctx.key_event) {
            return Some(Action::SelectNextSearchFilter);
        }
    }

    if ctx.state.search.context == SearchContext::Album
        || ctx.state.search.context == SearchContext::Artist
    {
        if ctx.keybinds.back.matches(ctx.key_event) {
            return Some(Action::BackToSearchResults);
        }
        if ctx.keybinds.play_all.matches(ctx.key_event) {
            return Some(Action::PlayAlbum);
        }
        if ctx.keybinds.queue_all.matches(ctx.key_event) {
            return Some(Action::QueueAlbum);
        }
    }

    None
}

fn search_input_key(ctx: &KeyContext) -> Option<Action> {
    if ctx.keybinds.toggle_focus.matches(ctx.key_event) {
        return Some(Action::ToggleFocus);
    }
    if ctx.keybinds.clear_search.matches(ctx.key_event) {
        return Some(Action::ClearSearch);
    }

    None
}

fn text_input_key(ctx: &KeyContext) -> Option<Action> {
    match ctx.key_event.code {
        KeyCode::Enter => Some(Action::Search),
        KeyCode::Backspace if !ctx.state.search.results_focused => Some(Action::BackspaceSearch),
        KeyCode::Char(c)
            if !ctx.state.search.results_focused
                && matches!(
                    ctx.key_event.modifiers,
                    KeyModifiers::NONE | KeyModifiers::SHIFT
                ) =>
        {
            Some(Action::InputChar(c))
        }
        _ => None,
    }
}

struct KeyContext<'a> {
    state: &'a AppState,
    keybinds: &'a Keybinds,
    key_event: KeyEvent,
}

fn playback_controls_available(state: &AppState) -> bool {
    state.player.now_playing.is_some() || state.player.attached_playback
}

pub fn action_from_mouse(
    state: &mut AppState,
    mouse_event: MouseEvent,
    screen_area: Rect,
) -> InputAction {
    if !state.settings.mouse_enabled {
        return InputAction::new(Action::None);
    }
    if !matches!(
        mouse_event.kind,
        MouseEventKind::Down(MouseButton::Left)
            | MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
    ) {
        return InputAction::new(Action::None);
    }

    let transport_height = state.settings.playbar_style.transport_height();
    let layout = layout::screen_layout(screen_area, transport_height);
    if layout::contains(layout.status_volume, mouse_event.column, mouse_event.row)
        && playback_controls_available(state)
    {
        return InputAction::new(match mouse_event.kind {
            MouseEventKind::ScrollUp => Action::VolumeUp,
            MouseEventKind::ScrollDown => Action::VolumeDown,
            MouseEventKind::Down(MouseButton::Left) => {
                volume_click_action(layout.status_volume, mouse_event.column)
            }
            _ => Action::None,
        });
    }

    if mouse_event.kind == MouseEventKind::ScrollUp {
        return InputAction::new(match state.current_view {
            View::Lyrics => Action::LyricsScrollUp,
            View::Search if state.search.results_focused => Action::SelectPrev,
            View::Queue => Action::QueueSelectPrev,
            _ => Action::None,
        });
    }
    if mouse_event.kind == MouseEventKind::ScrollDown {
        return InputAction::new(match state.current_view {
            View::Lyrics => Action::LyricsScrollDown,
            View::Search if state.search.results_focused => Action::SelectNext,
            View::Queue => Action::QueueSelectNext,
            _ => Action::None,
        });
    }

    if layout::contains(layout.nav_search, mouse_event.column, mouse_event.row) {
        return InputAction::new(Action::SwitchToSearch);
    }
    if layout::contains(layout.nav_queue, mouse_event.column, mouse_event.row) {
        return InputAction::new(Action::SwitchToQueue);
    }
    if layout::contains(layout.nav_help, mouse_event.column, mouse_event.row) {
        return InputAction::new(Action::SwitchToHelp);
    }

    match state.current_view {
        View::Search | View::Album | View::Artist => {
            if layout::contains(layout.search_input, mouse_event.column, mouse_event.row) {
                return InputAction::new(Action::FocusSearch);
            }
            if layout::contains(layout.results, mouse_event.column, mouse_event.row) {
                return action_from_results_click(state, layout.results, mouse_event.row);
            }
            if layout::contains(layout.recent, mouse_event.column, mouse_event.row) {
                return InputAction::new(Action::ShowDownloaded);
            }
            if layout::contains(layout.progress, mouse_event.column, mouse_event.row) {
                return InputAction::new(progress_click_action(
                    layout.progress,
                    mouse_event.column,
                ));
            }
            if layout::contains(layout.playback, mouse_event.column, mouse_event.row)
                && state.player.now_playing.is_some()
            {
                return InputAction::new(action_from_playback_click(
                    layout.playback,
                    mouse_event.column,
                ));
            }
            if layout::contains(layout.lyrics, mouse_event.column, mouse_event.row) {
                return InputAction::new(Action::SwitchToLyrics);
            }
        }
        View::Queue => {
            if layout::contains(layout.queue_list, mouse_event.column, mouse_event.row) {
                return action_from_queue_click(state, layout.queue_list, mouse_event.row);
            }
        }
        View::Lyrics => {
            let progress = layout::lyrics_progress_rect(screen_area, transport_height);
            if layout::contains(progress, mouse_event.column, mouse_event.row) {
                return InputAction::new(progress_click_action(progress, mouse_event.column));
            }
        }
        View::Help => {}
    }

    InputAction::new(Action::None)
}

fn action_from_results_click(state: &mut AppState, rect: Rect, row: u16) -> InputAction {
    let Some(index) = layout::row_to_list_index(rect, row) else {
        return InputAction::new(Action::None);
    };
    if index >= state.search.total_items() {
        return InputAction::new(Action::None);
    }

    state.search.results_focused = true;
    if state.search.selected == index {
        InputAction::new(Action::PlaySelected)
    } else {
        state.search.selected = index;
        InputAction::with_selected_cover_request(Action::None)
    }
}

fn action_from_queue_click(state: &mut AppState, rect: Rect, row: u16) -> InputAction {
    let Some(index) = layout::row_to_list_index(rect, row) else {
        return InputAction::new(Action::None);
    };
    if index >= state.queue.items.len() {
        return InputAction::new(Action::None);
    }

    if state.queue.selected == index {
        InputAction::new(Action::PlayQueueSelected)
    } else {
        state.queue.selected = index;
        InputAction::new(Action::None)
    }
}

fn can_use_global_text_shortcut(state: &AppState, key_event: KeyEvent) -> bool {
    key_event.modifiers == KeyModifiers::NONE
        && (state.current_view != View::Search || state.search.results_focused)
}

fn progress_click_action(rect: Rect, col: u16) -> Action {
    let midpoint = rect.x + rect.width / 2;
    if col < midpoint {
        Action::SeekBackward
    } else {
        Action::SeekForward
    }
}

fn volume_click_action(rect: Rect, col: u16) -> Action {
    let midpoint = rect.x + rect.width / 2;
    if col < midpoint {
        Action::VolumeDown
    } else {
        Action::VolumeUp
    }
}

fn action_from_playback_click(rect: Rect, col: u16) -> Action {
    match layout::playback_click_zone(rect, col) {
        PlaybackClickZone::Previous => Action::PlayQueuePrevious,
        PlaybackClickZone::SeekBackward => Action::SeekBackward,
        PlaybackClickZone::TogglePause => Action::TogglePause,
        PlaybackClickZone::Stop => Action::StopPlayback,
        PlaybackClickZone::SeekForward => Action::SeekForward,
        PlaybackClickZone::Next => Action::PlayQueueNext,
        PlaybackClickZone::NowPlaying => Action::SwitchToLyrics,
    }
}
