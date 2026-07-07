use crate::media::lyrics::Lyrics;

pub struct LyricsState {
    pub track_id: Option<u64>,
    pub loading: bool,
    pub requested_duration_seconds: Option<u32>,
    pub scroll: usize,
    pub lyrics: Option<Lyrics>,
    pub error: Option<String>,
}

impl LyricsState {
    pub fn new() -> Self {
        Self {
            track_id: None,
            loading: false,
            requested_duration_seconds: None,
            scroll: 0,
            lyrics: None,
            error: None,
        }
    }

    pub fn start_loading(&mut self, track_id: u64, duration_seconds: Option<u32>) {
        self.track_id = Some(track_id);
        self.loading = true;
        self.requested_duration_seconds = duration_seconds;
        self.scroll = 0;
        self.lyrics = None;
        self.error = None;
    }

    pub fn clear(&mut self) {
        self.track_id = None;
        self.loading = false;
        self.requested_duration_seconds = None;
        self.scroll = 0;
        self.lyrics = None;
        self.error = None;
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }
}

impl Default for LyricsState {
    fn default() -> Self {
        Self::new()
    }
}
