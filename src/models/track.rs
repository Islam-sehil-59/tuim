#[derive(Clone, Debug)]
pub struct Track {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub artist_id: u64,
    pub album: String,
    pub album_id: u64,
    pub cover_id: Option<String>,
    pub isrc: Option<String>,
}
