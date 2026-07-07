use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    state::AppState,
    ui::widgets::{
        footer::render_footer, status_bar::render_status_bar, transport::render_transport,
    },
};

pub struct QueuePane;

impl QueuePane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, state: &AppState) {
        let outer = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

        render_status_bar(frame, outer[0], state);
        render_queue(frame, outer[1], state);
        render_transport(frame, outer[2], state, "Playback");
        render_status(frame, outer[3], state);
        render_footer(frame, outer[4], state);
    }
}

impl Default for QueuePane {
    fn default() -> Self {
        Self::new()
    }
}

fn render_queue(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let palette = state.theme.palette;
    let mut items = vec![ListItem::new(Line::from(vec![
        Span::styled("  #   ", Style::default().fg(palette.table_header)),
        Span::styled("Title", Style::default().fg(palette.table_header)),
        Span::styled(
            format!("{:32}", ""),
            Style::default().fg(palette.table_header),
        ),
        Span::styled("Artist", Style::default().fg(palette.table_header)),
        Span::styled(
            format!("{:18}", ""),
            Style::default().fg(palette.table_header),
        ),
        Span::styled("Album", Style::default().fg(palette.table_header)),
    ]))];

    if state.queue.items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "Queue is empty",
            Style::default().fg(palette.muted_text),
        ))));
    } else {
        items.extend(state.queue.items.iter().enumerate().map(|(index, track)| {
            let marker = match state.queue.current_index {
                Some(current) if current == index && state.player.paused => "⏸",
                Some(current) if current == index => "▶",
                _ => " ",
            };
            ListItem::new(format!(
                "{marker} {:02}  {:34}  {:20}  {}",
                index + 1,
                truncate(&track.title, 34),
                truncate(&track.artist, 20),
                truncate(&track.album, 28)
            ))
        }));
    }

    let list = List::new(items)
        .style(Style::default().fg(palette.foreground))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Queue")
                .border_style(Style::default().fg(palette.focused_border)),
        )
        .highlight_style(
            Style::default()
                .fg(palette.selected_text)
                .bg(palette.selected_background)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if !state.queue.items.is_empty() {
        list_state.select(Some(state.queue.selected.saturating_add(1)));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_status(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let palette = state.theme.palette;
    let status = Paragraph::new(state.status.message.as_str())
        .style(Style::default().fg(palette.muted_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Status")
                .border_style(Style::default().fg(palette.border)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(status, area);
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
