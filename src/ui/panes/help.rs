use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::state::AppState;

pub struct HelpPane;

impl HelpPane {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, frame: &mut Frame, state: &AppState) {
        let palette = state.theme.palette;
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(12),
            Constraint::Length(6),
        ])
        .split(frame.area());
        let middle = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);
        let left = Layout::vertical([Constraint::Length(8), Constraint::Min(5)]).split(middle[0]);
        let right = Layout::vertical([Constraint::Length(7), Constraint::Min(6)]).split(middle[1]);

        let title = Paragraph::new("TUIM Help")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(palette.border)),
            );

        let now_playing = state
            .player
            .now_playing
            .as_ref()
            .map(|track| format!("{} - {}", track.artist, track.title))
            .unwrap_or_else(|| String::from("nothing"));
        let paused = if state.player.paused {
            "paused"
        } else {
            "playing"
        };

        let border_style = Style::default().fg(palette.border);
        let start = Paragraph::new(
            "\
Use the function keys to move between screens. Press L for Now Playing.
Use Tab to switch between typing in the search box and moving through results.
Use Left and Right on results to switch Tracks, Albums, and Artists.
When the search box is focused, normal letters type into it.
When results are focused, letters become commands.",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Start Here")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

        let search = Paragraph::new(
            "\
Type text      Search for music
Enter          Search, play/open, or show downloads when search is empty
Up / Down      Move selection
Ctrl+U         Clear search text
B              Back from album/artist detail
Left / Right   Switch Tracks, Albums, Artists",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Search And Results")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

        let library = Paragraph::new(
            "\
A              Add selected track to the end of queue
Shift+A        Add selected track after current song
Shift+D        Queue selected track for download
Click downloads panel to open downloaded tracks
Shift+P        Play all tracks in album/artist detail
Shift+Q        Queue all tracks in album/artist detail
V              Switch cover style: cover, rounded or static vinyl",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Library Actions")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

        let playback = Paragraph::new(
            "\
Space          Pause or resume
Left           Seek back 10 seconds
Right          Seek forward 10 seconds
Shift+Down     Volume down
Shift+Up       Volume up
M              Mute or unmute
X              Stop playback
Esc while playing detaches and keeps music running
Esc while paused stops playback
Lyrics         Load automatically when a song starts
L              Open Now Playing view
F2             Open queue",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Playback")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

        let queue = Paragraph::new(
            "\
Enter          Play selected queued track
N              Next queued track
P              Previous queued track
D              Remove selected queued track
F1             Return to search
Esc            Quit app",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Queue")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

        let footer = Paragraph::new(format!(
            "\
Views: F1 Search | F2 Queue | F3 Help | L Lyrics | Esc/q Close help
Mouse: click rows to select, click selected row again to open/play, wheel scrolls lists.
Now playing: {now_playing} ({paused})
Status: {}
",
            state.status.message
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Session")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

        frame.render_widget(title, chunks[0]);
        frame.render_widget(start, chunks[1]);
        frame.render_widget(search, left[0]);
        frame.render_widget(library, left[1]);
        frame.render_widget(playback, right[0]);
        frame.render_widget(queue, right[1]);
        frame.render_widget(footer, chunks[3]);
    }
}
