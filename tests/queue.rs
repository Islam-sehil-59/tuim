use tuim::{
    engine::queue as queue_engine,
    models::track::Track,
    state::{queue::Queue, search::SearchState},
};

#[test]
fn queue_can_hold_tracks_in_order() {
    let first = Track {
        id: 1,
        title: "First".to_string(),
        artist: "Artist".to_string(),
        artist_id: 10,
        album: "Album".to_string(),
        album_id: 20,
        cover_id: None,
        isrc: None,
    };
    let second = Track {
        id: 2,
        title: "Second".to_string(),
        artist: "Artist".to_string(),
        artist_id: 10,
        album: "Album".to_string(),
        album_id: 20,
        cover_id: None,
        isrc: None,
    };

    let mut queue = Queue::new();
    queue.add(first);
    queue.add(second);

    assert_eq!(queue.items.len(), 2);
    assert_eq!(queue.items[0].id, 1);
    assert_eq!(queue.items[1].id, 2);
    assert_eq!(queue.current().map(|track| track.id), Some(1));
    assert_eq!(queue.selected_track().map(|track| track.id), Some(1));
}

#[test]
fn queue_can_insert_after_current_track() {
    let mut queue = Queue::new();
    queue.add(track(1));
    queue.add(track(3));
    queue.add_next(track(2));

    let ids: Vec<u64> = queue.items.iter().map(|track| track.id).collect();

    assert_eq!(ids, vec![1, 2, 3]);
}

#[test]
fn queue_updates_current_index_when_removing_before_current() {
    let mut queue = Queue::new();
    queue.add(track(1));
    queue.add(track(2));
    queue.add(track(3));
    queue.next();

    let removed = queue.remove(0);

    assert_eq!(removed.map(|track| track.id), Some(1));
    assert_eq!(queue.current_index, Some(0));
    assert_eq!(queue.current().map(|track| track.id), Some(2));
}

#[test]
fn queue_clears_current_index_when_empty() {
    let mut queue = Queue::new();
    queue.add(track(1));

    queue.clear();

    assert!(queue.is_empty());
    assert_eq!(queue.current_index, None);
    assert_eq!(queue.selected, 0);
}

#[test]
fn queue_selection_tracks_removal() {
    let mut queue = Queue::new();
    queue.add(track(1));
    queue.add(track(2));
    queue.select_next();

    let removed = queue.remove_selected();

    assert_eq!(removed.map(|track| track.id), Some(2));
    assert_eq!(queue.selected, 0);
    assert_eq!(queue.selected_track().map(|track| track.id), Some(1));
}

#[test]
fn queue_moves_current_track_forward_and_backward() {
    let mut queue = Queue::new();
    queue.add(track(1));
    queue.add(track(2));
    queue.add(track(3));

    assert_eq!(queue.next().map(|track| track.id), Some(2));
    assert_eq!(queue.next().map(|track| track.id), Some(3));
    assert_eq!(queue.next().map(|track| track.id), None);
    assert_eq!(queue.previous().map(|track| track.id), Some(2));
}

#[test]
fn queue_engine_replaces_queue_with_search_context_tracks() {
    let mut search = SearchState::new();
    search.album_tracks = vec![track(10), track(11)];
    let mut queue = Queue::new();
    queue.add(track(1));

    let first = queue_engine::replace_with_search_context(&mut queue, &search);

    assert_eq!(first.map(|track| track.id), Ok(10));
    assert_eq!(
        queue.items.iter().map(|track| track.id).collect::<Vec<_>>(),
        vec![10, 11]
    );
    assert_eq!(queue.current().map(|track| track.id), Some(10));
}

#[test]
fn queue_engine_appends_search_context_tracks() {
    let mut search = SearchState::new();
    search.album_tracks = vec![track(10), track(11)];
    let mut queue = Queue::new();
    queue.add(track(1));

    let count = queue_engine::append_search_context(&mut queue, &search);

    assert_eq!(count, Ok(2));
    assert_eq!(
        queue.items.iter().map(|track| track.id).collect::<Vec<_>>(),
        vec![1, 10, 11]
    );
}

#[test]
fn queue_engine_reports_empty_search_context() {
    let search = SearchState::new();
    let mut queue = Queue::new();

    let result = queue_engine::replace_with_search_context(&mut queue, &search);

    assert_eq!(result.err(), Some(String::from("Album has no tracks.")));
    assert!(queue.is_empty());
}

fn track(id: u64) -> Track {
    Track {
        id,
        title: format!("Track {id}"),
        artist: "Artist".to_string(),
        artist_id: 10,
        album: "Album".to_string(),
        album_id: 20,
        cover_id: None,
        isrc: None,
    }
}
