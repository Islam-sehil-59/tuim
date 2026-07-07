use crate::{
    models::track::Track,
    state::{
        queue::Queue,
        search::{SearchContext, SearchState},
    },
};

pub enum QueueInsert {
    End,
    Next,
}

pub fn add_track(queue: &mut Queue, track: Track, insert: QueueInsert) {
    match insert {
        QueueInsert::End => queue.add(track),
        QueueInsert::Next => queue.add_next(track),
    }
}

pub fn replace_with_search_context(
    queue: &mut Queue,
    search: &SearchState,
) -> Result<Track, String> {
    let context_label = search_context_label(search.context);
    if search.album_tracks.is_empty() {
        return Err(match search.context {
            SearchContext::Artist => {
                String::from("Artist has no top tracks here. Open an album to play it.")
            }
            _ => format!("{context_label} has no tracks."),
        });
    }

    queue.clear();
    for track in search.album_tracks.clone() {
        add_track(queue, track, QueueInsert::End);
    }

    queue
        .current()
        .cloned()
        .ok_or_else(|| format!("{context_label} has no playable tracks."))
}

pub fn append_search_context(queue: &mut Queue, search: &SearchState) -> Result<usize, String> {
    let context_label = search_context_label_lower(search.context);
    if search.album_tracks.is_empty() {
        return Err(match search.context {
            SearchContext::Artist => {
                String::from("Artist has no top tracks here. Open an album to queue it.")
            }
            _ => format!("{context_label} has no tracks."),
        });
    }

    let count = search.album_tracks.len();
    for track in search.album_tracks.clone() {
        add_track(queue, track, QueueInsert::End);
    }

    Ok(count)
}

pub fn search_context_label(context: SearchContext) -> &'static str {
    match context {
        SearchContext::Artist => "Artist",
        _ => "Album",
    }
}

pub fn search_context_label_lower(context: SearchContext) -> &'static str {
    match context {
        SearchContext::Artist => "artist",
        _ => "album",
    }
}
