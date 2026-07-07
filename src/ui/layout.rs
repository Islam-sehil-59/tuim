use ratatui::layout::{Constraint, Layout, Rect};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackClickZone {
    Previous,
    SeekBackward,
    TogglePause,
    Stop,
    SeekForward,
    Next,
    NowPlaying,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenLayout {
    pub nav_search: Rect,
    pub nav_queue: Rect,
    pub nav_help: Rect,
    pub status_volume: Rect,
    pub search_input: Rect,
    pub results: Rect,
    pub recent: Rect,
    pub queue_list: Rect,
    pub playback: Rect,
    pub progress: Rect,
    pub lyrics: Rect,
}

pub fn screen_layout(area: Rect) -> ScreenLayout {
    let nav = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Percentage(33),
    ])
    .split(Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    });
    let visualizer_height = if area.height >= 34 { 8 } else { 5 };
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(3),
        Constraint::Length(visualizer_height),
        Constraint::Length(3),
    ])
    .split(area);
    let body = Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(outer[2]);
    let status_inner = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let status_chunks = Layout::horizontal([
        Constraint::Percentage(26),
        Constraint::Percentage(50),
        Constraint::Percentage(24),
    ])
    .split(status_inner);
    let right =
        Layout::vertical([Constraint::Percentage(58), Constraint::Percentage(42)]).split(body[1]);
    let top_player = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(right[0]);

    ScreenLayout {
        nav_search: nav[0],
        nav_queue: nav[1],
        nav_help: nav[2],
        status_volume: status_chunks[2],
        search_input: outer[1],
        results: body[0],
        recent: right[1],
        queue_list: queue_list_rect(area),
        playback: outer[3],
        progress: Rect {
            x: outer[3].x.saturating_add(1),
            y: outer[3].y.saturating_add(1),
            width: outer[3].width.saturating_sub(2),
            height: 1,
        },
        lyrics: top_player[1],
    }
}

pub fn queue_list_rect(area: Rect) -> Rect {
    Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(area)[1]
}

pub fn contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x
        && row >= rect.y
        && col < rect.x.saturating_add(rect.width)
        && row < rect.y.saturating_add(rect.height)
}

pub fn row_to_list_index(rect: Rect, row: u16) -> Option<usize> {
    if row <= rect.y || row >= rect.y.saturating_add(rect.height).saturating_sub(1) {
        return None;
    }

    Some(usize::from(row - rect.y - 1))
}

pub fn playback_click_zone(rect: Rect, col: u16) -> PlaybackClickZone {
    let inner_width = rect.width.saturating_sub(2).max(1);
    let relative = col.saturating_sub(rect.x.saturating_add(1));
    let zone = (usize::from(relative) * 7) / usize::from(inner_width);

    match zone {
        0 => PlaybackClickZone::Previous,
        1 => PlaybackClickZone::SeekBackward,
        2 => PlaybackClickZone::TogglePause,
        3 => PlaybackClickZone::Stop,
        4 => PlaybackClickZone::SeekForward,
        5 => PlaybackClickZone::Next,
        _ => PlaybackClickZone::NowPlaying,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_to_list_index_ignores_borders() {
        let rect = Rect {
            x: 4,
            y: 10,
            width: 20,
            height: 6,
        };

        assert_eq!(row_to_list_index(rect, 10), None);
        assert_eq!(row_to_list_index(rect, 11), Some(0));
        assert_eq!(row_to_list_index(rect, 14), Some(3));
        assert_eq!(row_to_list_index(rect, 15), None);
    }

    #[test]
    fn playback_click_zone_splits_controls_left_to_right() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 72,
            height: 5,
        };

        assert_eq!(playback_click_zone(rect, 1), PlaybackClickZone::Previous);
        assert_eq!(
            playback_click_zone(rect, 11),
            PlaybackClickZone::SeekBackward
        );
        assert_eq!(
            playback_click_zone(rect, 21),
            PlaybackClickZone::TogglePause
        );
        assert_eq!(playback_click_zone(rect, 31), PlaybackClickZone::Stop);
        assert_eq!(
            playback_click_zone(rect, 41),
            PlaybackClickZone::SeekForward
        );
        assert_eq!(playback_click_zone(rect, 51), PlaybackClickZone::Next);
        assert_eq!(playback_click_zone(rect, 61), PlaybackClickZone::NowPlaying);
    }
}
