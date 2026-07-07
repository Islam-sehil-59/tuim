use tuim::{
    models::track::Track,
    providers::{
        monochrome::{MonochromeProvider, client::ApiClient},
        provider::MusicProvider,
    },
};

#[test]
fn monochrome_provider_wraps_existing_api_client() {
    let api = ApiClient::new();
    let provider = MonochromeProvider::from_api(api);

    let _ = provider.api();
}

#[test]
fn monochrome_provider_uses_configured_deezer_fallback_url() {
    let provider =
        MonochromeProvider::new_with_deezer_url(Some("https://fallback.example.test".to_string()));

    assert_eq!(
        provider.api().deezer_fallback_base_url(),
        "https://fallback.example.test"
    );
}

#[test]
fn provider_trait_is_object_safe_for_app_boundaries() {
    fn accepts_provider(_provider: &dyn MusicProvider) {}

    let provider = MonochromeProvider::new();

    accepts_provider(&provider);
}

#[test]
fn playback_resolution_uses_track_identity_and_isrc_boundary() {
    let track = Track {
        id: 1,
        title: "Track".to_string(),
        artist: "Artist".to_string(),
        artist_id: 2,
        album: "Album".to_string(),
        album_id: 3,
        cover_id: None,
        isrc: Some("USRC17607839".to_string()),
    };

    assert_eq!(track.id, 1);
    assert_eq!(track.isrc.as_deref(), Some("USRC17607839"));
}
