#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum VisualizerMode {
    #[default]
    Disabled,
    Cava,
}

#[derive(Clone, Debug)]
pub struct VisualizerState {
    pub mode: VisualizerMode,
    pub available: bool,
    pub active: bool,
    pub last_error: Option<String>,
}

impl VisualizerState {
    pub fn new() -> Self {
        Self {
            mode: VisualizerMode::Cava,
            available: true,
            active: false,
            last_error: None,
        }
    }

    pub fn set_unavailable(&mut self, error: impl Into<String>) {
        self.available = false;
        self.active = false;
        self.last_error = Some(error.into());
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }
}

impl Default for VisualizerState {
    fn default() -> Self {
        Self::new()
    }
}
