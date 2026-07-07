use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    state::{
        AppState,
        search::{SearchContext, SearchFilter},
    },
    ui::{
        layout::{self, HomeLayoutMode, MIN_COVER_HEIGHT},
        widgets::{
            entity_preview::preview_lines, footer::render_footer, status_bar::render_status_bar,
            transport::render_transport,
        },
    },
};

pub struct SearchPane;

impl SearchPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, state: &AppState, supports_images: bool) -> Rect {
        let area = frame.area();
        let mode = layout::home_layout_mode(area);
        let outer = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(state.settings.playbar_style.transport_height()),
            Constraint::Length(3),
        ])
        .split(area);

        render_status_bar(frame, outer[0], state);
        render_search_input(frame, outer[1], state);

        let cover_rect = match mode {
            HomeLayoutMode::Wide => {
                let body =
                    Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                        .split(outer[2]);
                let right =
                    Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                        .split(body[1]);

                render_results(frame, body[0], state);
                render_cover(frame, right[0], state, supports_images);
                render_preview(frame, right[1], state, supports_images);
                right[0]
            }
            HomeLayoutMode::Medium => {
                let body =
                    Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                        .split(outer[2]);

                render_results(frame, body[0], state);
                render_cover(frame, body[1], state, supports_images);
                body[1]
            }
            HomeLayoutMode::Narrow => {
                let cover_height = MIN_COVER_HEIGHT.min(outer[2].height.saturating_sub(6).max(6));
                let body = Layout::vertical([Constraint::Length(cover_height), Constraint::Min(6)])
                    .split(outer[2]);

                render_cover(frame, body[0], state, supports_images);
                render_results(frame, body[1], state);
                body[0]
            }
        };

        render_transport(frame, outer[3], state, "Playback");
        render_footer(frame, outer[4], state);

        cover_rect
    }
}

impl Default for SearchPane {
    fn default() -> Self {
        Self::new()
    }
}

fn render_search_input(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.theme.palette;
    let border = if state.search.results_focused {
        palette.border
    } else {
        palette.focused_border
    };
    let search = Paragraph::new(state.search.query.as_str())
        .style(Style::default().fg(palette.foreground))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Search")
                .border_style(Style::default().fg(border)),
        );
    frame.render_widget(search, area);
}

fn render_results(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.theme.palette;
    let title = results_title(state);
    let items = result_items(state);
    let border = if state.search.results_focused {
        palette.focused_border
    } else {
        palette.border
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title)
                .border_style(Style::default().fg(border)),
        )
        .highlight_style(
            Style::default()
                .fg(palette.selected_text)
                .bg(palette.selected_background)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if state.search.total_items() > 0 {
        list_state.select(Some(state.search.selected));
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn results_title(state: &AppState) -> String {
    match state.search.context {
        SearchContext::Results => format!("Results  {}", filter_tabs(state.search.filter)),
        SearchContext::Album => state
            .search
            .current_album
            .as_ref()
            .map(|album| format!("Tracks  {} - {}", album.artist, album.title))
            .unwrap_or_else(|| "Tracks".to_string()),
        SearchContext::Artist => state
            .search
            .current_artist
            .as_ref()
            .map(|artist| format!("Artist  {}", artist.name))
            .unwrap_or_else(|| "Artist".to_string()),
    }
}

fn filter_tabs(active: SearchFilter) -> String {
    [
        SearchFilter::Tracks,
        SearchFilter::Albums,
        SearchFilter::Artists,
    ]
    .into_iter()
    .map(|filter| {
        if filter == active {
            format!("[{}]", filter.label())
        } else {
            filter.label().to_string()
        }
    })
    .collect::<Vec<_>>()
    .join("  ")
}

fn result_items(state: &AppState) -> Vec<ListItem<'static>> {
    if state.search.total_items() == 0 {
        let empty = match state.search.context {
            SearchContext::Results => "Type a query and press Enter to search",
            SearchContext::Album => "No album tracks found",
            SearchContext::Artist => "No artist content found",
        };
        return vec![ListItem::new(Line::from(Span::styled(
            empty,
            Style::default().fg(state.theme.palette.muted_text),
        )))];
    }

    match state.search.context {
        SearchContext::Results => search_result_items(state),
        SearchContext::Album => state
            .search
            .album_tracks
            .iter()
            .enumerate()
            .map(|(index, track)| {
                let marker = playback_marker(state, track.id);
                let cache = downloaded_marker(state, track.id);
                ListItem::new(format!(
                    "{marker}{cache} {:02}  {:32}  {:5}",
                    index + 1,
                    truncate(&track.title, 32),
                    "--:--"
                ))
            })
            .collect(),
        SearchContext::Artist => state
            .search
            .albums
            .iter()
            .map(|album| {
                ListItem::new(format!(
                    "  Album   {:34}  {:18}  {}",
                    truncate(&album.title, 34),
                    truncate(&album.artist, 18),
                    "open"
                ))
            })
            .chain(state.search.album_tracks.iter().map(|track| {
                let marker = playback_marker(state, track.id);
                let cache = downloaded_marker(state, track.id);
                ListItem::new(format!(
                    "{marker}{cache} Track   {:34}  {:18}  {}",
                    truncate(&track.title, 34),
                    truncate(&track.artist, 18),
                    "--:--"
                ))
            }))
            .collect(),
    }
}

fn search_result_items(state: &AppState) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    if matches!(state.search.filter, SearchFilter::Albums) {
        items.extend(state.search.albums.iter().map(|album| {
            ListItem::new(format!(
                "  Album   {:34}  {:18}  {:8}  {}",
                truncate(&album.title, 34),
                truncate(&album.artist, 18),
                "-",
                "open"
            ))
        }));
    }
    if matches!(state.search.filter, SearchFilter::Artists) {
        items.extend(state.search.artists.iter().map(|artist| {
            ListItem::new(format!(
                "  Artist  {:34}  {:18}  {:8}  {}",
                truncate(&artist.name, 34),
                "-",
                "-",
                "artist"
            ))
        }));
    }
    if matches!(state.search.filter, SearchFilter::Tracks) {
        items.extend(state.search.results.iter().map(|track| {
            let marker = playback_marker(state, track.id);
            let cache = downloaded_marker(state, track.id);
            ListItem::new(format!(
                "{marker}{cache} Track   {:34}  {:18}  {:8}  {}",
                truncate(&track.title, 34),
                truncate(&track.artist, 18),
                "-",
                truncate(&track.album, 18)
            ))
        }));
    }
    items
}

fn playback_marker(state: &AppState, track_id: u64) -> &'static str {
    match state.player.now_playing.as_ref() {
        Some(track) if track.id == track_id && state.player.paused => "⏸",
        Some(track) if track.id == track_id => "▶",
        _ => " ",
    }
}

fn downloaded_marker(state: &AppState, track_id: u64) -> &'static str {
    if state.library.is_downloaded(track_id) {
        "✓"
    } else {
        " "
    }
}

fn render_cover(frame: &mut Frame, area: Rect, state: &AppState, supports_images: bool) {
    let palette = state.theme.palette;
    let message = if state.cover.loading {
        "Loading cover art..."
    } else if !supports_images {
        "Image protocol unavailable"
    } else if state.cover.path.is_none() {
        "No cover available"
    } else {
        ""
    };
    let cover = Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(palette.muted_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Cover")
                .border_style(Style::default().fg(palette.border)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(cover, area);
}

fn render_preview(frame: &mut Frame, area: Rect, state: &AppState, supports_images: bool) {
    let palette = state.theme.palette;
    let preview = Paragraph::new(preview_lines(state, supports_images))
        .style(Style::default().fg(palette.foreground))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Preview")
                .border_style(Style::default().fg(palette.border)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(preview, area);
}

fn truncate(value: &str, width: usize) -> String {
    let mut chars = value.chars();
    let mut out = chars.by_ref().take(width).collect::<String>();
    if chars.next().is_some() && width > 1 {
        out.truncate(width.saturating_sub(1));
        out.push('…');
    }
    out
}
