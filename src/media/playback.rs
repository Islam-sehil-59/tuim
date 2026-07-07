#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaybackSourceKind {
    DirectUrl,
    DashManifest,
    HlsManifest,
    ProviderSpecific(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackSource {
    pub url: String,
    pub kind: PlaybackSourceKind,
    pub quality: Option<String>,
    pub format: Option<String>,
    pub headers: Vec<(String, String)>,
    pub is_preview: bool,
    pub preview_reason: Option<String>,
    pub expires_at: Option<String>,
}

impl PlaybackSource {
    pub fn new(url: impl Into<String>, kind: PlaybackSourceKind) -> Self {
        Self {
            url: url.into(),
            kind,
            quality: None,
            format: None,
            headers: Vec::new(),
            is_preview: false,
            preview_reason: None,
            expires_at: None,
        }
    }

    pub fn with_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = headers;
        self
    }
}
