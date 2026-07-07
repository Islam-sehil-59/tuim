pub mod image;
pub mod layout;
pub mod panes;
pub mod widgets;

use std::io;

use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::state::{AppState, view::View};
use crate::ui::image::KittyImageRenderer;
use crate::ui::panes::{help::HelpPane, lyrics::LyricsPane, queue::QueuePane, search::SearchPane};
use crate::visualizer::cava::CavaVisualizer;

pub struct Ui {
    search_pane: SearchPane,
    queue_pane: QueuePane,
    lyrics_pane: LyricsPane,
    help_pane: HelpPane,
    kitty: KittyImageRenderer,
    cava: CavaVisualizer,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            search_pane: SearchPane::new(),
            queue_pane: QueuePane::new(),
            lyrics_pane: LyricsPane::new(),
            help_pane: HelpPane::new(),
            kitty: KittyImageRenderer::new(),
            cava: CavaVisualizer::new(96, u16::MAX),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, state: &AppState) -> Option<Rect> {
        let bg = Style::default().bg(state.theme.palette.background);
        frame.render_widget(Paragraph::new("").style(bg), frame.area());

        let visualizer = self.cava.frame(state.player.now_playing.is_some());
        match state.current_view {
            View::Search | View::Album | View::Artist => Some(self.search_pane.render(
                frame,
                state,
                self.kitty.is_supported(),
            )),
            View::Queue => {
                self.queue_pane.render(frame, state);
                None
            }
            View::Lyrics => Some(self.lyrics_pane.render(frame, state, visualizer.as_ref())),
            View::Help => {
                self.help_pane.render(frame, state);
                None
            }
        }
    }

    pub fn sync_cover(&mut self, cover_rect: Rect, state: &AppState) -> io::Result<()> {
        self.kitty.sync_cover(cover_rect, state)
    }

    pub fn reset_cava(&mut self) {
        self.cava.reset();
    }

    pub fn sync_visualizer_state(&mut self, state: &mut AppState) {
        self.cava.update_state(&mut state.visualizer);
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        self.kitty.hide()
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}
