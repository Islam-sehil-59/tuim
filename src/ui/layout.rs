use ratatui::layout::{Constraint, Layout, Rect};

pub const MIN_COVER_WIDTH: u16 = 24;
pub const MIN_COVER_HEIGHT: u16 = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeLayoutMode {
    Wide,
    Medium,
    Narrow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LyricsLayoutMode {
    Wide,
    Medium,
    Narrow,
}

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

pub fn home_layout_mode(area: Rect) -> HomeLayoutMode {
    if area.width >= MIN_COVER_WIDTH * 5 && area.height >= 30 {
        HomeLayoutMode::Wide
    } else if area.width >= MIN_COVER_WIDTH * 3 + 14 && area.height >= 24 {
        HomeLayoutMode::Medium
    } else {
        HomeLayoutMode::Narrow
    }
}

pub fn lyrics_layout_mode(area: Rect) -> LyricsLayoutMode {
    if area.width >= MIN_COVER_WIDTH * 5 && area.height >= 30 {
        LyricsLayoutMode::Wide
    } else if area.width >= MIN_COVER_WIDTH * 3 + 14 && area.height >= 24 {
        LyricsLayoutMode::Medium
    } else {
        LyricsLayoutMode::Narrow
    }
}

pub fn screen_layout(area: Rect, transport_height: u16) -> ScreenLayout {
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
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(transport_height),
        Constraint::Length(3),
    ])
    .split(area);
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

    let mode = home_layout_mode(area);
    let (results, playback, lyrics) = match mode {
        HomeLayoutMode::Wide => {
            let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(outer[2]);
            let right =
                Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(body[1]);
            (body[0], outer[3], right[1])
        }
        HomeLayoutMode::Medium => {
            let body = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(outer[2]);
            (body[0], outer[3], Rect::default())
        }
        HomeLayoutMode::Narrow => {
            let body =
                Layout::vertical([Constraint::Length(14), Constraint::Min(6)]).split(outer[2]);
            (body[1], outer[3], Rect::default())
        }
    };

    ScreenLayout {
        nav_search: nav[0],
        nav_queue: nav[1],
        nav_help: nav[2],
        status_volume: status_chunks[2],
        search_input: outer[1],
        results,
        recent: Rect::default(),
        queue_list: queue_list_rect(area),
        playback,
        progress: Rect {
            x: playback.x.saturating_add(1),
            y: playback.y.saturating_add(1),
            width: playback.width.saturating_sub(2),
            height: 1,
        },
        lyrics,
    }
}

pub fn lyrics_progress_rect(area: Rect, transport_height: u16) -> Rect {
    match lyrics_layout_mode(area) {
        LyricsLayoutMode::Wide => {
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);
            let body = Layout::horizontal([Constraint::Percentage(34), Constraint::Percentage(66)])
                .split(chunks[1]);
            let inner = Rect {
                x: body[0].x.saturating_add(1),
                y: body[0].y.saturating_add(1),
                width: body[0].width.saturating_sub(2),
                height: body[0].height.saturating_sub(2),
            };
            let left_chunks =
                Layout::vertical([Constraint::Min(5), Constraint::Length(transport_height)])
                    .split(inner);
            Rect {
                x: left_chunks[1].x,
                y: left_chunks[1].y,
                width: left_chunks[1].width,
                height: 1,
            }
        }
        LyricsLayoutMode::Medium | LyricsLayoutMode::Narrow => {
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(transport_height),
                Constraint::Length(3),
            ])
            .split(area);
            Rect {
                x: chunks[2].x.saturating_add(1),
                y: chunks[2].y.saturating_add(1),
                width: chunks[2].width.saturating_sub(2),
                height: 1,
            }
        }
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

    #[test]
    fn home_layout_modes_follow_cover_preservation_breakpoints() {
        assert_eq!(
            home_layout_mode(Rect {
                x: 0,
                y: 0,
                width: 140,
                height: 36
            }),
            HomeLayoutMode::Wide
        );
        assert_eq!(
            home_layout_mode(Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 28
            }),
            HomeLayoutMode::Medium
        );
        assert_eq!(
            home_layout_mode(Rect {
                x: 0,
                y: 0,
                width: 72,
                height: 22
            }),
            HomeLayoutMode::Narrow
        );
    }

    #[test]
    fn lyrics_layout_modes_hide_lower_priority_regions_first() {
        assert_eq!(
            lyrics_layout_mode(Rect {
                x: 0,
                y: 0,
                width: 140,
                height: 36
            }),
            LyricsLayoutMode::Wide
        );
        assert_eq!(
            lyrics_layout_mode(Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 28
            }),
            LyricsLayoutMode::Medium
        );
        assert_eq!(
            lyrics_layout_mode(Rect {
                x: 0,
                y: 0,
                width: 72,
                height: 22
            }),
            LyricsLayoutMode::Narrow
        );
    }

    #[test]
    fn lyrics_progress_rect_tracks_responsive_transport_location() {
        let wide = lyrics_progress_rect(
            Rect {
                x: 0,
                y: 0,
                width: 140,
                height: 36,
            },
            2,
        );
        let narrow = lyrics_progress_rect(
            Rect {
                x: 0,
                y: 0,
                width: 72,
                height: 22,
            },
            2,
        );

        assert!(wide.x < 50);
        assert!(narrow.width > wide.width);
        assert_eq!(wide.height, 1);
        assert_eq!(narrow.height, 1);
    }
}
