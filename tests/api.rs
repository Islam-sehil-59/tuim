use tuim::providers::monochrome::{
    client::ApiClient,
    playback::{PlaybackResolution, PlaybackSourceKind},
};

#[test]
fn api_client_is_constructible_without_network_io() {
    let _client = ApiClient::new();
}

#[test]
fn playback_resolution_carries_source_metadata() {
    let resolution = PlaybackResolution {
        instance: "https://api.example.test".to_string(),
        source_kind: PlaybackSourceKind::DirectStreamUrl,
        source: "https://cdn.example.test/track.flac".to_string(),
        audio_quality: Some("FLAC".to_string()),
        presentation: Some("FULL".to_string()),
        preview_reason: None,
        manifest_mime_type: None,
        drm_protected: false,
    };

    assert_eq!(resolution.instance, "https://api.example.test");
    assert_eq!(resolution.source, "https://cdn.example.test/track.flac");
    assert_eq!(resolution.audio_quality.as_deref(), Some("FLAC"));
    assert_eq!(resolution.presentation.as_deref(), Some("FULL"));
    assert!(!resolution.drm_protected);
}

#[test]
fn playback_resolution_classifies_preview_only_sources() {
    let mut resolution = PlaybackResolution {
        instance: "https://api.example.test".to_string(),
        source_kind: PlaybackSourceKind::TrackManifestUri,
        source: "https://cdn.example.test/track.mpd".to_string(),
        audio_quality: Some("AACLC".to_string()),
        presentation: Some("FULL".to_string()),
        preview_reason: Some("FULL_REQUIRES_SUBSCRIPTION".to_string()),
        manifest_mime_type: Some("application/dash+xml".to_string()),
        drm_protected: false,
    };

    assert!(resolution.is_preview_only());
    assert!(!resolution.is_full_playback());

    resolution.preview_reason = None;
    assert!(!resolution.is_preview_only());
    assert!(resolution.is_full_playback());
}
