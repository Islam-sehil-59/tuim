use crate::media::playback::{
    PlaybackSource as NeutralPlaybackSource, PlaybackSourceKind as NeutralPlaybackSourceKind,
};

const MONOCHROME_REFERRER_HEADER: &str = "Referer";
const MONOCHROME_REFERRER_VALUE: &str = "https://monochrome.tf/";
const MONOCHROME_ORIGIN_HEADER: &str = "Origin";
const MONOCHROME_ORIGIN_VALUE: &str = "https://monochrome.tf";

#[derive(Clone, Debug)]
pub enum PlaybackSourceKind {
    DeezerIsrcStream,
    TrackManifestUri,
    InlineManifestFile,
    DirectStreamUrl,
}

#[derive(Clone, Debug)]
pub struct PlaybackResolution {
    pub instance: String,
    pub source_kind: PlaybackSourceKind,
    pub source: String,
    pub audio_quality: Option<String>,
    pub presentation: Option<String>,
    pub preview_reason: Option<String>,
    pub manifest_mime_type: Option<String>,
    pub drm_protected: bool,
}

impl PlaybackResolution {
    pub fn is_full_playback(&self) -> bool {
        self.preview_reason.is_none()
            && self
                .presentation
                .as_deref()
                .map(|presentation| presentation == "FULL")
                .unwrap_or(true)
    }

    pub fn is_preview_only(&self) -> bool {
        self.preview_reason.is_some() || self.presentation.as_deref() == Some("PREVIEW")
    }

    pub fn into_playback_source(self) -> NeutralPlaybackSource {
        let kind = match self.source_kind {
            PlaybackSourceKind::DeezerIsrcStream | PlaybackSourceKind::DirectStreamUrl => {
                NeutralPlaybackSourceKind::DirectUrl
            }
            PlaybackSourceKind::TrackManifestUri | PlaybackSourceKind::InlineManifestFile => {
                match self.manifest_mime_type.as_deref() {
                    Some(mime) if mime.eq_ignore_ascii_case("application/vnd.apple.mpegurl") => {
                        NeutralPlaybackSourceKind::HlsManifest
                    }
                    _ => NeutralPlaybackSourceKind::DashManifest,
                }
            }
        };
        let is_preview = self.is_preview_only();

        let headers = match self.source_kind {
            PlaybackSourceKind::DeezerIsrcStream | PlaybackSourceKind::DirectStreamUrl => {
                monochrome_playback_headers()
            }
            PlaybackSourceKind::TrackManifestUri | PlaybackSourceKind::InlineManifestFile => {
                Vec::new()
            }
        };

        NeutralPlaybackSource {
            url: self.source,
            kind,
            quality: self.audio_quality,
            format: self.manifest_mime_type,
            headers,
            is_preview,
            preview_reason: self.preview_reason,
            expires_at: None,
        }
    }
}

fn monochrome_playback_headers() -> Vec<(String, String)> {
    vec![
        (
            MONOCHROME_REFERRER_HEADER.to_string(),
            MONOCHROME_REFERRER_VALUE.to_string(),
        ),
        (
            MONOCHROME_ORIGIN_HEADER.to_string(),
            MONOCHROME_ORIGIN_VALUE.to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_resolution_to_provider_neutral_playback_source() {
        let source = PlaybackResolution {
            instance: "https://api.example.test".to_string(),
            source_kind: PlaybackSourceKind::DeezerIsrcStream,
            source: "https://cdn.example.test/track.flac".to_string(),
            audio_quality: Some("FLAC".to_string()),
            presentation: Some("FULL".to_string()),
            preview_reason: None,
            manifest_mime_type: None,
            drm_protected: false,
        }
        .into_playback_source();

        assert_eq!(source.url, "https://cdn.example.test/track.flac");
        assert_eq!(source.kind, NeutralPlaybackSourceKind::DirectUrl);
        assert_eq!(source.quality.as_deref(), Some("FLAC"));
        assert!(!source.is_preview);
        assert_eq!(source.headers, monochrome_playback_headers());
    }

    #[test]
    fn conversion_preserves_preview_status_and_manifest_format() {
        let source = PlaybackResolution {
            instance: "https://api.example.test".to_string(),
            source_kind: PlaybackSourceKind::TrackManifestUri,
            source: "https://cdn.example.test/manifest.mpd".to_string(),
            audio_quality: Some("AACLC".to_string()),
            presentation: Some("PREVIEW".to_string()),
            preview_reason: Some("FULL_REQUIRES_SUBSCRIPTION".to_string()),
            manifest_mime_type: Some("application/dash+xml".to_string()),
            drm_protected: false,
        }
        .into_playback_source();

        assert_eq!(source.kind, NeutralPlaybackSourceKind::DashManifest);
        assert_eq!(source.format.as_deref(), Some("application/dash+xml"));
        assert!(source.is_preview);
        assert_eq!(
            source.preview_reason.as_deref(),
            Some("FULL_REQUIRES_SUBSCRIPTION")
        );
        assert!(source.headers.is_empty());
    }
}
