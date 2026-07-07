#[derive(Clone, Debug)]
pub struct Artist {
    pub id: u64,
    pub name: String,
    pub picture_id: Option<String>,
    pub description: Option<String>,
    pub album_count: Option<usize>,
    pub track_count: Option<usize>,
}
