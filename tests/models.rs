use tuim::models::{album::Album, artist::Artist, track::Track};

#[test]
fn track_clone_preserves_provider_metadata() {
    let track = Track {
        id: 42,
        title: "Song".to_string(),
        artist: "Artist".to_string(),
        artist_id: 7,
        album: "Album".to_string(),
        album_id: 9,
        cover_id: Some("cover-id".to_string()),
        isrc: Some("USRC17607839".to_string()),
    };

    let cloned = track.clone();

    assert_eq!(cloned.id, 42);
    assert_eq!(cloned.artist_id, 7);
    assert_eq!(cloned.album_id, 9);
    assert_eq!(cloned.cover_id.as_deref(), Some("cover-id"));
    assert_eq!(cloned.isrc.as_deref(), Some("USRC17607839"));
}

#[test]
fn album_and_artist_models_allow_missing_images() {
    let album = Album {
        id: 1,
        title: "Album".to_string(),
        artist: "Artist".to_string(),
        cover_id: None,
        release_date: None,
        track_count: None,
    };
    let artist = Artist {
        id: 2,
        name: "Artist".to_string(),
        picture_id: None,
        description: None,
        album_count: None,
        track_count: None,
    };

    assert_eq!(album.cover_id, None);
    assert_eq!(artist.picture_id, None);
}
