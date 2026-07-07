pub struct StatusState {
    pub message: String,
}

impl StatusState {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
