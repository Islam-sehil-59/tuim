use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct SearchResponse {
    pub(super) data: SearchData,
}

#[derive(Deserialize)]
pub(super) struct SearchData {
    pub(super) items: Vec<SearchItem>,
}

#[derive(Deserialize)]
pub(super) struct SearchItem {
    pub(super) id: u64,
    pub(super) title: String,
    pub(super) artist: SearchArtist,
    pub(super) album: SearchAlbum,
    pub(super) isrc: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct SearchArtist {
    pub(super) id: u64,
    pub(super) name: String,
}

#[derive(Deserialize)]
pub(super) struct SearchAlbum {
    pub(super) id: u64,
    pub(super) title: String,
    pub(super) cover: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct AlbumResponse {
    pub(super) data: Option<AlbumData>,
    pub(super) album: Option<AlbumData>,
}

#[derive(Deserialize)]
pub(super) struct AlbumData {
    pub(super) id: u64,
    pub(super) title: String,
    pub(super) cover: Option<String>,
    pub(super) artist: SearchArtist,
}

#[derive(Deserialize)]
pub(super) struct ArtistResponse {
    pub(super) artist: Option<ArtistData>,
    pub(super) data: Option<ArtistData>,
}

#[derive(Deserialize)]
pub(super) struct ArtistData {
    pub(super) id: u64,
    pub(super) name: String,
    pub(super) picture: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct TrackManifestResponse {
    pub(super) data: TrackManifestEnvelope,
}

#[derive(Deserialize)]
pub(super) struct TrackManifestEnvelope {
    pub(super) data: TrackManifestResource,
}

#[derive(Deserialize)]
pub(super) struct TrackManifestResource {
    pub(super) attributes: TrackManifestAttributes,
}

#[derive(Deserialize)]
pub(super) struct TrackManifestAttributes {
    #[serde(rename = "trackPresentation")]
    pub(super) track_presentation: String,
    #[serde(rename = "previewReason")]
    pub(super) preview_reason: Option<String>,
    pub(super) uri: String,
    pub(super) formats: Vec<String>,
    #[serde(rename = "drmData")]
    pub(super) drm_data: Option<DrmData>,
}

#[derive(Deserialize)]
pub(super) struct DrmData {
    #[serde(rename = "drmSystem")]
    pub(super) _drm_system: String,
}

#[derive(Deserialize)]
pub(super) struct PlaybackResponse {
    pub(super) data: Option<PlaybackData>,
    url: Option<String>,
    #[serde(rename = "streamUrl")]
    stream_url: Option<String>,
}

impl PlaybackResponse {
    pub(super) fn url(&self) -> Option<&str> {
        self.url
            .as_deref()
            .or(self.stream_url.as_deref())
            .or_else(|| self.data.as_ref().and_then(PlaybackData::url))
    }
}

#[derive(Deserialize)]
pub(super) struct PlaybackData {
    pub(super) manifest: Option<String>,
    url: Option<String>,
    #[serde(rename = "streamUrl")]
    stream_url: Option<String>,
    #[serde(rename = "assetPresentation")]
    pub(super) asset_presentation: String,
    #[serde(rename = "audioQuality")]
    pub(super) audio_quality: String,
    #[serde(rename = "manifestMimeType")]
    pub(super) manifest_mime_type: String,
    #[serde(rename = "previewReason")]
    pub(super) preview_reason: Option<String>,
}

impl PlaybackData {
    fn url(&self) -> Option<&str> {
        self.url.as_deref().or(self.stream_url.as_deref())
    }
}
