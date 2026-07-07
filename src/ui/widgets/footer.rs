use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::state::{AppState, view::View};

pub fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.theme.palette;
    let mode = match state.current_view {
        View::Search => "SEARCH",
        View::Album => "ALBUM",
        View::Artist => "ARTIST",
        View::Queue => "QUEUE",
        View::Lyrics => "LYRICS",
        View::Help => "HELP",
    };
    let hints = match state.current_view {
        View::Lyrics => {
            "j/k scroll  space pause  left/right seek  shift+up/down volume  q back  ? help"
        }
        View::Queue => {
            "j/k move  enter play  n/p next/prev  shift+up/down volume  d remove  q back  ? help"
        }
        _ => {
            "tab focus  left/right tabs  enter open/play/downloads  shift+d download  shift+up/down volume  ? help"
        }
    };

    let line = Line::from(vec![
        Span::styled(
            format!("{mode}  "),
            Style::default()
                .fg(palette.selected_text)
                .bg(palette.selected_background)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(hints, Style::default().fg(palette.footer_text)),
        Span::styled(
            format!("   {}", state.status.message),
            Style::default().fg(palette.muted_text),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(line)
            .style(Style::default().fg(palette.foreground))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(palette.border)),
            ),
        area,
    );
}
