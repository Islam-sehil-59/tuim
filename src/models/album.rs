#[derive(Clone, Debug)]
pub struct Album {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub cover_id: Option<String>,
    pub release_date: Option<String>,
    pub track_count: Option<usize>,
}
