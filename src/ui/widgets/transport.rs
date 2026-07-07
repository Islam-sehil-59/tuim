use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
};

use crate::{
    player::mpv::PlaybackProgress, state::AppState, state::settings::PlaybarStyle,
    theme::palette::ThemePalette,
};

pub fn render_transport(frame: &mut Frame, area: Rect, state: &AppState, title: &str) {
    match state.settings.playbar_style {
        PlaybarStyle::Classic => render_classic(frame, area, state, title),
        PlaybarStyle::Modern => render_modern(frame, area, state),
    }
}

fn render_classic(frame: &mut Frame, area: Rect, state: &AppState, title: &str) {
    let palette = state.theme.palette;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(palette.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    let progress = state.player.progress.as_ref();
    let label = progress
        .map(PlaybackProgress::label)
        .unwrap_or_else(|| "--:-- / --:--".to_string());
    let ratio = progress.map(PlaybackProgress::ratio).unwrap_or(0.0);
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(palette.progress_fill)
                .bg(palette.progress_empty)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(label);
    let pause_icon = if state.player.paused { "" } else { "" };
    let controls = Paragraph::new(Line::from(vec![
        Span::styled(" prev", Style::default().fg(palette.foreground)),
        Span::styled("    -10", Style::default().fg(palette.foreground)),
        Span::styled(
            format!("   {pause_icon} play/pause"),
            Style::default()
                .fg(palette.accent_secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("    stop", Style::default().fg(palette.foreground)),
        Span::styled("   +10 ", Style::default().fg(palette.foreground)),
        Span::styled("   next ", Style::default().fg(palette.foreground)),
        Span::styled("   ", Style::default().fg(palette.muted_text)),
        Span::styled("   ", Style::default().fg(palette.muted_text)),
        Span::styled("    now playing", Style::default().fg(palette.foreground)),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(gauge, chunks[0]);
    frame.render_widget(controls, chunks[1]);
}

fn render_modern(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.theme.palette;
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);

    render_progress_line(frame, chunks[0], state, palette);
    render_controls(frame, chunks[1], state, palette);
}

fn render_progress_line(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let progress = state.player.progress.as_ref();
    let ratio = progress.map(PlaybackProgress::ratio).unwrap_or(0.0);
    let label = progress
        .map(PlaybackProgress::label)
        .unwrap_or_else(|| "--:-- / --:--".to_string());

    let width = area.width;
    let bar_width = width.saturating_sub(label.len() as u16 + 2);
    let filled = (bar_width as f64 * ratio) as u16;

    let line = Line::from(vec![
        Span::styled(
            "━".repeat(filled as usize),
            Style::default().fg(palette.progress_fill),
        ),
        Span::styled(
            "─".repeat((bar_width - filled) as usize),
            Style::default().fg(palette.muted_text),
        ),
        Span::styled(
            format!(" {} ", label),
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_controls(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let pause_icon = if state.player.paused { "" } else { "" };
    let controls = Paragraph::new(Line::from(vec![
        Span::styled(
            "    ",
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("", Style::default().fg(palette.foreground)),
        Span::styled("  ", Style::default().fg(palette.foreground)),
        Span::styled(
            pause_icon,
            Style::default()
                .fg(palette.accent_secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().fg(palette.foreground)),
        Span::styled("", Style::default().fg(palette.foreground)),
        Span::styled("  ", Style::default().fg(palette.foreground)),
        Span::styled("", Style::default().fg(palette.foreground)),
        Span::styled("  ", Style::default().fg(palette.foreground)),
        Span::styled(
            "",
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(controls, area);
}
