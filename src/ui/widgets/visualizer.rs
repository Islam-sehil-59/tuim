use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::{state::AppState, visualizer::cava::CavaFrame};

pub fn cava_lines(
    state: &AppState,
    visualizer_frame: Option<&CavaFrame>,
    max_width: u16,
    max_height: u16,
) -> Vec<Line<'static>> {
    let Some(frame) = visualizer_frame else {
        let message = if let Some(ref error) = state.visualizer.last_error {
            format!("CAVA: {error}")
        } else if state.player.now_playing.is_some() {
            "CAVA starting...".to_string()
        } else {
            "Play a song for CAVA.".to_string()
        };
        return vec![Line::from(Span::styled(
            message,
            Style::default()
                .fg(state.theme.palette.muted_text)
                .add_modifier(Modifier::DIM),
        ))];
    };

    cava_bar_lines(
        frame,
        usize::from(max_width).max(1),
        usize::from(max_height).max(1),
        state.theme.palette.visualizer,
    )
}

fn cava_bar_lines(
    frame: &CavaFrame,
    max_width: usize,
    height: usize,
    accent: ratatui::style::Color,
) -> Vec<Line<'static>> {
    let bar_width = 1;
    let bar_spacing = 1;
    let stride = bar_width + bar_spacing;
    let columns = (max_width / stride).max(1).min(frame.bars.len());
    let x_padding =
        max_width.saturating_sub(columns * bar_width + columns.saturating_sub(1) * bar_spacing) / 2;
    let values = frame
        .bars
        .iter()
        .take(columns)
        .map(|value| f32::from(*value) * height as f32 / f32::from(frame.max.max(1)))
        .collect::<Vec<_>>();

    let mut lines = Vec::with_capacity(height);
    for row in (0..height).rev() {
        let mut spans = Vec::new();
        if x_padding > 0 {
            spans.push(Span::raw(" ".repeat(x_padding)));
        }
        for (index, value) in values.iter().enumerate() {
            if index > 0 && bar_spacing > 0 {
                spans.push(Span::raw(" ".repeat(bar_spacing)));
            }
            spans.push(Span::styled(
                cava_cell(*value, row),
                Style::default().fg(accent),
            ));
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn cava_cell(column_height: f32, row: usize) -> &'static str {
    let symbols = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let fill = (column_height - row as f32).clamp(0.0, 0.999);
    if fill < 0.01 {
        return " ";
    }
    let index = (fill * 8.0).floor() as usize + 1;
    symbols[index.min(symbols.len() - 1)]
}
