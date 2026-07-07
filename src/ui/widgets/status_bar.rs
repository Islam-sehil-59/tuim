use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
};

use crate::{player::mpv::PlaybackProgress, state::AppState, theme::palette::ThemePalette};

pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.theme.palette;
    let (label, color) = playback_label(state, palette);
    let progress = state
        .player
        .progress
        .as_ref()
        .map(PlaybackProgress::label)
        .unwrap_or_else(|| "--:-- / --:--".to_string());
    let track = state
        .player
        .now_playing
        .as_ref()
        .map(|track| {
            format!(
                "{} - {} - {}",
                track.artist,
                if track.album.is_empty() {
                    "unknown source"
                } else {
                    track.album.as_str()
                },
                track.title
            )
        })
        .unwrap_or_else(|| {
            if state.player.attached_playback {
                "Attached playback session".to_string()
            } else {
                "No track loaded".to_string()
            }
        });
    let quality = quality_label(state);
    let volume = state
        .player
        .progress
        .as_ref()
        .and_then(|progress| progress.volume)
        .or(state.player.volume)
        .map(|volume| format!("Vol {volume}%"))
        .unwrap_or_else(|| "Vol --%".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 48 {
        let compact = Line::from(vec![
            Span::styled(
                format!("[{label}] "),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{quality} | "),
                Style::default().fg(palette.muted_text),
            ),
            Span::styled(track, Style::default().fg(palette.foreground)),
            Span::styled(
                format!(" | {volume}"),
                Style::default().fg(palette.muted_text),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(compact).style(Style::default().fg(palette.foreground)),
            inner,
        );
        return;
    }

    let chunks = Layout::horizontal([
        Constraint::Percentage(26),
        Constraint::Percentage(50),
        Constraint::Percentage(24),
    ])
    .split(inner);

    let left = Line::from(vec![
        Span::styled(
            format!(" {label} "),
            Style::default()
                .fg(palette.selected_text)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" - {quality}"),
            Style::default()
                .fg(palette.muted_text)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let center = Line::from(vec![
        Span::styled(track, Style::default().fg(palette.foreground)),
        Span::styled(
            format!("     {progress}"),
            Style::default().fg(palette.muted_text),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(left).style(Style::default().fg(palette.foreground)),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(center)
            .style(Style::default().fg(palette.foreground))
            .alignment(Alignment::Center),
        chunks[1],
    );
    render_volume(frame, chunks[2], state, &volume);
}

fn playback_label(
    state: &AppState,
    palette: ThemePalette,
) -> (&'static str, ratatui::style::Color) {
    if state.status.message.to_ascii_lowercase().contains("error") {
        return ("Error", palette.error);
    }

    if state.player.now_playing.is_none() && !state.player.attached_playback {
        return ("Stopped", palette.status_stopped);
    }

    if state.player.paused {
        ("Paused", palette.status_paused)
    } else {
        ("Playing", palette.status_playing)
    }
}

fn quality_label(state: &AppState) -> String {
    let codec = state
        .player
        .progress
        .as_ref()
        .and_then(|progress| progress.audio_codec.as_deref())
        .map(format_codec);
    let bitrate = state
        .player
        .progress
        .as_ref()
        .and_then(|progress| progress.audio_bitrate_kbps);

    match (codec, bitrate, state.player.source_quality.as_deref()) {
        (Some(codec), Some(bitrate), _) if is_lossless_codec(&codec) => {
            format!("{codec} Lossless ({bitrate} kbps)")
        }
        (Some(codec), Some(bitrate), _) => format!("{codec} ({bitrate} kbps)"),
        (Some(codec), None, _) if is_lossless_codec(&codec) => format!("{codec} Lossless"),
        (Some(codec), None, _) => codec,
        (None, Some(bitrate), _) => format!("{bitrate} kbps"),
        (None, None, Some(quality)) => format_quality(quality),
        (None, None, None) => String::from("-- kbps"),
    }
}

fn format_codec(codec: &str) -> String {
    match codec.to_ascii_lowercase().as_str() {
        "mp3" | "mp3float" => String::from("MP3"),
        "flac" => String::from("FLAC"),
        "aac" | "aac_latm" => String::from("AAC"),
        "opus" => String::from("OPUS"),
        "vorbis" => String::from("OGG"),
        other => other.to_ascii_uppercase(),
    }
}

fn format_quality(quality: &str) -> String {
    quality.replace('_', " ")
}

fn is_lossless_codec(codec: &str) -> bool {
    matches!(codec, "FLAC" | "ALAC" | "WAV")
}

fn render_volume(frame: &mut Frame, area: Rect, state: &AppState, label: &str) {
    let palette = state.theme.palette;
    let volume = state
        .player
        .progress
        .as_ref()
        .and_then(|progress| progress.volume)
        .or(state.player.volume)
        .unwrap_or(0)
        .min(100);

    if area.width < 18 {
        frame.render_widget(
            Paragraph::new(label.to_string())
                .alignment(Alignment::Right)
                .style(Style::default().fg(palette.muted_text)),
            area,
        );
        return;
    }

    let chunks = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Min(8),
        Constraint::Length(7),
    ])
    .split(area);
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(palette.progress_fill)
                .bg(palette.progress_empty)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(f64::from(volume) / 100.0)
        .label("");

    frame.render_widget(
        Paragraph::new("").style(Style::default().fg(palette.muted_text)),
        chunks[0],
    );
    frame.render_widget(gauge, chunks[1]);
    frame.render_widget(
        Paragraph::new(format!("{volume:>3}%"))
            .alignment(Alignment::Right)
            .style(Style::default().fg(palette.muted_text)),
        chunks[2],
    );
}
