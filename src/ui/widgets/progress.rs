use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Gauge},
};

use crate::player::mpv::PlaybackProgress;
use crate::theme::palette::ThemePalette;

pub fn render_progress(
    frame: &mut Frame,
    area: Rect,
    progress: Option<&PlaybackProgress>,
    palette: ThemePalette,
) {
    let label = progress
        .map(PlaybackProgress::label)
        .unwrap_or_else(|| String::from("--:-- / --:--"));
    let ratio = progress.map(PlaybackProgress::ratio).unwrap_or(0.0);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Progress"),
        )
        .gauge_style(
            Style::default()
                .fg(palette.progress_fill)
                .bg(palette.progress_empty)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(label);

    frame.render_widget(gauge, area);
}
