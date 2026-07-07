use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::{
    media::lyrics::Lyrics,
    state::AppState,
    state::settings::PlaybarStyle,
    ui::widgets::{footer::render_footer, status_bar::render_status_bar, visualizer::cava_lines},
    visualizer::cava::CavaFrame,
};

pub struct LyricsPane;

impl LyricsPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        state: &AppState,
        visualizer_frame: Option<&CavaFrame>,
    ) -> Rect {
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());
        let body = Layout::horizontal([Constraint::Percentage(32), Constraint::Percentage(68)])
            .split(chunks[1]);

        let palette = state.theme.palette;

        // ── Left window: cover + playback ──
        let now_playing = state
            .player
            .now_playing
            .as_ref()
            .map(|track| format!("{} - {}", track.artist, track.title))
            .unwrap_or_else(|| String::from("Nothing playing"));
        let cover_message = if state.playback_cover.loading {
            "Loading cover art..."
        } else if state.playback_cover.path.is_none() {
            "No cover available"
        } else {
            ""
        };

        let left_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(now_playing)
            .border_style(Style::default().fg(palette.border));
        let left_inner = left_block.inner(body[0]);
        frame.render_widget(left_block, body[0]);

        let playback_height = state.settings.playbar_style.transport_height();
        let left_chunks =
            Layout::vertical([Constraint::Min(5), Constraint::Length(playback_height)])
                .split(left_inner);

        let cover = Paragraph::new(cover_message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(palette.muted_text));
        frame.render_widget(cover, left_chunks[0]);

        render_playback_inline(frame, left_chunks[1], state);

        // ── Right window: lyrics + CAVA ──
        let right_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Lyrics")
            .border_style(Style::default().fg(palette.border));
        let right_inner = right_block.inner(body[1]);
        frame.render_widget(right_block, body[1]);

        let right_chunks =
            Layout::vertical([Constraint::Min(5), Constraint::Length(8)]).split(right_inner);

        let lyrics = Paragraph::new(full_lyrics_lines(
            state,
            right_chunks[0].height.saturating_sub(1),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        frame.render_widget(lyrics, right_chunks[0]);

        let visualizer = Paragraph::new(cava_lines(
            state,
            visualizer_frame,
            right_chunks[1].width.saturating_sub(1),
            right_chunks[1].height.saturating_sub(1),
        ))
        .alignment(Alignment::Left);
        frame.render_widget(visualizer, right_chunks[1]);

        // ── Top & bottom rows ──
        render_status_bar(frame, chunks[0], state);
        render_footer(frame, chunks[2], state);

        body[0]
    }
}

fn render_playback_inline(frame: &mut Frame, area: Rect, state: &AppState) {
    match state.settings.playbar_style {
        PlaybarStyle::Classic => render_playback_classic(frame, area, state),
        PlaybarStyle::Modern => render_playback_modern(frame, area, state),
    }
}

fn render_playback_classic(frame: &mut Frame, area: Rect, state: &AppState) {
    use ratatui::widgets::Gauge;

    use crate::player::mpv::PlaybackProgress;

    let palette = state.theme.palette;
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);

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
    frame.render_widget(gauge, chunks[0]);

    let pause_icon = if state.player.paused { "" } else { "" };
    let controls = Paragraph::new(Line::from(vec![
        Span::raw(" prev   "),
        Span::raw(" -10   "),
        Span::styled(
            format!("{pause_icon} play/pause"),
            Style::default()
                .fg(palette.accent_secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    stop   "),
        Span::raw("+10    "),
        Span::raw("next "),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(controls, chunks[1]);
}

fn render_playback_modern(frame: &mut Frame, area: Rect, state: &AppState) {
    use crate::player::mpv::PlaybackProgress;

    let palette = state.theme.palette;
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);

    let progress = state.player.progress.as_ref();
    let ratio = progress.map(PlaybackProgress::ratio).unwrap_or(0.0);
    let label = progress
        .map(PlaybackProgress::label)
        .unwrap_or_else(|| "--:-- / --:--".to_string());

    let width = chunks[0].width;
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
    frame.render_widget(Paragraph::new(line), chunks[0]);

    let pause_icon = if state.player.paused { "" } else { "" };
    let controls = Paragraph::new(Line::from(vec![
        Span::styled(
            "  ",
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("", Style::default().fg(palette.foreground)),
        Span::raw("  "),
        Span::styled(
            pause_icon,
            Style::default()
                .fg(palette.accent_secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("", Style::default().fg(palette.foreground)),
        Span::raw("  "),
        Span::styled("", Style::default().fg(palette.foreground)),
        Span::raw("  "),
        Span::styled(
            "",
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(controls, chunks[1]);
}

fn full_lyrics_lines(state: &AppState, height: u16) -> Vec<Line<'static>> {
    if state.player.now_playing.is_none() {
        return centered_message("Play a song to load lyrics.");
    }
    if state.lyrics.loading {
        return centered_message("Loading lyrics...");
    }
    if let Some(error) = &state.lyrics.error {
        return centered_message(error);
    }
    let Some(lyrics) = &state.lyrics.lyrics else {
        return centered_message("No lyrics loaded.");
    };

    if lyrics.has_synced_lines() {
        synced_lines(
            lyrics,
            state
                .player
                .progress
                .as_ref()
                .map(|progress| progress.position_secs)
                .unwrap_or(0.0),
            state.theme.palette.lyrics_current,
            state.theme.palette.lyrics_previous_next,
            height,
        )
    } else {
        plain_lines(
            lyrics.plain.as_deref().unwrap_or(""),
            state.lyrics.scroll,
            height,
        )
    }
}

fn synced_lines(
    lyrics: &Lyrics,
    position_secs: f64,
    accent: Color,
    muted: Color,
    height: u16,
) -> Vec<Line<'static>> {
    let position_ms = (position_secs.max(0.0) * 1_000.0) as u32;
    let current = lyrics
        .synced
        .iter()
        .rposition(|line| line.timestamp_ms <= position_ms)
        .unwrap_or(0);
    let expanded_current = height >= 10;
    let visible = if expanded_current {
        3
    } else {
        usize::from(height.clamp(3, 5))
    };
    let start = current.saturating_sub(visible / 2);

    let mut lyric_lines = Vec::new();
    for (index, line) in lyrics.synced.iter().skip(start).take(visible).enumerate() {
        if start + index == current {
            let next_timestamp = lyrics.synced.get(current + 1).map(|line| line.timestamp_ms);
            let highlighted =
                highlighted_word_count(&line.text, position_ms, line.timestamp_ms, next_timestamp);

            lyric_lines.push(Line::from(current_line_spans(
                &line.text,
                highlighted,
                accent,
                muted,
            )));
        } else {
            lyric_lines.push(Line::from(Span::styled(
                line.text.clone(),
                Style::default().fg(muted).add_modifier(Modifier::DIM),
            )));
        }
    }

    spaced_lines(lyric_lines, height)
}

fn current_line_spans(
    text: &str,
    highlighted: usize,
    accent: Color,
    muted: Color,
) -> Vec<Span<'static>> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![Span::raw(String::new())];
    }

    words
        .iter()
        .enumerate()
        .flat_map(|(index, word)| {
            let style = if index < highlighted {
                Style::default().fg(accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(muted).add_modifier(Modifier::BOLD)
            };
            [
                Span::styled((*word).to_string(), style),
                Span::raw(String::from(" ")),
            ]
        })
        .collect()
}

fn highlighted_word_count(
    text: &str,
    position_ms: u32,
    line_start_ms: u32,
    next_line_ms: Option<u32>,
) -> usize {
    let words = text.split_whitespace().count();
    if words == 0 {
        return 0;
    }

    let line_duration = next_line_ms
        .and_then(|next| next.checked_sub(line_start_ms))
        .unwrap_or(2_500)
        .max(1);
    let elapsed = position_ms.saturating_sub(line_start_ms).min(line_duration);
    ((elapsed as f64 / line_duration as f64) * words as f64)
        .ceil()
        .clamp(1.0, words as f64) as usize
}

fn plain_lines(lyrics: &str, scroll: usize, height: u16) -> Vec<Line<'static>> {
    let visible = usize::from((height / 2).clamp(1, 4));
    let lines: Vec<Line<'static>> = lyrics
        .lines()
        .filter(|line| !line.trim().is_empty())
        .skip(scroll)
        .take(visible)
        .map(|line| Line::from(line.trim().to_string()))
        .collect();

    if lines.is_empty() {
        centered_message("No plain lyrics available.")
    } else {
        spaced_lines(lines, height)
    }
}

fn spaced_lines(lines: Vec<Line<'static>>, height: u16) -> Vec<Line<'static>> {
    let mut spaced = Vec::new();
    let top_padding = usize::from(height.saturating_sub((lines.len() * 2) as u16) / 2);
    for _ in 0..top_padding.min(3) {
        spaced.push(Line::from(""));
    }

    for line in lines {
        spaced.push(line);
        spaced.push(Line::from(""));
    }

    spaced
}

fn centered_message(message: &str) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            message.to_string(),
            Style::default().add_modifier(Modifier::DIM),
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_highlight_count_tracks_line_progress() {
        assert_eq!(
            highlighted_word_count("one two three four", 1_000, 0, Some(2_000)),
            2
        );
    }
}
