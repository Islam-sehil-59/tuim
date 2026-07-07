use crate::models::track::Track;

pub struct Queue {
    pub items: Vec<Track>,
    pub current_index: Option<usize>,
    pub selected: usize,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: None,
            selected: 0,
        }
    }

    pub fn add(&mut self, track: Track) {
        self.items.push(track);
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
        self.clamp_selected();
    }

    pub fn add_next(&mut self, track: Track) {
        let insert_index = self
            .current_index
            .map(|index| index.saturating_add(1))
            .unwrap_or(self.items.len())
            .min(self.items.len());

        self.items.insert(insert_index, track);
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
        self.clamp_selected();
    }

    pub fn remove(&mut self, index: usize) -> Option<Track> {
        if index >= self.items.len() {
            return None;
        }

        let removed = self.items.remove(index);
        self.current_index = match (self.current_index, self.items.is_empty()) {
            (_, true) => None,
            (Some(current), false) if index < current => Some(current - 1),
            (Some(current), false) if current >= self.items.len() => Some(self.items.len() - 1),
            (current, false) => current,
        };
        self.clamp_selected();

        Some(removed)
    }

    pub fn current(&self) -> Option<&Track> {
        self.current_index.and_then(|index| self.items.get(index))
    }

    pub fn selected_track(&self) -> Option<&Track> {
        self.items.get(self.selected)
    }

    pub fn select_next(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }

        self.selected = (self.selected + 1).min(self.items.len() - 1);
        true
    }

    pub fn select_previous(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }

        self.selected = self.selected.saturating_sub(1);
        true
    }

    pub fn remove_selected(&mut self) -> Option<Track> {
        self.remove(self.selected)
    }

    pub fn set_current_to_selected(&mut self) -> Option<&Track> {
        if self.selected >= self.items.len() {
            return None;
        }

        self.current_index = Some(self.selected);
        self.current()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&Track> {
        let current = self.current_index?;
        let next = current.checked_add(1)?;
        if next >= self.items.len() {
            return None;
        }

        self.current_index = Some(next);
        self.current()
    }

    pub fn previous(&mut self) -> Option<&Track> {
        let current = self.current_index?;
        let previous = current.checked_sub(1)?;

        self.current_index = Some(previous);
        self.current()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current_index = None;
        self.selected = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn clamp_selected(&mut self) {
        if self.items.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.items.len() - 1);
        }
    }
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}
