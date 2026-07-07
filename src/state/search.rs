use crate::models::{album::Album, artist::Artist, track::Track};
use crate::providers::provider::SearchResults;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchContext {
    Results,
    Album,
    Artist,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchFilter {
    Tracks,
    Albums,
    Artists,
}

impl SearchFilter {
    pub fn label(self) -> &'static str {
        match self {
            SearchFilter::Tracks => "tracks",
            SearchFilter::Albums => "albums",
            SearchFilter::Artists => "artists",
        }
    }

    pub fn previous(self) -> Self {
        match self {
            SearchFilter::Tracks => SearchFilter::Artists,
            SearchFilter::Albums => SearchFilter::Tracks,
            SearchFilter::Artists => SearchFilter::Albums,
        }
    }

    pub fn next(self) -> Self {
        match self {
            SearchFilter::Tracks => SearchFilter::Albums,
            SearchFilter::Albums => SearchFilter::Artists,
            SearchFilter::Artists => SearchFilter::Tracks,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SelectedSearchItem<'a> {
    Album(&'a Album),
    Artist(&'a Artist),
    Track(&'a Track),
}

pub struct SearchState {
    pub query: String,
    pub results: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub selected: usize,
    pub results_focused: bool,
    pub context: SearchContext,
    pub filter: SearchFilter,
    pub current_album: Option<Album>,
    pub current_artist: Option<Artist>,
    pub album_tracks: Vec<Track>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            selected: 0,
            results_focused: false,
            context: SearchContext::Results,
            filter: SearchFilter::Tracks,
            current_album: None,
            current_artist: None,
            album_tracks: Vec::new(),
        }
    }

    pub fn selected_track(&self) -> Option<&Track> {
        match self.context {
            SearchContext::Results => match self.filter {
                SearchFilter::Tracks => self.results.get(self.selected),
                SearchFilter::Albums | SearchFilter::Artists => None,
            },
            SearchContext::Album => self.album_tracks.get(self.selected),
            SearchContext::Artist => {
                let track_index = self.selected.checked_sub(self.albums.len())?;
                self.album_tracks.get(track_index)
            }
        }
    }

    pub fn selected_item(&self) -> Option<SelectedSearchItem<'_>> {
        match self.context {
            SearchContext::Results => match self.filter {
                SearchFilter::Tracks => self
                    .results
                    .get(self.selected)
                    .map(SelectedSearchItem::Track),
                SearchFilter::Albums => self
                    .albums
                    .get(self.selected)
                    .map(SelectedSearchItem::Album),
                SearchFilter::Artists => self
                    .artists
                    .get(self.selected)
                    .map(SelectedSearchItem::Artist),
            },
            SearchContext::Album => self
                .album_tracks
                .get(self.selected)
                .map(SelectedSearchItem::Track),
            SearchContext::Artist => {
                if let Some(album) = self.albums.get(self.selected) {
                    return Some(SelectedSearchItem::Album(album));
                }

                let track_index = self.selected.checked_sub(self.albums.len())?;
                self.album_tracks
                    .get(track_index)
                    .map(SelectedSearchItem::Track)
            }
        }
    }

    pub fn total_items(&self) -> usize {
        match self.context {
            SearchContext::Results => match self.filter {
                SearchFilter::Tracks => self.results.len(),
                SearchFilter::Albums => self.albums.len(),
                SearchFilter::Artists => self.artists.len(),
            },
            SearchContext::Album | SearchContext::Artist => {
                self.album_tracks.len() + self.albums.len()
            }
        }
    }

    pub fn select_next(&mut self) -> bool {
        let total = self.total_items();
        if total == 0 {
            return false;
        }

        self.selected = (self.selected + 1).min(total - 1);
        true
    }

    pub fn select_previous(&mut self) -> bool {
        if self.total_items() == 0 {
            return false;
        }

        self.selected = self.selected.saturating_sub(1);
        true
    }

    pub fn set_results(&mut self, results: SearchResults) {
        self.selected = 0;
        self.results = results.tracks;
        self.albums = results.albums;
        self.artists = results.artists;
        self.context = SearchContext::Results;
        self.current_album = None;
        self.current_artist = None;
        self.album_tracks.clear();
        self.results_focused = self.total_items() > 0;
    }

    pub fn set_filter(&mut self, filter: SearchFilter) {
        self.filter = filter;
        self.selected = self.selected.min(self.total_items().saturating_sub(1));
        self.results_focused = self.total_items() > 0;
    }

    pub fn set_album_tracks(&mut self, album: Album, tracks: Vec<Track>) {
        self.selected = 0;
        self.context = SearchContext::Album;
        self.current_album = Some(album);
        self.current_artist = None;
        self.album_tracks = tracks;
        self.results_focused = !self.album_tracks.is_empty();
    }

    pub fn set_artist_results(&mut self, artist: Artist, albums: Vec<Album>, tracks: Vec<Track>) {
        self.selected = 0;
        self.context = SearchContext::Artist;
        self.current_artist = Some(artist);
        self.current_album = None;
        self.albums = albums;
        self.album_tracks = tracks;
        self.filter = SearchFilter::Tracks;
        self.results_focused = self.total_items() > 0;
    }

    pub fn return_to_results(&mut self) {
        self.selected = 0;
        self.context = SearchContext::Results;
        self.current_album = None;
        self.current_artist = None;
        self.album_tracks.clear();
        self.results_focused = self.total_items() > 0;
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}
