pub struct CoverState {
    pub request_key: Option<String>,
    pub loading: bool,
    pub path: Option<String>,
}

impl CoverState {
    pub fn new() -> Self {
        Self {
            request_key: None,
            loading: false,
            path: None,
        }
    }

    pub fn clear(&mut self) {
        self.request_key = None;
        self.loading = false;
        self.path = None;
    }
}

impl Default for CoverState {
    fn default() -> Self {
        Self::new()
    }
}
