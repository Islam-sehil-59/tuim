use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::state::{
    AppState,
    search::{SearchContext, SelectedSearchItem},
};

pub fn preview_lines(state: &AppState, supports_images: bool) -> Vec<Line<'static>> {
    let palette = state.theme.palette;
    let mut lines = Vec::new();

    if state.cover.loading {
        lines.push(Line::from(Span::styled(
            "Loading artwork...",
            Style::default().fg(palette.status_loading),
        )));
        lines.push(Line::from(""));
    } else if !supports_images {
        lines.push(Line::from(Span::styled(
            "Image protocol unavailable",
            Style::default().fg(palette.muted_text),
        )));
        lines.push(Line::from(""));
    }

    if state.search.context == SearchContext::Artist
        && let Some(artist) = &state.search.current_artist
    {
        lines.push(header("Artist", palette.accent));
        lines.push(strong(artist.name.clone(), palette.foreground));
        lines.push(meta(
            format!("Albums: {}", state.search.albums.len()),
            palette.table_header,
        ));
        lines.push(meta(
            format!("Top tracks: {}", state.search.album_tracks.len()),
            palette.table_header,
        ));
        if artist.picture_id.is_some() {
            lines.push(meta("Image: available".to_string(), palette.table_header));
        }
        lines.push(Line::from(""));
        if let Some(description) = &artist.description {
            lines.push(muteline(description.clone(), palette.muted_text));
        } else {
            lines.push(muteline(
                "Select an album to open it, or a top track to play it.".to_string(),
                palette.muted_text,
            ));
        }
        return lines;
    }

    match state.search.selected_item() {
        Some(SelectedSearchItem::Album(album)) => {
            lines.push(header("Album", palette.accent));
            lines.push(strong(album.title.clone(), palette.foreground));
            lines.push(muteline(album.artist.clone(), palette.muted_text));
            let tracks = if state.search.context == SearchContext::Album {
                state.search.album_tracks.len()
            } else {
                album.track_count.unwrap_or(0)
            };
            lines.push(meta(
                if tracks > 0 {
                    format!("{tracks} tracks")
                } else {
                    "Track list opens on Enter".to_string()
                },
                palette.table_header,
            ));
            if let Some(release_date) = &album.release_date {
                lines.push(meta(
                    format!("Released: {release_date}"),
                    palette.table_header,
                ));
            }
            if album.cover_id.is_some() {
                lines.push(meta("Artwork: available".to_string(), palette.table_header));
            }
            lines.push(Line::from(""));
            lines.push(muteline(
                "Enter opens tracklist. Shift+P plays all, Shift+Q queues all.".to_string(),
                palette.muted_text,
            ));
        }
        Some(SelectedSearchItem::Artist(artist)) => {
            lines.push(header("Artist", palette.accent));
            lines.push(strong(artist.name.clone(), palette.foreground));
            if let Some(album_count) = artist.album_count {
                lines.push(meta(format!("Albums: {album_count}"), palette.table_header));
            }
            if let Some(track_count) = artist.track_count {
                lines.push(meta(
                    format!("Top tracks: {track_count}"),
                    palette.table_header,
                ));
            }
            if artist.picture_id.is_some() {
                lines.push(meta("Image: available".to_string(), palette.table_header));
            }
            lines.push(meta(
                "Albums and top tracks open on Enter".to_string(),
                palette.table_header,
            ));
            lines.push(Line::from(""));
            if let Some(description) = &artist.description {
                lines.push(muteline(description.clone(), palette.muted_text));
            } else {
                lines.push(muteline(
                    "Discography and top tracks load in artist view.".to_string(),
                    palette.muted_text,
                ));
            }
        }
        Some(SelectedSearchItem::Track(track)) => {
            lines.push(header("Track", palette.accent));
            lines.push(strong(track.title.clone(), palette.foreground));
            lines.push(muteline(track.artist.clone(), palette.muted_text));
            lines.push(meta(track.album.clone(), palette.table_header));
            lines.push(Line::from(""));
            let lyrics = if state.lyrics.lyrics.is_some() {
                "Lyrics loaded"
            } else {
                "Lyrics available after playback/detail load"
            };
            lines.push(muteline(lyrics.to_string(), palette.muted_text));
        }
        None => {
            lines.push(header("Preview", palette.accent));
            lines.push(muteline(
                "Search for an album, artist, or track to inspect it here.".to_string(),
                palette.muted_text,
            ));
        }
    }

    lines
}

fn header(text: &str, color: ratatui::style::Color) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

fn strong(text: String, color: ratatui::style::Color) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

fn muteline(text: String, color: ratatui::style::Color) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(color)))
}

fn meta(text: String, color: ratatui::style::Color) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(color)))
}

#[cfg(test)]
mod tests {
    use crate::{
        models::{album::Album, artist::Artist},
        state::AppState,
    };

    use super::*;

    #[test]
    fn artist_detail_preview_uses_current_artist_not_first_album() {
        let mut state = AppState::new();
        state.search.set_artist_results(
            Artist {
                id: 7629451,
                name: "PNL".to_string(),
                picture_id: Some("979bb546-8007-4395-9257-20d521ea791e".to_string()),
                description: None,
                album_count: None,
                track_count: None,
            },
            vec![Album {
                id: 107105449,
                title: "Deux frères".to_string(),
                artist: "PNL".to_string(),
                cover_id: Some("85ee3a95-9627-41e1-834b-2df965d26004".to_string()),
                release_date: None,
                track_count: Some(16),
            }],
            Vec::new(),
        );

        let text = preview_lines(&state, true)
            .into_iter()
            .flat_map(|line| line.spans.into_iter().map(|span| span.content.into_owned()))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(text.contains("Artist"));
        assert!(text.contains("PNL"));
        assert!(text.contains("Albums: 1"));
        assert!(!text.contains("Deux frères"));
    }
}
